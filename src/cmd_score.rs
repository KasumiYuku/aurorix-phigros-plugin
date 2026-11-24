use std::sync::Arc;

use aurorix_plugin::{CommandSpec, Invocation, Outgoing, PluginError, ReplyKind};
use aurorix_template::{Arg, Args};
use rand::Rng;

use crate::backend::phi;
use crate::cards::player::Player;
use crate::cards::settlement::{
    ChartHistory, DifficultyScore, HistoryPoint, SettlementRequest, Standing,
};
use crate::cards::{b30, score as score_card, songinfo, songselect};
use crate::commands::{self, ok};
use crate::domain::span::{parse_span, Span};
use crate::domain::{overview, query, rks};
use crate::model::{
    Best, Binding, Difficulty, Grade, PlayRecord, Profile, Scored, Song, BEST_SIZE, COUNTED_BEST,
    COUNTED_PHI, MAX_SCORE, MONEY_UNITS, NUM_MONEY, NUM_PLAYABLE,
};
use crate::repo::ScoreEvent;
use crate::service::{put, PhigrosService};

const PICK_LIMIT: usize = 10;

const MAX_NOTES: i32 = 2500;

const HISTORY_LIMIT: usize = 8;
const SCORE_BOARD_ROWS: usize = 7;

const CHALLENGE_SPREAD: i32 = 2;

const SCORE_USAGE: &str = "用法：/p score <曲名> [难度]，难度取 EZ / HD / IN / AT（默认 IN），附加谱面可写 SP / LY";

const B30_USAGE: &str = "用法：/p b30 看前 30，/p b50 看前 50；两条命令都不接数量";

const PHI_USAGE: &str = "用法：/p ap30 看 All Perfect（只看满分曲，按定数降序，/p ap 是别名）；这条命令不接数量";

const FC_USAGE: &str =
    "用法：/p fc30 看 Full Combo 30（只看全连、但没满分的成绩，/p fc 是别名）；这条命令不接数量";

const ONE_GOOD_USAGE: &str = "用法：/p x30 看 1Good 30（只看全连、且分数正好差一个 Good 的成绩，/p 1g 是别名）；这条命令不接数量";

const CAL_USAGE: &str = "用法：/p cal <定数> <acc>，例如 /p cal 15.7 99.5（acc 可带 % 号）";

const HIS_USAGE: &str =
    "用法：/p his <曲名>，例如 /p his Cthugha（别名 hist / history / 历史 / 历程）";

const CHALLENGE_USAGE: &str = "用法：/p challenge [定数之和] [难度…]，顺序不限
  和数    40 / 41-43 / 40+ / -43（三张按定数取整相加，和必是整数）
  难度    EZ / HD / IN / AT
不填和数就随机挑一个凑得出来的数值
例如：/p challenge、/p challenge 40、/p challenge 41-43 IN、/p challenge 40+ AT";

const AIM_USAGE: &str = "用法：/p aim <曲名> [难度] <目标分>    反推打出该分要多少 Perfect / Good
      /p aim <物量> <目标分>           不查曲名，按物量直接算
      /p aim 课题 <曲名> [难度] <目标分> 课题模式（不计连击）
例：/p aim Rrhar'il AT 114514、/p aim 2222 114514";

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("b30")
            .alias("rks")
            .alias("best30")
            .describe("查询 RKS 与 Best 30")
            .handler(commands::on(service, run_b30)),
        CommandSpec::new("b50")
            .alias("best50")
            .describe("查询 RKS 与 Best 50")
            .handler(commands::on(service, run_b50)),
        CommandSpec::new("ap30")
            .alias("ap")
            .alias("满分榜")
            .alias("满分")
            .alias("allperfect")
            .describe("只看满分曲的榜面图（/p ap 是它的别名）")
            .handler(commands::on(service, run_ap30)),
        CommandSpec::new("fc30")
            .alias("fc")
            .alias("全连")
            .alias("全连榜")
            .alias("fullcombo")
            .describe("只看全连（但没满分）成绩的榜面图（/p fc 是它的别名）")
            .handler(commands::on(service, run_fc30)),
        CommandSpec::new("x30")
            .alias("1g")
            .alias("1g30")
            .alias("1good30")
            .alias("1good")
            .alias("差一个good")
            .describe("只看差一个 Good 成绩的榜面图（/p 1g 是它的别名）")
            .handler(commands::on(service, run_x30)),
        CommandSpec::new("score")
            .alias("单曲成绩")
            .describe("查询单曲成绩")
            .handler(commands::on(service, run_score)),
        CommandSpec::new("his")
            .alias("hist")
            .alias("history")
            .alias("历史")
            .alias("历程")
            .alias("成绩历史")
            .describe("查看某首曲子的历史成绩（四档现状 + 每档每次变化）")
            .handler(commands::on(service, run_his)),
        CommandSpec::new("cal")
            .alias("com")
            .alias("rks算")
            .describe("按定数与 ACC 计算单曲 RKS")
            .handler(commands::on(service, run_cal)),
        CommandSpec::new("challenge")
            .alias("randclg")
            .alias("课题")
            .alias("挑战")
            .describe("抽三张谱面，定数取整后相加等于指定值（不给和数就随机）")
            .handler(commands::on(service, run_challenge)),
        CommandSpec::new("aim")
            .alias("snipe")
            .alias("scoretarget")
            .alias("控分")
            .alias("打分")
            .alias("狙击分数")
            .describe("反推打出目标分需要多少 Perfect / Good")
            .handler(commands::on(service, run_aim)),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Best,
    Phi,
    Fc,
    OneGood,
}

impl Kind {
    fn title(self, size: usize) -> String {
        match self {
            Kind::Best => format!("Best {size}"),
            Kind::Phi => String::from("AP 30"),
            Kind::Fc => String::from("FC 30"),
            Kind::OneGood => String::from("1G 30"),
        }
    }

    fn list_tag(self) -> &'static str {
        match self {
            Kind::Phi => "AP",
            Kind::Fc => "FC",
            Kind::OneGood => "1G",
            Kind::Best => "",
        }
    }

    fn list_noun(self) -> &'static str {
        match self {
            Kind::Phi => "ALL PERFECT",
            Kind::Fc => "FULL COMBO",
            Kind::OneGood => "1 GOOD",
            Kind::Best => "成绩",
        }
    }

    fn usage(self) -> &'static str {
        match self {
            Kind::Phi => PHI_USAGE,
            Kind::Fc => FC_USAGE,
            Kind::OneGood => ONE_GOOD_USAGE,
            Kind::Best => B30_USAGE,
        }
    }
}

struct BoardData {
    best: Best,
    kind: Kind,
    size: usize,
    player: Player,
    synced: String,
    diffs: Vec<b30::B30DiffStat>,
}

impl BoardData {
    fn items(&self) -> Option<&[Scored]> {
        match self.kind {
            Kind::Phi => Some(&self.best.phi),
            Kind::Fc => Some(&self.best.fc),
            Kind::OneGood => Some(&self.best.one_good),
            Kind::Best => None,
        }
    }

    fn title(&self) -> String {
        self.kind.title(self.size)
    }

    fn avg_line(&self) -> String {
        match self.items() {
            Some(items) => format!("均值 RKS {}", rks_text(list_average(items))),
            None => {
                let best_count = rks::board_count(self.size);
                let best_avg = entries_average(&entries_of_best(&self.best, best_count));
                let mut parts = vec![format!("P3 平均 {}", rks_text(phi_average(&self.best)))];
                if best_count > 0 {
                    parts.push(format!("B{best_count} 平均 {}", rks_text(best_avg)));
                }
                parts.join(" · ")
            }
        }
    }

    fn footer(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        if self.best.unknown > 0 {
            match self.items() {
                Some(_) => lines.push(format!(
                    "有 {} 个谱面曲库未收录，不计入榜面",
                    self.best.unknown
                )),
                None => lines.push(format!(
                    "有 {} 个谱面曲库未收录，不计入 Best {}",
                    self.best.unknown, self.size
                )),
            }
        }
        match self.items() {
            Some(items) => {
                lines.push(format!(
                    "共 {} 首 {}；榜上列前 {} 条（按单曲等效 RKS 降序）",
                    items.len(),
                    self.kind.list_noun(),
                    items.len().min(BEST_SIZE)
                ));
                if self.kind == Kind::Phi {
                    lines.push(String::from(
                        "P1-P3 是 RKS 的加分项；数据来自你上次拉取的存档，/p pull 可更新",
                    ));
                } else {
                    lines.push(String::from("数据来自你上次拉取的存档，/p pull 可更新"));
                }
            }
            None => {
                let counted = rks::board_count(self.size);
                let shown = rks::overflow_shown(self.size);
                if self.best.top.len() > counted {
                    lines.push(format!(
                        "RKS 只计 P 位与前 {counted} 条，其后 {} 条仅供参照",
                        self.best.top.len() - counted
                    ));
                }
                if self.best.top.len() < counted + shown {
                    lines.push(format!("共 {} 条成绩，已全部列出", self.best.top.len()));
                }
                let other = if self.size > BEST_SIZE { "b30" } else { "b50" };
                lines.push(format!(
                    "发 /p {other} 可看另一档；数据来自你上次拉取的存档，/p pull 可更新"
                ));
            }
        }
        lines.join("\n")
    }

