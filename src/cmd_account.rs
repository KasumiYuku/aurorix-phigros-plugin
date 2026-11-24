use std::sync::Arc;

use aurorix_plugin::{
    CommandSpec, Invocation, InvocationSender, Outgoing, PluginError, Scene,
};
use aurorix_template::Args;
use chrono::{DateTime, Local};

use crate::backend::taptap::{DeviceLogin, DeviceState, POLL_MAX, POLL_MIN};
use crate::backend::Kind;
use crate::cards::overview as overview_card;
use crate::cards::sync as sync_card;
use crate::domain::overview as overview_domain;
use crate::domain::rks;
use crate::model::{
    grade_of, Best, Binding, Difficulty, DifficultyStat, Profile, Scored, Server, COUNTED_BEST,
    NUM_PLAYABLE,
};
use crate::commands::{self, ok};
use crate::repo::{LeaderBoard, RksPoint, ScoreEvent};
use crate::service::{now_seconds, put, server_label, PhigrosService, SyncReport};

const BIND_USAGE: &str = "用法：/p bind <sessionToken>，也可写作 /p bind cn <token> 指定国服";

const LOGIN_USAGE: &str = "用法：/p login [国服|国际服]，默认国服";

const ALREADY_BOUND: &str = "已经绑定了，可以使用 Phigros 相关能力了";

const LOGIN_SCAN: &str = "请在 TapTap 里点确认登录";

const LOGIN_TIMEOUT: &str = "Taptap 授权等待超时，请重新发送 /p login";

const TOKEN_INVALID: &str = "sessionToken 无效或已过期";

const ACTIVITY_WINDOW: i64 = 7 * 86_400;

const SECONDS_PER_DAY: i64 = 86_400;

const CHANGE_PREVIEW: usize = 8;

const DAY_LIMIT: usize = 10;

const ITEM_LIMIT: usize = 50;

const PER_DAY: usize = 10;

const BOARD_LIMIT: i64 = 10;

const MILESTONE_LIMIT: usize = 8;

const LOCAL_SCOPE: &str = "全服排名";

const ADOPT_LABEL: &str = "本次接入";

const SYNC_LABEL: &str = "本次同步";

const SYNC_TITLE: &str = "更新存档";

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("bind")
            .alias("绑定")
            .describe("绑定 Phigros 云端凭据")
            .handler(commands::on(service, run_bind)),
        CommandSpec::new("login")
            .alias("Taptap")
            .alias("登录")
            .describe("Taptap登录并绑定")
            .handler(commands::on(service, run_login)),
        CommandSpec::new("unbind")
            .alias("解绑")
            .describe("解除绑定（记录留在该账号名下）")
            .handler(commands::on(service, run_unbind)),
        CommandSpec::new("clean")
            .alias("清档")
            .describe("彻底删除本账号全部数据（不可恢复）")
            .handler(commands::on(service, run_clean)),
        CommandSpec::new("pull")
            .alias("update")
            .alias("更新数据")
            .alias("更新")
            .alias("更新存档")
            .describe("拉取最新存档")
            .handler(commands::on(service, run_pull)),
        CommandSpec::new("me")
            .alias("我的")
            .describe("查看绑定状态与账号档案")
            .handler(commands::on(service, run_me)),

    ]
}
async fn run_bind(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if bound(&service, &inv.user).await {
        return ok(commands::with_at(PhigrosService::notice(ALREADY_BOUND), &inv));
    }
    let raw = inv.text.trim().to_string();
    if wants_login(&raw) {
        return run_login(service, inv).await;
    }
    let Some((token, server)) = parse_bind(&raw) else {
        return ok(commands::with_at(PhigrosService::notice(BIND_USAGE), &inv));
    };
    let out = match commit(&service, &inv.user, &token, server).await {
        Ok(report) => bind_reply(&service, &report).await,
        Err(reason) => PhigrosService::fail(reason),
    };
    ok(commands::with_at(out, &inv))
}

async fn run_login(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let direct = matches!(inv.scene, Scene::C2c | Scene::Dm);
    let out = login_flow(
        &service,
        &inv.user,
        parse_server_arg(&inv.text),
        direct,
        inv.opt_sender.clone(),
    )
    .await;
    ok(commands::with_at(out, &inv))
}

pub async fn login_click(
    service: &Arc<PhigrosService>,
    user: &str,
    scene: Scene,
    sender: Option<InvocationSender>,
) -> Outgoing {
    login_flow(
        service,
        user,
        Some(Server::Cn),
        matches!(scene, Scene::C2c | Scene::Dm),
        sender,
    )
    .await
}

async fn login_flow(
    service: &Arc<PhigrosService>,
    user: &str,
    server: Option<Server>,
    direct: bool,
    sender: Option<InvocationSender>,
) -> Outgoing {
    if bound(service, user).await {
        return PhigrosService::notice(ALREADY_BOUND);
    }
    let Some(server) = server else {
        return PhigrosService::notice(LOGIN_USAGE);
    };
    let login = match service.flow().start(server).await {
        Ok(login) => login,
        Err(error) => return PhigrosService::fail(error.message),
    };
    service.put_login(user, login.clone());
    let out = crate::buttons::with_keyboard(
        service.text("account.login.ticket", &ticket_args(&login, direct)),
        crate::buttons::Page::LoginTicket {
            user: user.to_string(),
        },
    );
    if let Some(sender) = sender {
        tokio::spawn(watch_login(
            Arc::clone(service),
            user.to_string(),
            login,
            sender,
        ));
    }
    out
}

