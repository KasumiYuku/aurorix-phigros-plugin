pub mod legacy;

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use aurorix_db::{MigrationReport, MigrationStep, PluginDb};
use sqlx::PgPool;

use crate::model::{
    BandStat, Binding, ChartStat, Counters, Difficulty, LevelStat, PlayRecord, Profile, Server,
    ServerStats, BAND_MAX, PEER_THRESHOLDS, PEER_THRESHOLD_COUNT, STAT_MIN_USERS,
};

pub const META_TABLE: &str = "phg_meta";
pub const BINDING_TABLE: &str = "phg_binding";
pub const PROFILE_TABLE: &str = "phg_profile";
pub const RECORD_TABLE: &str = "phg_record";
pub const SCORE_EVENT_TABLE: &str = "phg_score_event";
pub const RKS_POINT_TABLE: &str = "phg_rks_point";
pub const RKS_DAILY_TABLE: &str = "phg_rks_daily";
pub const SNAPSHOT_TABLE: &str = "phg_snapshot";
pub const HISTORY_JOB_TABLE: &str = "phg_history_job";

pub const SCHEMA_VERSION: i64 = 19;

const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS phg_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS phg_binding (user_id TEXT PRIMARY KEY, server BIGINT NOT NULL, token TEXT NOT NULL, object_id TEXT NOT NULL DEFAULT '', nickname TEXT NOT NULL DEFAULT '', avatar TEXT NOT NULL DEFAULT '', save_modified_at BIGINT NOT NULL DEFAULT 0, bound_at BIGINT NOT NULL DEFAULT 0, last_sync_at BIGINT NOT NULL DEFAULT 0, status BIGINT NOT NULL DEFAULT 0, error TEXT NOT NULL DEFAULT '', pfp TEXT NOT NULL DEFAULT '', game_avatar TEXT NOT NULL DEFAULT '', joined_at BIGINT NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS phg_profile (user_id TEXT PRIMARY KEY, rks DOUBLE PRECISION NOT NULL DEFAULT 0, challenge_mode_rank BIGINT NOT NULL DEFAULT 0, counters TEXT NOT NULL DEFAULT '', self_intro TEXT NOT NULL DEFAULT '', background TEXT NOT NULL DEFAULT '', game_version BIGINT NOT NULL DEFAULT 0, save_version BIGINT NOT NULL DEFAULT 0, updated_at BIGINT NOT NULL DEFAULT 0);
CREATE INDEX IF NOT EXISTS phg_profile_rks ON phg_profile (rks DESC);
CREATE TABLE IF NOT EXISTS phg_record (user_id TEXT NOT NULL, song_id TEXT NOT NULL, difficulty BIGINT NOT NULL, score BIGINT NOT NULL, fc BIGINT NOT NULL DEFAULT 0, acc DOUBLE PRECISION NOT NULL DEFAULT 0, PRIMARY KEY (user_id, song_id, difficulty));
CREATE INDEX IF NOT EXISTS phg_record_user_score ON phg_record (user_id, score DESC);
CREATE INDEX IF NOT EXISTS phg_record_song_score ON phg_record (song_id, difficulty, score DESC, user_id);
CREATE TABLE IF NOT EXISTS phg_score_event (user_id TEXT NOT NULL, observed_at BIGINT NOT NULL, song_id TEXT NOT NULL, difficulty BIGINT NOT NULL, has_old BIGINT NOT NULL DEFAULT 0, old_score BIGINT NOT NULL DEFAULT 0, old_acc DOUBLE PRECISION NOT NULL DEFAULT 0, old_fc BIGINT NOT NULL DEFAULT 0, new_score BIGINT NOT NULL DEFAULT 0, new_acc DOUBLE PRECISION NOT NULL DEFAULT 0, new_fc BIGINT NOT NULL DEFAULT 0, old_rks DOUBLE PRECISION NOT NULL DEFAULT 0, new_rks DOUBLE PRECISION NOT NULL DEFAULT 0, PRIMARY KEY (user_id, observed_at, song_id, difficulty));
CREATE INDEX IF NOT EXISTS phg_score_event_user_at ON phg_score_event (user_id, observed_at DESC);
CREATE TABLE IF NOT EXISTS phg_rks_point (user_id TEXT NOT NULL, observed_at BIGINT NOT NULL, rks DOUBLE PRECISION NOT NULL DEFAULT 0, computed DOUBLE PRECISION NOT NULL DEFAULT 0, records BIGINT NOT NULL DEFAULT 0, phi BIGINT NOT NULL DEFAULT 0, challenge BIGINT NOT NULL DEFAULT 0, PRIMARY KEY (user_id, observed_at));
CREATE INDEX IF NOT EXISTS phg_rks_point_user_at ON phg_rks_point (user_id, observed_at DESC);
CREATE TABLE IF NOT EXISTS phg_rks_daily (user_id TEXT NOT NULL, day BIGINT NOT NULL, observed_at BIGINT NOT NULL, rks DOUBLE PRECISION NOT NULL DEFAULT 0, records BIGINT NOT NULL DEFAULT 0, phi BIGINT NOT NULL DEFAULT 0, challenge BIGINT NOT NULL DEFAULT 0, PRIMARY KEY (user_id, day));
CREATE TABLE IF NOT EXISTS phg_snapshot (owner TEXT NOT NULL, observed_at BIGINT NOT NULL, account TEXT NOT NULL DEFAULT '', session TEXT NOT NULL DEFAULT '', nickname TEXT NOT NULL DEFAULT '', rks DOUBLE PRECISION NOT NULL DEFAULT 0, challenge BIGINT NOT NULL DEFAULT 0, phi BIGINT NOT NULL DEFAULT 0, payload TEXT NOT NULL DEFAULT '', PRIMARY KEY (owner, observed_at));
CREATE INDEX IF NOT EXISTS phg_snapshot_account ON phg_snapshot (account);
CREATE TABLE IF NOT EXISTS phg_snapshot_claim (user_id TEXT NOT NULL, owner TEXT NOT NULL, observed_at BIGINT NOT NULL, claimed_at BIGINT NOT NULL DEFAULT 0, PRIMARY KEY (user_id, owner, observed_at));
CREATE TABLE IF NOT EXISTS phg_history_job (user_id TEXT NOT NULL PRIMARY KEY, pending BIGINT NOT NULL DEFAULT 1, attempts BIGINT NOT NULL DEFAULT 0, next_at BIGINT NOT NULL DEFAULT 0, synced_at BIGINT NOT NULL DEFAULT 0, error TEXT NOT NULL DEFAULT '');
CREATE INDEX IF NOT EXISTS phg_history_job_due ON phg_history_job (next_at) WHERE pending = 1;
CREATE TABLE IF NOT EXISTS phg_alias (song_id TEXT NOT NULL, alias TEXT NOT NULL, removed BIGINT NOT NULL DEFAULT 0, updated_at BIGINT NOT NULL DEFAULT 0, PRIMARY KEY (song_id, alias));
CREATE TABLE IF NOT EXISTS phg_block (user_id TEXT PRIMARY KEY, nickname TEXT NOT NULL DEFAULT '', object_id TEXT NOT NULL DEFAULT '', reason TEXT NOT NULL DEFAULT '', operator TEXT NOT NULL DEFAULT '', created_at BIGINT NOT NULL DEFAULT 0);
CREATE UNIQUE INDEX IF NOT EXISTS phg_binding_token ON phg_binding (token) WHERE token <> '';
CREATE UNIQUE INDEX IF NOT EXISTS phg_binding_object ON phg_binding (object_id) WHERE object_id <> '';
"#;

const MIGRATIONS: [MigrationStep; 1] = [MigrationStep {
    version: 1,
    name: "phg_core",
    sql: DDL,
}];

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("数据库错误：{0}")]
    Db(#[from] aurorix_db::DbError),
    #[error("SQL 错误：{0}")]
    Sql(#[from] sqlx::Error),
    #[error("序列化错误：{0}")]
    Json(#[from] serde_json::Error),
}

pub type RepoResult<T> = Result<T, RepoError>;

#[derive(Clone, Debug)]
pub struct Repo {
    db: Arc<PluginDb>,
}

impl Repo {
    pub async fn open(db: Arc<PluginDb>) -> RepoResult<MigrationReport> {
        let report = db.migrate(&MIGRATIONS).await?;
        Ok(report)
    }

    pub fn new(db: Arc<PluginDb>) -> Self {
        Self { db }
    }

    pub fn plugin_db(&self) -> &Arc<PluginDb> {
        &self.db
    }

    pub fn pool(&self) -> &PgPool {
        self.db.kv().pool()
    }

    pub async fn schema_version(&self) -> RepoResult<Option<i64>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM phg_meta WHERE key = 'schema_version'")
                .fetch_optional(self.pool())
                .await?;
        Ok(row.and_then(|(value,)| value.parse::<i64>().ok()))
    }

    pub async fn set_schema_version(&self, version: i64) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_meta (key, value) VALUES ('schema_version', $1) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
        )
        .bind(version.to_string())
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn binding(&self, user_id: &str) -> RepoResult<Option<Binding>> {
        let row = sqlx::query_as::<_, (String, i64, String, String, String, String, i64, i64, i64, i64, String, String, i64)>(
            "SELECT user_id, server, token, object_id, nickname, avatar, save_modified_at, bound_at, last_sync_at, status, error, game_avatar, joined_at FROM phg_binding WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|row| Binding {
            user_id: row.0,
            server: Server::from_code(row.1),
            token: row.2,
            object_id: row.3,
            nickname: row.4,
            avatar: row.5,
            save_modified_at: row.6,
            bound_at: row.7,
            last_sync_at: row.8,
            status: row.9,
            error: row.10,
            game_avatar: row.11,
            joined_at: row.12,
        }))
    }

    pub async fn put_binding(&self, binding: &Binding) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_binding (user_id, server, token, object_id, nickname, avatar, save_modified_at, bound_at, last_sync_at, status, error, game_avatar, joined_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT (user_id) DO UPDATE SET server = EXCLUDED.server, token = EXCLUDED.token, object_id = EXCLUDED.object_id, nickname = EXCLUDED.nickname, avatar = EXCLUDED.avatar, save_modified_at = EXCLUDED.save_modified_at, last_sync_at = EXCLUDED.last_sync_at, status = EXCLUDED.status, error = EXCLUDED.error, game_avatar = EXCLUDED.game_avatar",
        )
        .bind(&binding.user_id)
        .bind(binding.server.code())
        .bind(&binding.token)
        .bind(&binding.object_id)
        .bind(&binding.nickname)
        .bind(&binding.avatar)
        .bind(binding.save_modified_at)
        .bind(binding.bound_at)
        .bind(binding.last_sync_at)
        .bind(binding.status)
        .bind(&binding.error)
        .bind(&binding.game_avatar)
        .bind(binding.joined_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn mark_synced(&self, user_id: &str, modified_at: i64, synced_at: i64) -> RepoResult<()> {
        sqlx::query(
            "UPDATE phg_binding SET save_modified_at = $2, last_sync_at = $3, status = 1, error = '' WHERE user_id = $1",
        )
        .bind(user_id)
        .bind(modified_at)
        .bind(synced_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_id: &str) -> RepoResult<()> {
        let mut tx = self.pool().begin().await?;
        for table in [
            BINDING_TABLE,
            PROFILE_TABLE,
            RECORD_TABLE,
            SCORE_EVENT_TABLE,
            RKS_POINT_TABLE,
            RKS_DAILY_TABLE,
            HISTORY_JOB_TABLE,
        ] {
            let sql = format!("DELETE FROM {table} WHERE user_id = $1");
            sqlx::query(&sql).bind(user_id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn profile(&self, user_id: &str) -> RepoResult<Option<Profile>> {
        let row = sqlx::query_as::<_, (String, f64, i64, String, String, String, i64, i64, i64)>(
            "SELECT user_id, rks, challenge_mode_rank, counters, self_intro, background, game_version, save_version, updated_at FROM phg_profile WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|row| Profile {
            user_id: row.0,
            rks: row.1,
            challenge_mode_rank: row.2 as i32,
            counters: decode_counters(&row.3),
            self_intro: row.4,
            background: row.5,
            game_version: row.6 as i32,
            save_version: row.7 as i32,
            updated_at: row.8,
        }))
    }

    pub async fn put_profile(&self, profile: &Profile) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_profile (user_id, rks, challenge_mode_rank, counters, self_intro, background, game_version, save_version, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (user_id) DO UPDATE SET rks = EXCLUDED.rks, challenge_mode_rank = EXCLUDED.challenge_mode_rank, counters = EXCLUDED.counters, self_intro = EXCLUDED.self_intro, background = EXCLUDED.background, game_version = EXCLUDED.game_version, save_version = EXCLUDED.save_version, updated_at = EXCLUDED.updated_at",
        )
        .bind(&profile.user_id)
        .bind(profile.rks)
        .bind(i64::from(profile.challenge_mode_rank))
        .bind(encode_counters(&profile.counters))
        .bind(&profile.self_intro)
        .bind(&profile.background)
        .bind(i64::from(profile.game_version))
        .bind(i64::from(profile.save_version))
        .bind(profile.updated_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn records(&self, user_id: &str) -> RepoResult<Vec<PlayRecord>> {
        let rows = sqlx::query_as::<_, (String, i64, i64, i64, f64)>(
            "SELECT song_id, difficulty, score, fc, acc FROM phg_record WHERE user_id = $1 ORDER BY song_id, difficulty",
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Difficulty::from_index(row.1 as usize).map(|difficulty| PlayRecord {
                    song_id: row.0,
                    difficulty,
                    score: row.2 as i32,
                    fc: row.3 != 0,
                    acc: row.4,
                })
            })
            .collect())
    }

    pub async fn replace_records(&self, user_id: &str, records: &[PlayRecord]) -> RepoResult<()> {
        let mut tx = self.pool().begin().await?;
        sqlx::query("DELETE FROM phg_record WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        for record in records {
            sqlx::query(
                "INSERT INTO phg_record (user_id, song_id, difficulty, score, fc, acc) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (user_id, song_id, difficulty) DO UPDATE SET score = EXCLUDED.score, fc = EXCLUDED.fc, acc = EXCLUDED.acc",
            )
            .bind(user_id)
            .bind(&record.song_id)
            .bind(record.difficulty.index() as i64)
            .bind(i64::from(record.score))
            .bind(i64::from(u8::from(record.fc)))
            .bind(record.acc)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn count_bindings(&self) -> RepoResult<i64> {
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM phg_binding")
            .fetch_one(self.pool())
            .await?;
        Ok(row.0)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CountersJson {
    #[serde(default)]
    cleared: Vec<i32>,
    #[serde(default, rename = "fullCombo")]
    full_combo: Vec<i32>,
    #[serde(default)]
    phi: Vec<i32>,
    #[serde(default)]
    money: Vec<i32>,
}

fn encode_counters(counters: &Counters) -> String {
    let json = CountersJson {
        cleared: counters.cleared.to_vec(),
        full_combo: counters.full_combo.to_vec(),
        phi: counters.phi.to_vec(),
        money: counters.money.to_vec(),
    };
    serde_json::to_string(&json).unwrap_or_default()
}

fn decode_counters(raw: &str) -> Counters {
    let mut counters = Counters::default();
    if raw.trim().is_empty() {
        return counters;
    }
    let Ok(json) = serde_json::from_str::<CountersJson>(raw) else {
        return counters;
    };
    for index in 0..crate::model::NUM_PLAYABLE {
        counters.cleared[index] = json.cleared.get(index).copied().unwrap_or(0);
        counters.full_combo[index] = json.full_combo.get(index).copied().unwrap_or(0);
        counters.phi[index] = json.phi.get(index).copied().unwrap_or(0);
        counters.money[index] = json.money.get(index).copied().unwrap_or(0);
    }
    counters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_round_trip_keeps_column_names() {
        let mut counters = Counters::default();
        counters.cleared[0] = 7;
        counters.full_combo[1] = 3;
        counters.phi[2] = 1;
        counters.money[3] = 99;
        let raw = encode_counters(&counters);
        assert!(raw.contains("\"fullCombo\""));
        assert_eq!(decode_counters(&raw), counters);
        assert_eq!(decode_counters(""), Counters::default());
    }

    #[test]
    fn counters_tolerate_short_arrays() {
        let decoded = decode_counters(r#"{"cleared":[1,2],"fullCombo":[3],"phi":[],"money":[4,5,6,7]}"#);
        assert_eq!(decoded.cleared[1], 2);
        assert_eq!(decoded.cleared[2], 0);
        assert_eq!(decoded.full_combo[1], 0);
        assert_eq!(decoded.money[3], 7);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AliasEntry {
    pub song_id: String,
    pub alias: String,
    pub removed: bool,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockEntry {
    pub user_id: String,
    pub nickname: String,
    pub object_id: String,
    pub reason: String,
    pub operator: String,
    pub created_at: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RksPoint {
    pub observed_at: i64,
    pub rks: f64,
    pub computed: f64,
    pub records: i64,
    pub phi: i64,
    pub challenge: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RksDaily {
    pub day: i64,
    pub observed_at: i64,
    pub rks: f64,
    pub records: i64,
    pub phi: i64,
    pub challenge: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScoreEvent {
    pub observed_at: i64,
    pub song_id: String,
    pub difficulty: Difficulty,
    pub has_old: bool,
    pub old_score: i32,
    pub old_acc: f64,
    pub old_fc: bool,
    pub new_score: i32,
    pub new_acc: f64,
    pub new_fc: bool,
    pub old_rks: f64,
    pub new_rks: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub owner: String,
    pub observed_at: i64,
    pub account: String,
    pub session: String,
    pub nickname: String,
    pub rks: f64,
    pub challenge: i64,
    pub phi: i64,
    pub payload: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryJob {
    pub user_id: String,
    pub pending: bool,
    pub attempts: i64,
    pub next_at: i64,
    pub synced_at: i64,
    pub error: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LeaderRow {
    pub user_id: String,
    pub nickname: String,
    pub avatar: String,
    pub rks: f64,
    pub challenge: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LeaderBoard {
    pub total: i64,
    pub me_place: i64,
    pub me: Option<LeaderRow>,
    pub top: Vec<LeaderRow>,
}

impl Repo {
    pub async fn alias_entries(&self, song_id: &str) -> RepoResult<Vec<AliasEntry>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT song_id, alias, removed, updated_at FROM phg_alias WHERE song_id = $1 ORDER BY alias",
        )
        .bind(song_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| AliasEntry {
                song_id: row.0,
                alias: row.1,
                removed: row.2 != 0,
                updated_at: row.3,
            })
            .collect())
    }

    pub async fn all_aliases(&self) -> RepoResult<Vec<AliasEntry>> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64)>(
            "SELECT song_id, alias, removed, updated_at FROM phg_alias ORDER BY song_id, alias",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| AliasEntry {
                song_id: row.0,
                alias: row.1,
                removed: row.2 != 0,
                updated_at: row.3,
            })
            .collect())
    }

    pub async fn upsert_alias(&self, song_id: &str, alias: &str, now: i64) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_alias (song_id, alias, removed, updated_at) VALUES ($1,$2,0,$3) ON CONFLICT (song_id, alias) DO UPDATE SET removed = 0, updated_at = EXCLUDED.updated_at",
        )
        .bind(song_id)
        .bind(alias)
        .bind(now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn delete_alias(&self, song_id: &str, alias: &str, now: i64) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_alias (song_id, alias, removed, updated_at) VALUES ($1,$2,1,$3) ON CONFLICT (song_id, alias) DO UPDATE SET removed = 1, updated_at = EXCLUDED.updated_at",
        )
        .bind(song_id)
        .bind(alias)
        .bind(now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn drop_alias(&self, song_id: &str, alias: &str) -> RepoResult<()> {
        sqlx::query("DELETE FROM phg_alias WHERE song_id = $1 AND alias = $2")
            .bind(song_id)
            .bind(alias)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn reset_aliases(&self) -> RepoResult<u64> {
        let affected = sqlx::query("DELETE FROM phg_alias")
            .execute(self.pool())
            .await?;
        Ok(affected.rows_affected())
    }

    pub async fn blocks(&self) -> RepoResult<Vec<BlockEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, i64)>(
            "SELECT user_id, nickname, object_id, reason, operator, created_at FROM phg_block ORDER BY created_at",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| BlockEntry {
                user_id: row.0,
                nickname: row.1,
                object_id: row.2,
                reason: row.3,
                operator: row.4,
                created_at: row.5,
            })
            .collect())
    }

    pub async fn is_blocked(&self, user_id: &str) -> RepoResult<bool> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT user_id FROM phg_block WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await?;
        Ok(row.is_some())
    }

    pub async fn put_block(&self, entry: &BlockEntry) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_block (user_id, nickname, object_id, reason, operator, created_at) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (user_id) DO UPDATE SET nickname = EXCLUDED.nickname, object_id = EXCLUDED.object_id, reason = EXCLUDED.reason, operator = EXCLUDED.operator",
        )
        .bind(&entry.user_id)
        .bind(&entry.nickname)
        .bind(&entry.object_id)
        .bind(&entry.reason)
        .bind(&entry.operator)
        .bind(entry.created_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn remove_block(&self, user_id: &str) -> RepoResult<bool> {
        let affected = sqlx::query("DELETE FROM phg_block WHERE user_id = $1")
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(affected.rows_affected() > 0)
    }

    pub async fn history_job(&self, user_id: &str) -> RepoResult<Option<HistoryJob>> {
        let row = sqlx::query_as::<_, (String, i64, i64, i64, i64, String)>(
            "SELECT user_id, pending, attempts, next_at, synced_at, error FROM phg_history_job WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;
        Ok(row.map(|row| HistoryJob {
            user_id: row.0,
            pending: row.1 != 0,
            attempts: row.2,
            next_at: row.3,
            synced_at: row.4,
            error: row.5,
        }))
    }

    pub async fn put_history_job(&self, job: &HistoryJob) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_history_job (user_id, pending, attempts, next_at, synced_at, error) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (user_id) DO UPDATE SET pending = EXCLUDED.pending, attempts = EXCLUDED.attempts, next_at = EXCLUDED.next_at, synced_at = EXCLUDED.synced_at, error = EXCLUDED.error",
        )
        .bind(&job.user_id)
        .bind(i64::from(u8::from(job.pending)))
        .bind(job.attempts)
        .bind(job.next_at)
        .bind(job.synced_at)
        .bind(&job.error)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn due_history_jobs(&self, now: i64, limit: i64) -> RepoResult<Vec<HistoryJob>> {
        let rows = sqlx::query_as::<_, (String, i64, i64, i64, i64, String)>(
            "SELECT user_id, pending, attempts, next_at, synced_at, error FROM phg_history_job WHERE pending = 1 AND next_at <= $1 ORDER BY next_at LIMIT $2",
        )
        .bind(now)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| HistoryJob {
                user_id: row.0,
                pending: row.1 != 0,
                attempts: row.2,
                next_at: row.3,
                synced_at: row.4,
                error: row.5,
            })
            .collect())
    }

    pub async fn adopt_snapshots(&self, user_id: &str, account: &str) -> RepoResult<u64> {
        let affected = sqlx::query(
            "INSERT INTO phg_rks_point (user_id, observed_at, rks, computed, records, phi, challenge) SELECT $1, observed_at, rks, rks, 0, phi, challenge FROM phg_snapshot WHERE owner = $1 OR (account = $2 AND $2 <> '') ON CONFLICT (user_id, observed_at) DO NOTHING",
        )
        .bind(user_id)
        .bind(account)
        .execute(self.pool())
        .await?;
        Ok(affected.rows_affected())
    }

    pub async fn import_history(
        &self,
        user_id: &str,
        points: &[RksPoint],
        events: &[ScoreEvent],
    ) -> RepoResult<()> {
        if points.is_empty() && events.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool().begin().await?;
        for point in points {
            sqlx::query(
                "INSERT INTO phg_rks_point (user_id, observed_at, rks, computed, records, phi, challenge) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (user_id, observed_at) DO NOTHING",
            )
            .bind(user_id)
            .bind(point.observed_at)
            .bind(point.rks)
            .bind(point.computed)
            .bind(point.records)
            .bind(point.phi)
            .bind(point.challenge)
            .execute(&mut *tx)
            .await?;
        }
        for event in events {
            sqlx::query(
                "INSERT INTO phg_score_event (user_id, observed_at, song_id, difficulty, has_old, old_score, old_acc, old_fc, new_score, new_acc, new_fc, old_rks, new_rks) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT (user_id, observed_at, song_id, difficulty) DO NOTHING",
            )
            .bind(user_id)
            .bind(event.observed_at)
            .bind(&event.song_id)
            .bind(event.difficulty.index() as i64)
            .bind(i64::from(u8::from(event.has_old)))
            .bind(i64::from(event.old_score))
            .bind(event.old_acc)
            .bind(i64::from(u8::from(event.old_fc)))
            .bind(i64::from(event.new_score))
            .bind(event.new_acc)
            .bind(i64::from(u8::from(event.new_fc)))
            .bind(event.old_rks)
            .bind(event.new_rks)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn put_rks_point(&self, user_id: &str, point: &RksPoint) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_rks_point (user_id, observed_at, rks, computed, records, phi, challenge) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (user_id, observed_at) DO UPDATE SET rks = EXCLUDED.rks, computed = EXCLUDED.computed, records = EXCLUDED.records, phi = EXCLUDED.phi, challenge = EXCLUDED.challenge",
        )
        .bind(user_id)
        .bind(point.observed_at)
        .bind(point.rks)
        .bind(point.computed)
        .bind(point.records)
        .bind(point.phi)
        .bind(point.challenge)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn rks_points(&self, user_id: &str, since: i64) -> RepoResult<Vec<RksPoint>> {
        let rows = sqlx::query_as::<_, (i64, f64, f64, i64, i64, i64)>(
            "SELECT observed_at, rks, computed, records, phi, challenge FROM phg_rks_point WHERE user_id = $1 AND observed_at >= $2 ORDER BY observed_at",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| RksPoint {
                observed_at: row.0,
                rks: row.1,
                computed: row.2,
                records: row.3,
                phi: row.4,
                challenge: row.5,
            })
            .collect())
    }

    pub async fn put_rks_daily(&self, user_id: &str, daily: &RksDaily) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_rks_daily (user_id, day, observed_at, rks, records, phi, challenge) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (user_id, day) DO UPDATE SET observed_at = EXCLUDED.observed_at, rks = EXCLUDED.rks, records = EXCLUDED.records, phi = EXCLUDED.phi, challenge = EXCLUDED.challenge",
        )
        .bind(user_id)
        .bind(daily.day)
        .bind(daily.observed_at)
        .bind(daily.rks)
        .bind(daily.records)
        .bind(daily.phi)
        .bind(daily.challenge)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn rks_daily(&self, user_id: &str, limit: i64) -> RepoResult<Vec<RksDaily>> {
        let rows = sqlx::query_as::<_, (i64, i64, f64, i64, i64, i64)>(
            "SELECT day, observed_at, rks, records, phi, challenge FROM phg_rks_daily WHERE user_id = $1 ORDER BY day DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        let mut result: Vec<RksDaily> = rows
            .into_iter()
            .map(|row| RksDaily {
                day: row.0,
                observed_at: row.1,
                rks: row.2,
                records: row.3,
                phi: row.4,
                challenge: row.5,
            })
            .collect();
        result.reverse();
        Ok(result)
    }

    pub async fn put_score_events(
        &self,
        user_id: &str,
        events: &[ScoreEvent],
    ) -> RepoResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool().begin().await?;
        for event in events {
            sqlx::query(
                "INSERT INTO phg_score_event (user_id, observed_at, song_id, difficulty, has_old, old_score, old_acc, old_fc, new_score, new_acc, new_fc, old_rks, new_rks) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) ON CONFLICT (user_id, observed_at, song_id, difficulty) DO UPDATE SET has_old = EXCLUDED.has_old, old_score = EXCLUDED.old_score, old_acc = EXCLUDED.old_acc, old_fc = EXCLUDED.old_fc, new_score = EXCLUDED.new_score, new_acc = EXCLUDED.new_acc, new_fc = EXCLUDED.new_fc, old_rks = EXCLUDED.old_rks, new_rks = EXCLUDED.new_rks",
            )
            .bind(user_id)
            .bind(event.observed_at)
            .bind(&event.song_id)
            .bind(event.difficulty.index() as i64)
            .bind(i64::from(u8::from(event.has_old)))
            .bind(i64::from(event.old_score))
            .bind(event.old_acc)
            .bind(i64::from(u8::from(event.old_fc)))
            .bind(i64::from(event.new_score))
            .bind(event.new_acc)
            .bind(i64::from(u8::from(event.new_fc)))
            .bind(event.old_rks)
            .bind(event.new_rks)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn score_events(
        &self,
        user_id: &str,
        since: i64,
    ) -> RepoResult<Vec<ScoreEvent>> {
        let rows = sqlx::query_as::<_, (i64, String, i64, i64, i64, f64, i64, i64, f64, i64, f64, f64)>(
            "SELECT observed_at, song_id, difficulty, has_old, old_score, old_acc, old_fc, new_score, new_acc, new_fc, old_rks, new_rks FROM phg_score_event WHERE user_id = $1 AND observed_at >= $2 ORDER BY observed_at, song_id, difficulty",
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Difficulty::from_index(row.2 as usize).map(|difficulty| ScoreEvent {
                    observed_at: row.0,
                    song_id: row.1,
                    difficulty,
                    has_old: row.3 != 0,
                    old_score: row.4 as i32,
                    old_acc: row.5,
                    old_fc: row.6 != 0,
                    new_score: row.7 as i32,
                    new_acc: row.8,
                    new_fc: row.9 != 0,
                    old_rks: row.10,
                    new_rks: row.11,
                })
            })
            .collect())
    }

    pub async fn put_snapshot(&self, snapshot: &Snapshot) -> RepoResult<()> {
        sqlx::query(
            "INSERT INTO phg_snapshot (owner, observed_at, account, session, nickname, rks, challenge, phi, payload) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (owner, observed_at) DO NOTHING",
        )
        .bind(&snapshot.owner)
        .bind(snapshot.observed_at)
        .bind(&snapshot.account)
        .bind(&snapshot.session)
        .bind(&snapshot.nickname)
        .bind(snapshot.rks)
        .bind(snapshot.challenge)
        .bind(snapshot.phi)
        .bind(&snapshot.payload)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn snapshot_payload(
        &self,
        owner: &str,
    ) -> RepoResult<Option<(i64, String)>> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT observed_at, payload FROM phg_snapshot WHERE owner = $1 ORDER BY observed_at DESC LIMIT 1",
        )
        .bind(owner)
        .fetch_optional(self.pool())
        .await?;
        Ok(row)
    }

    pub async fn leader_board(&self, user_id: &str, limit: i64) -> RepoResult<LeaderBoard> {
        let total: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM phg_profile p WHERE NOT EXISTS (SELECT 1 FROM phg_block b WHERE b.user_id = p.user_id)",
        )
        .fetch_one(self.pool())
        .await?;
        let top_rows = sqlx::query_as::<_, (String, String, String, f64, i64)>(
            "SELECT p.user_id, b.nickname, b.avatar, p.rks, p.challenge_mode_rank FROM phg_profile p JOIN phg_binding b ON b.user_id = p.user_id WHERE NOT EXISTS (SELECT 1 FROM phg_block z WHERE z.user_id = p.user_id) ORDER BY p.rks DESC, p.user_id ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(self.pool())
        .await?;
        let me_row = sqlx::query_as::<_, (String, String, String, f64, i64)>(
            "SELECT p.user_id, b.nickname, b.avatar, p.rks, p.challenge_mode_rank FROM phg_profile p JOIN phg_binding b ON b.user_id = p.user_id WHERE p.user_id = $1 AND NOT EXISTS (SELECT 1 FROM phg_block z WHERE z.user_id = p.user_id)",
        )
        .bind(user_id)
        .fetch_optional(self.pool())
        .await?;
        let me_place = match me_row.as_ref() {
            Some(row) => {
                let ahead: (i64,) = sqlx::query_as(
                    "SELECT count(*) FROM phg_profile p WHERE p.rks > $1 AND NOT EXISTS (SELECT 1 FROM phg_block z WHERE z.user_id = p.user_id)",
                )
                .bind(row.3)
                .fetch_one(self.pool())
                .await?;
                ahead.0 + 1
            }
            None => 0,
        };
        Ok(LeaderBoard {
            total: total.0,
            me_place,
            me: me_row.map(leader_row),
            top: top_rows.into_iter().map(leader_row).collect(),
        })
    }
}

fn leader_row(row: (String, String, String, f64, i64)) -> LeaderRow {
    LeaderRow {
        user_id: row.0,
        nickname: row.1,
        avatar: row.2,
        rks: row.3,
        challenge: row.4,
    }
}

pub const SYNC_OK: i64 = 1;
pub const SYNC_FAILED: i64 = 2;

const SONG_BOARD_TOTAL: &str = "SELECT count(*) FROM phg_record r WHERE r.song_id = $1 AND r.difficulty = $2 AND NOT (r.score >= 1000000 AND r.acc < 99.999999) AND NOT EXISTS (SELECT 1 FROM phg_block z WHERE z.user_id = r.user_id)";

const SONG_BOARD_ROWS: &str = "SELECT t.user_id, COALESCE(NULLIF(b.nickname, ''), '未命名玩家'), t.score, t.acc, COALESCE(p.rks, 0), COALESCE(p.challenge_mode_rank, 0), COALESCE(b.avatar, '') FROM (SELECT r.user_id, r.score, r.acc, ROW_NUMBER() OVER (ORDER BY r.score DESC, r.acc DESC, r.user_id ASC) AS place FROM phg_record r WHERE r.song_id = $1 AND r.difficulty = $2 AND NOT (r.score >= 1000000 AND r.acc < 99.999999) AND NOT EXISTS (SELECT 1 FROM phg_block z WHERE z.user_id = r.user_id) ORDER BY r.score DESC, r.acc DESC, r.user_id ASC LIMIT $3) t LEFT JOIN phg_binding b ON b.user_id = t.user_id LEFT JOIN phg_profile p ON p.user_id = t.user_id ORDER BY t.place, t.user_id ASC";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepoStats {
    pub users: i64,
    pub records: i64,
    pub synced: i64,
    pub failed: i64,
    pub schema: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongBoardRow {
    pub user_id: String,
    pub nickname: String,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
    pub challenge_rank: i64,
    pub avatar: String,
}

impl Repo {
    pub async fn stats(&self) -> RepoResult<RepoStats> {
        let row: (i64, i64, i64, i64, String) = sqlx::query_as(
            "SELECT (SELECT count(*) FROM phg_binding), (SELECT count(*) FROM phg_record), (SELECT count(*) FROM phg_binding WHERE status = $1), (SELECT count(*) FROM phg_binding WHERE status = $2), COALESCE((SELECT value FROM phg_meta WHERE key = 'schema_version'), '0')",
        )
        .bind(SYNC_OK)
        .bind(SYNC_FAILED)
        .fetch_one(self.pool())
        .await?;
        Ok(RepoStats {
            users: row.0,
            records: row.1,
            synced: row.2,
            failed: row.3,
            schema: row.4.parse::<i64>().unwrap_or_default(),
        })
    }

    pub async fn song_board(
        &self,
        song_id: &str,
        difficulty: Difficulty,
        limit: usize,
    ) -> RepoResult<(Vec<SongBoardRow>, i64)> {
        if song_id.is_empty() || limit == 0 {
            return Ok((Vec::new(), 0));
        }
        let total: (i64,) = sqlx::query_as(SONG_BOARD_TOTAL)
            .bind(song_id)
            .bind(difficulty.index() as i64)
            .fetch_one(self.pool())
            .await?;
        if total.0 == 0 {
            return Ok((Vec::new(), 0));
        }
        let rows = sqlx::query_as::<_, (String, String, i64, f64, f64, i64, String)>(SONG_BOARD_ROWS)
            .bind(song_id)
            .bind(difficulty.index() as i64)
            .bind(limit as i64)
            .fetch_all(self.pool())
            .await?;
        let mut out: Vec<SongBoardRow> = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(SongBoardRow {
                user_id: row.0,
                nickname: row.1,
                score: row.2 as i32,
                acc: row.3,
                rks: row.4,
                challenge_rank: row.5,
                avatar: row.6,
            });
        }
        Ok((out, total.0))
    }
}

pub const SERVER_STATS_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
struct ServerCache {
    at: Option<Instant>,
    stats: ServerStats,
}

static SERVER_STATS: OnceLock<tokio::sync::Mutex<ServerCache>> = OnceLock::new();

fn server_cache() -> &'static tokio::sync::Mutex<ServerCache> {
    SERVER_STATS.get_or_init(|| tokio::sync::Mutex::new(ServerCache::default()))
}

impl Repo {
    pub async fn server_stats(
        &self,
        constants: impl Fn(&str, Difficulty) -> Option<f64>,
    ) -> RepoResult<ServerStats> {
        let mut cache = server_cache().lock().await;
        if cache.at.is_some_and(|at| at.elapsed() < SERVER_STATS_TTL) {
            return Ok(cache.stats.clone());
        }
        let levels = self.level_stats(&constants).await?;
        let bands = self.band_stats().await?;
        let charts = self.chart_stats().await?;
        cache.stats = ServerStats {
            levels,
            bands,
            charts,
        };
        cache.at = Some(Instant::now());
        Ok(cache.stats.clone())
    }

    async fn level_stats(
        &self,
        constants: &impl Fn(&str, Difficulty) -> Option<f64>,
    ) -> RepoResult<Vec<LevelStat>> {
        let rows = sqlx::query_as::<_, (String, i64, i64, f64)>(
            "SELECT song_id, difficulty, count(*), avg(acc) FROM phg_record GROUP BY song_id, difficulty",
        )
        .fetch_all(self.pool())
        .await?;
        Ok(bucket_levels(rows, constants))
    }

    async fn band_stats(&self) -> RepoResult<Vec<BandStat>> {
        let rows = sqlx::query_as::<_, (i64, i64)>(
            "SELECT LEAST(floor(rks), $1)::bigint, count(*) FROM phg_profile WHERE rks > 0 GROUP BY 1 ORDER BY 1",
        )
        .bind(i64::from(BAND_MAX))
        .fetch_all(self.pool())
        .await?;
        Ok(rows
            .into_iter()
            .filter(|(_, users)| *users >= i64::from(STAT_MIN_USERS))
            .map(|(band, users)| BandStat {
                band: band as i32,
                users: users as i32,
            })
            .collect())
    }

    async fn chart_stats(&self) -> RepoResult<Vec<ChartStat>> {
        let sql = chart_stat_sql();
        let mut query =
            sqlx::query_as::<_, (String, i64, i64, i64, f64, f64, i64, i64, i64, i64, i64, i64, i64)>(
                &sql,
            )
            .bind(i64::from(BAND_MAX));
        for threshold in PEER_THRESHOLDS {
            query = query.bind(threshold);
        }
        let rows = query.fetch_all(self.pool()).await?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                let (
                    song_id,
                    difficulty,
                    band,
                    users,
                    median,
                    p75,
                    hit100,
                    hit995,
                    hit99,
                    hit985,
                    hit98,
                    hit97,
                    hit96,
                ) = row;
                Difficulty::from_index(difficulty as usize).map(|difficulty| ChartStat {
                    song_id,
                    difficulty,
                    band: band as i32,
                    users: users as i32,
                    median,
                    p75,
                    hits: [
                        hit100 as i32,
                        hit995 as i32,
                        hit99 as i32,
                        hit985 as i32,
                        hit98 as i32,
                        hit97 as i32,
                        hit96 as i32,
                    ],
                })
            })
            .collect())
    }
}