    fn text(&self) -> Args {
        let mut args = Args::new();
        put(&mut args, "title", self.title());
        put(&mut args, "rks", rks_text(self.best.computed));
        put(&mut args, "official", rks_text(self.best.official));
        put(&mut args, "computed", rks_text(self.best.computed));
        int(&mut args, "records", self.best.records as i64);
        put(&mut args, "avgLine", self.avg_line());
        int(&mut args, "phiCount", self.best.phi.len() as i64);
        args.insert(String::from("rows"), Arg::List(self.text_rows()));
        put(&mut args, "footer", self.footer());
        args
    }

    fn text_rows(&self) -> Vec<Arg> {
        let mut rows: Vec<Arg> = Vec::new();
        match self.items() {
            Some(items) => {
                for (index, item) in items.iter().take(BEST_SIZE).enumerate() {
                    let label = if self.kind == Kind::Phi && index < self.best.podium.len() {
                        format!("P{}", index + 1)
                    } else {
                        format!("{}", index + 1)
                    };
                    rows.push(row_text(item, &label));
                }
            }
            None => {
                for (index, item) in self.best.podium.iter().enumerate() {
                    rows.push(row_text(item, &format!("P{}", index + 1)));
                }
                for (index, item) in self.best.top.iter().enumerate() {
                    rows.push(row_text(item, &format!("{}", index + 1)));
                }
            }
        }
        rows
    }

    fn image(&self) -> b30::B30Request {
        match self.items() {
            Some(items) => self.list_image(items),
            None => self.best_image(),
        }
    }

    fn best_image(&self) -> b30::B30Request {
        let mut req = b30::B30Request {
            player: self.player.clone(),
            count: self.size as i32,
            rks: self.best.official,
            computed: self.best.computed,
            records: self.best.records as i32,
            phi: self.best.phi.len() as i32,
            unknown: self.best.unknown as i32,
            suspect: self.best.suspect as i32,
            synced: self.synced.clone(),
            diffs: self.diffs.clone(),
            ..b30::B30Request::default()
        };
        req.backdrop = self
            .best
            .top
            .first()
            .or_else(|| self.best.podium.first())
            .map(|item| item.song_id().to_string())
            .unwrap_or_default();
        for index in 0..COUNTED_PHI {
            match self.best.podium.get(index) {
                Some(item) => req.entries.push(entry_of(item, index as i32 + 1, true)),
                None => req.entries.push(b30::B30Entry {
                    rank: index as i32 + 1,
                    podium: true,
                    slot: true,
                    ..b30::B30Entry::default()
                }),
            }
        }
        for (index, item) in self.best.top.iter().enumerate() {
            req.entries
                .push(entry_of(item, index as i32 + 1, false));
        }
        req.phi_avg = entries_average(&req.entries[..COUNTED_PHI.min(req.entries.len())]);
        req.best_count = rks::board_count(self.size) as i32;
        let counted = rks::board_count(self.size);
        let top = &req.entries[COUNTED_PHI.min(req.entries.len())..];
        req.best_avg = entries_average(&top[..top.len().min(counted)]);
        req
    }

    fn list_image(&self, items: &[Scored]) -> b30::B30Request {
        let shown = &items[..items.len().min(BEST_SIZE)];
        let title = self.title();
        let mut req = b30::B30Request {
            player: self.player.clone(),
            count: BEST_SIZE as i32,
            title: title.clone(),
            list_noun: String::from(self.kind.list_tag()),
            no_tier: true,
            rks: self.best.official,
            computed: self.best.computed,
            records: items.len() as i32,
            avg: list_average(shown),
            avg_label: format!("{} 平均", title.replace(' ', "")),
            phi: self.best.phi.len() as i32,
            unknown: self.best.unknown as i32,
            suspect: self.best.suspect as i32,
            synced: self.synced.clone(),
            diffs: self.diffs.clone(),
            ..b30::B30Request::default()
        };
        if let Some(first) = shown.first() {
            req.backdrop = first.song_id().to_string();
        }
        for (index, item) in shown.iter().enumerate() {
            let podium = self.kind == Kind::Phi && index < self.best.podium.len();
            req.entries.push(entry_of(item, index as i32 + 1, podium));
        }
        req
    }
}

async fn run_b30(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    run_board(service, inv, Kind::Best, BEST_SIZE).await
}

async fn run_b50(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    run_board(service, inv, Kind::Best, crate::model::BEST50_SIZE).await
}

async fn run_ap30(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    run_board(service, inv, Kind::Phi, BEST_SIZE).await
}

async fn run_fc30(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    run_board(service, inv, Kind::Fc, BEST_SIZE).await
}

async fn run_x30(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    run_board(service, inv, Kind::OneGood, BEST_SIZE).await
}

async fn run_board(
    service: Arc<PhigrosService>,
    inv: Invocation,
    kind: Kind,
    size: usize,
) -> Result<Option<Outgoing>, PluginError> {
    let size = match count_override(&inv.text) {
        Some(value) => clamp_size(value),
        None => size,
    };
    let binding = match service.require_binding(&inv.user).await {
        Ok(binding) => binding,
        Err(out) => return ok(*out),
    };
    let profile = service.profile(&inv.user).await.map_err(PluginError::handler)?;
    let records = service.records(&inv.user).await.map_err(PluginError::handler)?;
    let scored = service.judge(&records);
    let official = profile.as_ref().map(|item| item.rks).unwrap_or_default();
    let data = BoardData {
        best: service.board(&scored, official, size),
        kind,
        size,
        player: service.player(&binding, profile.as_ref()),
        synced: synced_text(&binding),
        diffs: diff_stats(&service, &records),
    };
    let req = data.image();
    let (width, height) = b30::canvas_size(&req);
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, data.title(), &req, move |kit| {
        b30::draw(kit, &root, &drawn)
    });
    let mut out = service.text("score.b30.image", &data.text());
    out = out.card(card);
    if kind == Kind::Best {
        out = crate::buttons::with_keyboard(out, crate::buttons::Page::Board { size });
    }
    if kind != Kind::Best && !inv.text.trim().is_empty() {
        return ok(commands::with_at(PhigrosService::notice(kind.usage()), &inv));
    }
    ok(commands::with_at(out, &inv))
}

pub async fn board_out(service: &Arc<PhigrosService>, user: &str, size: usize) -> Outgoing {
    let size = clamp_size(size as i64);
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = match service.profile(user).await {
        Ok(profile) => profile,
        Err(error) => return PhigrosService::fail(error.to_string()),
    };
    let records = match service.records(user).await {
        Ok(records) => records,
        Err(error) => return PhigrosService::fail(error.to_string()),
    };
    let scored = service.judge(&records);
    let official = profile.as_ref().map(|item| item.rks).unwrap_or_default();
    let data = BoardData {
        best: service.board(&scored, official, size),
        kind: Kind::Best,
        size,
        player: service.player(&binding, profile.as_ref()),
        synced: synced_text(&binding),
        diffs: diff_stats(service, &records),
    };
    let req = data.image();
    let (width, height) = b30::canvas_size(&req);
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, data.title(), &req, move |kit| {
        b30::draw(kit, &root, &drawn)
    });
    let out = service.text("score.b30.image", &data.text()).card(card);
    crate::buttons::with_keyboard(out, crate::buttons::Page::Board { size })
}
async fn run_score(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let Some(parsed) = query::parse_song_query(&inv.text).filter(|item| !item.text.trim().is_empty())
    else {
        return ok(PhigrosService::notice(SCORE_USAGE));
    };
    let binding = match service.require_binding(&inv.user).await {
        Ok(binding) => binding,
        Err(out) => return ok(*out),
    };
    let song = match resolve_song(&service, &parsed.text, "/p score ", "") {
        Resolved::Song(song) => song,
        Resolved::Basket(out) => return ok(out),
        Resolved::Missing(out) => return ok(out),
    };
    if !parsed.extra.is_empty() {
        return ok(extra_card(&service, song, &parsed.extra));
    }
    let records = service.records(&inv.user).await.map_err(PluginError::handler)?;
    let Some(level) = query::pick_score_difficulty(song, &records, parsed.difficulty) else {
        if parsed.difficulty == Some(Difficulty::Legacy) {
            return ok(PhigrosService::notice(format!(
                "《{}》没有旧谱（LEGACY）",
                song.title
            )));
        }
        return ok(PhigrosService::notice(SCORE_USAGE));
    };
    let profile = service.profile(&inv.user).await.map_err(PluginError::handler)?;
    let official = profile.as_ref().map(|item| item.rks).unwrap_or_default();
    let scored = service.judge(&records);
    let play = play_of(song, level, &scored);
    let standing = standing_of(&scored, &play, official);
    let events = service
        .repo()
        .score_events(&inv.user, 0)
        .await
        .map_err(|error| PluginError::handler(error.to_string()))?;
    let player = service.player(&binding, profile.as_ref());
    let ranking = score_ranking(&service, &play, &player, profile.as_ref(), &inv.user).await;
    let req = score_card::ScoreRequest {
        settlement: settlement_of(
            song,
            &play,
            player,
            standing,
            &scored,
            &events,
            gained_text(profile.as_ref()),
        ),
        ranking,
    };
    let (width, height) = score_card::canvas_size();
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, "单曲成绩", &req, move |kit| {
        score_card::draw(kit, &root, &drawn)
    });
    let title = song.title.clone();
    let out = PhigrosService::bare().card(card);
    let out = crate::buttons::with_keyboard(out, crate::buttons::Page::Song { title });
    ok(commands::with_at(out, &inv))
}