async fn run_unbind(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if service.binding(&inv.user).await.ok().flatten().is_none() {
        return ok(commands::with_at(
            PhigrosService::notice("尚未绑定任何账号，无需解绑"),
            &inv,
        ));
    }
    let out = unbind_confirm(&service, &inv.user).await;
    ok(commands::with_at(out, &inv))
}

async fn run_clean(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = clean_confirm(&service, &inv.user).await;
    ok(commands::with_at(out, &inv))
}

fn confirm_page(
    service: &PhigrosService,
    template: &str,
    page: crate::buttons::Page,
) -> Outgoing {
    let out = service.text(template, &Args::new());
    crate::buttons::with_keyboard(out, page)
}

pub async fn unbind_confirm(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    confirm_page(
        service,
        "account.unbind.confirm",
        crate::buttons::Page::UnbindConfirm {
            user: user.to_string(),
        },
    )
}

pub async fn clean_confirm(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    confirm_page(
        service,
        "account.clean.confirm",
        crate::buttons::Page::CleanConfirm {
            user: user.to_string(),
        },
    )
}

pub async fn login_ticket(_service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    let out = PhigrosService::notice("Taptap 授权等待中，可以点下方按钮取消");
    crate::buttons::with_keyboard(
        out,
        crate::buttons::Page::LoginTicket {
            user: user.to_string(),
        },
    )
}

pub async fn unbind_apply(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    let binding = match service.binding(user).await {
        Ok(Some(binding)) => binding,
        Ok(None) => return PhigrosService::notice("尚未绑定任何账号，无需解绑"),
        Err(error) => return PhigrosService::fail(error),
    };
    if let Err(error) = service.repo().delete_user(user).await {
        return PhigrosService::fail(format!("解绑失败：{error}"));
    }
    let mut args = Args::new();
    put(&mut args, "player", display_name(&binding));
    service.text("account.unbind.ok", &args)
}

pub fn unbind_cancel() -> Outgoing {
    PhigrosService::notice("已取消，没有改动绑定")
}

pub async fn clean_apply(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    let bound = matches!(service.binding(user).await, Ok(Some(_)));
    if let Err(error) = service.repo().delete_user(user).await {
        return PhigrosService::fail(format!("清除失败：{error}"));
    }
    if bound {
        return service.text("account.clean.ok", &Args::new());
    }
    PhigrosService::notice("还没有绑定任何账号，已经没有任何数据了")
}

pub fn clean_cancel() -> Outgoing {
    PhigrosService::notice("已取消，没有删除任何数据")
}

pub fn login_cancel(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    if service.take_login(user).is_some() {
        return PhigrosService::notice("已取消 Taptap 授权登录");
    }
    PhigrosService::notice("没有正在进行的 Taptap 授权登录")
}

async fn run_pull(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    if let Err(out) = service.require_binding(&inv.user).await {
        return ok(*out);
    }
    let before = service.records(&inv.user).await.map_err(PluginError::handler)?;
    let before_rks = rks::rks(&service.judge(&before));
    let report = match service.sync(&inv.user).await {
        Ok(report) => report,
        Err(reason) => return ok(commands::with_at(PhigrosService::fail(reason), &inv)),
    };
    let after_rks = settle_rks(&report);
    let mut args = Args::new();
    put(&mut args, "player", display_name(&report.binding));
    put(&mut args, "records", report.scored.len().to_string());
    put(&mut args, "modified", stamp_text(report.binding.save_modified_at));
    put(
        &mut args,
        "rks",
        rks_line(report.first_run, before_rks, after_rks),
    );
    put(
        &mut args,
        "changes",
        blockquote(&changes_text(&report, &service)),
    );
    let out = reply_with_sync(
        &service,
        "account.pull.ok",
        &args,
        (&report, before_rks, after_rks),
    )
    .await;
    ok(commands::with_at(out, &inv))
}

async fn run_me(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let binding = match service.require_binding(&inv.user).await {
        Ok(binding) => binding,
        Err(out) => return ok(*out),
    };
    let out = me_card(&service, &inv.user, &binding).await;
    ok(commands::with_at(out, &inv))
}

pub async fn me_out(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    match service.require_binding(user).await {
        Ok(binding) => me_card(service, user, &binding).await,
        Err(out) => *out,
    }
}

