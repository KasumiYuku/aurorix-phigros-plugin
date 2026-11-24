use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use aurorix_db::PluginDb;
use aurorix_plugin::{Attachment, Outgoing, PluginAssets, RenderCard, ReplyKind};
use aurorix_render::{CanvasKit, RenderContext};
use aurorix_template::{Arg, Args};

use crate::backend::client::{FetchedSave, SaveClient};
use crate::backend::phi;
use crate::backend::savefile::{self, SongLimits};
use crate::backend::taptap::{DeviceFlow, DeviceLogin};
use crate::cards::player::Player;
use crate::domain::catalog::Catalog;
use crate::domain::{best, query, rks};
use crate::model::{
    grade_of, Binding, Counters, Difficulty, PlayRecord, Profile, Progress, Scored, Server, Summary,
};
use crate::repo::{LeaderBoard, LeaderRow, Repo, RksDaily, RksPoint, ScoreEvent, Snapshot};
use crate::settings::{Settings, SettingsStore};

pub const COMMAND_PREFIX: &str = "p";

pub const SCORE_PREFIX: &str = "p score ";

pub const SONG_PREFIX: &str = "p song ";

pub const CATALOG_FILE: &str = "../data/phi.db";

pub const SETTINGS_FILE: &str = "../data/phg_settings.json";

pub const CATALOG_NAME: &str = "phi.db";

pub struct SyncReport {
    pub binding: Binding,
    pub profile: Profile,
    pub scored: Vec<Scored>,
    pub events: Vec<ScoreEvent>,
    pub first_run: bool,
    pub challenge: i32,
}

pub struct PhigrosService {
    catalog: Arc<Catalog>,
    assets: Arc<PluginAssets>,
    context: Arc<RenderContext>,
    client: Arc<SaveClient>,
    flow: DeviceFlow,
    db: Arc<PluginDb>,
    settings: Arc<SettingsStore>,
    logins: Mutex<HashMap<String, DeviceLogin>>,
    probes: ProbeSchedule,
    phi: Arc<phi::Source>,
    history: Arc<crate::history::Worker>,
}