async fn run_his(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let text = inv.text.trim().to_string();
    if text.is_empty() {
        return ok(PhigrosService::notice(HIS_USAGE));
    }
    let binding = match service.require_binding(&inv.user).await {
        Ok(binding) => binding,
        Err(out) => return ok(*out),
    };
    let song = match resolve_song(&service, &text, "/p his ", "") {
        Resolved::Song(song) => song,
        Resolved::Basket(out) => return ok(out),
        Resolved::Missing(out) => return ok(out),
    };
    let profile = service.profile(&inv.user).await.map_err(PluginError::handler)?;
    let records = service.records(&inv.user).await.map_err(PluginError::handler)?;
    let events = service
        .repo()
        .score_events(&inv.user, 0)
        .await
        .map_err(|error| PluginError::handler(error.to_string()))?;
    let req = songinfo_request(
        song,
        &records,
        &events,
        service.player(&binding, profile.as_ref()),
    );
    let (width, height) = songinfo::canvas_size(&req);
    let root = service.root().to_path_buf();
    let drawn = req.clone();
    let card = service.card(width, height, "游玩历史", &req, move |kit| {
        songinfo::draw(kit, &root, &drawn)
    });
    let out = service
        .text("songinfo.image", &songinfo_text(&req))
        .card(card);
    ok(commands::with_at(out, &inv))
}

async fn run_cal(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let Some((constant, acc)) = parse_cal(&inv.text) else {
        return ok(PhigrosService::notice(CAL_USAGE));
    };
    let mut args = Args::new();
    put(&mut args, "constant", format!("{constant:.1}"));
    put(&mut args, "acc", acc_short(acc));
    put(&mut args, "rks", rks_text(rks::single_rks(acc, constant)));
    ok(commands::with_at(service.text("score.cal", &args), &inv))
}

async fn run_challenge(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let raw = inv.text.trim().to_string();
    let out = challenge_out(&service, &inv.user, &raw).await;
    ok(commands::with_at(out, &inv))
}

pub async fn challenge_out(service: &Arc<PhigrosService>, user: &str, raw: &str) -> Outgoing {
    let Some(req) = parse_challenge(raw) else {
        return PhigrosService::notice(CHALLENGE_USAGE);
    };
    let Some(result) = solve_challenge(service, &req) else {
        return PhigrosService::notice(
            "这个和数在当前难度下凑不出来，换一个数值或换几档难度再试试",
        );
    };
    let binding = service.binding(user).await.unwrap_or_default();
    let profile = service.profile(user).await.unwrap_or_default();
    let player = match binding.as_ref() {
        Some(binding) => service.player(binding, profile.as_ref()),
        None => Player::default(),
    };
    let mut args = Args::new();
    let floors: Vec<String> = result
        .charts
        .iter()
        .map(|chart| chart.floor.to_string())
        .collect();
    let real: f64 = result.charts.iter().map(|chart| chart.constant).sum();
    int(&mut args, "sum", i64::from(result.sum));
    put(&mut args, "real", format!("{real:.1}"));
    int(&mut args, "count", result.charts.len() as i64);
    put(&mut args, "floors", floors.join("+"));
    let mut rows: Vec<Arg> = Vec::new();
    for chart in &result.charts {
        let mut row = Args::new();
        put(&mut row, "title", chart.title.clone());
        put(&mut row, "difficulty", chart.difficulty.name());
        put(&mut row, "constant", format!("{:.1}", chart.constant));
        int(&mut row, "floor", i64::from(chart.floor));
        rows.push(Arg::Map(row));
    }
    args.insert(String::from("rows"), Arg::List(rows));
    let request = challenge_request(service, &result, player);
    let (width, height) = songselect::canvas_size(&request);
    let root = service.root().to_path_buf();
    let drawn = request.clone();
    let card = service.card(width, height, "课题", &request, move |kit| {
        songselect::draw(kit, &root, &drawn)
    });
    let out = service.text("challenge.image", &args).card(card);
    crate::buttons::with_keyboard(
        out,
        crate::buttons::Page::Challenge {
            raw: raw.to_string(),
        },
    )
}

async fn run_aim(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let Some(req) = parse_aim(&inv.text) else {
        return ok(PhigrosService::notice(AIM_USAGE));
    };
    let result = if req.notes > 0 {
        match solve_aim(req.notes, req.target, req.mode) {
            Some(solution) => AimResult {
                song: String::new(),
                difficulty: Difficulty::In,
                constant: 0.0,
                has_chart: false,
                solution,
            },
            None => return ok(PhigrosService::notice(aim_range(req.notes, req.target))),
        }
    } else {
        let song = match resolve_song(&service, &req.title, "/p aim ", &aim_suffix(&req)) {
            Resolved::Song(song) => song,
            Resolved::Basket(out) => return ok(out),
            Resolved::Missing(out) => return ok(out),
        };
        match aim_by_song(song, req.level, req.target, req.mode) {
            Ok(result) => result,
            Err(reason) => return ok(PhigrosService::notice(reason)),
        }
    };
    let mut args = Args::new();
    put(&mut args, "head", aim_head(&result));
    put(&mut args, "mode", aim_mode(&result));
    int(&mut args, "notes", i64::from(result.solution.notes));
    put(&mut args, "target", score_text(aim_shown_score(&result)));
    int(&mut args, "perfect", i64::from(result.solution.shape.perfect));
    int(&mut args, "good", i64::from(result.solution.shape.good));
    int(&mut args, "broken", i64::from(result.solution.shape.broken));
    put(&mut args, "combo", aim_combo(&result));
    put(&mut args, "acc", acc_text(shape_acc(&result.solution.shape)));
    put(&mut args, "rks", aim_rks(&result));
    put(&mut args, "verdict", aim_verdict(&result));
    ok(commands::with_at(service.text("snipe", &args), &inv))
}

enum Resolved<'a> {
    Song(&'a Song),
    Basket(Outgoing),
    Missing(Outgoing),
}

fn resolve_song<'a>(
    service: &'a PhigrosService,
    text: &str,
    prefix: &str,
    suffix: &str,
) -> Resolved<'a> {
    let catalog = service.catalog();
    if let Some(song) = catalog.resolve(text) {
        return Resolved::Song(song);
    }
    let hits: Vec<&Song> = catalog
        .search(text, PICK_LIMIT)
        .into_iter()
        .map(|(song, _)| song)
        .collect();
    match hits.len() {
        0 => Resolved::Missing(PhigrosService::notice(format!(
            "没有找到《{text}》，可用 /p search {text} 试试"
        ))),
        1 => Resolved::Song(hits[0]),
        total => Resolved::Basket(basket(service, text, prefix, suffix, &hits, total)),
    }
}

fn basket(
    service: &PhigrosService,
    query: &str,
    prefix: &str,
    suffix: &str,
    hits: &[&Song],
    total: usize,
) -> Outgoing {
    if suffix.is_empty() {
        return service.pick(query, prefix, hits, total);
    }
    Outgoing::new(
        ReplyKind::Handler,
        query::pick_markdown_with(query, prefix, suffix, service.catalog(), hits, total),
    )
}

fn count_override(text: &str) -> Option<i64> {
    let token = text.split_whitespace().next()?;
    token.trim_start_matches('#').parse::<i64>().ok()
}

fn aim_suffix(req: &AimRequest) -> String {
    let mode = match req.mode {
        Mode::Normal => "",
        Mode::Challenge => " -c",
    };
    format!(" {} {}{mode}", req.level.name(), req.target)
}

fn clamp_size(value: i64) -> usize {
    if value <= 0 {
        return BEST_SIZE;
    }
    (value as usize).min(crate::model::BEST50_SIZE)
}

fn entry_of(item: &Scored, rank: i32, podium: bool) -> b30::B30Entry {
    b30::B30Entry {
        rank,
        song_id: item.song_id().to_string(),
        slot: false,
        title: item.title.clone(),
        diff: String::from(item.difficulty().name()),
        constant: item.constant,
        known: item.known,
        score: item.score(),
        acc: item.acc(),
        rks: item.rks,
        suggest: item.suggest,
        suggest_ok: item.suggest_ok,
        grade: if item.unrecorded() {
            String::new()
        } else {
            String::from(item.grade.name())
        },
        podium,
        cover_luma: item.cover_luma,
    }
}