async fn me_card(service: &PhigrosService, user: &str, binding: &Binding) -> Outgoing {
    let profile = match service.profile(user).await {
        Ok(profile) => profile.unwrap_or_default(),
        Err(error) => return PhigrosService::fail(error.to_string()),
    };
    let records = match service.records(user).await {
        Ok(records) => records,
        Err(error) => return PhigrosService::fail(error.to_string()),
    };
    let scored = service.judge(&records);
    let official = profile.rks;
    let best = service.board(&scored, official, scored.len().max(1));
    let repo = service.repo();
    let now = now_seconds();
    let points = repo.rks_points(user, 0).await.unwrap_or_default();
    let events = repo.score_events(user, 0).await.unwrap_or_default();
    let board = match service.online_board(user, binding, &profile, BOARD_LIMIT).await {
        Some(board) => board,
        None => repo
            .leader_board(user, BOARD_LIMIT)
            .await
            .unwrap_or_default(),
    };
    let stats = diff_stats(service, &scored);
    let data = MeData {
        binding,
        profile: &profile,
        best: &best,
        scored: &scored,
        stats: &stats,
        points: &points,
        events: &events,
        board: &board,
        now,
    };

    let req = overview_request(service, &data);
    let (width, height) = overview_card::canvas_size();
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, "玩家总览", &req, move |kit| {
        overview_card::draw(kit, &root, &drawn)
    });
    let out = PhigrosService::bare().card(card);
    crate::buttons::with_keyboard(out, crate::buttons::Page::Me)
}

async fn watch_login(
    service: Arc<PhigrosService>,
    user: String,
    login: DeviceLogin,
    sender: InvocationSender,
) {
    let interval = login.interval.clamp(POLL_MIN, POLL_MAX);
    let deadline = now_seconds() + login.expires_in as i64;
    let mut noticed = false;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        if now_seconds() >= deadline {
            let _ = sender
                .send(mention(&user, PhigrosService::notice(LOGIN_TIMEOUT)))
                .await;
            return;
        }
        let (session, state) = match service.flow().poll(&login).await {
            Ok(pair) => pair,
            Err(error) => {
                let _ = sender
                    .send(mention(&user, PhigrosService::fail(error.message)))
                    .await;
                return;
            }
        };
        if let Some(session) = session {
            let out = match commit(&service, &user, &session, Some(login.server)).await {
                Ok(report) => login_reply(&service, &report).await,
                Err(reason) => PhigrosService::fail(reason),
            };
            let _ = sender.send(mention(&user, out)).await;
            return;
        }
        if state == DeviceState::Scanned && !noticed {
            noticed = true;
            let _ = sender
                .send(mention(&user, PhigrosService::notice(LOGIN_SCAN)))
                .await;
        }
    }
}

async fn commit(
    service: &PhigrosService,
    user: &str,
    token: &str,
    server: Option<Server>,
) -> Result<SyncReport, String> {
    let candidates = match server {
        Some(server) => vec![server],
        None => vec![Server::Cn, Server::Global],
    };
    let mut token_error: Option<String> = None;
    let mut save_error: Option<String> = None;
    for candidate in candidates {
        match fetch(service, user, token, candidate).await {
            Ok(report) => return Ok(report),
            Err((Kind::TokenInvalid, message)) => {
                if token_error.is_none() {
                    token_error = Some(message);
                }
            }
            Err((Kind::SaveEmpty, message)) => {
                if save_error.is_none() {
                    save_error = Some(message);
                }
            }
            Err((_, message)) => return Err(message),
        }
    }
    Err(save_error
        .or(token_error)
        .unwrap_or_else(|| String::from(TOKEN_INVALID)))
}

async fn fetch(
    service: &PhigrosService,
    user: &str,
    token: &str,
    server: Server,
) -> Result<SyncReport, (Kind, String)> {
    let fetched = service
        .client()
        .archive(server, token)
        .await
        .map_err(|error| (error.kind, error.message))?;
    let binding = Binding {
        user_id: user.to_string(),
        token: token.to_string(),
        server,
        object_id: fetched.player.object_id.clone(),
        nickname: fetched.player.nickname.clone(),
        avatar: fetched.player.avatar.clone(),
        game_avatar: fetched.summary.avatar.clone(),
        save_modified_at: fetched.modified_at,
        bound_at: now_seconds(),
        last_sync_at: 0,
        status: 0,
        error: String::new(),
        joined_at: fetched.player.joined_at,
    };
    service
        .store(&binding, &fetched)
        .await
        .map_err(|reason| (Kind::ServiceDown, reason))
}

async fn bind_reply(service: &PhigrosService, report: &SyncReport) -> Outgoing {
    let mut args = bind_args(report);
    put(
        &mut args,
        "modified",
        stamp_text(report.binding.save_modified_at),
    );
    let after = settle_rks(report);
    reply_with_sync(service, "account.bind.ok", &args, (report, 0.0, after)).await
}

async fn login_reply(service: &PhigrosService, report: &SyncReport) -> Outgoing {
    let args = bind_args(report);
    let after = settle_rks(report);
    reply_with_sync(service, "account.login.ok", &args, (report, 0.0, after)).await
}

async fn reply_with_sync(
    service: &PhigrosService,
    id: &str,
    args: &Args,
    report: (&SyncReport, f64, f64),
) -> Outgoing {
    let (report, before_rks, after_rks) = report;
    let repo = service.repo();
    let events = repo
        .score_events(&report.binding.user_id, 0)
        .await
        .unwrap_or_default();
    let points = repo
        .rks_points(&report.binding.user_id, 0)
        .await
        .unwrap_or_default();
    let req = sync_request(service, report, &events, &points, before_rks, after_rks);
    let (width, height) = sync_card::canvas_size(&req);
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, SYNC_TITLE, &req, move |kit| {
        sync_card::draw(kit, &root, &drawn)
    });
    let caption = match id {
        "account.bind.ok" => "account.bind.ok.image",
        "account.login.ok" => "account.login.ok.image",
        _ => "",
    };
    let base = if caption.is_empty() {
        PhigrosService::bare()
    } else {
        service.text(caption, args)
    };
    crate::buttons::with_keyboard(base.card(card), crate::buttons::Page::BindOk)
}