impl PhigrosService {
    pub fn new(
        catalog: Arc<Catalog>,
        assets: Arc<PluginAssets>,
        context: Arc<RenderContext>,
        client: Arc<SaveClient>,
        db: Arc<PluginDb>,
    ) -> Arc<Self> {
        let constants: phi::Constants = {
            let catalog = Arc::clone(&catalog);
            Box::new(move |song_id: &str, difficulty: Difficulty| {
                catalog.song(song_id).and_then(|song| {
                    let chart = &song.charts[difficulty.index()];
                    chart.known().then_some(chart.constant)
                })
            })
        };
        Arc::new(Self {
            settings: Arc::clone(client.settings_store()),
            catalog,
            assets,
            context,
            client,
            flow: DeviceFlow::new(),
            db,
            logins: Mutex::new(HashMap::new()),
            probes: ProbeSchedule::default(),
            phi: Arc::new(phi::Source::new(phi::DEFAULT_BASE, constants)),
            history: Arc::new(crate::history::Worker::default()),
        })
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn root(&self) -> &Path {
        self.assets.root()
    }

    pub fn assets(&self) -> &Arc<PluginAssets> {
        &self.assets
    }

    pub fn context(&self) -> &RenderContext {
        self.apply_runtime_settings();
        &self.context
    }

    fn apply_runtime_settings(&self) {
        let settings = self.settings();
        aurorix_render::apply_settings(aurorix_render::RenderSettings {
            webp_enabled: settings.webp_enabled,
            render_budget_mb: settings.render_budget_mb.max(0) as u64,
            render_concurrency: settings.render_concurrency.max(0) as usize,
            render_queue: settings.render_queue.max(0) as usize,
        });
        aurorix_assets::set_url_cache_policy(aurorix_assets::UrlCachePolicy::new(
            settings.url_cache_seconds.max(0) as u64,
            settings.url_cache_verify,
            settings.url_cache_renewals.max(0) as u32,
            settings.url_cache_probe_percent.max(0) as u8,
        ));
    }

    pub fn client(&self) -> &SaveClient {
        &self.client
    }

    pub fn flow(&self) -> &DeviceFlow {
        &self.flow
    }

    pub fn settings(&self) -> Arc<Settings> {
        self.settings.get()
    }

    pub fn settings_store(&self) -> &Arc<SettingsStore> {
        &self.settings
    }

    pub fn phi(&self) -> &phi::Source {
        self.phi.set_online_rank(self.settings().online_rank);
        &self.phi
    }

    pub fn history_worker(&self) -> &crate::history::Worker {
        &self.history
    }

    pub fn repo(&self) -> Repo {
        Repo::new(self.db.clone())
    }

    pub fn put_login(&self, user: &str, login: DeviceLogin) {
        if let Ok(mut guard) = self.logins.lock() {
            guard.insert(user.to_string(), login);
        }
    }

    pub fn take_login(&self, user: &str) -> Option<DeviceLogin> {
        self.logins
            .lock()
            .ok()
            .and_then(|mut guard| guard.remove(user))
    }

    pub fn find_songs(&self, text: &str, limit: usize) -> Vec<(String, String, String)> {
        self.catalog
            .search(text, limit)
            .into_iter()
            .map(|(song, _)| {
                (
                    song.id.clone(),
                    song.title.clone(),
                    song.composer.clone(),
                )
            })
            .collect()
    }

    pub async fn binding(&self, user_id: &str) -> Result<Option<Binding>, String> {
        self.repo()
            .binding(user_id)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn profile(&self, user_id: &str) -> Result<Option<Profile>, String> {
        self.repo()
            .profile(user_id)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn records(&self, user_id: &str) -> Result<Vec<PlayRecord>, String> {
        self.ensure_fresh(user_id).await;
        self.records_local(user_id).await
    }

    async fn records_local(&self, user_id: &str) -> Result<Vec<PlayRecord>, String> {
        self.repo()
            .records(user_id)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn scored(&self, user_id: &str) -> Result<Vec<Scored>, String> {
        let records = self.records(user_id).await?;
        Ok(self.judge(&records))
    }

    pub async fn require_binding(&self, user_id: &str) -> Result<Binding, Box<Outgoing>> {
        match self.binding(user_id).await {
            Ok(Some(binding)) => Ok(binding),
            Ok(None) => Err(Box::new(Self::unbound())),
            Err(error) => Err(Box::new(Self::fail(error))),
        }
    }

    pub fn judge(&self, records: &[PlayRecord]) -> Vec<Scored> {
        let mut out: Vec<Scored> = Vec::with_capacity(records.len());
        for record in records {
            let Some(song) = self.catalog.song(&record.song_id) else {
                continue;
            };
            let chart = &song.charts[record.difficulty.index()];
            let known = chart.known();
            let constant = chart.constant;
            out.push(Scored {
                record: record.clone(),
                title: song.title.clone(),
                composer: song.composer.clone(),
                constant,
                known,
                notes: chart.notes.combo(),
                rks: if known {
                    rks::single_rks(record.acc, constant)
                } else {
                    0.0
                },
                grade: grade_of(record.score, record.fc),
                rank: 0,
                suggest: 0.0,
                suggest_ok: false,
                suspect: record.is_suspect(),
                variant: song.variant(),
                cover_luma: song.cover_luma,
            });
        }
        out
    }

    pub fn board(&self, scored: &[Scored], official: f64, size: usize) -> crate::model::Best {
        best::board(scored, official, size)
    }

    pub async fn online_board(
        &self,
        user: &str,
        binding: &Binding,
        profile: &Profile,
        limit: i64,
    ) -> Option<LeaderBoard> {
        let me = LeaderRow {
            user_id: user.to_string(),
            nickname: binding.nickname.clone(),
            avatar: binding.avatar_key(),
            rks: profile.rks,
            challenge: i64::from(profile.challenge_mode_rank),
        };
        self.phi().board(&me, limit.max(0) as usize).await.ok()
    }

    pub fn player(&self, binding: &Binding, profile: Option<&Profile>) -> Player {
        crate::cards::player::player_of(binding, profile)
    }

    pub async fn sync(&self, user_id: &str) -> Result<SyncReport, String> {
        let binding = self
            .binding(user_id)
            .await?
            .ok_or_else(|| String::from("未绑定 Phigros 账号"))?;
        if save_wait(&self.settings(), &binding, now_seconds()) > 0 {
            return self.current_report(&binding).await;
        }
        self.pull(user_id).await
    }

    async fn pull(&self, user_id: &str) -> Result<SyncReport, String> {
        let binding = self
            .binding(user_id)
            .await?
            .ok_or_else(|| String::from("未绑定 Phigros 账号"))?;
        let fetched = self
            .client
            .archive(binding.server, &binding.token)
            .await
            .map_err(|error| error.to_string())?;
        self.store(&binding, &fetched).await
    }

    async fn current_report(&self, binding: &Binding) -> Result<SyncReport, String> {
        let profile = self.profile(&binding.user_id).await?.unwrap_or_default();
        let records = self.records_local(&binding.user_id).await?;
        Ok(SyncReport {
            binding: binding.clone(),
            challenge: profile.challenge_mode_rank,
            profile,
            scored: self.judge(&records),
            events: Vec::new(),
            first_run: false,
        })
    }

    async fn ensure_fresh(&self, user_id: &str) {
        let settings = self.settings();
        if !settings.fresh_enabled || settings.fresh_budget_ms <= 0 {
            return;
        }
        let Ok(Some(binding)) = self.binding(user_id).await else {
            return;
        };
        if binding.token.is_empty() {
            return;
        }
        let now = now_seconds();
        if save_wait(&settings, &binding, now) > 0 || !self.probes.due(user_id, now) {
            return;
        }
        let budget = std::time::Duration::from_millis(settings.fresh_budget_ms as u64);
        let probe = self
            .client
            .probe(binding.server, &binding.token, &binding.object_id);
        let Ok(Ok(stamp)) = tokio::time::timeout(budget, probe).await else {
            self.probes.note(user_id, now, false);
            return;
        };
        if stamp.modified_at == binding.save_modified_at {
            self.probes.note(user_id, now, false);
            return;
        }
        self.probes.note(user_id, now, true);
        let _ = self.pull(user_id).await;
    }

    pub async fn store(
        &self,
        binding: &Binding,
        fetched: &FetchedSave,
    ) -> Result<SyncReport, String> {
        let limits = SongLimits::compute(self.catalog.game_version(), self.catalog.songs())
            .for_game_version(fetched.summary.game_version);
        savefile::validate(&fetched.archive, &fetched.summary, limits)
            .map_err(|error| format!("存档校验失败：{error}"))?;
        let archive = savefile::sanitize(&fetched.archive);
        let repo = self.repo();
        let previous = repo.records(&binding.user_id).await.map_err(|e| e.to_string())?;
        let fresh = match repo.binding(&binding.user_id).await {
            Ok(Some(old)) => !old.token.is_empty() && old.token != binding.token,
            _ => false,
        };
        let scored = self.judge(&archive.records);
        let official = fetched.summary.ranking_score;
        let computed = rks::rks(&scored);
        let card = self.board(&scored, official, scored.len().max(1));
        let now = now_seconds();
        let mut synced = binding.clone();
        synced.object_id = fetched.object_id.clone();
        synced.nickname = if fetched.player.nickname.is_empty() {
            binding.nickname.clone()
        } else {
            fetched.player.nickname.clone()
        };
        synced.avatar = fetched.player.avatar.clone();
        synced.game_avatar = fetched.summary.avatar.clone();
        synced.joined_at = fetched.player.joined_at;
        synced.save_modified_at = fetched.modified_at;
        synced.last_sync_at = now;
        synced.status = 1;
        synced.error.clear();
        let profile = Profile {
            user_id: binding.user_id.clone(),
            rks: official,
            challenge_mode_rank: fetched.summary.challenge_mode_rank,
            self_intro: archive.user.self_intro.clone(),
            background: archive.user.background.clone(),
            game_version: fetched.summary.game_version,
            save_version: fetched.summary.save_version,
            updated_at: now,
            counters: counters_of(&fetched.summary, &archive.progress),
        };
        let events = diff(&self.catalog, &previous, &archive.records, now);
        let point = RksPoint {
            observed_at: now,
            rks: official,
            computed,
            records: card.records as i64,
            phi: card.phi.len() as i64,
            challenge: i64::from(fetched.summary.challenge_mode_rank),
        };
        let daily = RksDaily {
            day: now / 86_400,
            observed_at: now,
            rks: official,
            records: point.records,
            phi: point.phi,
            challenge: point.challenge,
        };
        let snapshot = Snapshot {
            owner: binding.user_id.clone(),
            observed_at: now,
            account: fetched.object_id.clone(),
            session: String::new(),
            nickname: synced.nickname.clone(),
            rks: official,
            challenge: point.challenge,
            phi: point.phi,
            payload: payload_of(&scored, computed),
        };
        repo.put_binding(&synced).await.map_err(|e| e.to_string())?;
        repo.put_profile(&profile).await.map_err(|e| e.to_string())?;
        repo.replace_records(&binding.user_id, &archive.records)
            .await
            .map_err(|e| e.to_string())?;
        repo.put_score_events(&binding.user_id, &events)
            .await
            .map_err(|e| e.to_string())?;
        repo.put_rks_point(&binding.user_id, &point)
            .await
            .map_err(|e| e.to_string())?;
        repo.put_rks_daily(&binding.user_id, &daily)
            .await
            .map_err(|e| e.to_string())?;
        repo.put_snapshot(&snapshot).await.map_err(|e| e.to_string())?;
        crate::history::schedule(&repo, &binding.user_id, fresh).await;
        self.history.poke();
        Ok(SyncReport {
            binding: synced,
            profile,
            scored,
            events,
            first_run: previous.is_empty(),
            challenge: fetched.summary.challenge_mode_rank,
        })
    }

    pub fn card<P>(
        &self,
        width: i32,
        height: i32,
        label: impl Into<String>,
        input: &impl Debug,
        paint: P,
    ) -> RenderCard
    where
        P: Fn(&CanvasKit<'_>) + Send + Sync + 'static,
    {
        let label = label.into();
        let request = crate::cards::card_request(width, height);
        RenderCard::new(request, label.clone(), move |kit| {
            crate::cards::with_device_scale(kit, width, height, || paint(kit));
        })
        .with_cache_key(fingerprint(&label, input))
    }

    pub async fn refresh_aliases(&self) -> Result<(), String> {
        let entries = self
            .repo()
            .all_aliases()
            .await
            .map_err(|error| error.to_string())?;
        self.catalog.set_aliases(&entries);
        Ok(())
    }

    pub fn fill(&self, id: &str, args: &Args) -> String {
        aurorix_template::fill_for(crate::PLUGIN_ID, id, args).unwrap_or_default()
    }

    pub fn text(&self, id: &str, args: &Args) -> Outgoing {
        let body = self.fill(id, args);
        if body.trim().is_empty() {
            return Outgoing::new(ReplyKind::Handler, id.to_string());
        }
        Outgoing::new(ReplyKind::Handler, body)
    }

    pub fn bare() -> Outgoing {
        Outgoing::new(ReplyKind::Handler, String::new())
    }

    pub fn fail(reason: impl Into<String>) -> Outgoing {
        let mut args = Args::new();
        put(&mut args, "reason", reason.into());
        let body = aurorix_template::fill_for(crate::PLUGIN_ID, "fail", &args)
            .unwrap_or_else(|_| String::from("出现了一些小问题"));
        Outgoing::new(ReplyKind::Handler, body)
    }

    pub fn notice(reason: impl Into<String>) -> Outgoing {
        let mut args = Args::new();
        put(&mut args, "reason", reason.into());
        let body = aurorix_template::fill_for(crate::PLUGIN_ID, "notice", &args)
            .unwrap_or_else(|_| String::from("请注意"));
        Outgoing::new(ReplyKind::Handler, body)
    }

    pub fn unbound() -> Outgoing {
        let args = Args::new();
        let body = aurorix_template::fill_for(crate::PLUGIN_ID, "unbound", &args)
            .unwrap_or_else(|_| String::from("请先绑定 Phigros 账号"));
        let out = Outgoing::new(ReplyKind::Handler, body);
        crate::buttons::with_keyboard(out, crate::buttons::Page::Unbound)
    }

    pub fn pick(&self, raw: &str, prefix: &str, candidates: &[&crate::model::Song], total: usize) -> Outgoing {
        let body = query::pick_markdown(raw, prefix, &self.catalog, candidates, total);
        Outgoing::new(ReplyKind::Handler, body)
    }

    pub fn attachment(&self, key: &str, fallback: &str) -> Option<Attachment> {
        let root = self.root();
        let mut key = String::from(key);
        if !root.join(&key).is_file() {
            key = String::from(fallback);
        }
        let path = root.join(&key);
        let data = std::fs::read(&path).ok()?;
        Some(Attachment::new(
            data,
            file_name(&key),
            mime_of(&key).to_string(),
        ))
    }
}

pub fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|span| span.as_secs() as i64)
        .unwrap_or_default()
}

pub fn put(args: &mut Args, key: &str, value: impl Into<String>) {
    args.insert(key.to_string(), Arg::Str(value.into()));
}

pub fn fingerprint(tag: &str, input: &impl Debug) -> u64 {
    let mut sink = Fnv(0xCBF2_9CE4_8422_2325);
    let _ = write!(sink, "{tag}|{input:?}");
    sink.0
}

struct Fnv(u64);

impl Write for Fnv {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        for byte in text.as_bytes() {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01B3);
        }
        Ok(())
    }
}

fn file_name(key: &str) -> String {
    key.rsplit('/').next().unwrap_or(key).to_string()
}

fn mime_of(key: &str) -> &'static str {
    if key.ends_with(".png") {
        return "image/png";
    }
    if key.ends_with(".jpg") || key.ends_with(".jpeg") {
        return "image/jpeg";
    }
    if key.ends_with(".mp3") {
        return "audio/mpeg";
    }
    "application/octet-stream"
}