fn entries_of_best(best: &Best, count: usize) -> Vec<b30::B30Entry> {
    best.top
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| entry_of(item, index as i32 + 1, false))
        .collect()
}

fn entries_average(entries: &[b30::B30Entry]) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;
    for entry in entries {
        if entry.rks > 0.0 {
            sum += entry.rks;
            count += 1;
        }
    }
    if count == 0 {
        return 0.0;
    }
    sum / f64::from(count)
}

fn phi_average(best: &Best) -> f64 {
    let mut sum = 0.0;
    let mut count = 0;
    for item in &best.podium {
        if item.rks > 0.0 {
            sum += item.rks;
            count += 1;
        }
    }
    if count == 0 {
        return 0.0;
    }
    sum / f64::from(count)
}

fn list_average(items: &[Scored]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }
    let sum: f64 = items.iter().map(|item| item.rks).sum();
    sum / items.len() as f64
}

fn diff_stats(service: &PhigrosService, records: &[PlayRecord]) -> Vec<b30::B30DiffStat> {
    let catalog = service.catalog();
    let mut seen: Vec<(String, usize)> = Vec::new();
    let mut played = [0_i32; NUM_PLAYABLE];
    for record in records {
        let index = record.difficulty.index();
        if index >= NUM_PLAYABLE {
            continue;
        }
        let id = catalog
            .song(&record.song_id)
            .map(|song| {
                if song.variant() {
                    song.variant_of.clone()
                } else {
                    song.id.clone()
                }
            })
            .unwrap_or_else(|| record.song_id.clone());
        let key = (id, index);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        played[index] += 1;
    }
    let mut total = [0_i32; NUM_PLAYABLE];
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
    Difficulty::playable_all()
        .iter()
        .enumerate()
        .map(|(index, difficulty)| b30::B30DiffStat {
            label: String::from(difficulty.name()),
            played: played[index],
            total: total[index],
        })
        .collect()
}

fn row_text(item: &Scored, rank: &str) -> Arg {
    let mut row = Args::new();
    put(&mut row, "rank", rank);
    put(&mut row, "title", item.title.clone());
    put(&mut row, "difficulty", item.difficulty().name());
    put(&mut row, "constant", constant_text(item.constant, item.known));
    put(&mut row, "score", score_text(item.score()));
    put(&mut row, "acc", acc_text(item.acc()));
    put(&mut row, "grade", item.grade.name());
    put(&mut row, "suggest", suggest_text(item));
    Arg::Map(row)
}

fn play_of(song: &Song, level: Difficulty, scored: &[Scored]) -> Scored {
    if let Some(item) = scored
        .iter()
        .find(|item| item.song_id() == song.id && item.difficulty() == level)
    {
        return item.clone();
    }
    let mut play = Scored {
        record: PlayRecord {
            song_id: song.id.clone(),
            difficulty: level,
            ..PlayRecord::default()
        },
        title: song.title.clone(),
        composer: song.composer.clone(),
        grade: Grade::New,
        variant: song.variant(),
        cover_luma: song.cover_luma,
        ..Scored::default()
    };
    if let Some(chart) = song.chart(level) {
        play.known = true;
        play.constant = chart.constant;
        play.notes = chart.notes.combo();
    }
    play
}

fn standing_of(scored: &[Scored], item: &Scored, official: f64) -> Standing {
    let mut ranked: Vec<&Scored> = scored
        .iter()
        .filter(|entry| !entry.unrecorded() && entry.counted())
        .collect();
    ranked.sort_by(|left, right| {
        right
            .rks
            .partial_cmp(&left.rks)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.score().cmp(&left.score()))
    });
    let mut out = Standing {
        played: !item.unrecorded(),
        rks: item.rks,
        rank: 0,
        total: ranked.len() as i32,
        suggest: 0.0,
        suggest_ok: false,
    };
    for (index, entry) in ranked.iter().enumerate() {
        if entry.song_id() == item.song_id() && entry.difficulty() == item.difficulty() {
            out.rank = index as i32 + 1;
            break;
        }
    }
    if !item.known || item.unrecorded() || item.score() >= MAX_SCORE {
        return out;
    }
    let pivot = if out.rank > 0 {
        ranked[(out.rank as usize - 1).min(COUNTED_BEST - 1)].rks
    } else if ranked.is_empty() {
        0.0
    } else {
        ranked[(ranked.len() - 1).min(COUNTED_BEST - 1)].rks
    };
    let step = rks::round_up(official) * BEST_SIZE as f64;
    if let Some(acc) = rks::required_acc(pivot + step, item.constant) {
        out.suggest = acc;
        out.suggest_ok = true;
    }
    out
}

fn board_window(len: usize, self_index: Option<usize>, want: usize) -> usize {
    if len <= want {
        return 0;
    }
    let Some(index) = self_index else {
        return 0;
    };
    let start = index.saturating_sub(want / 2);
    if start + want > len {
        return len - want;
    }
    start
}

struct Ranking {
    rows: Vec<score_card::ScoreRankRow>,
    total: i32,
    global: bool,
    self_rank: i32,
}

async fn score_ranking(
    service: &Arc<PhigrosService>,
    play: &Scored,
    player: &Player,
    profile: Option<&Profile>,
    user: &str,
) -> score_card::ScoreRanking {
    let Some(rank) = ranking_of(service, play, user).await else {
        return score_card::ScoreRanking::default();
    };
    let hidden = service.repo().is_blocked(user).await.unwrap_or_default();
    assemble(rank, play, player, profile, hidden)
}

async fn ranking_of(service: &Arc<PhigrosService>, play: &Scored, user: &str) -> Option<Ranking> {
    if let Some(rank) = online_ranking(service, play, user).await {
        return Some(rank);
    }
    local_ranking(service, play, user).await
}

async fn online_ranking(
    service: &Arc<PhigrosService>,
    play: &Scored,
    user: &str,
) -> Option<Ranking> {
    let binding = service.binding(user).await.ok().flatten()?;
    if binding.token.is_empty() {
        return None;
    }
    let ask = phi::BoardAsk {
        song_id: play.song_id().to_string(),
        difficulty: play.difficulty(),
        token: binding.token,
    };
    let phi::SongRank {
        total,
        me_rank,
        rows,
    } = service.phi().song_board(&ask).await.ok()?;
    if rows.is_empty() {
        return None;
    }
    let rows = rows
        .into_iter()
        .map(|row| {
            let dan = overview::dan_of(row.challenge);
            score_card::ScoreRankRow {
                rank: row.rank,
                name: row.name,
                avatar: row.avatar,
                rks: row.rks,
                dan_level: dan.level,
                dan_progress: dan.progress,
                score: row.score,
                acc: row.acc,
                self_row: me_rank > 0 && row.rank == me_rank,
                hidden: false,
            }
        })
        .collect();
    Some(Ranking {
        rows,
        total,
        global: true,
        self_rank: me_rank,
    })
}

async fn local_ranking(
    service: &Arc<PhigrosService>,
    play: &Scored,
    user: &str,
) -> Option<Ranking> {
    let (rows, total) = service
        .repo()
        .song_board(play.song_id(), play.difficulty(), SCORE_BOARD_ROWS)
        .await
        .ok()?;
    if total <= 0 && rows.is_empty() {
        return None;
    }
    let mut self_rank = play.rank as i32;
    let mut out: Vec<score_card::ScoreRankRow> = Vec::with_capacity(rows.len());
    for (offset, row) in rows.iter().enumerate() {
        let self_row = row.user_id == user;
        if self_row {
            self_rank = offset as i32 + 1;
        }
        let dan = overview::dan_of(row.challenge_rank);
        out.push(score_card::ScoreRankRow {
            rank: offset as i32 + 1,
            name: row.nickname.clone(),
            avatar: row.avatar.clone(),
            rks: row.rks,
            dan_level: dan.level,
            dan_progress: dan.progress,
            score: row.score,
            acc: row.acc,
            self_row,
            hidden: false,
        });
    }
    Some(Ranking {
        rows: out,
        total: total as i32,
        global: false,
        self_rank,
    })
}

fn assemble(
    rank: Ranking,
    play: &Scored,
    player: &Player,
    profile: Option<&Profile>,
    hidden: bool,
) -> score_card::ScoreRanking {
    let mut out = score_card::ScoreRanking {
        diff: play.difficulty().name().to_string(),
        total: rank.total,
        global: rank.global,
        rows: Vec::new(),
    };
    let self_index = rank.rows.iter().position(|row| row.self_row);
    let start = board_window(rank.rows.len(), self_index, SCORE_BOARD_ROWS);
    for row in rank.rows.iter().skip(start).take(SCORE_BOARD_ROWS) {
        out.rows.push(score_card::ScoreRankRow {
            hidden: row.self_row && hidden,
            ..row.clone()
        });
    }
    if rank.self_rank > 0 && !out.rows.iter().any(|row| row.self_row) {
        let dan = overview::dan_of(
            profile
                .map(|item| item.challenge_mode_rank as i64)
                .unwrap_or_default(),
        );
        out.rows.push(score_card::ScoreRankRow {
            rank: rank.self_rank,
            name: player.name.clone(),
            avatar: player.avatar.clone(),
            rks: player.rks,
            dan_level: dan.level,
            dan_progress: dan.progress,
            score: play.score(),
            acc: play.acc(),
            self_row: true,
            hidden,
        });
    }
    if out.rows.len() > SCORE_BOARD_ROWS {
        if let Some(last) = out.rows.pop() {
            out.rows.truncate(SCORE_BOARD_ROWS - 1);
            out.rows.push(last);
        }
    }
    out
}