fn bind_args(report: &SyncReport) -> Args {
    let mut args = Args::new();
    put(&mut args, "player", display_name(&report.binding));
    put(&mut args, "server", server_label(report.binding.server));
    put(&mut args, "rks", rks_text(settle_rks(report)));
    put(&mut args, "records", report.scored.len().to_string());
    args
}

fn settle_rks(report: &SyncReport) -> f64 {
    let computed = rks::rks(&report.scored);
    if computed > 0.0 {
        computed
    } else {
        report.profile.rks
    }
}

struct MeData<'a> {
    binding: &'a Binding,
    profile: &'a Profile,
    best: &'a Best,
    scored: &'a [Scored],
    stats: &'a [DifficultyStat],
    points: &'a [RksPoint],
    events: &'a [ScoreEvent],
    board: &'a LeaderBoard,
    now: i64,
}

fn overview_request(service: &PhigrosService, data: &MeData<'_>) -> overview_card::OverviewRequest {
    let curve = data
        .points
        .iter()
        .filter(|point| point.rks > 0.0)
        .map(|point| overview_card::CurvePoint {
            at: point.observed_at,
            rks: point.rks,
        })
        .collect();
    let milestones = overview_domain::milestones(
        data.events,
        data.points,
        data.binding.joined_at,
        data.now,
    );
    overview_card::OverviewRequest {
        backdrop: backdrop_of(data.best),
        player: service.player(data.binding, Some(data.profile)),
        server: String::from(crate::service::server_label(data.binding.server)),
        self_intro: data.profile.self_intro.trim().to_string(),
        curve,
        diffs: overview_diffs(data),
        collection: overview_collection(service, data),
        rank: overview_domain::rank_rows(data.board, false, LOCAL_SCOPE),
        milestones: overview_domain::pick_milestones(&milestones, MILESTONE_LIMIT),
        synced: synced_text(data.binding),
    }
}

fn overview_diffs(data: &MeData<'_>) -> Vec<overview_card::OverviewDiff> {
    let (cleared, full_combo, phi) = stats_of(data.scored);
    Difficulty::playable_all()
        .iter()
        .enumerate()
        .map(|(index, level)| {
            let stat = data.stats.get(index).copied().unwrap_or_default();
            let (best, worst) = diff_extremes(data.scored, *level);
            overview_card::OverviewDiff {
                label: level.name().to_string(),
                played: stat.played as i32,
                total: stat.total as i32,
                cleared: cleared[index],
                full_combo: full_combo[index],
                phi: phi[index],
                best,
                worst,
            }
        })
        .collect()
}

fn overview_collection(
    service: &PhigrosService,
    data: &MeData<'_>,
) -> overview_card::OverviewCollection {
    let mut charts = 0_i32;
    let mut songs: Vec<&str> = Vec::new();
    for item in data.scored {
        if item.unrecorded() || item.variant {
            continue;
        }
        charts += 1;
        if !songs.contains(&item.song_id()) {
            songs.push(item.song_id());
        }
    }
    let total_charts: usize = data.stats.iter().map(|stat| stat.total).sum();
    let total_songs = service
        .catalog()
        .songs()
        .iter()
        .filter(|song| !song.variant())
        .count();
    overview_card::OverviewCollection {
        unlocked_charts: charts,
        total_charts: total_charts as i32,
        unlocked_songs: songs.len() as i32,
        total_songs: total_songs as i32,
    }
}

fn backdrop_of(best: &Best) -> String {
    if let Some(item) = best.top.first() {
        return item.song_id().to_string();
    }
    if let Some(item) = best.podium.first() {
        return item.song_id().to_string();
    }
    String::new()
}

fn sync_backdrop(best: &Best, days: &[Day]) -> String {
    for day in days {
        for event in &day.items {
            if !event.song_id.is_empty() {
                return event.song_id.clone();
            }
        }
    }
    backdrop_of(best)
}

fn stats_of(scored: &[Scored]) -> ([i32; NUM_PLAYABLE], [i32; NUM_PLAYABLE], [i32; NUM_PLAYABLE]) {
    let mut cleared = [0_i32; NUM_PLAYABLE];
    let mut full_combo = [0_i32; NUM_PLAYABLE];
    let mut phi = [0_i32; NUM_PLAYABLE];
    for item in scored {
        let index = item.difficulty().index();
        if index >= NUM_PLAYABLE {
            continue;
        }
        if item.score() >= 700_000 {
            cleared[index] += 1;
        }
        if item.score() >= crate::model::MAX_SCORE && !item.suspect {
            phi[index] += 1;
        }
        if item.fc() {
            full_combo[index] += 1;
        }
    }
    (cleared, full_combo, phi)
}