fn chart_stat_sql() -> String {
    let mut sql = String::from(
        "SELECT r.song_id, r.difficulty, LEAST(floor(p.rks), $1)::bigint, count(*), percentile_cont(0.5) WITHIN GROUP (ORDER BY r.acc), percentile_cont(0.75) WITHIN GROUP (ORDER BY r.acc)",
    );
    for index in 0..PEER_THRESHOLD_COUNT {
        sql.push_str(&format!(", count(*) FILTER (WHERE r.acc >= ${})", index + 2));
    }
    sql.push_str(
        " FROM phg_record r JOIN phg_profile p ON p.user_id = r.user_id WHERE p.rks > 0 GROUP BY 1, 2, 3",
    );
    sql
}

fn bucket_levels(
    rows: Vec<(String, i64, i64, f64)>,
    constants: &impl Fn(&str, Difficulty) -> Option<f64>,
) -> Vec<LevelStat> {
    let mut buckets: BTreeMap<i32, (i64, f64)> = BTreeMap::new();
    for (song_id, difficulty, records, acc) in rows {
        let Some(difficulty) = Difficulty::from_index(difficulty as usize) else {
            continue;
        };
        let Some(constant) = constants(&song_id, difficulty) else {
            continue;
        };
        if constant <= 0.0 {
            continue;
        }
        let slot = buckets.entry((constant + 0.5).floor() as i32).or_default();
        slot.0 += records;
        slot.1 += acc * records as f64;
    }
    buckets
        .into_iter()
        .filter(|(_, (records, _))| *records >= i64::from(STAT_MIN_USERS))
        .map(|(constant, (records, sum))| LevelStat {
            constant: f64::from(constant),
            records,
            acc: sum / records as f64,
        })
        .collect()
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn song_board_sql_filters_blocks_and_suspects() {
        assert!(SONG_BOARD_ROWS.contains("phg_block"));
        assert!(SONG_BOARD_ROWS.contains("ROW_NUMBER"));
        assert!(SONG_BOARD_ROWS.contains("phg_profile"));
        assert!(SONG_BOARD_ROWS.contains("phg_binding"));
        assert!(SONG_BOARD_TOTAL.contains("NOT (r.score >= 1000000"));
        assert!(SONG_BOARD_ROWS.contains("NOT (r.score >= 1000000"));
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;

    fn constant(song_id: &str, difficulty: Difficulty) -> Option<f64> {
        if song_id == "known" && difficulty == Difficulty::In {
            Some(14.6)
        } else {
            None
        }
    }

    #[test]
    fn level_buckets_round_constants_and_drop_thin_buckets() {
        let rows = vec![
            (String::from("known"), Difficulty::In.index() as i64, 20, 98.0),
            (String::from("known"), Difficulty::At.index() as i64, 10, 96.0),
            (String::from("known"), Difficulty::In.index() as i64, 15, 97.0),
        ];
        let out = bucket_levels(rows, &constant);
        assert_eq!(out.len(), 1);
        assert!((out[0].constant - 15.0).abs() < 1e-9);
        assert_eq!(out[0].records, 35);
        assert!((out[0].acc - (98.0 * 20.0 + 97.0 * 15.0) / 35.0).abs() < 1e-9);
    }

    #[test]
    fn chart_stat_sql_binds_every_peer_threshold() {
        let sql = chart_stat_sql();
        assert_eq!(
            sql.matches("FILTER (WHERE r.acc >= $").count(),
            PEER_THRESHOLD_COUNT
        );
        assert!(sql.contains("percentile_cont(0.5)"));
        assert!(sql.contains("percentile_cont(0.75)"));
        assert!(sql.contains("GROUP BY 1, 2, 3"));
        assert!(!sql.to_lowercase().contains("select *"));
    }
}