fn settlement_of(
    song: &Song,
    play: &Scored,
    player: Player,
    standing: Standing,
    scored: &[Scored],
    events: &[ScoreEvent],
    gained: String,
) -> SettlementRequest {
    let played = !play.unrecorded();
    let mut req = SettlementRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        diff: String::from(play.difficulty().name()),
        constant: play.constant,
        known: play.known,
        played,
        score: play.score(),
        acc: play.acc(),
        history: 0,
        fc: play.fc(),
        grade: if played {
            String::from(play.grade.name())
        } else {
            String::new()
        },
        standing,
        scores: difficulty_scores(song, scored),
        chart_history: chart_history(events, song, play.difficulty()),
        gained,
        ..SettlementRequest::default()
    };
    req.player = player;
    req
}

fn difficulty_scores(song: &Song, scored: &[Scored]) -> Vec<DifficultyScore> {
    let mut out: Vec<DifficultyScore> = Vec::with_capacity(NUM_PLAYABLE);
    for level in Difficulty::playable_all() {
        let play = play_of(song, *level, scored);
        let standing = standing_of(scored, &play, 0.0);
        out.push(DifficultyScore {
            diff: String::from(level.name()),
            played: !play.unrecorded(),
            score: play.score(),
            acc: play.acc(),
            fc: play.fc(),
            phi: play.score() >= MAX_SCORE,
            rks: standing.rks,
            rank: standing.rank,
            total: standing.total,
            grade: if play.unrecorded() {
                String::new()
            } else {
                String::from(play.grade.name())
            },
        });
    }
    out
}

fn chart_history(events: &[ScoreEvent], song: &Song, level: Difficulty) -> ChartHistory {
    let mut points: Vec<HistoryPoint> = Vec::new();
    for event in events {
        if event.song_id != song.id || event.difficulty != level {
            continue;
        }
        let Some(at) = chrono::DateTime::from_timestamp(event.observed_at, 0) else {
            continue;
        };
        points.push(HistoryPoint {
            at,
            score: event.new_score,
        });
    }
    let gain: i32 = events
        .iter()
        .filter(|event| {
            event.song_id == song.id && event.difficulty == level && event.has_old
        })
        .map(|event| (event.new_score - event.old_score).max(0))
        .sum();
    ChartHistory { points, gain }
}

fn songinfo_request(
    song: &Song,
    records: &[PlayRecord],
    events: &[ScoreEvent],
    player: Player,
) -> songinfo::SongInfoRequest {
    let mut req = songinfo::SongInfoRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.trim().to_string(),
        player,
        levels: Vec::new(),
    };
    for level in Difficulty::playable_all().iter().rev() {
        let mut item = songinfo::SongInfoLevel {
            diff: String::from(level.name()),
            has: false,
            played: false,
            score: 0,
            acc: 0.0,
            grade: String::new(),
            rks: 0.0,
            combo: 0,
            history: Vec::new(),
            changes: 0,
            constant: 0.0,
        };
        if let Some(chart) = song.chart(*level) {
            item.has = true;
            item.constant = chart.constant;
            item.combo = chart.notes.combo();
        }
        if let Some(record) = records
            .iter()
            .find(|record| record.song_id == song.id && record.difficulty == *level)
        {
            if record.score > 0 {
                item.played = true;
                item.score = record.score;
                item.acc = record.acc;
                item.grade = String::from(crate::model::grade_of(record.score, record.fc).name());
                item.rks = rks::single_rks(record.acc, item.constant);
            }
        }
        let (history, changes) = chart_points(events, song, *level);
        item.history = history;
        item.changes = changes;
        if item.history.is_empty() {
            continue;
        }
        req.levels.push(item);
    }
    req
}

fn chart_points(
    events: &[ScoreEvent],
    song: &Song,
    level: Difficulty,
) -> (Vec<songinfo::SongInfoPoint>, i32) {
    let matched: Vec<&ScoreEvent> = events
        .iter()
        .filter(|event| event.song_id == song.id && event.difficulty == level)
        .collect();
    let mut moved = 0;
    for event in &matched {
        if !event.has_old || event.new_score != event.old_score {
            moved += 1;
        }
    }
    let mut out: Vec<songinfo::SongInfoPoint> = Vec::new();
    for event in matched.iter().rev() {
        if out.len() >= HISTORY_LIMIT {
            break;
        }
        if event.has_old && event.new_score == event.old_score {
            continue;
        }
        let Some(at) = chrono::DateTime::from_timestamp(event.observed_at, 0) else {
            continue;
        };
        out.push(songinfo::SongInfoPoint {
            at,
            score: event.new_score,
            acc: event.new_acc,
            rks: event.new_rks,
            grade: String::from(
                crate::model::grade_of(event.new_score, event.new_fc).name(),
            ),
            delta: if event.has_old {
                event.new_score - event.old_score
            } else {
                0
            },
            first: !event.has_old,
        });
    }
    (out, moved)
}

fn songinfo_text(req: &songinfo::SongInfoRequest) -> Args {
    let mut args = Args::new();
    put(&mut args, "title", req.title.clone());
    put(&mut args, "levels", req.levels.len().to_string());
    let mut rows: Vec<Arg> = Vec::new();
    for level in &req.levels {
        let mut row = Args::new();
        put(&mut row, "diff", level.diff.clone());
        put(&mut row, "constant", constant_text(level.constant, level.has));
        put(
            &mut row,
            "score",
            if level.played {
                score_text(level.score)
            } else {
                String::from("-")
            },
        );
        put(
            &mut row,
            "acc",
            if level.played {
                acc_short(level.acc)
            } else {
                String::from("-")
            },
        );
        put(
            &mut row,
            "grade",
            if level.played {
                level.grade.clone()
            } else {
                String::from("-")
            },
        );
        put(
            &mut row,
            "rks",
            if level.played && level.has && level.rks > 0.0 {
                rks_text(level.rks)
            } else {
                String::from("-")
            },
        );
        put(&mut row, "note", songinfo_note(level));
        rows.push(Arg::Map(row));
    }
    args.insert(String::from("rows"), Arg::List(rows));
    put(
        &mut args,
        "footer",
        "成绩来自你上次拉取的存档，/p pull 可更新",
    );
    args
}

fn songinfo_note(level: &songinfo::SongInfoLevel) -> String {
    if !level.has {
        return String::from("无此谱面");
    }
    if !level.played {
        return String::from("尚无成绩");
    }
    if level.history.is_empty() {
        return String::from("还没有变化记录");
    }
    format!("共 {} 次", level.changes)
}

fn extra_card(service: &PhigrosService, song: &Song, kind: &str) -> Outgoing {
    let found = song.extra.iter().find(|extra| {
        extra.kind.eq_ignore_ascii_case(kind) || extra.display().eq_ignore_ascii_case(kind)
    });
    let Some(extra) = found else {
        return PhigrosService::notice(format!("《{}》没有 {kind} 谱面", song.title));
    };
    let mut args = Args::new();
    put(&mut args, "title", song.title.clone());
    put(&mut args, "kind", extra.kind.clone());
    put(&mut args, "constant", dash(&extra.label));
    put(&mut args, "charter", dash(&extra.charter));
    put(
        &mut args,
        "notes",
        if extra.notes.combo() > 0 {
            extra.notes.combo().to_string()
        } else {
            String::from("-")
        },
    );
    service.text("score.extra", &args)
}

fn parse_cal(text: &str) -> Option<(f64, f64)> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    if fields.len() != 2 {
        return None;
    }
    let constant = fields[0].trim_end_matches('%').parse::<f64>().ok()?;
    let acc = fields[1].trim_end_matches('%').parse::<f64>().ok()?;
    if !(constant > 0.0 && constant <= crate::model::MAX_CONSTANT) {
        return None;
    }
    if !(0.0..=100.0).contains(&acc) {
        return None;
    }
    Some((constant, acc))
}

struct ChallengeChart {
    song_id: String,
    title: String,
    difficulty: Difficulty,
    constant: f64,
    floor: i32,
}

struct ChallengeResult {
    charts: Vec<ChallengeChart>,
    sum: i32,
}

struct ChallengeRequest {
    target: Option<Span>,
    difficulties: Vec<Difficulty>,
}