fn diff_extremes(scored: &[Scored], level: Difficulty) -> (i32, i32) {
    let mut best = 0_i32;
    let mut worst = 0_i32;
    for item in scored {
        if item.difficulty() != level || item.unrecorded() || !item.counted() {
            continue;
        }
        if best == 0 || item.score() > best {
            best = item.score();
        }
        if worst == 0 || item.score() < worst {
            worst = item.score();
        }
    }
    (best, worst)
}

fn diff_stats(service: &PhigrosService, scored: &[Scored]) -> Vec<DifficultyStat> {
    let catalog = service.catalog();
    let mut seen: Vec<(String, usize)> = Vec::new();
    let mut played = [0_usize; NUM_PLAYABLE];
    for item in scored {
        let index = item.difficulty().index();
        if index >= NUM_PLAYABLE {
            continue;
        }
        let id = catalog
            .song(item.song_id())
            .map(|song| {
                if song.variant() {
                    song.variant_of.clone()
                } else {
                    song.id.clone()
                }
            })
            .unwrap_or_else(|| item.song_id().to_string());
        let key = (id, index);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        played[index] += 1;
    }
    let mut total = [0_usize; NUM_PLAYABLE];
    for song in catalog.songs() {
        if song.variant() {
            continue;
        }
        for difficulty in song.difficulties() {
            let index = difficulty.index();
            if index < NUM_PLAYABLE {
                total[index] += 1;
            }
        }
    }
    (0..NUM_PLAYABLE)
        .map(|index| DifficultyStat {
            played: played[index],
            total: total[index],
        })
        .collect()
}

struct Day {
    at: i64,
    total: usize,
    items: Vec<ScoreEvent>,
}

fn event_days(events: &[ScoreEvent], days: usize, per_day: usize) -> Vec<Day> {
    let offset = i64::from(Local::now().offset().local_minus_utc());
    let mut buckets: Vec<(i64, Vec<ScoreEvent>)> = Vec::new();
    for event in events {
        let index = (event.observed_at + offset).div_euclid(SECONDS_PER_DAY);
        match buckets.iter_mut().find(|(key, _)| *key == index) {
            Some((_, items)) => items.push(event.clone()),
            None => buckets.push((index, vec![event.clone()])),
        }
    }
    buckets.sort_by_key(|(key, _)| std::cmp::Reverse(*key));
    buckets.truncate(days);
    buckets
        .into_iter()
        .map(|(_, mut items)| {
            items.sort_by(|left, right| {
                impact(right)
                    .partial_cmp(&impact(left))
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(right.new_score.cmp(&left.new_score))
                    .then(right.song_id.cmp(&left.song_id))
                    .then(right.difficulty.index().cmp(&left.difficulty.index()))
            });
            let at = items
                .iter()
                .map(|event| event.observed_at)
                .max()
                .unwrap_or_default();
            let total = items.len();
            items.truncate(per_day);
            Day { at, total, items }
        })
        .collect()
}

fn impact(event: &ScoreEvent) -> f64 {
    event.new_rks - event.old_rks
}

fn sync_groups(
    days: &[Day],
    set: &[String],
    service: &PhigrosService,
    changed: bool,
    save_modified: i64,
) -> Vec<sync_card::SyncGroup> {
    let mut out: Vec<sync_card::SyncGroup> = Vec::new();
    let mut left = ITEM_LIMIT;
    for (index, day) in days.iter().enumerate() {
        if day.items.is_empty() || left == 0 {
            continue;
        }
        let label = if index == 0 && changed && day.at >= save_modified - 1 {
            String::from(SYNC_LABEL)
        } else {
            String::new()
        };
        let mut group = sync_card::SyncGroup {
            at: DateTime::from_timestamp(day.at, 0),
            label,
            items: Vec::new(),
            total: day.total as i32,
        };
        for event in day.items.iter().take(left) {
            group.items.push(sync_item(event, set, service));
        }
        left -= group.items.len();
        out.push(group);
    }
    out
}

fn sync_item(event: &ScoreEvent, set: &[String], service: &PhigrosService) -> sync_card::SyncItem {
    let mut item = sync_card::SyncItem {
        song_id: event.song_id.clone(),
        title: event.song_id.clone(),
        difficulty: String::from(event.difficulty.name()),
        constant: 0.0,
        known: false,
        old_score: event.old_score,
        new_score: event.new_score,
        old_acc: event.old_acc,
        new_acc: event.new_acc,
        delta: event.new_score - event.old_score,
        rks: event.new_rks,
        grade: String::from(grade_of(event.new_score, event.new_fc).name()),
        in_best: set.contains(&chart_key(&event.song_id, event.difficulty)),
        rks_delta: event.new_rks - event.old_rks,
    };
    if !event.has_old {
        item.old_score = 0;
        item.old_acc = 0.0;
        item.delta = 0;
    }
    if let Some(song) = service.catalog().song(&event.song_id) {
        if !song.title.is_empty() {
            item.title = song.title.clone();
        }
        if let Some(chart) = song.chart(event.difficulty) {
            item.constant = chart.constant;
            item.known = chart.known();
        }
    }
    item
}

