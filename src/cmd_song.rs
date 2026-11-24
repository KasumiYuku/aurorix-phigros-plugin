use std::collections::BTreeMap;
use std::sync::Arc;

use aurorix_plugin::{CommandSpec, Invocation, Outgoing, PluginError};
use aurorix_template::{Arg, Args};

use crate::cards;
use crate::cmd_query::constant_text;
use crate::commands;
use crate::domain::catalog::{self, Catalog, SCORE_PREFIX};
use crate::domain::overview;
use crate::domain::query;
use crate::domain::span::{self, Span};
use crate::model::{Chart, Difficulty, Notes, Song};
use crate::service::PhigrosService;

pub const ALIAS_PREFIX: &str = "p alias ";
pub const SEARCH_PREFIX: &str = "p search ";
pub const RAND_PREFIX: &str = "p rand ";
pub const PLAY_PREFIX: &str = "p play ";
pub const ILL_PREFIX: &str = "p ill ";

pub const SONG_USAGE: &str = "用法：/p song <曲名>";
pub const ALIAS_USAGE: &str = "用法：/p alias <曲名>，看这首的全部别名；改别名请到管理台";
pub const ALIAS_READONLY: &str = "改别名请到管理台：/p alias 只用来查看";
pub const SEARCH_USAGE: &str = "用法：/p search <定数> <难度> <曲名>，顺序不限\n例如 /p search 15 IN、/p search クロ AT、/p search 光";
pub const RAND_USAGE: &str = "用法：/p rand [定数] [难度]\n定数支持 14、14-15、14+、>14、>=14、≥14、<15、≤15，难度支持 EZ/HD/IN/AT\n例如 /p rand >14 IN、/p rand 8.5-12.0、/p rand at 14-15";
pub const PLAY_USAGE: &str = "用法：/p play <曲名>，例如 /p play Protoflicker";
pub const ILL_USAGE: &str = "用法：/p ill <曲名>，例如 /p ill Protoflicker";

pub const PICK_MISSING: &str = "没有找到匹配的曲目";
pub const NO_AUDIO: &str = "这首还没有音频";
pub const NO_ILLUST: &str = "这首还没有曲绘";
pub const NO_CHART_FOR_DIFFICULTY: &str = "这首歌没有 %s 这一档";
pub const RAND_EMPTY: &str = "该条件下没有可随机的谱面";

pub const SEARCH_SCAN: usize = 64;
pub const ALIAS_WIDTH: usize = 40;
pub const ALIAS_SEP: &str = " · ";

const LABEL_SONG: &str = "曲目图鉴";
const LABEL_SEARCH: &str = "检索结果";
const LABEL_SELECT: &str = "随机谱面";
const SONG_RANK_MAX: usize = 12;

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("song")
            .alias("info")
            .alias("曲目")
            .alias("图鉴")
            .describe("查看曲目图鉴")
            .handler(commands::on(service, run_song)),
        CommandSpec::new("alias")
            .alias("别名")
            .describe("查看曲目的全部别名")
            .handler(commands::on(service, run_alias)),
        CommandSpec::new("search")
            .alias("检索")
            .describe("检索曲库")
            .handler(commands::on(service, run_search)),
        CommandSpec::new("rand")
            .alias("随机")
            .describe("随机一张谱面")
            .handler(commands::on(service, run_rand)),
        CommandSpec::new("play")
            .alias("听")
            .alias("点歌")
            .alias("播放")
            .alias("music")
            .describe("点一首歌听")
            .handler(commands::on(service, run_play)),
        CommandSpec::new("ill")
            .alias("曲绘")
            .alias("看曲绘")
            .alias("查看曲绘")
            .describe("查看曲目曲绘")
            .handler(commands::on(service, run_ill)),
    ]
}