fn parse_challenge(text: &str) -> Option<ChallengeRequest> {
    let mut req = ChallengeRequest {
        target: None,
        difficulties: Vec::new(),
    };
    for token in text.split_whitespace() {
        if let Some(span) = parse_span(token) {
            req.target = Some(whole_span(span)?);
            continue;
        }
        let level = Difficulty::parse(token)?;
        if level == Difficulty::Legacy {
            return None;
        }
        req.difficulties.push(level);
    }
    Some(req)
}

fn whole_span(span: Span) -> Option<Span> {
    for value in [span.min, span.max] {
        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }
    }
    Some(span)
}

fn challenge_buckets(
    service: &PhigrosService,
    levels: &[Difficulty],
) -> Vec<(i32, Vec<(usize, Difficulty)>)> {
    let mut buckets: Vec<(i32, Vec<(usize, Difficulty)>)> = Vec::new();
    for (index, song) in service.catalog().songs().iter().enumerate() {
        if song.variant() {
            continue;
        }
        for level in levels {
            let Some(chart) = song.chart(*level) else {
                continue;
            };
            let floor = chart.constant as i32;
            match buckets.iter_mut().find(|(value, _)| *value == floor) {
                Some((_, list)) => list.push((index, *level)),
                None => buckets.push((floor, vec![(index, *level)])),
            }
        }
    }
    buckets.sort_by_key(|(floor, _)| *floor);
    buckets
}

fn solve_challenge(service: &PhigrosService, req: &ChallengeRequest) -> Option<ChallengeResult> {
    let levels = if req.difficulties.is_empty() {
        Difficulty::playable_all().to_vec()
    } else {
        req.difficulties.clone()
    };
    let buckets = challenge_buckets(service, &levels);
    if buckets.is_empty() {
        return None;
    }
    let (low, high) = target_totals(req.target, max_sum(&buckets))?;
    let mut totals: Vec<i32> = (low..=high).collect();
    let mut rng = rand::thread_rng();
    for index in (1..totals.len()).rev() {
        let swap = rng.gen_range(0..=index);
        totals.swap(index, swap);
    }
    for sum in totals {
        if let Some(charts) = pick_challenge(service, &buckets, sum, &mut rng) {
            return Some(ChallengeResult { charts, sum });
        }
    }
    None
}

fn target_totals(target: Option<Span>, ceiling: i32) -> Option<(i32, i32)> {
    let Some(span) = target else {
        return Some((0, ceiling));
    };
    let mut low = 0;
    if span.min_set {
        low = span.min as i32;
        if span.min_open {
            low += 1;
        }
    }
    let mut high = ceiling;
    if span.max_set {
        let top = span.max as i32;
        if top < high {
            high = top;
        }
        if span.max_open {
            high -= 1;
        }
    }
    if low > high || high < 0 {
        return None;
    }
    Some((low, high))
}

fn max_sum(buckets: &[(i32, Vec<(usize, Difficulty)>)]) -> i32 {
    let mut best = 0;
    for left in buckets {
        for middle in buckets.iter().filter(|item| item.0 >= left.0) {
            for right in buckets.iter().filter(|item| item.0 >= middle.0) {
                if !enough(buckets, &[left.0, middle.0, right.0]) {
                    continue;
                }
                best = best.max(left.0 + middle.0 + right.0);
            }
        }
    }
    best
}

fn enough(buckets: &[(i32, Vec<(usize, Difficulty)>)], picks: &[i32]) -> bool {
    for floor in picks {
        let need = picks.iter().filter(|item| *item == floor).count();
        let have = buckets
            .iter()
            .find(|(value, _)| value == floor)
            .map(|(_, list)| list.len())
            .unwrap_or_default();
        if have < need {
            return false;
        }
    }
    true
}

fn pick_challenge(
    service: &PhigrosService,
    buckets: &[(i32, Vec<(usize, Difficulty)>)],
    sum: i32,
    rng: &mut impl Rng,
) -> Option<Vec<ChallengeChart>> {
    let mut combos: Vec<[i32; 3]> = Vec::new();
    for left in buckets {
        for middle in buckets.iter().filter(|item| item.0 >= left.0) {
            for right in buckets.iter().filter(|item| item.0 >= middle.0) {
                if left.0 + middle.0 + right.0 != sum {
                    continue;
                }
                if !enough(buckets, &[left.0, middle.0, right.0]) {
                    continue;
                }
                combos.push([left.0, middle.0, right.0]);
            }
        }
    }
    if combos.is_empty() {
        return None;
    }
    let spread = |combo: &[i32; 3]| combo[2] - combo[0];
    let best = combos.iter().map(spread).min()?;
    let kept: Vec<[i32; 3]> = combos
        .into_iter()
        .filter(|combo| spread(combo) <= best + CHALLENGE_SPREAD)
        .collect();
    let chosen = kept[rng.gen_range(0..kept.len())];
    let mut used: Vec<(usize, Difficulty)> = Vec::with_capacity(3);
    let mut charts: Vec<ChallengeChart> = Vec::with_capacity(3);
    for floor in chosen {
        let (_, list) = buckets.iter().find(|(value, _)| *value == floor)?;
        let start = rng.gen_range(0..list.len());
        let mut found = None;
        for offset in 0..list.len() {
            let entry = list[(start + offset) % list.len()];
            if used.contains(&entry) {
                continue;
            }
            found = Some(entry);
            break;
        }
        let (index, level) = found?;
        used.push((index, level));
        let song = service.catalog().songs().get(index)?;
        let chart = song.chart(level)?;
        charts.push(ChallengeChart {
            song_id: song.id.clone(),
            title: song.title.clone(),
            difficulty: level,
            constant: chart.constant,
            floor,
        });
    }
    Some(charts)
}

fn challenge_request(
    service: &PhigrosService,
    result: &ChallengeResult,
    player: Player,
) -> songselect::SongSelectRequest {
    let mut req = songselect::SongSelectRequest {
        backdrop: String::new(),
        song_id: String::new(),
        title: String::new(),
        composer: String::new(),
        diff: String::new(),
        constant: 0.0,
        known: false,
        levels: Vec::new(),
        rows: Vec::new(),
        strip: Vec::new(),
        player,
    };
    let Some(first) = result.charts.first() else {
        return req;
    };
    req.backdrop = first.song_id.clone();
    req.song_id = first.song_id.clone();
    req.title = first.title.clone();
    if let Some(song) = service.catalog().song(&first.song_id) {
        req.composer = song.composer.clone();
    }
    req.diff = String::from(first.difficulty.name());
    req.constant = first.constant;
    req.known = true;
    let mut rows: Vec<songselect::SongSelectRow> =
        Vec::with_capacity(songselect::SEL_ROW_SEL + result.charts.len());
    for _ in 0..songselect::SEL_ROW_SEL {
        rows.push(songselect::SongSelectRow {
            title: String::new(),
            constant: 0.0,
            known: false,
            selected: false,
        });
    }
    for chart in &result.charts {
        rows.push(songselect::SongSelectRow {
            title: chart.title.clone(),
            constant: chart.constant,
            known: true,
            selected: chart.song_id == first.song_id && chart.difficulty == first.difficulty,
        });
    }
    req.rows = rows;
    let mut best = 0;
    for (index, chart) in result.charts.iter().enumerate() {
        let top = &result.charts[best];
        if chart.floor > top.floor || (chart.floor == top.floor && chart.constant > top.constant) {
            best = index;
        }
    }
    for (index, chart) in result.charts.iter().enumerate() {
        let badge = chart.floor.to_string();
        req.levels.push(songselect::SongSelectLevel {
            diff: String::from(chart.difficulty.name()),
            const_text: badge.clone(),
            active: index == best,
        });
        req.strip.push(songselect::SongSelectThumb {
            song_id: chart.song_id.clone(),
            badge,
        });
    }
    req
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Challenge,
}

#[derive(Clone, Copy, Default)]
struct Shape {
    perfect: i32,
    good: i32,
    broken: i32,
    combo: i32,
}

#[derive(Clone, Copy)]
struct Solution {
    shape: Shape,
    mode: Mode,
    notes: i32,
    target: i32,
    reached: i32,
    exact: bool,
}

struct AimRequest {
    title: String,
    level: Difficulty,
    notes: i32,
    target: i32,
    mode: Mode,
}

struct AimResult {
    song: String,
    difficulty: Difficulty,
    constant: f64,
    has_chart: bool,
    solution: Solution,
}

fn shape_acc(shape: &Shape) -> f64 {
    let notes = shape_notes(shape);
    if notes == 0 {
        return 0.0;
    }
    f64::from(100 * shape.perfect + 65 * shape.good) / f64::from(notes)
}

fn shape_notes(shape: &Shape) -> i32 {
    shape.perfect + shape.good + shape.broken
}

fn score_of(mode: Mode, notes: i32, shape: &Shape) -> i32 {
    if notes <= 0 {
        return 0;
    }
    let den = 20_i64 * i64::from(notes);
    let weighted = 20 * i64::from(shape.perfect) + 13 * i64::from(shape.good);
    let num = match mode {
        Mode::Challenge => 1_000_000 * weighted,
        Mode::Normal => 900_000 * weighted + 2_000_000 * i64::from(shape.combo),
    };
    ((num + den / 2) / den) as i32
}