fn adopt_group(
    groups: Vec<sync_card::SyncGroup>,
    report: &SyncReport,
    set: &[String],
) -> Vec<sync_card::SyncGroup> {
    if !report.first_run {
        return groups;
    }
    let mut charted: Vec<String> = Vec::new();
    for group in &groups {
        for item in &group.items {
            charted.push(format!("{}|{}", item.song_id, item.difficulty));
        }
    }
    let mut sorted: Vec<&Scored> = report
        .scored
        .iter()
        .filter(|item| !item.suspect)
        .collect();
    sorted.sort_by(|left, right| {
        right
            .rks
            .partial_cmp(&left.rks)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(right.score().cmp(&left.score()))
    });
    let mut items: Vec<sync_card::SyncItem> = Vec::new();
    for item in sorted {
        if items.len() >= PER_DAY {
            break;
        }
        if charted.contains(&chart_key(item.song_id(), item.difficulty())) {
            continue;
        }
        items.push(adopt_item(item, set));
    }
    if items.is_empty() {
        return groups;
    }
    let total = report.scored.iter().filter(|item| !item.suspect).count() as i32;
    let head = sync_card::SyncGroup {
        at: DateTime::from_timestamp(report.binding.save_modified_at, 0),
        label: String::from(ADOPT_LABEL),
        items,
        total,
    };
    std::iter::once(head).chain(groups).collect()
}

fn adopt_item(item: &Scored, set: &[String]) -> sync_card::SyncItem {
    let title = if item.title.is_empty() {
        item.song_id().to_string()
    } else {
        item.title.clone()
    };
    sync_card::SyncItem {
        song_id: item.song_id().to_string(),
        title,
        difficulty: String::from(item.difficulty().name()),
        constant: item.constant,
        known: item.known,
        old_score: 0,
        new_score: item.score(),
        old_acc: 0.0,
        new_acc: item.acc(),
        delta: 0,
        rks: item.rks,
        grade: String::from(grade_of(item.score(), item.fc()).name()),
        in_best: set.contains(&chart_key(item.song_id(), item.difficulty())),
        rks_delta: item.rks,
    }
}

fn best_set(best: &Best) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for item in best.top.iter().take(COUNTED_BEST) {
        out.push(chart_key(item.song_id(), item.difficulty()));
    }
    for item in &best.podium {
        out.push(chart_key(item.song_id(), item.difficulty()));
    }
    out
}

fn chart_key(song_id: &str, level: Difficulty) -> String {
    format!("{song_id}|{}", level.name())
}

fn sync_request(
    service: &PhigrosService,
    report: &SyncReport,
    events: &[ScoreEvent],
    points: &[RksPoint],
    before_rks: f64,
    after_rks: f64,
) -> sync_card::SyncRequest {
    let binding = &report.binding;
    let profile = &report.profile;
    let best = service.board(&report.scored, profile.rks, report.scored.len().max(1));
    let set = best_set(&best);
    let changed = if report.first_run {
        !report.scored.is_empty()
    } else {
        !events.is_empty() || report.scored.iter().any(|item| item.suspect)
    };
    let days = event_days(events, DAY_LIMIT, PER_DAY);
    let groups = sync_groups(&days, &set, service, changed, binding.save_modified_at);
    let groups = adopt_group(groups, report, &set);
    let stats = diff_stats(service, &report.scored);
    let played: usize = stats.iter().map(|stat| stat.played).sum();
    let charts: usize = stats.iter().map(|stat| stat.total).sum();
    let (recent7, has_recent) = recent7(points, now_seconds());
    sync_card::SyncRequest {
        player: service.player(binding, Some(profile)),
        backdrop: sync_backdrop(&best, &days),
        at: DateTime::from_timestamp(now_seconds(), 0),
        save_modified: DateTime::from_timestamp(binding.save_modified_at, 0),
        first_sync: report.first_run,
        changed,
        rks_before: before_rks,
        rks_after: after_rks,
        groups,
        total: report.scored.len() as i32,
        played: played as i32,
        charts: charts as i32,
        recent7,
        has_recent,
    }
}

fn recent7(points: &[RksPoint], now: i64) -> (f64, bool) {
    let Some(base) = carry_back(points, now - ACTIVITY_WINDOW) else {
        return (0.0, false);
    };
    let Some(latest) = points.last() else {
        return (0.0, false);
    };
    let has = points.len() > 1 || (base - latest.rks).abs() > f64::EPSILON;
    (latest.rks - base, has)
}

fn carry_back(points: &[RksPoint], at: i64) -> Option<f64> {
    let first = points.first()?;
    let mut value: Option<f64> = None;
    for point in points {
        if point.observed_at > at {
            break;
        }
        value = Some(point.rks);
    }
    if value.is_some() {
        return value;
    }
    let last = points.last()?;
    if first.observed_at < last.observed_at {
        return Some(first.rks);
    }
    None
}

fn changes_text(report: &SyncReport, service: &PhigrosService) -> String {
    let mut out = String::new();
    let added = pick(report, |event| !event.has_old);
    let improved = pick(report, |event| event.has_old && event.new_score > event.old_score);
    let dropped = pick(report, |event| event.has_old && event.new_score < event.old_score);
    let suspect = report.scored.iter().filter(|item| item.suspect).count();
    if report.first_run {
        out.push_str(&format!("首次同步，已导入 {} 条成绩", report.scored.len()));
    } else if added.is_empty() && improved.is_empty() && dropped.is_empty() && suspect == 0 {
        out.push_str("本次没有发现新的成绩");
    } else {
        out.push_str(&summary(&added, &improved, &dropped));
    }
    write_group(&mut out, "新增", &added, service);
    write_group(&mut out, "↑", &improved, service);
    write_group(&mut out, "↓", &dropped, service);
    write_suspect(&mut out, report);
    out
}

