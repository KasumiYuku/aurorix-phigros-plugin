use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use aurorix_plugin::{CommandSpec, Invocation, Outgoing, PluginError};
use aurorix_template::{Arg, Args};

use crate::cards;
use crate::commands;
use crate::domain::catalog::Catalog;
use crate::domain::query::{self, ListGroup, ListOrder, Ordering, Parsed};
use crate::domain::rks;
use crate::domain::span::Span;
use crate::model::{
    Chapter, Difficulty, Grade, Notes, Scored, Song, CLEAR_SCORE, GRADE_COUNT,
    MAX_SCORE, NUM_PLAYABLE,
};
use crate::service::PhigrosService;
use crate::settings::Settings;

pub const LIST_PREFIX: &str = "p list ";
pub const STATS_PREFIX: &str = "p stats ";
pub const SUGGEST_PREFIX: &str = "p suggest ";
pub const TABLE_PREFIX: &str = "p table ";
pub const CHAP_PREFIX: &str = "p chap ";

pub const LIST_DEFAULT: usize = 300;
pub const LIST_MAX: usize = 300;
pub const STATS_SHORT: usize = 6;
pub const STATS_SHORT_MAX: usize = 20;

pub const STATS_CONST_MAX: usize = 30;

const PHI_PER_PAGE: usize = 4;

const ENTRY_PER_PAGE: usize = 10;

const LABEL_LIST: &str = "成绩清单";
const LABEL_STATS: &str = "成绩统计";
const LABEL_SUGGEST: &str = "推分建议";
const LABEL_TABLE: &str = "定数表";
const LABEL_CHAPTER: &str = "章节进度";
const LABEL_CHAPTER_DETAIL: &str = "章节详情";

const CHAP_FOOTER: &str = "数据来自你上次拉取的存档，/p pull 可更新";

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("list")
            .alias("筛选")
            .alias("成绩筛选")
            .describe("按条件筛选成绩")
            .handler(commands::on(service, run_list)),
        CommandSpec::new("stats")
            .alias("统计")
            .alias("区间")
            .describe("定数区间成绩统计")
            .handler(commands::on(service, run_stats)),
        CommandSpec::new("suggest")
            .alias("push")
            .alias("推分")
            .describe("推分建议")
            .handler(commands::on(service, run_suggest)),
        CommandSpec::new("table")
            .alias("const")
            .alias("定数")
            .alias("定数表")
            .describe("按定数 / 难度 / 章节查曲库里的谱面")
            .handler(commands::on(service, run_table)),
        CommandSpec::new("chap")
            .alias("章节")
            .alias("章节进度")
            .describe("查看各章节完成度 / 单章详情")
            .handler(commands::on(service, run_chap)),
    ]
}