fn is_mode_token(token: &str) -> bool {
    matches!(
        token.to_lowercase().as_str(),
        "课题" | "课题模式" | "challenge" | "-c" | "--challenge"
    )
}

fn parse_aim(text: &str) -> Option<AimRequest> {
    let mut fields: Vec<String> = Vec::new();
    let mut mode = Mode::Normal;
    for field in text.split_whitespace() {
        if is_mode_token(field) {
            mode = Mode::Challenge;
            continue;
        }
        fields.push(field.to_string());
    }
    if fields.len() < 2 {
        return None;
    }
    let target = fields.pop()?.parse::<i32>().ok()?;
    if !(0..=MAX_SCORE).contains(&target) {
        return None;
    }
    let mut level = Difficulty::In;
    if let Some(parsed) = fields.last().and_then(|token| Difficulty::parse(token)) {
        if !parsed.valid() {
            return None;
        }
        level = parsed;
        fields.pop();
    }
    if fields.is_empty() {
        return None;
    }
    if fields.len() == 1 {
        if let Ok(notes) = fields[0].parse::<i32>() {
            if notes <= 0 || notes > MAX_NOTES {
                return None;
            }
            return Some(AimRequest {
                title: String::new(),
                level,
                notes,
                target,
                mode,
            });
        }
    }
    Some(AimRequest {
        title: fields.join(" "),
        level,
        notes: 0,
        target,
        mode,
    })
}

fn aim_range(notes: i32, target: i32) -> String {
    format!(
        "物量取 1~{MAX_NOTES}、目标分取 0~{MAX_SCORE}（收到物量 {notes}、目标分 {target}）"
    )
}

fn aim_by_song(
    song: &Song,
    level: Difficulty,
    target: i32,
    mode: Mode,
) -> Result<AimResult, String> {
    let Some(chart) = song.chart(level) else {
        let names: Vec<&str> = song.difficulties().iter().map(|item| item.name()).collect();
        let list = if names.is_empty() {
            String::from("谱面信息")
        } else {
            names.join(" / ")
        };
        return Err(format!(
            "《{}》没有 {} 谱面（它有 {list}）",
            song.title,
            level.name()
        ));
    };
    let notes = chart.notes.combo();
    if notes <= 0 {
        return Err(format!(
            "《{}》{} 的物量暂缺；查到物量后可用 /p aim <物量> {target} 直接算",
            song.title,
            level.name()
        ));
    }
    match solve_aim(notes, target, mode) {
        Some(solution) => Ok(AimResult {
            song: song.title.clone(),
            difficulty: level,
            constant: chart.constant,
            has_chart: chart.known(),
            solution,
        }),
        None => Err(aim_range(notes, target)),
    }
}

fn solve_aim(notes: i32, target: i32, mode: Mode) -> Option<Solution> {
    if notes <= 0 || notes > MAX_NOTES || !(0..=MAX_SCORE).contains(&target) {
        return None;
    }
    if let Some(shape) = exact_shape(notes, target, mode) {
        return Some(Solution {
            shape,
            mode,
            notes,
            target,
            reached: target,
            exact: true,
        });
    }
    let shape = nearest_shape(notes, target, mode)?;
    Some(Solution {
        shape,
        mode,
        notes,
        target,
        reached: score_of(mode, notes, &shape),
        exact: false,
    })
}

fn molecular(notes: i32, target: i32, mode: Mode) -> Option<i64> {
    let lo = (2 * i64::from(target) - 1) * i64::from(notes);
    let hi = (2 * i64::from(target) + 1) * i64::from(notes);
    let mut value = lo / 10_000;
    if lo > 0 && lo % 10_000 != 0 {
        value += 1;
    }
    if value < 0 || value * 10_000 >= hi {
        return None;
    }
    if mode == Mode::Challenge && value % 10 != 0 {
        return None;
    }
    Some(value)
}

fn exact_shape(notes: i32, target: i32, mode: Mode) -> Option<Shape> {
    let value = molecular(notes, target, mode)?;
    let shape = match_molecular(notes, mode, value)?;
    if score_of(mode, notes, &shape) != target {
        return None;
    }
    Some(shape)
}

fn match_molecular(notes: i32, mode: Mode, value: i64) -> Option<Shape> {
    for good in 0..=notes {
        if let Some(shape) = match_good(notes, mode, value, good) {
            return Some(shape);
        }
    }
    None
}

fn match_good(notes: i32, mode: Mode, value: i64, good: i32) -> Option<Shape> {
    if good < 0 || good > notes {
        return None;
    }
    if mode == Mode::Challenge {
        if value % 10 != 0 {
            return None;
        }
        let rest = value / 10 - 13 * i64::from(good);
        if rest < 0 || rest % 20 != 0 {
            return None;
        }
        let perfect = (rest / 20) as i32;
        if perfect + good > notes {
            return None;
        }
        return Some(Shape {
            perfect,
            good,
            broken: notes - perfect - good,
            combo: perfect + good,
        });
    }
    let term = value - 117 * i64::from(good);
    if term < 0 || term % 20 != 0 {
        return None;
    }
    pick_perfect(notes, good, term / 20)
}

fn pick_perfect(notes: i32, good: i32, quotient: i64) -> Option<Shape> {
    let low = 0.max(ceil_div(quotient - i64::from(notes), 9))
        .max(ceil_div(quotient - i64::from(good), 10));
    let mut high = (notes - good).min((quotient / 9) as i32);
    if low > high {
        return None;
    }
    let perfect = notes - good;
    if perfect >= low && perfect <= high && quotient - 9 * i64::from(perfect) == i64::from(notes) {
        return Some(Shape {
            perfect,
            good,
            broken: 0,
            combo: notes,
        });
    }
    high = high.min(notes - good - 1);
    if high < low {
        return None;
    }
    let feasible = |value: i32| -> bool {
        let broken = notes - value - good;
        (quotient - 9 * i64::from(value)) * i64::from(broken + 1) >= i64::from(value + good)
    };
    let (mut left, mut right) = (low, high);
    while left < right {
        let middle = (left + right + 1) / 2;
        if feasible(middle) {
            left = middle;
        } else {
            right = middle - 1;
        }
    }
    if !feasible(left) {
        return None;
    }
    let mut best = low;
    for candidate in [left, clamp_int((quotient + 5) as i32 / 10, low, left)] {
        if closer(candidate, best, quotient) {
            best = candidate;
        }
    }
    Some(Shape {
        perfect: best,
        good,
        broken: notes - best - good,
        combo: (quotient - 9 * i64::from(best)) as i32,
    })
}

const NEAREST_SPAN: i64 = 512;

fn nearest_shape(notes: i32, target: i32, mode: Mode) -> Option<Shape> {
    let ideal = (2 * i64::from(target) * i64::from(notes) + 5000) / 10_000;
    let mut below: Option<(Shape, i32)> = None;
    let mut above: Option<(Shape, i32)> = None;
    let mut step = 0;
    while step <= NEAREST_SPAN && (below.is_none() || above.is_none()) {
        let offsets: [i64; 2] = if step == 0 {
            [ideal, ideal]
        } else {
            [ideal + step, ideal - step]
        };
        let count = if step == 0 { 1 } else { 2 };
        for value in &offsets[..count] {
            if *value < 0 || (below.is_some() && above.is_some()) {
                continue;
            }
            let Some(shape) = match_molecular(notes, mode, *value) else {
                continue;
            };
            let score = score_of(mode, notes, &shape);
            if score <= target {
                let off = target - score;
                if below.is_none() || off < below.map(|item| item.1).unwrap_or(i32::MAX) {
                    below = Some((shape, off));
                }
            } else {
                let off = score - target;
                if above.is_none() || off < above.map(|item| item.1).unwrap_or(i32::MAX) {
                    above = Some((shape, off));
                }
            }
        }
        step += 1;
    }
    match (below, above) {
        (Some((shape, low)), Some((other, high))) => {
            if low < high {
                Some(shape)
            } else {
                Some(other)
            }
        }
        (Some((shape, _)), None) => Some(shape),
        (None, Some((shape, _))) => Some(shape),
        (None, None) => None,
    }
}

fn ceil_div(a: i64, b: i64) -> i32 {
    let mut quotient = a / b;
    if a > 0 && a % b != 0 {
        quotient += 1;
    }
    quotient as i32
}

fn clamp_int(value: i32, low: i32, high: i32) -> i32 {
    value.max(low).min(high)
}

fn closer(left: i32, right: i32, quotient: i64) -> bool {
    let lhs = (10 * i64::from(left) - quotient).abs();
    let rhs = (10 * i64::from(right) - quotient).abs();
    if lhs != rhs {
        return lhs < rhs;
    }
    left > right
}

fn aim_shown_score(result: &AimResult) -> i32 {
    if result.solution.exact {
        result.solution.target
    } else {
        result.solution.reached
    }
}