fn pick<F>(report: &SyncReport, keep: F) -> Vec<&ScoreEvent>
where
    F: Fn(&ScoreEvent) -> bool,
{
    report.events.iter().filter(|event| keep(event)).collect()
}

fn summary(added: &[&ScoreEvent], improved: &[&ScoreEvent], dropped: &[&ScoreEvent]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !added.is_empty() {
        parts.push(format!("新增 {} 首", added.len()));
    }
    if !improved.is_empty() {
        parts.push(format!("↑ {} 首", improved.len()));
    }
    if !dropped.is_empty() {
        parts.push(format!("↓ {} 首", dropped.len()));
    }
    parts.join(" · ")
}

fn write_group(out: &mut String, title: &str, items: &[&ScoreEvent], service: &PhigrosService) {
    if items.is_empty() {
        return;
    }
    out.push_str("\n\n");
    out.push_str(title);
    for (index, event) in items.iter().enumerate() {
        if index >= CHANGE_PREVIEW {
            out.push_str(&format!("\n  … 等 {} 首", items.len()));
            break;
        }
        out.push_str(&format!(
            "\n  {} [{}] {} · {}",
            title_of(service, &event.song_id),
            event.difficulty.name(),
            score_text(event.new_score),
            acc_short(event.new_acc)
        ));
    }
}

fn write_suspect(out: &mut String, report: &SyncReport) {
    let items: Vec<&Scored> = report.scored.iter().filter(|item| item.suspect).collect();
    if items.is_empty() {
        return;
    }
    out.push_str("\n\n数据不自洽（不进 B30 / RKS，请核对存档）");
    for (index, item) in items.iter().enumerate() {
        if index >= CHANGE_PREVIEW {
            out.push_str(&format!("\n  … 等 {} 条", items.len()));
            break;
        }
        out.push_str(&format!(
            "\n  {} [{}] {} · {} —— 满分却不满 100% 准确率",
            item.title,
            item.difficulty().name(),
            score_text(item.score()),
            acc_short(item.acc())
        ));
    }
}

fn title_of(service: &PhigrosService, song_id: &str) -> String {
    match service.catalog().song(song_id) {
        Some(song) if !song.title.is_empty() => song.title.clone(),
        _ => song_id.to_string(),
    }
}