async fn run_song(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = song_out(&service, &inv.user, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_alias(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = alias_out(&service, &inv.user, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_search(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = search_out(&service, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_rand(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = random_out(&service, &inv.user, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

async fn run_play(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = play_out(&service, inv.text.trim()).await;
    commands::ok(out)
}

async fn run_ill(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let out = ill_out(&service, inv.text.trim()).await;
    commands::ok(commands::with_at(out, &inv))
}

#[derive(Clone, Debug, PartialEq)]
pub enum SongMatch<'a> {
    Found(&'a Song),
    Candidates(Vec<&'a Song>, usize),
    Missing,
}

pub fn resolve_song<'a>(catalog: &'a Catalog, text: &str) -> SongMatch<'a> {
    let hits = catalog.search(text, SEARCH_SCAN);
    if hits.is_empty() {
        return SongMatch::Missing;
    }
    let strong: Vec<&Song> = hits
        .iter()
        .filter(|(_, score)| *score >= SCORE_PREFIX)
        .map(|(song, _)| *song)
        .collect();
    if strong.len() == 1 {
        return SongMatch::Found(strong[0]);
    }
    let total = hits.len();
    let rows: Vec<&Song> = hits
        .into_iter()
        .map(|(song, _)| song)
        .take(query::PICK_ROWS)
        .collect();
    SongMatch::Candidates(rows, total)
}

pub fn healthy_chart(song: &Song, difficulty: Difficulty) -> Option<&Chart> {
    song.chart(difficulty)
}

fn dash(text: &str) -> String {
    if text.trim().is_empty() {
        String::from("-")
    } else {
        text.to_string()
    }
}

fn note_parts(notes: Notes) -> String {
    if notes.combo() == 0 {
        return String::new();
    }
    format!(
        "Tap {} / Drag {} / Hold {} / Flick {}",
        notes.tap, notes.drag, notes.hold, notes.flick
    )
}

pub fn notes_line(notes: Notes) -> String {
    let parts = note_parts(notes);
    if parts.is_empty() {
        return String::from("-");
    }
    format!("{}（{}）", notes.combo(), parts)
}

pub fn bpm_value(song: &Song) -> String {
    let text = song.bpm_text.trim();
    if !text.is_empty() {
        return text.to_string();
    }
    if song.bpm <= 0 {
        return String::new();
    }
    song.bpm.to_string()
}

pub fn bpm_text(song: &Song) -> String {
    let value = bpm_value(song);
    if value.is_empty() {
        String::from("-")
    } else {
        value
    }
}

pub fn song_version(song: &Song) -> String {
    let label = song.version_label.trim();
    if !label.is_empty() {
        return label.to_string();
    }
    if song.version > 0 {
        return format!("v{}", song.version);
    }
    String::new()
}

pub fn version_text(song: &Song) -> String {
    if song.version <= 0 {
        return String::from("-");
    }
    format!("v{}", song.version)
}

pub fn change_version(change: &crate::model::ConstChange) -> String {
    let label = change.version_label.trim();
    if !label.is_empty() {
        return label.to_string();
    }
    if change.version > 0 {
        return format!("v{}", change.version);
    }
    String::new()
}

pub fn variant_title(catalog: &Catalog, song: &Song) -> String {
    if !song.variant() {
        return String::new();
    }
    match catalog.song(&song.variant_of) {
        Some(origin) if !origin.title.is_empty() => origin.title.clone(),
        _ => song.variant_of.clone(),
    }
}

pub fn variant_line(title: &str) -> String {
    if title.trim().is_empty() {
        return String::new();
    }
    format!("本曲是《{title}》的曲绘变体，谱面与本体相同")
}

pub fn levels_line(song: &Song) -> String {
    let parts: Vec<String> = song
        .difficulties()
        .iter()
        .filter_map(|level| {
            song.chart(*level)
                .map(|chart| format!("{} {}", level.name(), constant_text(chart.constant, true)))
        })
        .collect();
    if parts.is_empty() {
        return String::from("无谱面数据");
    }
    parts.join(" / ")
}

pub fn history_block(song: &Song) -> String {
    let mut lines: Vec<String> = Vec::new();
    for level in song.difficulties() {
        let Some(chart) = song.chart(level) else {
            continue;
        };
        for change in &chart.history {
            lines.push(format!(
                "  {} v{} {} → {}",
                level.name(),
                change.version,
                constant_text(change.from, true),
                constant_text(change.to, true)
            ));
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("定数调整：\n{}", lines.join("\n"))
}

pub fn detail_footer(song: &Song) -> String {
    if !song.sp.trim().is_empty() {
        return format!("SP 说明：{}", song.sp.replace('\n', " "));
    }
    format!("发 /p score {} 查看你的成绩", song.title)
}

pub fn display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| {
            let code = ch as u32;
            let wide = (0x1100..=0x115f).contains(&code)
                || (0x2e80..=0xa4cf).contains(&code)
                || (0xac00..=0xd7a3).contains(&code)
                || (0xf900..=0xfaff).contains(&code)
                || (0xfe30..=0xfe6f).contains(&code)
                || (0xff00..=0xff60).contains(&code)
                || (0xffe0..=0xffe6).contains(&code);
            if wide {
                2
            } else {
                1
            }
        })
        .sum()
}

pub fn alias_lines(aliases: &[String]) -> String {
    let sep_width = display_width(ALIAS_SEP);
    let mut out = String::new();
    let mut width = 0usize;
    for (index, alias) in aliases.iter().enumerate() {
        let item = alias.trim();
        if item.is_empty() {
            continue;
        }
        let item_width = display_width(item);
        if out.is_empty() {
            out.push_str(item);
            width = item_width;
            continue;
        }
        let _ = index;
        if width + sep_width + item_width > ALIAS_WIDTH {
            out.push('\n');
            width = 0;
        } else {
            out.push_str(ALIAS_SEP);
            width += sep_width;
        }
        out.push_str(item);
        width += item_width;
    }
    out
}

pub fn song_request(catalog: &Catalog, song: &Song) -> cards::song::SongRequest {
    let mut levels: Vec<cards::song::SongLevel> = Vec::new();
    for level in song.all_difficulties() {
        let Some(chart) = song.chart(level) else {
            continue;
        };
        levels.push(cards::song::SongLevel {
            diff: level.name().to_string(),
            constant: chart.constant,
            known: chart.known(),
            const_text: String::new(),
            notes: chart.notes,
            charter: chart.charter.clone(),
        });
    }
    for extra in &song.extra {
        levels.push(cards::song::SongLevel {
            diff: extra.display(),
            constant: 0.0,
            known: true,
            const_text: extra.label.clone(),
            notes: extra.notes,
            charter: extra.charter.clone(),
        });
    }
    let mut history: Vec<cards::song::SongConstChange> = Vec::new();
    for level in song.all_difficulties() {
        let Some(chart) = song.chart(level) else {
            continue;
        };
        for change in &chart.history {
            history.push(cards::song::SongConstChange {
                diff: level.name().to_string(),
                version: change_version(change),
                from: change.from,
                to: change.to,
            });
        }
    }
    cards::song::SongRequest {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        illustrator: song.illustrator.clone(),
        chapter: catalog.chapter_display(&song.chapter),
        length: song.length.clone(),
        bpm: bpm_value(song),
        version: song_version(song),
        aliases: song.aliases.clone(),
        variant_of: variant_title(catalog, song),
        sp: song.sp.clone(),
        ranking: cards::song::SongRanking::default(),
        history,
        levels,
    }
}

fn level_constant(level: &cards::song::SongLevel) -> String {
    let custom = level.const_text.trim();
    if !custom.is_empty() {
        return custom.to_string();
    }
    format!("{:.1}", level.constant)
}

fn map_arg(pairs: Vec<(&str, Arg)>) -> Arg {
    let mut out: BTreeMap<String, Arg> = BTreeMap::new();
    for (key, value) in pairs {
        out.insert(key.to_string(), value);
    }
    Arg::Map(out)
}

fn detail_args(request: &cards::song::SongRequest, song: &Song) -> Args {
    let mut charts: Vec<Arg> = Vec::new();
    for level in &request.levels {
        charts.push(map_arg(vec![
            ("difficulty", Arg::Str(level.diff.clone())),
            ("constant", Arg::Str(level_constant(level))),
            ("charter", Arg::Str(dash(&level.charter))),
            ("notes", Arg::Str(notes_line(level.notes))),
        ]));
    }
    let mut args = Args::new();
    commands::put(&mut args, "title", request.title.clone());
    commands::put(&mut args, "composer", dash(&request.composer));
    commands::put(&mut args, "illustrator", dash(&request.illustrator));
    commands::put(&mut args, "chapter", dash(&request.chapter));
    commands::put(&mut args, "bpm", bpm_text(song));
    commands::put(&mut args, "length", dash(&request.length));
    commands::put(&mut args, "version", version_text(song));
    commands::put(&mut args, "variant", variant_line(&request.variant_of));
    args.insert("charts".to_string(), Arg::List(charts));
    commands::put(&mut args, "history", history_block(song));
    commands::put(&mut args, "footer", detail_footer(song));
    args
}

fn rank_per(levels: usize) -> usize {
    if levels == 0 {
        return 0;
    }
    (SONG_RANK_MAX / levels).max(1)
}

fn rank_levels(song: &Song, asked: Option<Difficulty>) -> Vec<Difficulty> {
    let levels = song.difficulties();
    let mut order: Vec<Difficulty> = Vec::with_capacity(levels.len());
    if let Some(asked) = asked {
        for level in &levels {
            if *level == asked {
                order.push(*level);
            }
        }
    }
    for level in levels.iter().rev() {
        if Some(*level) == asked {
            continue;
        }
        order.push(*level);
    }
    order
}

async fn song_ranking(
    service: &Arc<PhigrosService>,
    song: &Song,
    asked: Option<Difficulty>,
    user: &str,
) -> cards::song::SongRanking {
    let order = rank_levels(song, asked);
    if order.is_empty() {
        return cards::song::SongRanking::default();
    }
    let repo = service.repo();
    let per = rank_per(order.len());
    let mut out = cards::song::SongRanking::default();
    for level in order {
        let Ok((rows, total)) = repo.song_board(&song.id, level, per).await else {
            continue;
        };
        out.total += total as i32;
        for (index, row) in rows.iter().enumerate() {
            if out.rows.len() >= SONG_RANK_MAX {
                break;
            }
            let dan = overview::dan_of(row.challenge_rank);
            out.rows.push(cards::song::SongRankRow {
                diff: level.name().to_string(),
                rank: index as i32 + 1,
                name: row.nickname.clone(),
                avatar: row.avatar.clone(),
                score: row.score,
                acc: row.acc,
                dan_level: dan.level,
                dan_progress: dan.progress,
                self_row: !user.is_empty() && row.user_id == user,
            });
        }
    }
    out
}

pub async fn song_out(service: &Arc<PhigrosService>, user: &str, raw: &str) -> Outgoing {
    let Some(query) = query::parse_song_query(raw) else {
        return PhigrosService::notice(SONG_USAGE);
    };
    let catalog = service.catalog();
    let song = match resolve_song(catalog, &query.text) {
        SongMatch::Found(song) => song,
        SongMatch::Candidates(songs, total) => {
            return service.pick(raw, crate::service::SONG_PREFIX, &songs, total)
        }
        SongMatch::Missing => return PhigrosService::fail(PICK_MISSING),
    };
    let mut request = song_request(catalog, song);
    request.ranking = song_ranking(service, song, query.difficulty, user).await;
    let args = detail_args(&request, song);
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Song {
            title: song.title.clone(),
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::song::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_SONG,
        &request,
        move |kit| cards::song::draw(kit, &root, &drawn),
    );
    let mut out = service.text("song.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

async fn aliases_of(service: &Arc<PhigrosService>, song: &Song) -> Vec<String> {
    let mut aliases = song.aliases.clone();
    let Ok(entries) = service.repo().alias_entries(&song.id).await else {
        return aliases;
    };
    for entry in entries {
        let name = entry.alias.trim().to_string();
        if name.is_empty() {
            continue;
        }
        if entry.removed {
            aliases.retain(|item| item != &name);
        } else if !aliases.contains(&name) {
            aliases.push(name);
        }
    }
    aliases
}

pub async fn alias_out(service: &Arc<PhigrosService>, _user: &str, raw: &str) -> Outgoing {
    let text = raw.trim();
    if let Some((head, _)) = text.split_once(char::is_whitespace) {
        if matches!(head, "add" | "添加" | "del" | "delete" | "移除") {
            return PhigrosService::notice(ALIAS_READONLY);
        }
    }
    let Some(query) = query::parse_song_query(text) else {
        return PhigrosService::notice(ALIAS_USAGE);
    };
    let catalog = service.catalog();
    let song = match resolve_song(catalog, &query.text) {
        SongMatch::Found(song) => song,
        SongMatch::Candidates(songs, total) => {
            return service.pick(raw, ALIAS_PREFIX, &songs, total)
        }
        SongMatch::Missing => return PhigrosService::fail(PICK_MISSING),
    };
    let aliases = aliases_of(service, song).await;
    let mut args = Args::new();
    commands::put(&mut args, "title", song.title.clone());
    commands::put(&mut args, "aliases", alias_lines(&aliases));
    service.text("song.alias", &args)
}

pub fn search_row(
    catalog: &Catalog,
    song: &Song,
    picked: Difficulty,
) -> cards::search::SearchRow {
    let mut levels: Vec<cards::search::SearchLevel> = Vec::new();
    for level in song.all_difficulties() {
        let Some(chart) = song.chart(level) else {
            continue;
        };
        levels.push(cards::search::SearchLevel {
            diff: level.name().to_string(),
            const_text: constant_text(chart.constant, chart.known()),
            active: level == picked,
        });
    }
    cards::search::SearchRow {
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        group: catalog.chapter_display(&song.chapter),
        levels,
    }
}

fn title_hits(song: &Song, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let probe = catalog::normalize_title(needle);
    if probe.is_empty() {
        return true;
    }
    let mut names: Vec<&str> = vec![song.title.as_str(), song.composer.as_str()];
    names.extend(song.aliases.iter().map(|alias| alias.as_str()));
    names
        .iter()
        .any(|name| catalog::normalize_title(name).contains(&probe))
}

pub async fn search_out(service: &Arc<PhigrosService>, raw: &str) -> Outgoing {
    let Some(mut parsed) = query::parse(raw) else {
        return PhigrosService::notice(SEARCH_USAGE);
    };
    crate::cmd_query::resolve_chapter_keyword(service.catalog(), &mut parsed);
    let head = query::condition_text(&parsed);
    if head.is_empty() {
        return PhigrosService::notice(SEARCH_USAGE);
    }
    let catalog = service.catalog();
    let limit = if parsed.limit > 0 {
        parsed.limit.min(cards::search::SEARCH_MAX)
    } else {
        service
            .settings()
            .search_limit
            .min(cards::search::SEARCH_MAX)
    };
    let mut hits: Vec<(&Song, Difficulty, f64)> = Vec::new();
    for song in catalog.songs() {
        if song.variant() {
            continue;
        }
        if !title_hits(song, &parsed.query.title) {
            continue;
        }
        let mut best: Option<(Difficulty, f64)> = None;
        for level in song.all_difficulties() {
            if !parsed.query.difficulties.is_empty()
                && !parsed.query.difficulties.contains(&level)
            {
                continue;
            }
            let Some(chart) = song.chart(level) else {
                continue;
            };
            if parsed.query.constant.on() && !parsed.query.constant.contains(chart.constant) {
                continue;
            }
            if best
                .map(|(_, constant)| chart.constant > constant)
                .unwrap_or(true)
            {
                best = Some((level, chart.constant));
            }
        }
        let Some((level, constant)) = best else {
            continue;
        };
        hits.push((song, level, constant));
    }
    if hits.is_empty() {
        return PhigrosService::notice(format!("没有符合条件的曲目（{head}）"));
    }
    let total = hits.len();
    let ordering = match query::order_explicit(raw) {
        true => parsed.order,
        false => query::Ordering {
            key: query::ListOrder::Title,
            reverse: false,
        },
    };
    hits.sort_by(|left, right| {
        let base = match ordering.key {
            query::ListOrder::Constant => right
                .2
                .partial_cmp(&left.2)
                .unwrap_or(std::cmp::Ordering::Equal),
            _ => left.0.title.to_lowercase().cmp(&right.0.title.to_lowercase()),
        };
        let ordered = if ordering.reverse {
            base.reverse()
        } else {
            base
        };
        ordered.then_with(|| left.0.title.to_lowercase().cmp(&right.0.title.to_lowercase()))
    });
    let rows: Vec<cards::search::SearchRow> = hits
        .iter()
        .take(limit)
        .map(|(song, level, _)| search_row(catalog, song, *level))
        .collect();
    let shown = rows.len();
    let note = if total > shown {
        format!("命中较多，只画前 {shown} 首 —— 共 {total} 首")
    } else {
        String::new()
    };
    let backdrop = rows.first().map(|row| row.song_id.clone()).unwrap_or_default();
    let request = cards::search::SearchRequest {
        head: head.clone(),
        note,
        backdrop,
        rows,
    };
    let mut args = Args::new();
    commands::put(&mut args, "condition", head);
    commands::put(&mut args, "total", total.to_string());
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Plain,
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::search::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_SEARCH,
        &request,
        move |kit| cards::search::draw(kit, &root, &drawn),
    );
    let mut out = service.text("song.search.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

pub fn parse_rand(raw: &str) -> Option<(Span, Vec<Difficulty>)> {
    let mut filter = Span::default();
    let mut levels: Vec<Difficulty> = Vec::new();
    for field in raw.split_whitespace() {
        if let Some(level) = Difficulty::parse(field) {
            if level.valid() && !levels.contains(&level) {
                levels.push(level);
            }
            continue;
        }
        let span = span::parse_constant(field)?;
        if filter.on() {
            return None;
        }
        filter = span;
    }
    Some((filter, levels))
}

pub fn select_rows(
    all: &[Song],
    target_id: &str,
    difficulty: Difficulty,
) -> Vec<cards::songselect::SongSelectRow> {
    #[derive(Clone, Copy)]
    struct Cand<'a> {
        song: &'a Song,
        constant: f64,
        known: bool,
    }
    let mut pool: Vec<Cand> = Vec::with_capacity(all.len());
    let mut index: Option<usize> = None;
    for song in all {
        let Some(chart) = song.chart(difficulty) else {
            continue;
        };
        if song.id == target_id {
            index = Some(pool.len());
        }
        pool.push(Cand {
            song,
            constant: chart.constant,
            known: chart.known(),
        });
    }
    let Some(pick) = index else {
        return Vec::new();
    };
    if pool.is_empty() {
        return Vec::new();
    }
    let selected = pool[pick];
    pool.sort_by(|left, right| {
        right
            .known
            .cmp(&left.known)
            .then_with(|| {
                left.constant
                    .partial_cmp(&right.constant)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let mut index = pool
        .iter()
        .position(|item| item.song.id == selected.song.id)
        .unwrap_or_default();
    let mut unique: Vec<Cand> = Vec::with_capacity(pool.len());
    for (position, item) in pool.iter().enumerate() {
        if position > 0 {
            let last = unique[unique.len() - 1];
            if item.known == last.known && item.constant == last.constant {
                continue;
            }
        }
        unique.push(*item);
    }
    if unique.len() >= cards::songselect::SEL_ROWS {
        if let Some(slot) = unique
            .iter()
            .position(|item| item.known == selected.known && item.constant == selected.constant)
        {
            unique[slot] = selected;
        }
        pool = unique;
        index = pool
            .iter()
            .position(|item| item.song.id == selected.song.id)
            .unwrap_or_default();
    }
    let mut rows: Vec<cards::songselect::SongSelectRow> =
        Vec::with_capacity(cards::songselect::SEL_ROWS);
    for slot in 0..cards::songselect::SEL_ROWS {
        let offset = slot as isize - cards::songselect::SEL_ROW_SEL as isize;
        let cursor = index as isize + offset;
        if cursor < 0 || cursor >= pool.len() as isize {
            rows.push(empty_row());
            continue;
        }
        let item = pool[cursor as usize];
        rows.push(cards::songselect::SongSelectRow {
            title: item.song.title.clone(),
            constant: item.constant,
            known: item.known,
            selected: item.song.id == target_id,
        });
    }
    rows
}

fn empty_row() -> cards::songselect::SongSelectRow {
    cards::songselect::SongSelectRow {
        title: String::new(),
        constant: 0.0,
        known: false,
        selected: false,
    }
}

pub fn select_levels(song: &Song, active: Difficulty) -> Vec<cards::songselect::SongSelectLevel> {
    let mut out: Vec<cards::songselect::SongSelectLevel> = Vec::new();
    for level in song.difficulties() {
        let text = match song.chart(level) {
            Some(chart) if chart.known() => format!("{}", chart.constant.floor() as i32),
            _ => String::from("?"),
        };
        out.push(cards::songselect::SongSelectLevel {
            diff: level.name().to_string(),
            const_text: text,
            active: level == active,
        });
    }
    out
}

async fn rand_card(
    service: &Arc<PhigrosService>,
    song: &Song,
    difficulty: Difficulty,
    raw: &str,
    user: &str,
) -> Outgoing {
    let catalog = service.catalog();
    let chart = song.chart(difficulty);
    let player = rand_player(service, user).await;
    let request = cards::songselect::SongSelectRequest {
        backdrop: song.id.clone(),
        song_id: song.id.clone(),
        title: song.title.clone(),
        composer: song.composer.clone(),
        diff: difficulty.name().to_string(),
        constant: chart.map(|item| item.constant).unwrap_or_default(),
        known: chart.map(|item| item.known()).unwrap_or_default(),
        levels: select_levels(song, difficulty),
        rows: select_rows(catalog.songs(), &song.id, difficulty),
        strip: Vec::new(),
        player,
    };
    let mut args = Args::new();
    commands::put(&mut args, "title", song.title.clone());
    commands::put(&mut args, "composer", dash(&song.composer));
    commands::put(&mut args, "difficulty", difficulty.name());
    commands::put(
        &mut args,
        "constant",
        constant_text(
            request.constant,
            request.known,
        ),
    );
    commands::put(&mut args, "levels", levels_line(song));
    let keyboard = crate::buttons::keyboard(
        crate::buttons::Page::Rand {
            raw: raw.to_string(),
            title: song.title.clone(),
        },
        &crate::buttons::ButtonCtx { page: 1, pages: 1 },
    );
    let root = service.root().to_path_buf();
    let (width, height) = cards::songselect::canvas_size(&request);
    let drawn = request.clone();
    let card = service.card(
        width,
        height,
        LABEL_SELECT,
        &request,
        move |kit| cards::songselect::draw(kit, &root, &drawn),
    );
    let mut out = service.text("song.rand.image", &args).card(card);
    if let Some(keyboard) = keyboard {
        out = out.keyboard(keyboard);
    }
    out
}

async fn rand_player(service: &Arc<PhigrosService>, user: &str) -> cards::player::Player {
    if user.is_empty() {
        return cards::player::Player::default();
    }
    let Ok(Some(binding)) = service.binding(user).await else {
        return cards::player::Player::default();
    };
    let profile = service.profile(user).await.ok().flatten();
    service.player(&binding, profile.as_ref())
}

pub async fn random_out(service: &Arc<PhigrosService>, user: &str, raw: &str) -> Outgoing {
    let Some((filter, levels)) = parse_rand(raw) else {
        return PhigrosService::notice(RAND_USAGE);
    };
    let catalog = service.catalog();
    let mut pool: Vec<(&Song, Difficulty)> = Vec::new();
    for song in catalog.songs() {
        if song.variant() {
            continue;
        }
        for level in song.difficulties() {
            if !levels.is_empty() && !levels.contains(&level) {
                continue;
            }
            let Some(chart) = song.chart(level) else {
                continue;
            };
            if filter.on() && !filter.contains(chart.constant) {
                continue;
            }
            pool.push((song, level));
        }
    }
    if pool.is_empty() {
        return PhigrosService::notice(RAND_EMPTY);
    }
    let pick = rand_index(pool.len());
    let (song, difficulty) = pool[pick];
    rand_card(service, song, difficulty, raw, user).await
}

fn rand_index(len: usize) -> usize {
    use rand::Rng;
    if len <= 1 {
        return 0;
    }
    rand::thread_rng().gen_range(0..len)
}

pub async fn play_out(service: &Arc<PhigrosService>, raw: &str) -> Outgoing {
    let Some(query) = query::parse_song_query(raw) else {
        return PhigrosService::notice(PLAY_USAGE);
    };
    let catalog = service.catalog();
    let song = match resolve_song(catalog, &query.text) {
        SongMatch::Found(song) => song,
        SongMatch::Candidates(songs, total) => {
            return service.pick(raw, PLAY_PREFIX, &songs, total)
        }
        SongMatch::Missing => return PhigrosService::fail(PICK_MISSING),
    };
    if song.audio_url.trim().is_empty() {
        return PhigrosService::notice(NO_AUDIO);
    }
    let mut args = Args::new();
    commands::put(&mut args, "cover", song.ill_url.clone());
    commands::put(&mut args, "link", song.wiki_url.clone());
    commands::put(&mut args, "title", song.title.clone());
    commands::put(&mut args, "artist", song.composer.clone());
    service
        .text("song.play", &args)
        .voice(song.audio_url.clone())
        .no_footer()
}

pub async fn ill_out(service: &Arc<PhigrosService>, raw: &str) -> Outgoing {
    let Some(query) = query::parse_song_query(raw) else {
        return PhigrosService::notice(ILL_USAGE);
    };
    let catalog = service.catalog();
    let song = match resolve_song(catalog, &query.text) {
        SongMatch::Found(song) => song,
        SongMatch::Candidates(songs, total) => {
            return service.pick(raw, ILL_PREFIX, &songs, total)
        }
        SongMatch::Missing => return PhigrosService::fail(PICK_MISSING),
    };
    if let Some(level) = query.difficulty {
        if song.chart(level).is_none() {
            return PhigrosService::notice(
                NO_CHART_FOR_DIFFICULTY.replace("%s", level.name()),
            );
        }
    }
    if song.ill_url.trim().is_empty() {
        return PhigrosService::notice(NO_ILLUST);
    }
    let key = cards::illust_key(service.root(), &song.id);
    let fallback = format!("illust/{}.jpg", song.id);
    let Some(attachment) = service.attachment(&key, &fallback) else {
        return PhigrosService::notice(NO_ILLUST);
    };
    let mut args = Args::new();
    commands::put(&mut args, "title", song.title.clone());
    commands::put(&mut args, "composer", dash(&song.composer));
    commands::put(&mut args, "illustrator", dash(&song.illustrator));
    service.text("song.ill", &args).image(attachment)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn song(id: &str, title: &str, chapter: &str, levels: &[(Difficulty, f64)]) -> Song {
        let mut item = Song {
            id: id.to_string(),
            title: title.to_string(),
            composer: String::from("Aurorix"),
            illustrator: String::from("Aurorix"),
            chapter: chapter.to_string(),
            length: String::from("02:00"),
            bpm: 180,
            version: 3,
            version_label: String::from("3.9.0"),
            ..Song::default()
        };
        for (level, constant) in levels {
            item.charts[level.index()] = Chart {
                difficulty: *level,
                constant: *constant,
                charter: String::from("charter"),
                notes: Notes {
                    tap: 100,
                    drag: 20,
                    hold: 10,
                    flick: 5,
                },
                ..Chart::default()
            };
        }
        item
    }

    #[test]
    fn alias_lines_wrap_by_display_width() {
        let aliases = vec![
            String::from("short"),
            String::from("另一个长一点的别名"),
            String::from("third"),
        ];
        let text = alias_lines(&aliases);
        assert_eq!(display_width("霓虹灯牌"), 8);
        assert_eq!(display_width("abc"), 3);
        assert!(text.contains(ALIAS_SEP));
        for line in text.lines() {
            assert!(display_width(line) <= ALIAS_WIDTH + 4);
        }
        assert_eq!(alias_lines(&[]), String::new());
        assert_eq!(alias_lines(&[String::from("solo")]), String::from("solo"));
    }

    #[test]
    fn select_rows_pins_the_target_to_the_middle() {
        let mut songs: Vec<Song> = Vec::new();
        for step in 0..12 {
            let constant = 10.0 + step as f64;
            songs.push(song(
                &format!("s{step}"),
                &format!("Song {step}"),
                "Chapter 5 霓虹灯牌",
                &[(Difficulty::In, constant)],
            ));
        }
        let rows = select_rows(&songs, "s6", Difficulty::In);
        assert_eq!(rows.len(), cards::songselect::SEL_ROWS);
        assert!(rows[cards::songselect::SEL_ROW_SEL].selected);
        assert_eq!(rows[cards::songselect::SEL_ROW_SEL].title, "Song 6");
        let selected: Vec<&cards::songselect::SongSelectRow> =
            rows.iter().filter(|row| row.selected).collect();
        assert_eq!(selected.len(), 1);
        let filled: Vec<f64> = rows
            .iter()
            .filter(|row| !row.title.is_empty())
            .map(|row| row.constant)
            .collect();
        assert!(filled.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(select_rows(&songs, "missing", Difficulty::In).is_empty());
        assert!(select_rows(&songs, "s0", Difficulty::At).is_empty());
    }

    #[test]
    fn select_levels_mark_only_the_active_difficulty() {
        let item = song(
            "a",
            "Alpha",
            "Chapter 5 霓虹灯牌",
            &[(Difficulty::Ez, 4.0), (Difficulty::In, 14.7)],
        );
        let levels = select_levels(&item, Difficulty::In);
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].diff, "EZ");
        assert_eq!(levels[0].const_text, "4");
        assert_eq!(levels[1].const_text, "14");
        assert!(levels[1].active);
        assert!(!levels[0].active);
    }

    #[test]
    fn parse_rand_reads_one_span_and_many_levels() {
        let (filter, levels) = parse_rand(">14 IN").expect("定数与难度");
        assert!(filter.on());
        assert!(filter.contains(15.0));
        assert!(!filter.contains(13.0));
        assert_eq!(levels, vec![Difficulty::In]);
        let (filter, levels) = parse_rand("").expect("空条件");
        assert!(!filter.on());
        assert!(levels.is_empty());
        assert!(parse_rand("15+ AT").is_some());
        assert!(
            parse_rand(">10 >11").is_none(),
            "两个定数条件是打错了"
        );
        assert!(parse_rand("14 nonsense").is_none());
    }

    #[test]
    fn song_text_helpers_follow_go() {
        let mut item = song(
            "a",
            "Alpha",
            "Chapter 5 霓虹灯牌",
            &[(Difficulty::In, 15.5)],
        );
        assert_eq!(bpm_value(&item), "180");
        assert_eq!(song_version(&item), "3.9.0");
        assert_eq!(version_text(&item), "v3");
        assert_eq!(levels_line(&item), "IN 15.5");
        assert_eq!(notes_line(item.charts[Difficulty::In.index()].notes), "135（Tap 100 / Drag 20 / Hold 10 / Flick 5）");
        assert_eq!(detail_footer(&item), "发 /p score Alpha 查看你的成绩");
        item.sp = String::from("愚人节\n谱面");
        assert_eq!(detail_footer(&item), "SP 说明：愚人节 谱面");
        item.variant_of = String::from("a");
        assert!(variant_line("Alpha").contains("《Alpha》"));
        assert_eq!(variant_line("  "), String::new());
        assert_eq!(history_block(&item), String::new());
    }
}