fn aim_head(result: &AimResult) -> String {
    if result.song.is_empty() {
        return String::from("按物量算");
    }
    let mut head = format!("{} {}", result.song, result.difficulty.name());
    if result.has_chart {
        head.push(' ');
        head.push_str(&constant_text(result.constant, true));
    }
    head
}

fn aim_mode(result: &AimResult) -> &'static str {
    if result.solution.mode == Mode::Challenge {
        "课题模式（不计连击）"
    } else {
        "普通模式"
    }
}

fn aim_combo(result: &AimResult) -> String {
    if result.solution.mode == Mode::Challenge {
        return String::from("不计连击");
    }
    result.solution.shape.combo.to_string()
}

fn aim_rks(result: &AimResult) -> String {
    if result.solution.mode == Mode::Challenge || !result.has_chart {
        return String::from("—");
    }
    let value = rks::single_rks(shape_acc(&result.solution.shape), result.constant);
    if value <= 0.0 {
        return format!("{}（ACC 不到 70%，这条成绩不计单曲 RKS）", rks_text(0.0));
    }
    rks_text(value)
}

fn aim_verdict(result: &AimResult) -> String {
    let solution = &result.solution;
    let target = score_text(solution.target);
    if !solution.exact {
        return format!(
            "{target} 打不出来：这张谱每 {} 分一档，最接近的是 {}（差 {}）",
            lattice_step(solution.notes),
            score_text(solution.reached),
            (solution.reached - solution.target).abs()
        );
    }
    if solution.mode == Mode::Challenge {
        return format!("按这套方案的构成打，课题模式结算就是 {target}（不计连击，断连放哪都行）");
    }
    format!(
        "按这套方案的构成打就是 {target}，连击要正好 {}（这张谱一连 {} 分）",
        solution.shape.combo,
        per_combo(solution.notes)
    )
}

fn per_combo(notes: i32) -> i32 {
    if notes <= 0 || notes > 100_000 {
        return 0;
    }
    (100_000 + notes / 2) / notes
}

fn lattice_step(notes: i32) -> i32 {
    if notes <= 0 || notes > 5000 {
        return 0;
    }
    ((5000 + notes / 2) / notes).max(1)
}

fn gained_text(profile: Option<&Profile>) -> String {
    let Some(profile) = profile else {
        return String::new();
    };
    for index in (0..NUM_MONEY).rev() {
        let value = profile.counters.money[index];
        if value != 0 {
            return format!("{:.2} {}", f64::from(value), MONEY_UNITS[index]);
        }
    }
    String::new()
}

fn synced_text(binding: &Binding) -> String {
    let at = if binding.save_modified_at > 0 {
        binding.save_modified_at
    } else {
        binding.last_sync_at
    };
    if at <= 0 {
        return String::new();
    }
    match chrono::DateTime::from_timestamp(at, 0) {
        Some(stamp) => stamp
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        None => String::new(),
    }
}

fn rks_text(value: f64) -> String {
    format!("{value:.4}")
}

fn acc_text(value: f64) -> String {
    format!("{value:.4}%")
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

fn constant_text(value: f64, known: bool) -> String {
    if !known {
        return String::from("未收录");
    }
    format!("{value:.1}")
}

fn suggest_text(item: &Scored) -> String {
    if item.unrecorded() {
        return String::from("尚未游玩");
    }
    if item.score() >= MAX_SCORE {
        return String::from("已满分");
    }
    if !item.known {
        return String::from("曲库未收录该谱面，无法计算");
    }
    if !item.suggest_ok {
        return String::from("已达该定数上限");
    }
    format!("{} 即可提分", acc_short(item.suggest))
}

fn dash(text: &str) -> String {
    if text.trim().is_empty() {
        return String::from("-");
    }
    text.to_string()
}
fn int(args: &mut Args, key: &str, value: i64) {
    args.insert(key.to_string(), Arg::Int(value));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_override_reads_leading_number_only() {
        assert_eq!(count_override("36"), Some(36));
        assert_eq!(count_override("#12 IN"), Some(12));
        assert_eq!(count_override("best30"), None);
        assert_eq!(count_override(""), None);
        assert_eq!(clamp_size(0), BEST_SIZE);
        assert_eq!(clamp_size(36), 36);
        assert_eq!(clamp_size(999), crate::model::BEST50_SIZE);
    }

    #[test]
    fn cal_parse_accepts_percent_and_rejects_out_of_range() {
        assert_eq!(parse_cal("15.7 99.5"), Some((15.7, 99.5)));
        assert_eq!(parse_cal("15.7% 99.5%"), Some((15.7, 99.5)));
        assert_eq!(parse_cal("15.7"), None);
        assert_eq!(parse_cal("19 99"), None);
        assert_eq!(parse_cal("15.7 101"), None);
    }

    #[test]
    fn aim_parse_keeps_title_and_notes_paths_apart() {
        let chart = parse_aim("Rrhar'il AT 114514").unwrap();
        assert_eq!(chart.title, "Rrhar'il");
        assert_eq!(chart.level, Difficulty::At);
        assert_eq!(chart.target, 114514);
        assert_eq!(chart.notes, 0);
        let notes = parse_aim("2222 114514").unwrap();
        assert_eq!(notes.notes, 2222);
        assert!(parse_aim("2222 114514").is_some_and(|item| item.title.is_empty()));
        let challenge = parse_aim("课题 BANGING STRIKE IN 907890").unwrap();
        assert_eq!(challenge.mode, Mode::Challenge);
        assert_eq!(challenge.title, "BANGING STRIKE");
        assert!(parse_aim("114514").is_none());
    }

    #[test]
    fn aim_solver_hits_target_or_nearest() {
        let exact = solve_aim(1000, 1_000_000, Mode::Normal).unwrap();
        assert_eq!(exact.shape.perfect, 1000);
        assert!(exact.exact);
        let value = score_of(Mode::Normal, 1000, &exact.shape);
        assert_eq!(value, 1_000_000);
        let near = solve_aim(1000, 999_685, Mode::Normal).unwrap();
        assert_eq!(near.reached, 999_685);
        assert_eq!(shape_notes(&near.shape), 1000);
        let challenge = solve_aim(800, 900_000, Mode::Challenge).unwrap();
        assert_eq!(
            score_of(Mode::Challenge, 800, &challenge.shape),
            challenge.reached
        );
        assert!(solve_aim(0, 900_000, Mode::Normal).is_none());
        assert!(solve_aim(3000, 900_000, Mode::Normal).is_none());
    }

    #[test]
    fn challenge_totals_bounded_by_bucket_pool() {
        let buckets: Vec<(i32, Vec<(usize, Difficulty)>)> = vec![
            (12, vec![(0, Difficulty::In), (1, Difficulty::In)]),
            (
                13,
                vec![(2, Difficulty::In), (3, Difficulty::Hd), (4, Difficulty::In)],
            ),
        ];
        assert_eq!(max_sum(&buckets), 13 + 13 + 13);
        assert_eq!(target_totals(None, max_sum(&buckets)), Some((0, 39)));
        assert_eq!(target_totals(Some(Span::between(40.0, 50.0)), 39), None);
        assert_eq!(target_totals(Some(Span::at_least(30.0)), 39), Some((30, 39)));
        assert_eq!(target_totals(Some(Span::at_most(12.0)), 39), Some((0, 12)));
        assert_eq!(target_totals(Some(Span::greater_than(38.0)), 39), Some((39, 39)));
        assert_eq!(target_totals(Some(Span::greater_than(39.0)), 39), None);
        assert_eq!(target_totals(Some(Span::less_than(0.0)), 39), None);
        assert!(!enough(&buckets, &[13, 13, 13, 13]));
    }

    fn sample_rank_row(rank: i32, self_row: bool) -> score_card::ScoreRankRow {
        score_card::ScoreRankRow {
            rank,
            self_row,
            ..score_card::ScoreRankRow::default()
        }
    }

    #[test]
    fn ranking_keeps_the_source_and_parks_self_last_when_missing() {
        let play = Scored::default();
        let player = Player::default();
        let online = Ranking {
            rows: vec![sample_rank_row(23_661, true), sample_rank_row(23_662, false)],
            total: 109_646,
            global: true,
            self_rank: 23_661,
        };
        let drawn = assemble(online, &play, &player, None, false);
        assert!(drawn.global);
        assert_eq!(drawn.total, 109_646);
        assert_eq!(score_card::score_board_count(&drawn), "全服共 109'646 人");
        assert_eq!(drawn.rows.len(), 2);
        assert!(drawn.rows[0].self_row);
        let local = Ranking {
            rows: vec![sample_rank_row(1, false), sample_rank_row(2, false)],
            total: 119,
            global: false,
            self_rank: 42,
        };
        let drawn = assemble(local, &play, &player, None, false);
        assert!(!drawn.global);
        assert_eq!(score_card::score_board_count(&drawn), "共 119 人");
        assert_eq!(drawn.rows.len(), 3);
        assert!(drawn.rows[2].self_row);
        assert_eq!(drawn.rows[2].rank, 42);
    }
}