fn counters_of(summary: &Summary, progress: &Progress) -> Counters {
    Counters {
        cleared: summary.cleared,
        full_combo: summary.full_combo,
        phi: summary.phi,
        money: progress.money,
    }
}

fn save_wait(settings: &Settings, binding: &Binding, now: i64) -> i64 {
    let interval = settings.save_min_interval_seconds;
    if interval <= 0 || binding.last_sync_at <= 0 {
        return 0;
    }
    (binding.last_sync_at + interval - now).max(0)
}

const PROBE_BASE_SECONDS: i64 = 10;

const PROBE_MAX_SECONDS: i64 = 300;

#[derive(Default)]
struct ProbeSchedule {
    entries: Mutex<HashMap<String, (i64, i64)>>,
}

impl ProbeSchedule {
    fn due(&self, user_id: &str, now: i64) -> bool {
        let guard = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match guard.get(user_id) {
            Some((checked, interval)) => now >= checked + interval,
            None => true,
        }
    }

    fn note(&self, user_id: &str, now: i64, changed: bool) {
        let mut guard = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let next = match guard.get(user_id) {
            Some((_, interval)) if !changed => (interval * 2).min(PROBE_MAX_SECONDS),
            _ => PROBE_BASE_SECONDS,
        };
        guard.insert(user_id.to_string(), (now, next));
    }
}