fn blockquote(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    text.split('\n')
        .map(|line| {
            if line.trim().is_empty() {
                String::from(">")
            } else {
                format!("> {line}")
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn rks_line(first_sync: bool, before: f64, after: f64) -> String {
    if first_sync {
        return format!("RKS {}", rks_text(after));
    }
    if before <= 0.0 && after <= 0.0 {
        return String::new();
    }
    format!(
        "RKS {} → {}（{}）",
        rks_text(before),
        rks_text(after),
        signed_text(after - before)
    )
}

fn ticket_args(login: &DeviceLogin, direct: bool) -> Args {
    let mut args = Args::new();
    put(&mut args, "url", login.url.clone());
    put(&mut args, "expires", minutes_text(login.expires_in));
    put(&mut args, "hint", ticket_hint(direct));
    args
}

fn ticket_hint(direct: bool) -> &'static str {
    if direct {
        "也可以直接把 sessionToken 用 `/p bind <sessionToken>` 发给我"
    } else {
        "也可以私聊我发送 `/p bind <sessionToken>` 直接绑定"
    }
}

fn minutes_text(seconds: u64) -> String {
    let minutes = seconds / 60;
    if minutes < 1 {
        return String::from("1 分钟");
    }
    format!("{minutes} 分钟")
}

fn mention(user: &str, out: Outgoing) -> Outgoing {
    Outgoing { at: false, ..out }.mention_user(user.to_string())
}

async fn bound(service: &PhigrosService, user: &str) -> bool {
    matches!(service.binding(user).await, Ok(Some(_)))
}

fn wants_login(raw: &str) -> bool {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    match fields.len() {
        0 => true,
        1 => parse_server(fields[0]).is_some(),
        _ => false,
    }
}

fn parse_bind(raw: &str) -> Option<(String, Option<Server>)> {
    let mut fields: Vec<&str> = raw.split_whitespace().collect();
    let mut server = None;
    if fields.len() >= 2 {
        if let Some(parsed) = parse_server(fields[0]) {
            server = Some(parsed);
            fields.remove(0);
        }
    }
    if fields.len() != 1 {
        return None;
    }
    let token = fields[0].trim();
    if token.is_empty() {
        return None;
    }
    Some((token.to_string(), server))
}

fn parse_server_arg(raw: &str) -> Option<Server> {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    match fields.len() {
        0 => Some(Server::Cn),
        1 => parse_server(fields[0]),
        _ => None,
    }
}

fn parse_server(text: &str) -> Option<Server> {
    match text.trim().to_lowercase().as_str() {
        "cn" | "国服" | "中国" | "china" => Some(Server::Cn),
        "gb" | "global" | "国际" | "国际服" => Some(Server::Global),
        _ => None,
    }
}

fn display_name(binding: &Binding) -> String {
    let name = binding.nickname.trim();
    if name.is_empty() {
        return String::from("未命名玩家");
    }
    name.to_string()
}

fn stamp_text(seconds: i64) -> String {
    if seconds <= 0 {
        return String::from("-");
    }
    match DateTime::from_timestamp(seconds, 0) {
        Some(stamp) => stamp
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        None => String::from("-"),
    }
}

fn synced_text(binding: &Binding) -> String {
    if binding.save_modified_at > 0 {
        return stamp_text(binding.save_modified_at);
    }
    if binding.last_sync_at > 0 {
        return stamp_text(binding.last_sync_at);
    }
    String::new()
}

fn rks_text(value: f64) -> String {
    format!("{value:.4}")
}

fn signed_text(value: f64) -> String {
    format!("{value:+.4}")
}

fn acc_short(value: f64) -> String {
    format!("{value:.2}%")
}

fn score_text(value: i32) -> String {
    let digits = value.to_string();
    if value < 0 {
        return digits;
    }
    let mut out = String::with_capacity(digits.len() + 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(at: i64, song_id: &str, old_score: i32, new_score: i32) -> ScoreEvent {
        ScoreEvent {
            observed_at: at,
            song_id: song_id.to_string(),
            difficulty: Difficulty::In,
            has_old: old_score > 0,
            old_score,
            old_acc: 0.0,
            old_fc: false,
            new_score,
            new_acc: 99.0,
            new_fc: false,
            old_rks: 0.0,
            new_rks: (new_score - old_score) as f64 / 1000.0,
        }
    }

    #[test]
    fn parse_bind_takes_optional_server_prefix() {
        assert_eq!(
            parse_bind("  token12345 "),
            Some((String::from("token12345"), None))
        );
        assert_eq!(
            parse_bind("cn token12345"),
            Some((String::from("token12345"), Some(Server::Cn)))
        );
        assert_eq!(
            parse_bind("国际服 token12345"),
            Some((String::from("token12345"), Some(Server::Global)))
        );
        assert_eq!(parse_bind("cn xx yy"), None);
        assert_eq!(parse_bind("   "), None);
        assert_eq!(parse_bind("cn"), Some((String::from("cn"), None)));
    }

    #[test]
    fn wants_login_only_when_no_credential_given() {
        assert!(wants_login(""));
        assert!(wants_login("   "));
        assert!(wants_login("国服"));
        assert!(wants_login("cn"));
        assert!(!wants_login("token12345"));
        assert!(!wants_login("cn token12345"));
    }

    #[test]
    fn parse_server_arg_defaults_to_cn_and_rejects_extra_words() {
        assert_eq!(parse_server_arg(""), Some(Server::Cn));
        assert_eq!(parse_server_arg("cn"), Some(Server::Cn));
        assert_eq!(parse_server_arg(" 国际服 "), Some(Server::Global));
        assert_eq!(parse_server_arg("cn global"), None);
        assert_eq!(parse_server_arg("火星服"), None);
    }

    #[test]
    fn event_days_splits_by_day_and_orders_by_impact() {
        let base = 1_700_000_000_i64;
        let events = vec![
            event(base, "a", 900_000, 950_000),
            event(base, "b", 0, 800_000),
            event(base + 2 * SECONDS_PER_DAY, "c", 0, 990_000),
        ];
        let days = event_days(&events, 10, 10);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].total, 1);
        assert_eq!(days[0].items[0].song_id, "c");
        assert_eq!(days[1].total, 2);
        assert_eq!(days[1].items[0].song_id, "b");
        assert_eq!(days[1].items[1].song_id, "a");
    }

    #[test]
    fn sync_backdrop_prefers_this_sync_changes() {
        let base = 1_700_000_000_i64;
        let days = event_days(&[event(base, "changed", 0, 900_000)], 10, 10);
        assert_eq!(sync_backdrop(&Best::default(), &days), "changed");
        assert_eq!(sync_backdrop(&Best::default(), &[]), "");
    }

    #[test]
    fn event_days_honours_day_and_item_limits() {
        let base = 1_700_000_000_i64;
        let mut events: Vec<ScoreEvent> = Vec::new();
        for day in 0..3 {
            for song in 0..3 {
                events.push(event(
                    base + day * SECONDS_PER_DAY,
                    &format!("s{song}"),
                    100_000 + song,
                    200_000,
                ));
            }
        }
        let days = event_days(&events, 2, 2);
        assert_eq!(days.len(), 2);
        assert!(days.iter().all(|day| day.items.len() <= 2));
        assert_eq!(days[0].total, 3);
    }

    #[test]
    fn rks_line_reports_first_sync_and_skips_empty_sides() {
        assert_eq!(rks_line(true, 0.0, 15.2), "RKS 15.2000");
        assert_eq!(rks_line(false, 0.0, 0.0), "");
        assert_eq!(
            rks_line(false, 15.1, 15.2),
            "RKS 15.1000 → 15.2000（+0.1000）"
        );
    }
}
