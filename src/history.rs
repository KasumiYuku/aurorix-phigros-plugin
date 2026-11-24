use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{Mutex as AsyncMutex, Notify};

use crate::backend::phi::PhiError;
use crate::repo::{HistoryJob, Repo};
use crate::service::{now_seconds, PhigrosService};

pub const HISTORY_BUDGET: Duration = Duration::from_secs(25);

pub const POLL: Duration = Duration::from_secs(30);

pub const FLOOR: Duration = Duration::from_secs(1);

pub const BATCH: i64 = 16;

pub const EMPTY_RETRY: i64 = 24 * 3600;

pub const BACKOFF: [i64; 5] = [60, 300, 1800, 7200, 21_600];

pub fn backoff_after(attempts: i64) -> i64 {
    if attempts < 1 {
        return BACKOFF[0];
    }
    let index = (attempts as usize).min(BACKOFF.len()) - 1;
    BACKOFF[index]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Rearm,
    Defer,
}

fn verdict(error: PhiError) -> Verdict {
    match error {
        PhiError::NoHistory | PhiError::Offline => Verdict::Rearm,
        PhiError::InvalidToken | PhiError::Unavailable => Verdict::Defer,
    }
}

#[derive(Default)]
pub struct Worker {
    wake: Notify,
    locks: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
}

impl Worker {
    pub fn poke(&self) {
        self.wake.notify_one();
    }

    pub async fn wait(&self, span: Duration) {
        tokio::select! {
            _ = self.wake.notified() => {}
            _ = tokio::time::sleep(span) => {}
        }
    }

    fn lock_for(&self, user_id: &str) -> Arc<AsyncMutex<()>> {
        let mut guard = self
            .locks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(guard.entry(user_id.to_string()).or_default())
    }
}

pub fn spawn(service: Arc<PhigrosService>) {
    tokio::spawn(async move {
        run(service).await;
    });
}

async fn run(service: Arc<PhigrosService>) {
    loop {
        let wait = round(&service).await;
        service.history_worker().wait(wait).await;
    }
}

async fn round(service: &Arc<PhigrosService>) -> Duration {
    let repo = service.repo();
    let Ok(jobs) = repo.due_history_jobs(now_seconds(), BATCH).await else {
        return POLL;
    };
    for job in &jobs {
        one(service, job).await;
    }
    if jobs.len() as i64 == BATCH {
        FLOOR
    } else {
        POLL
    }
}

async fn one(service: &Arc<PhigrosService>, job: &HistoryJob) {
    let repo = service.repo();
    let binding = match repo.binding(&job.user_id).await {
        Ok(Some(binding)) => binding,
        Ok(None) => return finish(&repo, &job.user_id).await,
        Err(_) => return defer(&repo, job).await,
    };
    if binding.token.is_empty() {
        return finish(&repo, &job.user_id).await;
    }
    let archive = match tokio::time::timeout(
        HISTORY_BUDGET,
        service.phi().history(&binding.token),
    )
    .await
    {
        Ok(Ok(archive)) => archive,
        Ok(Err(error)) => return settle(&repo, job, error).await,
        Err(_) => return defer(&repo, job).await,
    };
    let lock = service.history_worker().lock_for(&job.user_id);
    let _guard = lock.lock().await;
    let current = match repo.binding(&job.user_id).await {
        Ok(Some(current)) => current,
        Ok(None) => return,
        Err(_) => return defer(&repo, job).await,
    };
    if current.token != binding.token {
        return;
    }
    let _ = repo
        .adopt_snapshots(&job.user_id, &current.object_id)
        .await;
    if repo
        .import_history(&job.user_id, &archive.points, &archive.events)
        .await
        .is_err()
    {
        return defer(&repo, job).await;
    }
    if archive.is_empty() {
        return rearm(&repo, job).await;
    }
    finish(&repo, &job.user_id).await;
}

async fn settle(repo: &Repo, job: &HistoryJob, error: PhiError) {
    match verdict(error) {
        Verdict::Rearm => rearm(repo, job).await,
        Verdict::Defer => defer(repo, job).await,
    }
}

async fn defer(repo: &Repo, job: &HistoryJob) {
    let attempts = job.attempts + 1;
    let mut next = job.clone();
    next.pending = true;
    next.attempts = attempts;
    next.next_at = now_seconds() + backoff_after(attempts);
    let _ = repo.put_history_job(&next).await;
}

async fn rearm(repo: &Repo, job: &HistoryJob) {
    let mut next = job.clone();
    next.pending = true;
    next.attempts = 0;
    next.next_at = now_seconds() + EMPTY_RETRY;
    let _ = repo.put_history_job(&next).await;
}

async fn finish(repo: &Repo, user_id: &str) {
    let mut job = repo.history_job(user_id).await.ok().flatten().unwrap_or_default();
    job.user_id = user_id.to_string();
    job.pending = false;
    job.next_at = 0;
    job.synced_at = now_seconds();
    job.error.clear();
    let _ = repo.put_history_job(&job).await;
}

pub async fn schedule(repo: &Repo, user_id: &str, fresh: bool) {
    let mut job = repo.history_job(user_id).await.ok().flatten().unwrap_or_default();
    job.user_id = user_id.to_string();
    job.pending = true;
    job.next_at = 0;
    if fresh {
        job.attempts = 0;
        job.error.clear();
    }
    let _ = repo.put_history_job(&job).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_walks_the_table_and_stays_on_the_last_step() {
        assert_eq!(backoff_after(0), 60);
        assert_eq!(backoff_after(1), 60);
        assert_eq!(backoff_after(2), 300);
        assert_eq!(backoff_after(3), 1800);
        assert_eq!(backoff_after(4), 7200);
        assert_eq!(backoff_after(5), 21_600);
        assert_eq!(backoff_after(6), 21_600);
        assert_eq!(backoff_after(500), 21_600);
    }

    #[test]
    fn missing_history_and_offline_rearm_instead_of_backing_off() {
        assert_eq!(verdict(PhiError::NoHistory), Verdict::Rearm);
        assert_eq!(verdict(PhiError::Offline), Verdict::Rearm);
        assert_eq!(verdict(PhiError::Unavailable), Verdict::Defer);
        assert_eq!(verdict(PhiError::InvalidToken), Verdict::Defer);
        assert_eq!(EMPTY_RETRY, 86_400);
    }

    #[test]
    fn budgets_match_the_go_worker() {
        assert_eq!(HISTORY_BUDGET, Duration::from_secs(25));
        assert_eq!(BATCH, 16);
        assert_eq!(FLOOR, Duration::from_secs(1));
    }
}