fn payload_of(scored: &[Scored], computed: f64) -> String {
    let records = scored
        .iter()
        .filter(|item| item.counted())
        .count();
    format!("{{\"records\":{records},\"computed\":{computed:.4}}}")
}

fn diff(
    catalog: &Catalog,
    previous: &[PlayRecord],
    current: &[PlayRecord],
    observed_at: i64,
) -> Vec<ScoreEvent> {
    let mut old = HashMap::with_capacity(previous.len());
    for record in previous {
        old.insert((record.song_id.clone(), record.difficulty), record.clone());
    }
    let mut events: Vec<ScoreEvent> = Vec::new();
    for record in current {
        let before = old.get(&(record.song_id.clone(), record.difficulty));
        if let Some(before) = before {
            if before.score == record.score && (before.acc - record.acc).abs() < 0.0001 && before.fc == record.fc {
                continue;
            }
        }
        let constant = catalog
            .song(&record.song_id)
            .map(|song| song.charts[record.difficulty.index()].constant)
            .unwrap_or_default();
        events.push(ScoreEvent {
            observed_at,
            song_id: record.song_id.clone(),
            difficulty: record.difficulty,
            has_old: before.is_some(),
            old_score: before.map(|item| item.score).unwrap_or_default(),
            old_acc: before.map(|item| item.acc).unwrap_or_default(),
            old_fc: before.map(|item| item.fc).unwrap_or_default(),
            new_score: record.score,
            new_acc: record.acc,
            new_fc: record.fc,
            old_rks: before
                .map(|item| rks::single_rks(item.acc, constant))
                .unwrap_or_default(),
            new_rks: rks::single_rks(record.acc, constant),
        });
    }
    events
}

pub fn catalog_path(root: &Path) -> PathBuf {
    let settings = SettingsStore::load(Some(settings_path(root))).get();
    let file = settings.data_file.trim();
    if !file.is_empty() {
        return PathBuf::from(file);
    }
    let dir = settings.data_dir.trim();
    if !dir.is_empty() {
        return PathBuf::from(dir).join(CATALOG_NAME);
    }
    root.join(CATALOG_FILE)
}

pub fn server_label(server: Server) -> &'static str {
    match server {
        Server::Cn => "国服",
        Server::Global => "国际服",
    }
}

pub fn settings_path(root: &Path) -> PathBuf {
    root.join(SETTINGS_FILE)
}