async fn run_list(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = list_out(&service, &inv.user, inv.text.trim(), 1).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_stats(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = stats_out(&service, &inv.user, inv.text.trim(), STATS_SHORT).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_suggest(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = suggest_out(&service, &inv.user, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_table(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = table_out(&service, &inv.user, inv.text.trim(), 1).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_chap(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let text = inv.text.trim().to_string();
    let out = if text.is_empty() {
        chapter_out(&service, &inv.user).await
    } else {
        chapter_detail_out(&service, &inv.user, &text).await
    };
    commands::ok(commands::with_at(out, &inv))
}

pub fn score_text(score: i32) -> String {
    let digits = score.to_string();
    if score < 0 {
        return digits;
    }
    let mut out = String::with_capacity(digits.len() + 2);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(ch);
    }
    out
}

pub fn acc_text(acc: f64) -> String {
    format!("{acc:.4}%")
}

pub fn acc_short(acc: f64) -> String {
    format!("{acc:.2}%")
}

pub fn constant_text(value: f64, known: bool) -> String {
    if !known {
        return String::from("未收录");
    }
    format!("{value:.1}")
}

pub fn rks_text(value: f64) -> String {
    format!("{value:.4}")
}

pub fn signed_text(value: f64) -> String {
    format!("{value:+.4}")
}

pub fn percent_text(played: usize, total: usize) -> String {
    if total == 0 {
        return String::from("-");
    }
    let ratio = played as f64 / total as f64 * 100.0;
    format!("{}%", ratio.round() as i64)
}

pub fn page_limit(settings: &Settings, requested: usize) -> usize {
    if requested == 0 {
        settings.list_limit
    } else {
        requested.min(settings.list_max)
    }
}

pub fn page_count(total: usize, limit: usize) -> usize {
    if limit == 0 || total == 0 {
        return 1;
    }
    total.div_ceil(limit)
}

pub fn page_slice(total: usize, page: usize, limit: usize) -> (usize, usize) {
    if limit == 0 {
        return (0, 0);
    }
    let pages = page_count(total, limit);
    let page = page.clamp(1, pages);
    let start = (page - 1) * limit;
    (start, (start + limit).min(total))
}

pub fn list_usage() -> String {
    format!(
        "用法：/p list [条件…] [状态] [排序] [分组] [nN]，顺序不限、可任意组合\n  定数    15 / 15+ / 14-16（裸数字，写整数算整档：15 = 15.0-15.9）\n  难度    ez / hd / in / at　评级 s / a / v / fc / phi\n  成绩    acc99+ / score990000+ / rks15+　曲名或曲包名 直接写片段\n  状态    nofc 未全连 / nophi 未满分 / counted 只算计入 RKS / unknown 未收录 / variant 变体\n  排序    byconst / byrks / byacc / byscore / byrank / bytitle，加 desc 倒序（默认按曲名 A-Z）\n  分组    gdiff（默认）/ gconst / ggrade / gnone\n  条数    n{LIST_DEFAULT}（默认 {LIST_DEFAULT}，上限 {LIST_MAX}）\n例如：/p list 14-16 in nofc byconst n{LIST_MAX}、/p list acc99+ desc"
    )
}

pub fn stats_usage() -> String {
    format!(
        "用法：/p stats <定数区间> [难度] [开关…]，顺序不限\n  细分    d / diff 只看难度 · c / const 只看定数（0.1 档）\n  缺口    nofc 未全连 · nophi 未满分 · noclear 未通关 · unknown 未收录\n  nN      短板列 N 条（默认 {STATS_SHORT}，n0 不列）\n  默认    完成度与定数细分都画，短板 {STATS_SHORT} 条，看这一段全部成绩\n例如：/p stats 14-16、/p stats 15+ in、/p stats 15 nofc n10、/p stats 15 c noclear"
    )
}

pub fn push_usage() -> String {
    String::from("用法：/p push [定数区间] [难度]\n例如：/p push、/p push 15+、/p push 14-16 IN")
}

pub fn table_usage() -> String {
    format!(
        "用法：/p table [关键词/曲包名] [定数] [难度] [nN] [按定数|按曲名]\n定数写法：14（整档）· 13.5（精确）· 10-15 · >=14 · 14+\n曲包：直接写包名（方舟蜃景）或写成 曲包:方舟蜃景\n条数：nN；不写默认画 {LIST_DEFAULT} 行，最多 {LIST_MAX}"
    )
}

fn map_arg(pairs: Vec<(&str, Arg)>) -> Arg {
    let mut out: BTreeMap<String, Arg> = BTreeMap::new();
    for (key, value) in pairs {
        out.insert(key.to_string(), value);
    }
    Arg::Map(out)
}

fn scored_key(song_id: &str, difficulty: Difficulty) -> String {
    format!("{song_id}#{}", difficulty.index())
}

fn record_map(scored: &[Scored]) -> HashMap<String, &Scored> {
    let mut out: HashMap<String, &Scored> = HashMap::new();
    for item in scored {
        if item.unrecorded() {
            continue;
        }
        out.insert(scored_key(item.song_id(), item.difficulty()), item);
    }
    out
}

#[derive(Clone, Debug, Default)]
pub struct Filters {
    pub constant: Span,
    pub difficulties: Vec<Difficulty>,
    pub titles: Option<HashSet<String>>,
    pub chapters: Option<HashSet<String>>,
    pub score: Span,
    pub acc: Span,
    pub rks: Span,
    pub grades: Vec<Grade>,
    pub un_fc: bool,
    pub un_phi: bool,
    pub counted: bool,
    pub unknown: bool,
    pub variant: bool,
}

impl Filters {
    pub fn chart(&self, song: &Song, difficulty: Difficulty, constant: f64) -> bool {
        if !self.difficulties.is_empty() && !self.difficulties.contains(&difficulty) {
            return false;
        }
        if self.constant.on() && !self.constant.contains(constant) {
            return false;
        }
        self.song(song)
    }

    pub fn song(&self, song: &Song) -> bool {
        if let Some(titles) = &self.titles {
            if !titles.contains(&song.id) {
                return false;
            }
        }
        if let Some(chapters) = &self.chapters {
            if !chapters.contains(&song.id) {
                return false;
            }
        }
        true
    }
}

fn chapter_matches(chapter: &Chapter, needle: &str) -> bool {
    chapter.name.to_lowercase().contains(needle)
        || chapter.display.to_lowercase().contains(needle)
        || chapter
            .aliases
            .iter()
            .any(|alias| alias.to_lowercase().contains(needle))
}

fn chapter_named(chapters: &[Chapter], needle: &str) -> bool {
    chapters.iter().any(|chapter| {
        chapter.name.trim().to_lowercase() == needle
            || chapter.display.trim().to_lowercase() == needle
            || chapter
                .aliases
                .iter()
                .any(|alias| alias.trim().to_lowercase() == needle)
    })
}

fn resolve_chapter_in(chapters: &[Chapter], song_hit: bool, parsed: &mut Parsed) {
    if !parsed.query.chapter.trim().is_empty() || parsed.query.title.trim().is_empty() {
        return;
    }
    let needle = parsed.query.title.trim().to_lowercase();
    let hit = chapter_named(chapters, &needle)
        || (!song_hit && chapters.iter().any(|chapter| chapter_matches(chapter, &needle)));
    if !hit {
        return;
    }
    parsed.query.chapter = needle;
    parsed.query.title.clear();
}

pub fn resolve_chapter_keyword(catalog: &Catalog, parsed: &mut Parsed) {
    if !parsed.query.chapter.trim().is_empty() || parsed.query.title.trim().is_empty() {
        return;
    }
    let needle = parsed.query.title.trim().to_lowercase();
    let song_hit = !catalog.search(&needle, 1).is_empty();
    resolve_chapter_in(catalog.chapters(), song_hit, parsed);
}

fn chapter_ids(catalog: &Catalog, needle: &str) -> HashSet<String> {
    let matched: HashSet<&str> = catalog
        .chapters()
        .iter()
        .filter(|chapter| chapter_matches(chapter, needle))
        .map(|chapter| chapter.name.as_str())
        .collect();
    catalog
        .songs()
        .iter()
        .filter(|song| matched.contains(song.chapter.as_str()))
        .map(|song| song.id.clone())
        .collect()
}

pub fn build_filters(catalog: &Catalog, parsed: &Parsed) -> Filters {
    let query = &parsed.query;
    let titles = (!query.title.is_empty()).then(|| {
        catalog
            .search(&query.title, LIST_MAX * 2)
            .into_iter()
            .map(|(song, _)| song.id.clone())
            .collect()
    });
    let chapters = (!query.chapter.is_empty())
        .then(|| chapter_ids(catalog, &query.chapter.to_lowercase()));
    Filters {
        constant: query.constant,
        difficulties: query.difficulties.clone(),
        titles,
        chapters,
        score: query.filter.score,
        acc: query.filter.acc,
        rks: query.filter.rks,
        grades: query.filter.grades.clone(),
        un_fc: query.states.un_fc,
        un_phi: query.states.un_phi,
        counted: query.states.counted,
        unknown: query.states.unknown,
        variant: query.states.variant,
    }
}

pub fn matches(filters: &Filters, item: &Scored) -> bool {
    if filters.constant.on() && (!item.known || !filters.constant.contains(item.constant)) {
        return false;
    }
    if !filters.difficulties.is_empty() && !filters.difficulties.contains(&item.difficulty()) {
        return false;
    }
    if let Some(titles) = &filters.titles {
        if !titles.contains(item.song_id()) {
            return false;
        }
    }
    if let Some(chapters) = &filters.chapters {
        if !chapters.contains(item.song_id()) {
            return false;
        }
    }
    if filters.score.on() && !filters.score.contains(f64::from(item.score())) {
        return false;
    }
    if filters.acc.on() && !filters.acc.contains(item.acc()) {
        return false;
    }
    if filters.rks.on() && !filters.rks.contains(item.rks) {
        return false;
    }
    if !filters.grades.is_empty() && !filters.grades.contains(&item.grade) {
        return false;
    }
    if filters.counted && !item.counted() {
        return false;
    }
    if filters.un_fc && item.fc() {
        return false;
    }
    if filters.un_phi && item.grade == Grade::Phi {
        return false;
    }
    if filters.unknown && item.known {
        return false;
    }
    if filters.variant && !item.variant {
        return false;
    }
    true
}

fn rank_map(scored: &[Scored]) -> HashMap<String, i32> {
    let mut ordered: Vec<&Scored> = scored.iter().filter(|item| !item.unrecorded()).collect();
    ordered.sort_by(|left, right| {
        right
            .rks
            .partial_cmp(&left.rks)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.score().cmp(&left.score()))
    });
    ordered
        .iter()
        .enumerate()
        .map(|(index, item)| {
            (
                scored_key(item.song_id(), item.difficulty()),
                index as i32 + 1,
            )
        })
        .collect()
}

fn list_row(item: &Scored, ranks: &HashMap<String, i32>) -> cards::list::ListRow {
    let (next, need) = crate::model::next_grade(item.score());
    cards::list::ListRow {
        song_id: item.song_id().to_string(),
        title: item.title.clone(),
        composer: item.composer.clone(),
        group: String::new(),
        rank: ranks
            .get(&scored_key(item.song_id(), item.difficulty()))
            .copied()
            .unwrap_or_default(),
        diff: item.difficulty().name().to_string(),
        constant: item.constant,
        known: item.known,
        variant: item.variant,
        score: item.score(),
        acc: item.acc(),
        grade: item.grade.name().to_string(),
        rks: item.rks,
        next: if item.score() >= MAX_SCORE {
            String::new()
        } else {
            next.name().to_string()
        },
        need,
    }
}

fn auto_band_label(order: ListOrder, row: &cards::list::ListRow) -> String {
    match order {
        ListOrder::Rks => match row.rks {
            value if value >= 16.0 => String::from("RKS 16.00+"),
            value if value >= 15.5 => String::from("RKS 15.50 - 15.99"),
            value if value >= 15.0 => String::from("RKS 15.00 - 15.49"),
            value if value >= 14.0 => String::from("RKS 14.00 - 14.99"),
            value if value >= 13.0 => String::from("RKS 13.00 - 13.99"),
            _ => String::from("RKS < 13.00"),
        },
        ListOrder::Constant => match row.known && row.constant > 0.0 {
            true => format!("{:.1}", row.constant),
            false => String::from("定数未知"),
        },
        ListOrder::Acc => match row.acc {
            value if value >= 100.0 => String::from("ACC 100.00%"),
            value if value >= 99.5 => String::from("ACC 99.50%+"),
            value if value >= 99.0 => String::from("ACC 99.00% - 99.49%"),
            value if value >= 98.0 => String::from("ACC 98.00% - 98.99%"),
            _ => String::from("ACC < 98.00%"),
        },
        _ => match row.grade.to_uppercase().as_str() {
            "PHI" | "Φ" => String::from("φ (1,000,000)"),
            "V" => String::from("V (960,000+)"),
            "S" => String::from("S"),
            "A" => String::from("A"),
            other => format!("{other} 及以下"),
        },
    }
}

fn group_label(row: &cards::list::ListRow, group: ListGroup) -> String {
    match group {
        ListGroup::Difficulty => row.diff.clone(),
        ListGroup::Constant => constant_text(row.constant, row.known),
        ListGroup::Grade => row.grade.clone(),
        ListGroup::None => String::new(),
    }
}

fn sort_rows(rows: &mut [cards::list::ListRow], ordering: Ordering) {
    rows.sort_by(|left, right| {
        let base = match ordering.key {
            ListOrder::Constant => right
                .constant
                .partial_cmp(&left.constant)
                .unwrap_or(std::cmp::Ordering::Equal),
            ListOrder::Rks => right
                .rks
                .partial_cmp(&left.rks)
                .unwrap_or(std::cmp::Ordering::Equal),
            ListOrder::Acc => right
                .acc
                .partial_cmp(&left.acc)
                .unwrap_or(std::cmp::Ordering::Equal),
            ListOrder::Score => right.score.cmp(&left.score),
            ListOrder::Rank => left.rank.cmp(&right.rank),
            ListOrder::Title => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
        };
        let ordered = if ordering.reverse {
            base.reverse()
        } else {
            base
        };
        ordered.then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
    });
}

fn group_text(group: ListGroup) -> &'static str {
    match group {
        ListGroup::Difficulty => "分难度",
        ListGroup::Constant => "分定数",
        ListGroup::Grade => "分评级",
        ListGroup::None => "不分带",
    }
}

fn order_text(ordering: Ordering) -> String {
    let key = match ordering.key {
        ListOrder::Constant => "定数",
        ListOrder::Rks => "单曲 RKS",
        ListOrder::Acc => "ACC",
        ListOrder::Score => "分数",
        ListOrder::Rank => "全库名次",
        ListOrder::Title => "曲名",
    };
    let direction = match (ordering.key, ordering.reverse) {
        (ListOrder::Rank, false) => "从前往后",
        (ListOrder::Rank, true) => "从后往前",
        (_, false) => "从高到低",
        (_, true) => "从低到高",
    };
    format!("按{key}{direction}")
}

pub async fn list_out(
    service: &Arc<PhigrosService>,
    user: &str,
    raw: &str,
    page: usize,
) -> Outgoing {
    let Some(mut parsed) = query::parse(raw) else {
        return PhigrosService::notice(list_usage());
    };
    resolve_chapter_keyword(service.catalog(), &mut parsed);
    if !query::order_explicit(raw) {
        parsed.order = query::Ordering {
            key: query::ListOrder::Title,
            reverse: false,
        };
    }
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = service.profile(user).await.ok().flatten();
    let scored = match service.scored(user).await {
        Ok(scored) => scored,
        Err(error) => return PhigrosService::fail(error),
    };
    let head = {
        let text = query::condition_text(&parsed);
        if text.is_empty() {
            String::from("全部成绩")
        } else {
            text
        }
    };
    let filters = build_filters(service.catalog(), &parsed);
    let ranks = rank_map(&scored);
    let mut rows: Vec<cards::list::ListRow> = scored
        .iter()
        .filter(|item| !item.unrecorded() && matches(&filters, item))
        .map(|item| list_row(item, &ranks))
        .collect();
    sort_rows(&mut rows, parsed.order);
    let total = rows.len();
    if total == 0 {
        return PhigrosService::notice(format!("没有符合条件的成绩（{head}）"));
    }
    let limit = page_limit(&service.settings(), parsed.limit);
    let pages = page_count(total, limit);
    let current = page.clamp(1, pages);
    let (start, end) = page_slice(total, current, limit);
    let auto_band = parsed.group == ListGroup::None && total >= 40;
    for row in rows.iter_mut() {
        row.group = match auto_band {
            true => auto_band_label(parsed.order.key, row),
            false => group_label(row, parsed.group),
        };
    }
    let shown = rows[start..end].to_vec();
    let backdrop = shown
        .first()
        .map(|row| row.song_id.clone())
        .unwrap_or_default();
    let mut acc_sum = 0.0;
    let mut rks_sum = 0.0;
    let mut max_rks = 0.0_f64;
    for row in &rows {
        acc_sum += row.acc;
        rks_sum += row.rks;
        if row.rks > max_rks {
            max_rks = row.rks;
        }
    }
    let count = rows.len() as f64;
    let average_acc = acc_sum / count;
    let average_rks = rks_sum / count;
    let request = cards::list::ListRequest {
        player: service.player(&binding, profile.as_ref()),
        head: head.clone(),
        note: format!(
            "命中 {total} 条 · ACC 均 {} · RKS 均 {} · 最高 {} · {} · {} · 第 {current}/{pages} 页",
            acc_short(average_acc),
            rks_text(average_rks),
            rks_text(max_rks),
            order_text(parsed.order),
            group_text(parsed.group)
        ),
        backdrop,
        stats: vec![
            cards::list::ListStat {
                value: total.to_string(),
                label: String::from("命中"),
            },
            cards::list::ListStat {
                value: acc_short(average_acc),
                label: String::from("平均 ACC"),
            },
            cards::list::ListStat {
                value: rks_text(average_rks),
                label: String::from("平均 RKS"),
            },
            cards::list::ListStat {
                value: rks_text(max_rks),
                label: String::from("最高 RKS"),
            },
        ],
        rows: shown.clone(),
    };
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::List {
            raw: raw.to_string(),
            page: current,
        },
        &crate::buttons::ButtonCtx {
            page: current,
            pages,
        },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::list::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_LIST,
        &request,
        move |kit| cards::list::draw(kit, &root, &drawn),
    );
    let mut out = PhigrosService::bare().card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatsDetail {
    Both,
    Difficulty,
    Constant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatsView {
    All,
    UnFc,
    UnPhi,
    Uncleared,
}

impl StatsView {
    fn keeps(self, item: &Scored) -> bool {
        match self {
            StatsView::All => true,
            StatsView::UnFc => !item.fc(),
            StatsView::UnPhi => item.score() < MAX_SCORE,
            StatsView::Uncleared => item.score() < CLEAR_SCORE,
        }
    }

    fn name(self) -> &'static str {
        match self {
            StatsView::All => "全部",
            StatsView::UnFc => "未fc",
            StatsView::UnPhi => "未phi",
            StatsView::Uncleared => "未通关",
        }
    }
    fn short_note(self) -> &'static str {
        match self {
            StatsView::All => "这一段里 ACC 最低的若干张",
            StatsView::UnFc => "这一段里还没全连的，ACC 高的在前",
            StatsView::UnPhi => "这一段里还没满分的，ACC 高的在前",
            StatsView::Uncleared => "这一段里还没通关的，分数高的在前",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StatsOptions {
    detail: StatsDetail,
    view: StatsView,
    short: usize,
}

impl StatsOptions {
    fn new(short: usize) -> Self {
        Self {
            detail: StatsDetail::Both,
            view: StatsView::All,
            short,
        }
    }
}

fn stats_options(raw: &str, short: usize) -> StatsOptions {
    let mut options = StatsOptions::new(short);
    for token in raw.split_whitespace() {
        let token = crate::domain::span::fold_symbols(token.trim()).to_lowercase();
        match token.as_str() {
            "d" | "diff" => options.detail = StatsDetail::Difficulty,
            "c" | "const" => options.detail = StatsDetail::Constant,
            "nofc" => options.view = StatsView::UnFc,
            "nophi" => options.view = StatsView::UnPhi,
            "noclear" => options.view = StatsView::Uncleared,
            other => {
                if let Some(count) = other.strip_prefix('n').and_then(|rest| rest.parse().ok()) {
                    options.short = count;
                }
            }
        }
    }
    options
}

struct RangeScan {
    bands: Vec<cards::stats::StatsBand>,
    consts: Vec<cards::stats::StatsConst>,
    total: i32,
    played: i32,
    cleared: i32,
    full_combo: i32,
    phi: i32,
    acc_sum: f64,
}

#[derive(Default)]
struct TierScan {
    total: i32,
    played: i32,
    acc_sum: f64,
}
fn scan_range(
    catalog: &Catalog,
    records: &HashMap<String, &Scored>,
    filters: &Filters,
) -> RangeScan {
    let mut levels: Vec<cards::stats::StatsBand> = Vec::with_capacity(NUM_PLAYABLE);
    let mut tiers: BTreeMap<i32, TierScan> = BTreeMap::new();
    let mut scan = RangeScan {
        bands: Vec::new(),
        consts: Vec::new(),
        total: 0,
        played: 0,
        cleared: 0,
        full_combo: 0,
        phi: 0,
        acc_sum: 0.0,
    };
    for difficulty in Difficulty::playable_all() {
        if !filters.difficulties.is_empty() && !filters.difficulties.contains(difficulty) {
            continue;
        }
        let mut band = cards::stats::StatsBand {
            name: difficulty.name().to_string(),
            ..cards::stats::StatsBand::default()
        };
        for song in catalog.songs() {
            if song.variant() || !filters.song(song) {
                continue;
            }
            let Some(chart) = song.chart(*difficulty) else {
                continue;
            };
            if filters.constant.on() && !filters.constant.contains(chart.constant) {
                continue;
            }
            band.total += 1;
            scan.total += 1;
            let tier = tiers
                .entry((chart.constant * 10.0).round() as i32)
                .or_default();
            tier.total += 1;
            let Some(item) = records.get(&scored_key(&song.id, *difficulty)) else {
                continue;
            };
            band.played += 1;
            band.avg_acc += item.acc();
            scan.played += 1;
            scan.acc_sum += item.acc();
            tier.played += 1;
            tier.acc_sum += item.acc();
            if item.score() >= CLEAR_SCORE {
                band.cleared += 1;
                scan.cleared += 1;
            }
            if item.fc() || item.score() >= MAX_SCORE {
                band.full_combo += 1;
                scan.full_combo += 1;
            }
            if item.score() >= MAX_SCORE {
                band.phi += 1;
                scan.phi += 1;
            }
        }
        if band.played > 0 {
            band.avg_acc /= f64::from(band.played);
        }
        levels.push(band);
    }
    scan.bands = levels;
    scan.consts = tiers
        .into_iter()
        .map(|(tier, item)| cards::stats::StatsConst {
            constant: f64::from(tier) / 10.0,
            total: item.total,
            played: item.played,
            avg_acc: if item.played > 0 {
                item.acc_sum / f64::from(item.played)
            } else {
                0.0
            },
        })
        .collect();
    scan
}

fn grade_counts(scored: &[Scored], filters: &Filters) -> Vec<cards::stats::StatsGrade> {
    let mut counts = [0_i32; GRADE_COUNT];
    for item in scored {
        if item.unrecorded() || !matches(filters, item) {
            continue;
        }
        counts[item.grade.index()] += 1;
    }
    Grade::ALL
        .iter()
        .rev()
        .filter_map(|grade| {
            let count = counts[grade.index()];
            (count > 0).then(|| cards::stats::StatsGrade {
                name: grade.name().to_string(),
                count,
            })
        })
        .collect()
}

fn short_rows(
    scored: &[Scored],
    filters: &Filters,
    options: StatsOptions,
) -> Vec<cards::stats::StatsRow> {
    if options.short == 0 {
        return Vec::new();
    }
    let mut items: Vec<&Scored> = scored
        .iter()
        .filter(|item| {
            !item.unrecorded()
                && item.known
                && matches(filters, item)
                && options.view.keeps(item)
        })
        .collect();
    items.sort_by(|left, right| {
        left.acc()
            .partial_cmp(&right.acc())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if options.view != StatsView::All {
        items.sort_by(|left, right| {
            if options.view == StatsView::Uncleared {
                right.score().cmp(&left.score())
            } else {
                right
                    .acc()
                    .partial_cmp(&left.acc())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });
    }
    items
        .into_iter()
        .take(options.short.min(STATS_SHORT_MAX))
        .map(|item| cards::stats::StatsRow {
            song_id: item.song_id().to_string(),
            title: item.title.clone(),
            diff: item.difficulty().name().to_string(),
            constant: item.constant,
            score: item.score(),
            acc: item.acc(),
            grade: item.grade.name().to_string(),
            gap: (MAX_SCORE - item.score()).max(0),
        })
        .collect()
}

pub async fn stats_out(
    service: &Arc<PhigrosService>,
    user: &str,
    raw: &str,
    short: usize,
) -> Outgoing {
    let Some(mut parsed) = query::parse(raw) else {
        return PhigrosService::notice(stats_usage());
    };
    resolve_chapter_keyword(service.catalog(), &mut parsed);
    let options = stats_options(raw, short);
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = service.profile(user).await.ok().flatten();
    let scored = match service.scored(user).await {
        Ok(scored) => scored,
        Err(error) => return PhigrosService::fail(error),
    };
    let head = {
        let text = query::condition_text(&parsed);
        if text.is_empty() {
            String::from("全部定数")
        } else {
            text
        }
    };
    let filters = build_filters(service.catalog(), &parsed);
    let records = record_map(&scored);
    let scan = scan_range(service.catalog(), &records, &filters);
    if scan.total == 0 {
        return PhigrosService::notice(format!("这一区间里没有谱面（{head}）"));
    }
    let grades = grade_counts(&scored, &filters);
    let shorts = short_rows(&scored, &filters, options);
    let backdrop = shorts
        .first()
        .map(|row| row.song_id.clone())
        .unwrap_or_default();
    let bands = if options.detail == StatsDetail::Constant {
        Vec::new()
    } else {
        scan.bands.clone()
    };
    let mut consts = if options.detail == StatsDetail::Difficulty {
        Vec::new()
    } else {
        scan.consts.clone()
    };
    let mut const_note = String::from("谱面数 / 已玩");
    if consts.len() > STATS_CONST_MAX {
        let dropped = consts.len() - STATS_CONST_MAX;
        const_note = format!("只画了最高的 {STATS_CONST_MAX} 档（略过 {dropped} 档）");
        consts = consts.split_off(dropped);
    }
    let mut note_parts: Vec<String> = vec![format!("共 {} 张谱面", scan.total)];
    if options.detail != StatsDetail::Both {
        note_parts.push(String::from(match options.detail {
            StatsDetail::Difficulty => "只看难度",
            _ => "只看定数",
        }));
    }
    if options.view != StatsView::All {
        note_parts.push(format!("只看{}", options.view.name()));
    }
    let request = cards::stats::StatsRequest {
        player: service.player(&binding, profile.as_ref()),
        head: head.clone(),
        note: note_parts.join(" · "),
        backdrop,
        stats: vec![
            cards::stats::ListStat {
                value: percent_text(scan.played as usize, scan.total as usize),
                label: String::from("完成度"),
            },
            cards::stats::ListStat {
                value: format!("{}/{}", scan.played, scan.total),
                label: String::from("已打"),
            },
            cards::stats::ListStat {
                value: scan.cleared.to_string(),
                label: String::from("通关"),
            },
            cards::stats::ListStat {
                value: scan.full_combo.to_string(),
                label: String::from("FC"),
            },
            cards::stats::ListStat {
                value: scan.phi.to_string(),
                label: String::from("Phi"),
            },
            cards::stats::ListStat {
                value: if scan.played > 0 {
                    acc_short(scan.acc_sum / f64::from(scan.played))
                } else {
                    String::from("-")
                },
                label: String::from("均 ACC"),
            },
        ],
        bands,
        consts,
        const_note,
        grades,
        short_title: if shorts.is_empty() {
            String::new()
        } else {
            String::from("短板")
        },
        short: shorts,
        short_note: String::from(options.view.short_note()),
    };
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Stats {
            raw: raw.to_string(),
            page: 1,
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::stats::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_STATS,
        &request,
        move |kit| cards::stats::draw(kit, &root, &drawn),
    );
    let mut out = PhigrosService::bare().card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

fn band_note(band: i32) -> String {
    format!("定数 {band}")
}

fn phi_group(
    scored: &[Scored],
    picks: &[crate::rank::phi::PhiCandidate],
    prior: &crate::rank::prior::Prior,
) -> cards::suggest::SuggestGroup {
    let mut tier = cards::suggest::PUSH_DISTANT;
    let mut cand: Vec<cards::suggest::SuggestCandidate> = Vec::with_capacity(picks.len());
    for pick in picks {
        let item = &scored[pick.index];
        let gap = 100.0 - item.acc();
        let peer = prior.peer(item.song_id(), item.difficulty());
        tier = tier.min(crate::rank::push::tier_of_gap(gap));
        cand.push(cards::suggest::SuggestCandidate {
            song_id: item.song_id().to_string(),
            title: item.title.clone(),
            diff: item.difficulty().name().to_string(),
            constant: item.constant,
            known: item.known,
            score: item.score(),
            acc: item.acc(),
            rks: item.rks,
            target: item.constant,
            target_acc: 100.0,
            gap,
            via_phi: true,
            gain: pick.gain,
            peer_median: peer.map(|peer| peer.median).unwrap_or_default(),
            peer_users: peer.map(|peer| peer.users).unwrap_or_default(),
            rate: peer.map(|peer| peer.rate(100.0)).unwrap_or_default(),
            counted: true,
        });
    }
    cards::suggest::SuggestGroup {
        tier,
        phi: true,
        note: String::from("满分候选 · 按 RKS 增量降序"),
        cand,
    }
}

pub async fn suggest_out(service: &Arc<PhigrosService>, user: &str, raw: &str) -> Outgoing {
    let Some(mut parsed) = query::parse_constant_query(raw) else {
        if !raw.trim().is_empty() {
            return PhigrosService::notice(push_usage());
        }
        return suggest_page(service, user, raw, None, String::from("全部定数")).await;
    };
    resolve_chapter_keyword(service.catalog(), &mut parsed);
    let head = {
        let text = query::condition_text(&parsed);
        if text.is_empty() {
            String::from("全部定数")
        } else {
            text
        }
    };
    suggest_page(service, user, raw, Some(parsed), head).await
}

async fn suggest_page(
    service: &Arc<PhigrosService>,
    user: &str,
    raw: &str,
    parsed: Option<Parsed>,
    head: String,
) -> Outgoing {
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = service.profile(user).await.ok().flatten();
    let scored = match service.scored(user).await {
        Ok(scored) => scored,
        Err(error) => return PhigrosService::fail(error),
    };
    let filters = parsed
        .as_ref()
        .map(|parsed| build_filters(service.catalog(), parsed))
        .unwrap_or_default();
    let computed = rks::rks(&scored);
    let stats = match service
        .repo()
        .server_stats(|song_id, difficulty| {
            service
                .catalog()
                .song(song_id)
                .and_then(|song| song.constant(difficulty))
        })
        .await
    {
        Ok(stats) => stats,
        Err(error) => return PhigrosService::fail(error.to_string()),
    };
    let prior = crate::rank::prior::prior_from(&stats, crate::model::band_of(computed));
    let board = crate::rank::push::push(&scored, 0, &prior);
    let cutoff = board.cutoff;
    let out_of_reach = board.out_of_reach;
    let out_of_reach_phi = board.out_of_reach_phi;
    let mut bands: Vec<(i32, cards::suggest::SuggestGroup)> = Vec::new();
    let mut total = 0;
    let per_page = service.settings().push_per_page as i32;
    for pushed in board.candidates.iter() {
        if !matches(&filters, &scored[pushed.index]) {
            continue;
        }
        if total >= per_page {
            continue;
        }
        total += 1;
        let tier = crate::rank::push::tier_of_gap(pushed.gap);
        let band = pushed.band;
        let candidate = cards::suggest::SuggestCandidate {
            song_id: pushed.song_id.clone(),
            title: pushed.title.clone(),
            diff: pushed.difficulty.name().to_string(),
            constant: pushed.constant,
            known: true,
            score: pushed.score,
            acc: pushed.acc,
            rks: pushed.rks,
            target: rks::single_rks(pushed.target_acc, pushed.constant),
            target_acc: pushed.target_acc,
            gap: pushed.gap,
            via_phi: pushed.via_phi,
            gain: pushed.gain,
            peer_median: pushed.peer_median,
            peer_users: pushed.peer_users,
            rate: pushed.rate,
            counted: true,
        };
        match bands.iter_mut().find(|(slot, _)| *slot == band) {
            Some((_, group)) => {
                group.cand.push(candidate);
                if tier < group.tier {
                    group.tier = tier;
                }
            }
            None => bands.push((
                band,
                cards::suggest::SuggestGroup {
                    tier,
                    phi: false,
                    note: band_note(band),
                    cand: vec![candidate],
                },
            )),
        }
    }
    bands.sort_by_key(|(band, _)| *band);
    let phi_picks = crate::rank::phi::candidates(&scored, PHI_PER_PAGE);
    let mut groups: Vec<cards::suggest::SuggestGroup> = Vec::new();
    if !phi_picks.is_empty() {
        groups.push(phi_group(&scored, &phi_picks, &prior));
    }
    groups.extend(bands.into_iter().map(|(_, group)| group));
    let scope = crate::rank::entry::EntryScope {
        span: parsed
            .as_ref()
            .map(|item| item.query.constant)
            .unwrap_or_default(),
        levels: parsed
            .as_ref()
            .map(|item| item.query.difficulties.clone())
            .unwrap_or_default(),
    };
    let entry =
        crate::rank::entry::board(&scored, service.catalog(), &scope, ENTRY_PER_PAGE, &prior);
    if groups.is_empty() && entry.candidates.is_empty() {
        return PhigrosService::notice(String::from(
            "这一档没有要推的谱面：已打过的都到顶了，剩下的曲线也都进不了 B27",
        ));
    }
    let backdrop = groups
        .iter()
        .flat_map(|group| group.cand.iter())
        .map(|cand| cand.song_id.clone())
        .chain(
            entry
                .candidates
                .iter()
                .map(|candidate| candidate.song_id.clone()),
        )
        .next()
        .unwrap_or_default();
    let request = cards::suggest::SuggestRequest {
        player: service.player(&binding, profile.as_ref()),
        backdrop,
        filter: head.clone(),
        groups,
        total,
        cutoff,
        rks: board.rks,
        step: board.step,
        out_of_reach,
        empty: String::from("这一档没有要推的谱面"),
        out_of_reach_phi,
        entry: entry
            .candidates
            .iter()
            .map(|candidate| cards::suggest::SuggestEntry {
                song_id: candidate.song_id.clone(),
                title: candidate.title.clone(),
                diff: candidate.difficulty.name().to_string(),
                constant: candidate.constant,
                notes: candidate.notes,
                target: candidate.target,
                required_acc: candidate.required_acc,
                via_phi: candidate.via_phi,
                reach_acc: candidate.reach_acc,
                reach_ok: candidate.reach_ok,
                gain: candidate.gain,
                short: candidate.short,
            })
            .collect(),
        entry_total: entry.total,
        entry_unreached: entry.unreached,
        entry_too_hard: entry.too_hard,
    };
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Suggest {
            raw: raw.to_string(),
            page: 1,
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::suggest::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_SUGGEST,
        &request,
        move |kit| cards::suggest::draw(kit, &root, &drawn),
    );
    let mut out = PhigrosService::bare().card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetOrder {
    #[default]
    Constant,
    Notes,
    Title,
    Bpm,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SheetCell {
    pub constant: f64,
    pub notes: Notes,
    pub charter: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SheetRow {
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub cells: Vec<SheetCell>,
    pub matched: usize,
    pub pick: Difficulty,
    pub best: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sheet {
    pub rows: Vec<SheetRow>,
    pub levels: Vec<Difficulty>,
    pub total: usize,
    pub songs: usize,
    pub min_const: f64,
    pub max_const: f64,
}

#[derive(Clone, Debug, Default)]
pub struct SheetQuery<'a> {
    pub constant: Span,
    pub levels: Vec<Difficulty>,
    pub chapters: Option<&'a HashSet<String>>,
    pub songs: Option<&'a HashSet<String>>,
    pub order: SheetOrder,
    pub limit: usize,
}

fn sheet_columns(levels: &[Difficulty]) -> Vec<Difficulty> {
    let mut out: Vec<Difficulty> = Difficulty::playable_all().to_vec();
    if levels.contains(&Difficulty::Legacy) {
        out.push(Difficulty::Legacy);
    }
    out
}

fn sheet_levels(levels: &[Difficulty]) -> Vec<Difficulty> {
    if levels.is_empty() {
        return Difficulty::playable_all().to_vec();
    }
    let mut out: Vec<Difficulty> = Vec::with_capacity(levels.len());
    for level in levels {
        if level.valid() && !out.contains(level) {
            out.push(*level);
        }
    }
    out
}

fn sheet_notes(row: &SheetRow) -> i32 {
    row.cells
        .iter()
        .filter(|cell| cell.constant > 0.0)
        .map(|cell| cell.notes.combo())
        .max()
        .unwrap_or_default()
}

pub fn build_sheet(songs: &[Song], query: &SheetQuery<'_>) -> Sheet {
    let levels = sheet_levels(&query.levels);
    let mut out = Sheet {
        levels: sheet_columns(&query.levels),
        ..Sheet::default()
    };
    let width = out.levels.len();
    for song in songs {
        if song.variant() {
            continue;
        }
        if let Some(names) = query.chapters {
            if !names.contains(song.chapter.as_str()) {
                continue;
            }
        }
        if let Some(ids) = query.songs {
            if !ids.contains(song.id.as_str()) {
                continue;
            }
        }
        let mut row = SheetRow {
            song_id: song.id.clone(),
            title: song.title.clone(),
            composer: song.composer.clone(),
            cells: vec![SheetCell::default(); width],
            ..SheetRow::default()
        };
        for (index, level) in out.levels.clone().iter().enumerate() {
            let Some(chart) = song.chart(*level) else {
                continue;
            };
            row.cells[index] = SheetCell {
                constant: chart.constant,
                notes: chart.notes,
                charter: chart.charter.trim().to_string(),
            };
            if !levels.contains(level) || !query.constant.contains(chart.constant) {
                continue;
            }
            row.matched += 1;
            out.total += 1;
            if out.total == 1 || chart.constant < out.min_const {
                out.min_const = chart.constant;
            }
            if out.total == 1 || chart.constant > out.max_const {
                out.max_const = chart.constant;
            }
            if chart.constant > row.best {
                row.best = chart.constant;
                row.pick = *level;
            }
        }
        if row.matched == 0 {
            continue;
        }
        out.rows.push(row);
    }
    out.songs = out.rows.len();
    let bpm: HashMap<String, i32> = songs
        .iter()
        .map(|song| (song.id.clone(), song.bpm))
        .collect();
    out.rows.sort_by(|left, right| {
        let base = match query.order {
            SheetOrder::Notes => sheet_notes(right).cmp(&sheet_notes(left)),
            SheetOrder::Title => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
            SheetOrder::Bpm => bpm
                .get(&right.song_id)
                .copied()
                .unwrap_or_default()
                .cmp(&bpm.get(&left.song_id).copied().unwrap_or_default()),
            SheetOrder::Constant => right
                .best
                .partial_cmp(&left.best)
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        base.then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
    });
    if query.limit > 0 && out.rows.len() > query.limit {
        out.rows.truncate(query.limit);
    }
    out
}

fn sheet_order(order: ListOrder) -> SheetOrder {
    match order {
        ListOrder::Title => SheetOrder::Title,
        _ => SheetOrder::Constant,
    }
}

fn sheet_head(parsed: &Parsed) -> String {
    let text = query::condition_text(parsed);
    if text.is_empty() {
        String::from("全部定数")
    } else {
        text
    }
}

pub async fn table_out(
    service: &Arc<PhigrosService>,
    _user: &str,
    raw: &str,
    _page: usize,
) -> Outgoing {
    let mut parsed = match query::parse_constant_query(raw) {
        Some(parsed) => parsed,
        None => match query::parse(raw) {
            Some(parsed) => parsed,
            None => return PhigrosService::notice(table_usage()),
        },
    };
    resolve_chapter_keyword(service.catalog(), &mut parsed);
    let filters = build_filters(service.catalog(), &parsed);
    let chapters = filters.chapters.as_ref().map(|ids| {
        service
            .catalog()
            .songs()
            .iter()
            .filter(|song| ids.contains(&song.id))
            .map(|song| song.chapter.clone())
            .collect::<HashSet<String>>()
    });
    let limit = page_limit(&service.settings(), parsed.limit);
    let sheet = build_sheet(
        service.catalog().songs(),
        &SheetQuery {
            constant: parsed.query.constant,
            levels: parsed.query.difficulties.clone(),
            chapters: chapters.as_ref(),
            songs: filters.titles.as_ref(),
            order: sheet_order(parsed.order.key),
            limit,
        },
    );
    let head = sheet_head(&parsed);
    if sheet.total == 0 {
        return PhigrosService::notice(format!("没有符合条件的谱面（{head}）"));
    }
    let levels = sheet.levels.clone();
    let mut sections: Vec<cards::table::TableSection> = Vec::new();
    for row in &sheet.rows {
        let label = format!("{}", row.best.floor() as i32);
        let cells: Vec<cards::table::TableCell> = row
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let level = levels.get(index).copied().unwrap_or(Difficulty::Ez);
                let hit = cell.constant > 0.0
                    && parsed.query.constant.contains(cell.constant)
                    && (parsed.query.difficulties.is_empty()
                        || parsed.query.difficulties.contains(&level));
                cards::table::TableCell {
                    constant: cell.constant,
                    hit,
                }
            })
            .collect();
        let entry = cards::table::TableRow {
            song_id: row.song_id.clone(),
            title: row.title.clone(),
            sub: row.composer.clone(),
            cells,
        };
        match sections.iter_mut().find(|section| section.label == label) {
            Some(section) => section.rows.push(entry),
            None => sections.push(cards::table::TableSection {
                label,
                rows: vec![entry],
            }),
        }
    }
    let backdrop = sheet
        .rows
        .first()
        .map(|row| row.song_id.clone())
        .unwrap_or_default();
    let note = format!(
        "命中 {} 张谱面 · {} 首 · 定数 {} - {}",
        sheet.total,
        sheet.songs,
        constant_text(sheet.min_const, true),
        constant_text(sheet.max_const, true)
    );
    let request = cards::table::TableRequest {
        name: cards::table::TABLE_NAME.to_string(),
        title: format!("定数表 · {head}"),
        note: note.clone(),
        backdrop,
        cols: levels.iter().map(|level| level.name().to_string()).collect(),
        sections,
    };
    let mut args = Args::new();
    commands::put(&mut args, "note", note);
    commands::put(&mut args, "title", request.title.clone());
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Table {
            raw: raw.to_string(),
            page: 1,
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::table::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_TABLE,
        &request,
        move |kit| cards::table::draw(kit, &root, &drawn),
    );
    let mut out = service.text("table.sheet.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

pub fn match_chapter(chapters: &[Chapter], text: &str) -> Option<Chapter> {
    let needle = text.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    chapters
        .iter()
        .find(|chapter| {
            chapter.name.to_lowercase().contains(&needle)
                || chapter.display.to_lowercase().contains(&needle)
                || chapter
                    .aliases
                    .iter()
                    .any(|alias| alias.to_lowercase().contains(&needle))
        })
        .cloned()
}

pub fn chapter_not_found(text: &str) -> String {
    format!(
        "没有找到「{text}」这一章，发 /p chap 可以看全部章节（章节名可以只写一段，如「霓虹灯牌」）"
    )
}

pub async fn chapter_out(service: &Arc<PhigrosService>, user: &str) -> Outgoing {
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = service.profile(user).await.ok().flatten();
    let scored = match service.scored(user).await {
        Ok(scored) => scored,
        Err(error) => return PhigrosService::fail(error),
    };
    let catalog = service.catalog();
    let mut records = record_map(&scored);
    for item in &scored {
        if item.unrecorded() {
            continue;
        }
        let Some(song) = catalog.song(item.song_id()) else {
            continue;
        };
        if !song.variant() {
            continue;
        }
        records
            .entry(scored_key(&song.variant_of, item.difficulty()))
            .or_insert(item);
    }
    let mut chapters: Vec<cards::chapter::ChapterProgress> = Vec::new();
    for chapter in catalog.chapters() {
        let mut entry = cards::chapter::ChapterProgress {
            name: if chapter.display.is_empty() {
                chapter.name.clone()
            } else {
                chapter.display.clone()
            },
            banner: chapter.name.clone(),
            ..cards::chapter::ChapterProgress::default()
        };
        for song in catalog.songs() {
            if song.variant() || song.chapter != chapter.name {
                continue;
            }
            for difficulty in Difficulty::playable_all() {
                if song.chart(*difficulty).is_none() {
                    continue;
                }
                entry.total += 1;
                let Some(item) = records.get(&scored_key(&song.id, *difficulty)) else {
                    continue;
                };
                entry.played += 1;
                if item.score() >= CLEAR_SCORE {
                    entry.cleared += 1;
                }
                if item.fc() || item.score() >= MAX_SCORE {
                    entry.full_combo += 1;
                }
                if item.score() >= MAX_SCORE {
                    entry.phi += 1;
                }
            }
        }
        if entry.total > 0 {
            chapters.push(entry);
        }
    }
    if chapters.is_empty() {
        return PhigrosService::notice(String::from("曲库里还没有能统计的章节"));
    }
    let played: i32 = chapters.iter().map(|chapter| chapter.played).sum();
    let total: i32 = chapters.iter().map(|chapter| chapter.total).sum();
    let backdrop = scored
        .iter()
        .filter(|item| item.known && !item.unrecorded())
        .max_by(|left, right| {
            left.constant
                .partial_cmp(&right.constant)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|item| item.song_id().to_string())
        .unwrap_or_default();
    let root = service.root().to_path_buf();
    let request = cards::chapter::ChapterRequest {
        chapters: chapters.clone(),
        backdrop,
        player: service.player(&binding, profile.as_ref()),
    };
    let mut rows: Vec<Arg> = Vec::new();
    for chapter in &chapters {
        rows.push(map_arg(vec![
            ("name", Arg::Str(chapter.name.clone())),
            (
                "percent",
                Arg::Str(percent_text(
                    chapter.played as usize,
                    chapter.total as usize,
                )),
            ),
            ("played", Arg::Int(i64::from(chapter.played))),
            ("total", Arg::Int(i64::from(chapter.total))),
            ("cleared", Arg::Int(i64::from(chapter.cleared))),
            ("fullCombo", Arg::Int(i64::from(chapter.full_combo))),
            ("phi", Arg::Int(i64::from(chapter.phi))),
        ]));
    }
    let mut args = Args::new();
    commands::put(&mut args, "title", "章节进度");
    commands::put(
        &mut args,
        "percent",
        percent_text(played as usize, total as usize),
    );
    commands::put(&mut args, "played", played.to_string());
    commands::put(&mut args, "total", total.to_string());
    commands::put(&mut args, "chapters", chapters.len().to_string());
    args.insert("rows".to_string(), Arg::List(rows));
    commands::put(&mut args, "footer", CHAP_FOOTER);
    let names: Vec<String> = chapters.iter().map(|c| c.name.clone()).collect();
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Chapter {
            chapters: names.clone(),
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let (width, height) = cards::chapter::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_CHAPTER,
        &request,
        move |kit| cards::chapter::draw(kit, &root, &drawn),
    );
    let mut out = service.text("chapter.progress.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

pub async fn chapter_detail_out(
    service: &Arc<PhigrosService>,
    user: &str,
    text: &str,
) -> Outgoing {
    let catalog = service.catalog();
    let Some(chapter) = match_chapter(catalog.chapters(), text) else {
        return PhigrosService::notice(chapter_not_found(text));
    };
    let binding = match service.require_binding(user).await {
        Ok(binding) => binding,
        Err(out) => return *out,
    };
    let profile = service.profile(user).await.ok().flatten();
    let scored = match service.scored(user).await {
        Ok(scored) => scored,
        Err(error) => return PhigrosService::fail(error),
    };
    let records = record_map(&scored);
    let levels: Vec<Difficulty> = Difficulty::playable_all().to_vec();
    let mut level_stats: Vec<cards::chapterdetail::ChapterDetailLevel> = levels
        .iter()
        .map(|difficulty| cards::chapterdetail::ChapterDetailLevel {
            diff: difficulty.name().to_string(),
            ..cards::chapterdetail::ChapterDetailLevel::default()
        })
        .collect();
    let mut grades = [0_usize; GRADE_COUNT];
    let mut songs: Vec<cards::chapterdetail::ChapterDetailSong> = Vec::new();
    let mut total = 0_usize;
    let mut played = 0_usize;
    let mut full_combo = 0_usize;
    let mut phi = 0_usize;
    let mut acc_sum = 0.0_f64;
    for song in catalog.songs() {
        if song.variant() || song.chapter != chapter.name {
            continue;
        }
        let mut cells: Vec<cards::chapterdetail::ChapterDetailCell> = levels
            .iter()
            .map(|_| cards::chapterdetail::ChapterDetailCell::default())
            .collect();
        let mut best_acc = 0.0_f64;
        let mut top_const = 0.0_f64;
        for (index, difficulty) in levels.iter().enumerate() {
            let Some(chart) = song.chart(*difficulty) else {
                continue;
            };
            cells[index].constant = chart.constant;
            level_stats[index].total += 1;
            total += 1;
            if chart.constant > top_const {
                top_const = chart.constant;
            }
            let Some(item) = records.get(&scored_key(&song.id, *difficulty)) else {
                continue;
            };
            cells[index].grade = item.grade.name().to_string();
            cells[index].score = item.score();
            cells[index].acc = item.acc();
            cells[index].rks = item.rks;
            best_acc = best_acc.max(item.acc());
            level_stats[index].played += 1;
            level_stats[index].acc += item.acc();
            played += 1;
            acc_sum += item.acc();
            if item.fc() || item.score() >= MAX_SCORE {
                full_combo += 1;
            }
            if item.score() >= MAX_SCORE {
                phi += 1;
            }
            if item.score() > 0 {
                grades[item.grade.index()] += 1;
            }
        }
        songs.push(cards::chapterdetail::ChapterDetailSong {
            song_id: song.id.clone(),
            title: song.title.clone(),
            sub: format!("{} · {}", constant_text(top_const, true), song.composer),
            cells,
            best_acc,
        });
    }
    if total == 0 {
        return PhigrosService::notice(format!(
            "这一章在曲库里还没有谱面（{}）",
            chapter.name
        ));
    }
    for level in level_stats.iter_mut() {
        if level.played > 0 {
            level.acc /= level.played as f64;
        }
    }
    let title = if chapter.display.is_empty() {
        chapter.name.clone()
    } else {
        chapter.display.clone()
    };
    let request = cards::chapterdetail::ChapterDetailRequest {
        title: title.clone(),
        note: format!(
            "{} · 完成度 {}",
            chapter.name,
            percent_text(played, total)
        ),
        player: service.player(&binding, profile.as_ref()),
        backdrop: String::new(),
        banner: chapter.name.clone(),
        total,
        played,
        full_combo,
        phi,
        avg_acc: acc_sum / played.max(1) as f64,
        grades,
        levels: level_stats,
        songs,
    };
    let mut args = Args::new();
    commands::put(&mut args, "title", title);
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::ChapterDetail {
            name: chapter.name.clone(),
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::chapterdetail::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_CHAPTER_DETAIL,
        &request,
        move |kit| cards::chapterdetail::draw(kit, &root, &drawn),
    );
    let mut out = service.text("chapter.detail.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Chart;

    fn song(id: &str, title: &str, levels: &[Difficulty], constant: f64) -> Song {
        let mut item = Song {
            id: id.to_string(),
            title: title.to_string(),
            composer: String::from("Aurorix"),
            chapter: String::from("Chapter 5 霓虹灯牌"),
            ..Song::default()
        };
        for level in levels {
            item.charts[level.index()] = Chart {
                difficulty: *level,
                constant,
                ..Chart::default()
            };
        }
        item
    }

    fn scored(id: &str, level: Difficulty, score: i32, acc: f64) -> Scored {
        let grade = crate::model::grade_of(score, false);
        Scored {
            record: crate::model::PlayRecord {
                song_id: id.to_string(),
                difficulty: level,
                score,
                acc,
                fc: false,
            },
            title: id.to_string(),
            constant: 15.0,
            known: true,
            rks: rks::single_rks(acc, 15.0),
            grade,
            ..Scored::default()
        }
    }

    #[test]
    fn pagination_clamps_to_bounds() {
        let settings = Settings::default();
        assert_eq!(page_limit(&settings, 0), LIST_DEFAULT);
        assert_eq!(page_limit(&settings, LIST_MAX + 50), LIST_MAX);
        assert_eq!(page_count(0, 10), 1);
        assert_eq!(page_count(95, 10), 10);
        assert_eq!(page_slice(95, 1, 10), (0, 10));
        assert_eq!(page_slice(95, 10, 10), (90, 95));
        assert_eq!(page_slice(95, 99, 10), (90, 95));
        assert_eq!(page_slice(95, 0, 10), (0, 10));
        assert_eq!(page_slice(0, 3, 10), (0, 0));
    }

    #[test]
    fn filters_read_every_dimension() {
        let item = scored("a", Difficulty::In, 960_000, 98.0);
        let mut filters = Filters::default();
        assert!(matches(&filters, &item));
        filters.constant = Span::between(15.0, 15.9);
        assert!(matches(&filters, &item));
        filters.constant = Span::between(16.0, 16.9);
        assert!(!matches(&filters, &item));
        let mut filters = Filters {
            grades: vec![Grade::V],
            ..Filters::default()
        };
        assert!(matches(&filters, &item));
        filters.grades = vec![Grade::Fc];
        assert!(!matches(&filters, &item));
        let mut titles = HashSet::new();
        titles.insert(String::from("a"));
        let filters = Filters {
            titles: Some(titles),
            ..Filters::default()
        };
        assert!(matches(&filters, &item));
        let filters = Filters {
            titles: Some(HashSet::new()),
            ..Filters::default()
        };
        assert!(!matches(&filters, &item));
        let filters = Filters {
            un_fc: true,
            ..Filters::default()
        };
        assert!(matches(&filters, &item));
    }

    #[test]
    fn sheet_truncates_and_keeps_legacy_only_when_named() {
        let songs = vec![
            song("a", "Alpha", &[Difficulty::Ez, Difficulty::In], 14.0),
            song("b", "Beta", &[Difficulty::In], 16.0),
            song("c", "Gamma", &[Difficulty::In], 15.0),
        ];
        let query = SheetQuery {
            constant: Span::at_least(10.0),
            levels: Vec::new(),
            chapters: None,
            songs: None,
            order: SheetOrder::Constant,
            limit: 2,
        };
        let sheet = build_sheet(&songs, &query);
        assert_eq!(sheet.levels.len(), NUM_PLAYABLE);
        assert_eq!(sheet.total, 4);
        assert_eq!(sheet.songs, 3);
        assert_eq!(sheet.rows.len(), 2);
        assert_eq!(sheet.rows[0].song_id, "b");
        assert_eq!(sheet.rows[1].song_id, "c");
        assert_eq!(sheet.min_const, 14.0);
        assert_eq!(sheet.max_const, 16.0);
        let query = SheetQuery {
            levels: vec![Difficulty::In, Difficulty::Legacy],
            limit: 0,
            ..SheetQuery::default()
        };
        let sheet = build_sheet(&songs, &query);
        assert_eq!(sheet.levels.len(), NUM_PLAYABLE + 1);
        assert_eq!(sheet.rows.len(), 3);
    }

    #[test]
    fn sheet_skips_variants_and_other_chapters() {
        let mut variant = song("v", "Variant", &[Difficulty::In], 15.0);
        variant.variant_of = String::from("a");
        let mut other = song("o", "Other", &[Difficulty::In], 15.0);
        other.chapter = String::from("Chapter 6 方舟蜃景");
        let songs = vec![song("a", "Alpha", &[Difficulty::In], 15.0), variant, other];
        let wanted: HashSet<String> = HashSet::from([String::from("Chapter 5 霓虹灯牌")]);
        let query = SheetQuery {
            chapters: Some(&wanted),
            ..SheetQuery::default()
        };
        let sheet = build_sheet(&songs, &query);
        assert_eq!(sheet.rows.len(), 1);
        assert_eq!(sheet.rows[0].song_id, "a");
    }

    #[test]
    fn sheet_keeps_only_named_songs() {
        let songs = vec![
            song("a", "Alpha", &[Difficulty::In], 15.0),
            song("b", "Beta", &[Difficulty::In], 16.0),
        ];
        let picked: HashSet<String> = HashSet::from([String::from("b")]);
        let query = SheetQuery {
            songs: Some(&picked),
            ..SheetQuery::default()
        };
        let sheet = build_sheet(&songs, &query);
        assert_eq!(sheet.rows.len(), 1);
        assert_eq!(sheet.rows[0].song_id, "b");
        let none: HashSet<String> = HashSet::new();
        let query = SheetQuery {
            songs: Some(&none),
            ..SheetQuery::default()
        };
        assert!(build_sheet(&songs, &query).rows.is_empty());
    }

    #[test]
    fn chapter_matching_accepts_partial_names() {
        let chapters = vec![
            Chapter {
                name: String::from("Chapter 5 霓虹灯牌"),
                display: String::from("霓虹灯牌"),
                aliases: vec![String::from("ch5")],
            },
            Chapter {
                name: String::from("Chapter 6 方舟蜃景"),
                display: String::from("方舟蜃景"),
                aliases: Vec::new(),
            },
        ];
        assert_eq!(
            match_chapter(&chapters, "霓虹").map(|chapter| chapter.name),
            Some(String::from("Chapter 5 霓虹灯牌"))
        );
        assert_eq!(
            match_chapter(&chapters, "ch5").map(|chapter| chapter.display),
            Some(String::from("霓虹灯牌"))
        );
        assert!(match_chapter(&chapters, "   ").is_none());
        assert!(match_chapter(&chapters, "不存在").is_none());
        assert!(chapter_not_found("x").contains("「x」"));
    }

    #[test]
    fn bare_chapter_name_becomes_a_chapter_filter() {
        let chapters = vec![
            Chapter {
                name: String::from("Chapter 6 方舟蜃景"),
                display: String::from("方舟蜃景"),
                aliases: Vec::new(),
            },
            Chapter {
                name: String::from("Chapter 8 微波炉"),
                display: String::from("微波炉"),
                aliases: vec![String::from("wbl")],
            },
        ];
        let mut parsed = query::parse("方舟蜃景").expect("要能解析");
        resolve_chapter_in(&chapters, false, &mut parsed);
        assert_eq!(parsed.query.chapter, "方舟蜃景");
        assert!(parsed.query.title.is_empty(), "曲包名不该再当关键词");
        let mut partial = query::parse("方舟").expect("要能解析");
        resolve_chapter_in(&chapters, false, &mut partial);
        assert_eq!(partial.query.chapter, "方舟");
        let mut song = query::parse("Spasmodic").expect("要能解析");
        resolve_chapter_in(&chapters, true, &mut song);
        assert_eq!(song.query.title, "Spasmodic", "有命中曲目就还是关键词");
        assert!(song.query.chapter.is_empty());
        let mut wbl = query::parse("wbl").expect("要能解析");
        resolve_chapter_in(&chapters, true, &mut wbl);
        assert_eq!(wbl.query.chapter, "wbl", "别名精确命中优先于曲名");
        let mut explicit = query::parse("曲包:方舟蜃景").expect("要能解析");
        resolve_chapter_in(&chapters, false, &mut explicit);
        assert_eq!(explicit.query.chapter, "方舟蜃景");
    }

    #[test]
    fn percent_and_score_formatting_follow_go() {
        assert_eq!(percent_text(0, 0), "-");
        assert_eq!(percent_text(1, 3), "33%");
        assert_eq!(percent_text(3, 3), "100%");
        assert_eq!(score_text(1_000_000), "1'000'000");
        assert_eq!(score_text(999), "999");
        assert_eq!(constant_text(15.5, true), "15.5");
        assert_eq!(constant_text(0.0, false), "未收录");
        assert_eq!(acc_short(99.456), "99.46%");
    }
}
