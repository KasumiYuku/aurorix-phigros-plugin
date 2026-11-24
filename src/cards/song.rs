use std::path::Path;
use std::sync::OnceLock;

use aurorix_render::skia_safe::{Image, Paint, Rect};
use aurorix_render::{CanvasKit, Color, GradientStop, LinearGradient};

use super::memo;
use super::player;
use super::settlement;
use super::sketch;
use crate::model::Notes;

pub const CANVAS_W: f32 = 1920.0;

pub const MARGIN_X: f32 = 100.0;

pub const CONTENT_RIGHT: f32 = 1820.0;

pub const CONTENT_W: f32 = 1720.0;

pub const HEAD_TOP: f32 = 24.0;

pub const HEAD_TITLE_SIZE: f32 = 52.0;

pub const HEAD_NOTE_SIZE: f32 = 13.0;

pub const HEAD_H: f32 = HEAD_TITLE_SIZE + 10.0 + HEAD_NOTE_SIZE;

pub const HEAD_TITLE_BASELINE: f32 = HEAD_TOP + HEAD_TITLE_SIZE;

pub const HEAD_NOTE_BASELINE: f32 = HEAD_TOP + HEAD_H;

pub const BLOCKS_TOP: f32 = HEAD_TOP + HEAD_H + 28.0;

pub const BLOCK_GAP: f32 = 24.0;

pub const CARD_PAD: f32 = 28.0;

pub const CARD_TITLE_SIZE: f32 = 22.0;

pub const CARD_TIGHT_GAP: f32 = 16.0;

pub const CARD_RADIUS: f32 = 16.0;

pub const CARD_FROST: f32 = 22.0;

pub const CARD_NOTE_SIZE: f32 = 13.0;

pub const CARD_TEXT_SIZE: f32 = 18.0;

pub const CARD_SUB_SIZE: f32 = 16.0;

pub const MID_RATIO: f32 = 0.36;

pub const COVER_W: f32 = 900.0;

pub const COVER_H: f32 = COVER_W * 1080.0 / 2048.0;

pub const COVER_PAD: f32 = 36.0;

pub const COVER_TITLE_SIZE: f32 = 40.0;

pub const COVER_NOTE_SIZE: f32 = 15.0;

pub const COVER_SHADE: f32 = COVER_H * 0.62;

pub const COVER_SHADE_TOP: f32 = 234.0;

pub const RANK_GAP: f32 = 28.0;

pub const RANK_ROWS: usize = 12;

pub const RANK_HEAD_DROP: f32 = 38.0;

pub const RANK_DIFF_RIGHT: f32 = 26.0;

pub const RANK_INDEX_RIGHT: f32 = 78.0;

pub const RANK_AVATAR_X: f32 = 90.0;

pub const RANK_AVATAR_SIZE: f32 = 28.0;

pub const RANK_AVATAR_RADIUS: f32 = CARD_RADIUS * 0.5;

pub const RANK_BADGE_W: f32 = 61.0;

pub const RANK_BADGE_GAP: f32 = 12.0;

pub const RANK_BADGE_RATIO: f32 = 20.0 / 61.0;

pub const RANK_SCORE_W: f32 = 120.0;

pub const RANK_ACC_W: f32 = 90.0;

pub const RANK_COL_GAP: f32 = 24.0;

pub const RANK_DOT_GAP: f32 = 11.0;

pub const RANK_DOT: f32 = 3.0;

pub const RANK_EMPTY_RATIO: f32 = 0.19;

pub const INFO_ALIAS_LABEL_DROP: f32 = CARD_TIGHT_GAP * 1.4;

pub const INFO_ALIAS_PITCH: f32 = CARD_TEXT_SIZE + CARD_TIGHT_GAP * 0.5;

pub const INFO_ALIAS_VALUE_DROP: f32 = 12.0;

pub const INFO_ALIAS_MAX_LINES: usize = 4;

pub const INFO_HIST_ROW_H: f32 = 40.0;

pub const NOTE_PITCH: f32 = CARD_TEXT_SIZE + CARD_TIGHT_GAP * 0.6;

pub const NOTE_MAX_LINES: usize = 3;

pub const TITLE_SIZE_STEP: f32 = 2.0;

pub const ROW_H: f32 = 72.0;

pub const ROW_TEXT_DROP: f32 = 32.0;

pub const ROW_BAR_DROP: f32 = 50.0;

pub const BAR_H: f32 = 8.0;

pub const COL_GAP: f32 = 24.0;

pub const CHIP_W: f32 = 96.0;

pub const CHIP_H: f32 = 34.0;

pub const CHIP_ALPHA: u8 = 0x2E;

pub const ROW_VALUE_SIZE: f32 = 21.0;

pub const ROW_TEXT_SIZE: f32 = 17.0;

pub const HEAD_SIZE: f32 = 15.0;

pub const INFO_COLS: usize = 4;

pub const INFO_ROWS: usize = 2;

pub const INFO_ROW_H: f32 = 56.0;

pub const INFO_VALUE_DROP: f32 = 12.0;

pub const TIP_DROP: f32 = 32.0;

pub const BOTTOM_MARGIN: f32 = 32.0;

pub const TITLE_SIZE_MAX: f32 = 52.0;

pub const TITLE_SIZE_MIN: f32 = 28.0;

pub const COL_WEIGHTS: [f32; 8] = [0.075, 0.085, 0.26, 0.095, 0.095, 0.095, 0.095, 0.10];

pub const NOTE_KINDS: [&str; 4] = ["Tap", "Drag", "Hold", "Flick"];

pub const NOTE_INKS: [Color; 4] = [
    Color::new(0x54, 0xD6, 0xFF, 0xFF),
    Color::new(0xFF, 0xC4, 0x4D, 0xFF),
    Color::new(0x5A, 0xE8, 0xC0, 0xFF),
    Color::new(0xFF, 0x5C, 0x8A, 0xFF),
];

pub const SHEET_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0E);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const LABEL_INK: Color = Color::new(0xB4, 0xBA, 0xC6, 0xFF);

pub const NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const PLACEHOLDER_INK: Color = Color::new(0x8A, 0x92, 0xA2, 0xFF);

pub const ACCENT_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

pub const DOT_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x5E);

pub const SHADE_INK: Color = Color::new(0x00, 0x00, 0x00, 0xFF);

pub const COVER_FALLBACK: Color = Color::new(0x22, 0x26, 0x30, 0xFF);

pub const HEAD_TITLE: &str = "SONG";

pub const HEAD_NOTE: &str = "曲目详情";

pub const MATRIX_TITLE: &str = "难度与物量";

pub const INFO_TITLE: &str = "曲目资料";

pub const INFO_ALIAS_LABEL: &str = "别名";

pub const INFO_HIST_LABEL: &str = "定数变迁";

pub const INFO_LEVEL_COUNT_LABEL: &str = "难度数";

pub const RANK_TITLE: &str = "谱面排行";

pub const RANK_EMPTY: &str = "暂无成绩";

pub const RANK_TOTAL_SUFFIX: &str = " 条成绩";

pub const INFO_ALIAS_SUFFIX: &str = " 个别名";

pub const LEVEL_COUNT_SUFFIX: &str = " 档";

pub const VARIANT_PREFIX: &str = "本曲是《";

pub const VARIANT_SUFFIX: &str = "》的曲绘变体，谱面与本体相同";

pub const SEG_SEPARATOR: &str = "   ·   ";

pub const ALIAS_SEPARATOR: &str = " / ";

pub const ELLIPSIS_TAIL: &str = " / …";

pub const UNKNOWN_CONST: &str = "未收录";

pub const DASH: &str = "—";

pub const ARROW_UP: &str = "↑";

pub const ARROW_DOWN: &str = "↓";

pub const ARROW_FLAT: &str = "→";

pub const FOOTER_TEXT: &str = "成绩来自上次同步记录 · 使用 /p update 可更新";

pub const MATRIX_EMPTY: &str = "曲库还没收录这首曲子的谱面";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongLevel {
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub const_text: String,
    pub notes: Notes,
    pub charter: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongConstChange {
    pub diff: String,
    pub version: String,
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongRankRow {
    pub diff: String,
    pub rank: i32,
    pub name: String,
    pub avatar: String,
    pub score: i32,
    pub acc: f64,
    pub dan_level: i32,
    pub dan_progress: i32,
    pub self_row: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongRanking {
    pub total: i32,
    pub rows: Vec<SongRankRow>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongRequest {
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub illustrator: String,
    pub chapter: String,
    pub length: String,
    pub bpm: String,
    pub version: String,
    pub aliases: Vec<String>,
    pub variant_of: String,
    pub sp: String,
    pub ranking: SongRanking,
    pub history: Vec<SongConstChange>,
    pub levels: Vec<SongLevel>,
}

pub fn rows(req: &SongRequest) -> usize {
    req.levels.len().max(1)
}

pub fn hist_rows(req: &SongRequest) -> usize {
    req.history.len()
}

fn font_book() -> &'static aurorix_render::FontBook {
    static BOOK: OnceLock<aurorix_render::FontBook> = OnceLock::new();
    BOOK.get_or_init(|| aurorix_render::FontBook::new(64, None))
}

fn measure(family: &str, size: f32, value: &str) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    let style = sketch::text_style(family, size, SCORE_INK);
    aurorix_render::measure_text(font_book(), value, &style).unwrap_or(0.0)
}

fn fits(family: &str, size: f32, value: &str, width: f32) -> bool {
    measure(family, size, value).round() <= width.round()
}

fn collapse_lines(value: &str) -> String {
    value
        .split(['\n', '\r'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn wrap_lines(
    body: &str,
    max_lines: usize,
    width: f32,
    family: &str,
    size: f32,
) -> Vec<String> {
    let body = collapse_lines(body);
    if body.trim().is_empty() || max_lines == 0 {
        return Vec::new();
    }
    let mut lines: Vec<String> = Vec::with_capacity(max_lines);
    let mut current = String::new();
    let mut truncated = false;
    for word in body.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if fits(family, size, &candidate, width) {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            if lines.len() >= max_lines {
                truncated = true;
                break;
            }
        }
        if fits(family, size, word, width) {
            current.push_str(word);
            continue;
        }
        for ch in word.chars() {
            let mut probe = current.clone();
            probe.push(ch);
            if !current.is_empty() && !fits(family, size, &probe, width) {
                lines.push(std::mem::take(&mut current));
                if lines.len() >= max_lines {
                    truncated = true;
                    break;
                }
            }
            current.push(ch);
        }
        if truncated {
            break;
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    if truncated {
        if let Some(last) = lines.last_mut() {
            *last = sketch::truncate_width(last, width, |probe| measure(family, size, probe));
        }
    }
    lines
}

pub fn variant_note(variant_of: &str) -> String {
    let origin = variant_of.trim();
    if origin.is_empty() {
        return String::new();
    }
    format!("{VARIANT_PREFIX}{origin}{VARIANT_SUFFIX}")
}

pub fn note_body(req: &SongRequest) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(2);
    let sp = collapse_lines(&req.sp);
    if !sp.trim().is_empty() {
        parts.push(sp);
    }
    let note = variant_note(&req.variant_of);
    if !note.is_empty() {
        parts.push(note);
    }
    parts.join(SEG_SEPARATOR)
}

pub fn note_lines(req: &SongRequest) -> Vec<String> {
    wrap_lines(
        &note_body(req),
        NOTE_MAX_LINES,
        CONTENT_W,
        sketch::FONT_BODY,
        CARD_TEXT_SIZE,
    )
}

pub fn alias_fits_cell(req: &SongRequest) -> bool {
    let joined = req.aliases.join(ALIAS_SEPARATOR);
    let joined = joined.trim();
    if joined.is_empty() {
        return true;
    }
    fits(
        sketch::FONT_DISPLAY,
        CARD_TEXT_SIZE,
        joined,
        info_value_w(rows(req)),
    )
}

pub fn alias_lines(req: &SongRequest) -> Vec<String> {
    if alias_fits_cell(req) {
        return Vec::new();
    }
    let items: Vec<String> = req
        .aliases
        .iter()
        .map(|alias| alias.trim().to_string())
        .filter(|alias| !alias.is_empty())
        .collect();
    if items.is_empty() {
        return Vec::new();
    }
    let width = alias_value_w(rows(req));
    let mut lines: Vec<String> = Vec::with_capacity(INFO_ALIAS_MAX_LINES);
    let mut current = String::new();
    let mut truncated = false;
    for item in items {
        let candidate = if current.is_empty() {
            item.clone()
        } else {
            format!("{current}{ALIAS_SEPARATOR}{item}")
        };
        if fits(sketch::FONT_DISPLAY, CARD_TEXT_SIZE, &candidate, width) {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
            if lines.len() >= INFO_ALIAS_MAX_LINES {
                truncated = true;
                break;
            }
        }
        current = item;
    }
    if !truncated && !current.is_empty() {
        lines.push(current);
    }
    if lines.len() > INFO_ALIAS_MAX_LINES {
        lines.truncate(INFO_ALIAS_MAX_LINES);
        truncated = true;
    }
    if truncated {
        if let Some(last) = lines.last_mut() {
            last.push_str(ELLIPSIS_TAIL);
        }
    }
    lines
}

pub fn card_content(rect: Rect, title_size: f32, has_title: bool) -> Rect {
    let mut top = CARD_PAD;
    if has_title && title_size > 0.0 {
        top += title_size + CARD_TIGHT_GAP;
    }
    Rect::from_xywh(
        rect.left() + CARD_PAD,
        rect.top() + top,
        rect.width() - 2.0 * CARD_PAD,
        rect.height() - CARD_PAD - top,
    )
}

pub fn main_card() -> Rect {
    Rect::from_xywh(MARGIN_X, BLOCKS_TOP, CONTENT_W, COVER_H + 2.0 * CARD_PAD)
}

pub fn main_content() -> Rect {
    card_content(main_card(), 0.0, false)
}

pub fn cover_rect() -> Rect {
    let c = main_content();
    Rect::from_xywh(c.left(), c.top(), COVER_W, COVER_H)
}

pub fn cover_title_w() -> f32 {
    COVER_W - 2.0 * COVER_PAD
}

pub fn cover_title_at() -> (f32, f32) {
    let cover = cover_rect();
    (cover.left() + COVER_PAD, cover.bottom() - COVER_PAD)
}

pub fn cover_chapter_at() -> (f32, f32) {
    let (x, baseline) = cover_title_at();
    (x, baseline - COVER_TITLE_SIZE - 12.0)
}

pub fn rank_content() -> Rect {
    let c = main_content();
    let x = cover_rect().right() + RANK_GAP;
    Rect::from_xywh(x, c.top(), c.right() - x, c.height())
}

pub fn rank_head_at() -> (f32, f32) {
    let c = rank_content();
    (c.left(), c.top() + CARD_SUB_SIZE)
}

pub fn rank_note_at() -> (f32, f32) {
    let c = rank_content();
    (c.right(), c.top() + CARD_SUB_SIZE)
}

pub fn rank_rows_top() -> f32 {
    rank_content().top() + RANK_HEAD_DROP
}

pub fn rank_pitch() -> f32 {
    (rank_content().bottom() - rank_rows_top()) / RANK_ROWS as f32
}

pub fn rank_row_base(index: usize) -> f32 {
    rank_rows_top() + (index as f32 + 0.5) * rank_pitch() + CARD_SUB_SIZE * MID_RATIO
}

pub fn rank_empty_at() -> (f32, f32, f32) {
    let c = rank_content();
    let top = rank_rows_top();
    let size = (c.bottom() - top) * RANK_EMPTY_RATIO;
    (
        c.left() + c.width() / 2.0,
        (top + c.bottom()) / 2.0 + size * MID_RATIO,
        size,
    )
}

pub fn rank_diff_right() -> f32 {
    rank_content().left() + RANK_DIFF_RIGHT
}

pub fn rank_index_right() -> f32 {
    rank_content().left() + RANK_INDEX_RIGHT
}

pub fn rank_avatar(index: usize) -> Rect {
    let c = rank_content();
    let mid = rank_row_base(index) - CARD_SUB_SIZE * MID_RATIO;
    Rect::from_xywh(
        c.left() + RANK_AVATAR_X,
        mid - RANK_AVATAR_SIZE / 2.0,
        RANK_AVATAR_SIZE,
        RANK_AVATAR_SIZE,
    )
}

pub fn rank_badge_h() -> f32 {
    RANK_BADGE_W * RANK_BADGE_RATIO
}

pub fn rank_badge(index: usize) -> Rect {
    let height = rank_badge_h();
    let mid = rank_row_base(index) - CARD_SUB_SIZE * MID_RATIO;
    Rect::from_xywh(
        rank_avatar(index).right() + RANK_BADGE_GAP,
        mid - height / 2.0,
        RANK_BADGE_W,
        height,
    )
}

pub fn rank_badge_text_size() -> f32 {
    24.0 * RANK_BADGE_W / 100.0
}

pub fn rank_name_left() -> f32 {
    rank_badge(0).right() + RANK_BADGE_GAP
}

pub fn rank_score_right() -> f32 {
    rank_content().right()
}

pub fn rank_acc_right() -> f32 {
    rank_score_right() - RANK_SCORE_W - RANK_COL_GAP
}

pub fn rank_name_w() -> f32 {
    rank_acc_right() - RANK_ACC_W - rank_name_left() - CARD_TIGHT_GAP
}

pub fn header_h() -> f32 {
    CARD_TIGHT_GAP * 0.8 + CARD_NOTE_SIZE + CARD_TIGHT_GAP * 1.2
}

pub fn matrix_content_h(rows: usize) -> f32 {
    header_h() + rows.max(1) as f32 * ROW_H
}

pub fn matrix_card(rows: usize) -> Rect {
    Rect::from_xywh(
        MARGIN_X,
        main_card().bottom() + BLOCK_GAP,
        CONTENT_W,
        matrix_content_h(rows) + 2.0 * CARD_PAD + CARD_TITLE_SIZE + CARD_TIGHT_GAP,
    )
}

pub fn matrix_content(rows: usize) -> Rect {
    card_content(matrix_card(rows), CARD_TITLE_SIZE, true)
}

pub fn header_base(rows: usize) -> f32 {
    matrix_content(rows).top() + CARD_TIGHT_GAP * 0.8 + CARD_NOTE_SIZE
}

pub fn row_top(rows: usize, index: usize) -> f32 {
    header_base(rows) + CARD_TIGHT_GAP * 1.2 + index as f32 * ROW_H
}

pub fn row_base(rows: usize, index: usize) -> f32 {
    row_top(rows, index) + ROW_TEXT_DROP
}

pub fn row_bar(rows: usize, index: usize) -> Rect {
    let left = note_group_left(rows);
    Rect::from_xywh(
        left,
        row_top(rows, index) + ROW_BAR_DROP,
        matrix_content(rows).right() - left,
        BAR_H,
    )
}

pub fn col_right(rows: usize, index: usize) -> f32 {
    let c = matrix_content(rows);
    let total: f32 = COL_WEIGHTS.iter().sum();
    let mut x = c.left();
    for weight in COL_WEIGHTS.iter().take(index + 1) {
        x += c.width() * weight / total;
    }
    x
}

pub fn diff_chip(rows: usize, index: usize) -> Rect {
    let base = row_base(rows, index);
    Rect::from_xywh(
        matrix_content(rows).left(),
        base - CHIP_H / 2.0,
        CHIP_W,
        CHIP_H,
    )
}

pub fn diff_text_at(rows: usize, index: usize) -> (f32, f32) {
    let chip = diff_chip(rows, index);
    (
        chip.left() + chip.width() / 2.0,
        chip.top() + chip.height() / 2.0 + CARD_SUB_SIZE * MID_RATIO,
    )
}

pub fn const_right(rows: usize) -> f32 {
    col_right(rows, 1)
}

pub fn charter_x(rows: usize) -> f32 {
    col_right(rows, 1) + COL_GAP
}

pub fn charter_w(rows: usize) -> f32 {
    col_right(rows, 2) - charter_x(rows) - COL_GAP
}

pub fn note_group_left(rows: usize) -> f32 {
    col_right(rows, 2)
}

pub fn note_col_right(rows: usize, index: usize) -> f32 {
    col_right(rows, 3 + index)
}

pub fn total_right(rows: usize) -> f32 {
    col_right(rows, 7)
}

pub fn info_content_h() -> f32 {
    INFO_ROWS as f32 * INFO_ROW_H
}

pub fn alias_h(alias_lines: usize) -> f32 {
    if alias_lines == 0 {
        return 0.0;
    }
    INFO_ALIAS_LABEL_DROP
        + CARD_NOTE_SIZE
        + INFO_ALIAS_VALUE_DROP
        + alias_lines as f32 * INFO_ALIAS_PITCH
}

pub fn hist_h(hist_rows: usize) -> f32 {
    if hist_rows == 0 {
        return 0.0;
    }
    INFO_ALIAS_LABEL_DROP + CARD_NOTE_SIZE + hist_rows as f32 * INFO_HIST_ROW_H
}

pub fn info_card(rows: usize, hist_rows: usize, alias_lines: usize) -> Rect {
    Rect::from_xywh(
        MARGIN_X,
        matrix_card(rows).bottom() + BLOCK_GAP,
        CONTENT_W,
        info_content_h()
            + alias_h(alias_lines)
            + hist_h(hist_rows)
            + 2.0 * CARD_PAD
            + CARD_TITLE_SIZE
            + CARD_TIGHT_GAP,
    )
}

pub fn info_content(rows: usize) -> Rect {
    card_content(info_card(rows, 0, 0), CARD_TITLE_SIZE, true)
}

pub fn info_cell(rows: usize, col: usize, row: usize) -> Rect {
    let c = info_content(rows);
    let w = c.width() / INFO_COLS as f32;
    Rect::from_xywh(
        c.left() + col as f32 * w,
        c.top() + row as f32 * INFO_ROW_H,
        w,
        INFO_ROW_H,
    )
}

pub fn info_label_at(rows: usize, col: usize, row: usize) -> (f32, f32) {
    let cell = info_cell(rows, col, row);
    (cell.left(), cell.top() + CARD_NOTE_SIZE)
}

pub fn info_value_at(rows: usize, col: usize, row: usize) -> (f32, f32) {
    let cell = info_cell(rows, col, row);
    (
        cell.left(),
        cell.top() + CARD_NOTE_SIZE + INFO_VALUE_DROP + CARD_TEXT_SIZE,
    )
}

pub fn info_value_w(rows: usize) -> f32 {
    info_content(rows).width() / INFO_COLS as f32 - CARD_TIGHT_GAP * 1.5
}

pub fn alias_label_at(rows: usize) -> (f32, f32) {
    let c = info_content(rows);
    (
        c.left(),
        c.top() + info_content_h() + INFO_ALIAS_LABEL_DROP + CARD_NOTE_SIZE,
    )
}

pub fn alias_value_at(rows: usize, index: usize) -> (f32, f32) {
    let (x, baseline) = alias_label_at(rows);
    (
        x,
        baseline + INFO_ALIAS_VALUE_DROP + CARD_TEXT_SIZE + index as f32 * INFO_ALIAS_PITCH,
    )
}

pub fn alias_value_w(rows: usize) -> f32 {
    info_content(rows).width()
}

pub fn hist_label_at(rows: usize, alias_lines: usize) -> (f32, f32) {
    let c = info_content(rows);
    (
        c.left(),
        c.top() + info_content_h() + alias_h(alias_lines) + INFO_ALIAS_LABEL_DROP + CARD_NOTE_SIZE,
    )
}

pub fn hist_row_base(rows: usize, alias_lines: usize, index: usize) -> f32 {
    let (_, baseline) = hist_label_at(rows, alias_lines);
    baseline + CARD_TIGHT_GAP + CARD_TEXT_SIZE + index as f32 * INFO_HIST_ROW_H
}

pub fn note_at(rows: usize, hist_rows: usize, alias_lines: usize, line: usize) -> (f32, f32) {
    let top = info_card(rows, hist_rows, alias_lines).bottom() + INFO_ALIAS_LABEL_DROP;
    (MARGIN_X, top + CARD_TEXT_SIZE + line as f32 * NOTE_PITCH)
}

pub fn note_block_h(note_lines: usize) -> f32 {
    if note_lines == 0 {
        return 0.0;
    }
    INFO_ALIAS_LABEL_DROP + note_lines as f32 * NOTE_PITCH
}

pub fn tip_baseline(rows: usize, hist_rows: usize, alias_lines: usize, note_lines: usize) -> f32 {
    info_card(rows, hist_rows, alias_lines).bottom() + note_block_h(note_lines) + TIP_DROP
}

pub fn canvas_height(rows: usize, hist_rows: usize, alias_lines: usize, note_lines: usize) -> f32 {
    info_card(rows, hist_rows, alias_lines).bottom()
        + note_block_h(note_lines)
        + TIP_DROP
        + CARD_NOTE_SIZE
        + BOTTOM_MARGIN
}

pub fn canvas_size(req: &SongRequest) -> (i32, i32) {
    (
        CANVAS_W.round() as i32,
        canvas_height(
            rows(req),
            hist_rows(req),
            alias_lines(req).len(),
            note_lines(req).len(),
        )
        .round() as i32,
    )
}

pub fn const_text(level: &SongLevel) -> String {
    if !level.known {
        return UNKNOWN_CONST.to_string();
    }
    let custom = level.const_text.trim();
    if !custom.is_empty() {
        return custom.to_string();
    }
    if level.constant <= 0.0 {
        return DASH.to_string();
    }
    sketch::one_decimal(level.constant)
}

pub fn notes_of(level: &SongLevel) -> [i32; 4] {
    [
        level.notes.tap,
        level.notes.drag,
        level.notes.hold,
        level.notes.flick,
    ]
}

pub fn count_or_dash(value: i32) -> String {
    if value <= 0 {
        DASH.to_string()
    } else {
        value.to_string()
    }
}

pub fn score_text(score: i32) -> String {
    sketch::group_digits(&score.to_string())
}

pub fn rank_label(row: &SongRankRow) -> String {
    format!("#{}", row.rank)
}

pub fn rank_note_text(ranking: &SongRanking) -> String {
    if ranking.total <= 0 {
        return RANK_EMPTY.to_string();
    }
    format!(
        "{}{RANK_TOTAL_SUFFIX}",
        sketch::group_digits(&ranking.total.to_string())
    )
}

pub fn history_label(change: &SongConstChange) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(2);
    let diff = change.diff.trim();
    if !diff.is_empty() {
        parts.push(diff);
    }
    let version = change.version.trim();
    if !version.is_empty() {
        parts.push(version);
    }
    parts.join(" ")
}

pub fn const_change_text(change: &SongConstChange) -> String {
    let arrow = if change.to > change.from {
        ARROW_UP
    } else if change.to < change.from {
        ARROW_DOWN
    } else {
        ARROW_FLAT
    };
    format!(
        "{} {arrow} {}",
        sketch::one_decimal(change.from),
        sketch::one_decimal(change.to)
    )
}

fn fill_gradient(kit: &CanvasKit<'_>, area: Rect, stops: [(f32, Color); 5]) {
    let gradient = LinearGradient::vertical(
        stops
            .iter()
            .map(|(offset, color)| GradientStop::new(*offset, *color)),
    );
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas().draw_rect(area, &paint);
}

fn shade_color(step: f32) -> Color {
    let ratio = step.powf(1.6);
    let alpha = (COVER_SHADE_TOP * ratio).round().clamp(0.0, 255.0) as u8;
    Color::new(SHADE_INK.r, SHADE_INK.g, SHADE_INK.b, alpha)
}

fn draw_cover_shade(kit: &CanvasKit<'_>) {
    let cover = sketch::ph_rect(
        cover_rect().left(),
        cover_rect().top(),
        cover_rect().width(),
        cover_rect().height(),
    );
    sketch::clip_round(kit, cover, CARD_RADIUS, || {
        let area = Rect::from_xywh(
            cover.left(),
            cover.bottom() - COVER_SHADE,
            cover.width(),
            COVER_SHADE,
        );
        fill_gradient(
            kit,
            area,
            [
                (0.0, shade_color(0.0)),
                (0.25, shade_color(0.25)),
                (0.5, shade_color(0.5)),
                (0.75, shade_color(0.75)),
                (1.0, shade_color(1.0)),
            ],
        );
    });
}

fn draw_cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    let area = sketch::ph_rect(
        cover_rect().left(),
        cover_rect().top(),
        cover_rect().width(),
        cover_rect().height(),
    );
    let key = super::illust_key(root, song_id);
    let mut drawn = false;
    if !song_id.is_empty() {
        if let Some(texture) = memo::texture(kit, &key) {
            if let Some(image) = texture.image() {
                sketch::clip_round(kit, area, CARD_RADIUS, || {
                    sketch::image_cover(kit, image, area);
                });
                drawn = true;
            }
        }
    }
    if !drawn {
        sketch::fill_round(kit, area, CARD_RADIUS, COVER_FALLBACK);
    }
}

fn draw_card(
    kit: &CanvasKit<'_>,
    background: Option<&Image>,
    canvas: Rect,
    area: Rect,
    title: &str,
    sub: &str,
) -> Rect {
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    if let Some(image) = background {
        let bounds = Rect::from_ltrb(
            frame.left() - 24.0,
            frame.top() - 24.0,
            frame.right() + 24.0,
            frame.bottom() + 24.0,
        );
        sketch::clip_round(kit, frame, CARD_RADIUS, || {
            sketch::blur_layer(kit, bounds, CARD_FROST, |inner| {
                sketch::image_nearest(inner, image, canvas);
            });
        });
    }
    sketch::fill_round(kit, frame, CARD_RADIUS, SHEET_INK);
    let baseline = area.top() + CARD_PAD + CARD_TITLE_SIZE;
    if !title.is_empty() {
        let style = sketch::text_style(sketch::FONT_DISPLAY, CARD_TITLE_SIZE, SCORE_INK);
        sketch::draw_text(
            kit,
            &style,
            title,
            area.left() + CARD_PAD,
            baseline,
            SCORE_INK,
        );
    }
    if !sub.is_empty() {
        let style = sketch::text_style(sketch::FONT_BODY, CARD_NOTE_SIZE, NOTE_INK);
        sketch::text_right(
            kit,
            &style,
            sub,
            area.right() - CARD_PAD,
            baseline,
            NOTE_INK,
        );
    }
    card_content(area, CARD_TITLE_SIZE, !title.is_empty())
}

fn fit_title(kit: &CanvasKit<'_>, value: &str, max_width: f32) -> (String, f32) {
    let mut size = TITLE_SIZE_MAX;
    while size > TITLE_SIZE_MIN {
        let style = sketch::text_style(sketch::FONT_DISPLAY, size, SCORE_INK);
        if sketch::text_width(kit, &style, value).round() <= max_width.round() {
            return (value.to_string(), size);
        }
        size -= TITLE_SIZE_STEP;
    }
    let style = sketch::text_style(sketch::FONT_DISPLAY, TITLE_SIZE_MIN, SCORE_INK);
    (
        sketch::ellipsize(kit, &style, value, max_width),
        TITLE_SIZE_MIN,
    )
}

fn draw_head(kit: &CanvasKit<'_>) {
    let style = sketch::text_style(sketch::FONT_DISPLAY, HEAD_TITLE_SIZE, SCORE_INK);
    sketch::draw_text(
        kit,
        &style,
        HEAD_TITLE,
        MARGIN_X,
        HEAD_TITLE_BASELINE,
        SCORE_INK,
    );
    let note = sketch::text_style(sketch::FONT_BODY, HEAD_NOTE_SIZE, PLACEHOLDER_INK);
    sketch::draw_text(
        kit,
        &note,
        HEAD_NOTE,
        MARGIN_X,
        HEAD_NOTE_BASELINE,
        PLACEHOLDER_INK,
    );
}

fn draw_rank_dots(kit: &CanvasKit<'_>, base: f32, from: f32, to: f32) {
    let from = from + CARD_TIGHT_GAP;
    let to = to - CARD_TIGHT_GAP;
    if RANK_DOT < 1.0 || to - from < RANK_DOT_GAP {
        return;
    }
    let y = (base - CARD_SUB_SIZE * 0.3).round();
    let mut x = from.round();
    while x + RANK_DOT <= to {
        sketch::ph_fill(kit, Rect::from_xywh(x, y, RANK_DOT, RANK_DOT), DOT_INK);
        x += RANK_DOT_GAP;
    }
}

fn draw_rank_rows(kit: &CanvasKit<'_>, root: &Path, req: &SongRequest) {
    let (head_x, head_base) = rank_head_at();
    let head_style = sketch::text_style(sketch::FONT_DISPLAY, CARD_SUB_SIZE, LABEL_INK);
    sketch::draw_text(kit, &head_style, RANK_TITLE, head_x, head_base, LABEL_INK);

    let (note_x, note_base) = rank_note_at();
    let note_style = sketch::text_style(sketch::FONT_BODY, CARD_NOTE_SIZE, NOTE_INK);
    let note = rank_note_text(&req.ranking);
    let avail = note_x - rank_name_left();
    let fitted = sketch::ellipsize(kit, &note_style, &note, avail);
    sketch::text_right(kit, &note_style, &fitted, note_x, note_base, NOTE_INK);

    if req.ranking.rows.is_empty() {
        let (x, baseline, size) = rank_empty_at();
        let style = sketch::text_style(sketch::FONT_DISPLAY, size, PLACEHOLDER_INK);
        sketch::text_center(kit, &style, RANK_EMPTY, x, baseline, PLACEHOLDER_INK);
        return;
    }

    let name_w = rank_name_w();
    let diff_style = sketch::text_style(sketch::FONT_DISPLAY, CARD_SUB_SIZE, SCORE_INK);
    let name_style = sketch::text_style(sketch::FONT_BODY, CARD_SUB_SIZE, LABEL_INK);
    let value_style = sketch::text_style(sketch::FONT_BODY, CARD_SUB_SIZE, NOTE_INK);
    let badge_size = rank_badge_text_size();
    let badge_style = sketch::text_style(sketch::FONT_BODY, badge_size, SCORE_INK);
    for (index, row) in req.ranking.rows.iter().enumerate() {
        if index >= RANK_ROWS {
            break;
        }
        let base = rank_row_base(index);
        let (index_ink, name_ink) = if index < 3 || row.self_row {
            (ACCENT_INK, SCORE_INK)
        } else {
            (NOTE_INK, LABEL_INK)
        };
        let avatar = rank_avatar(index);
        if let Some(image) = settlement::avatar_image(kit, root, &row.avatar) {
            let frame =
                sketch::ph_rect(avatar.left(), avatar.top(), avatar.width(), avatar.height());
            sketch::clip_round(kit, frame, RANK_AVATAR_RADIUS, || {
                sketch::image_fill(kit, image.image(), frame);
            });
        }
        let badge = rank_badge(index);
        player::rank_frame(kit, root, badge, row.dan_level);
        let mid = badge.top() + badge.height() / 2.0;
        sketch::text_center(
            kit,
            &badge_style,
            &row.dan_progress.to_string(),
            badge.left() + badge.width() / 2.0,
            mid + badge_size * MID_RATIO,
            SCORE_INK,
        );
        sketch::text_right(
            kit,
            &diff_style,
            &row.diff,
            rank_diff_right(),
            base,
            memo::difficulty_chip(&row.diff),
        );
        sketch::text_right(
            kit,
            &diff_style,
            &rank_label(row),
            rank_index_right(),
            base,
            index_ink,
        );
        let name = sketch::ellipsize(kit, &name_style, &row.name, name_w);
        sketch::draw_text(kit, &name_style, &name, rank_name_left(), base, name_ink);
        let name_end = rank_name_left() + sketch::text_width(kit, &name_style, &name);
        draw_rank_dots(kit, base, name_end, rank_acc_right() - RANK_ACC_W);
        sketch::text_right(
            kit,
            &value_style,
            &sketch::acc_percent(row.acc),
            rank_acc_right(),
            base,
            NOTE_INK,
        );
        sketch::text_right(
            kit,
            &diff_style,
            &score_text(row.score),
            rank_score_right(),
            base,
            SCORE_INK,
        );
    }
}

fn draw_main(
    kit: &CanvasKit<'_>,
    root: &Path,
    background: Option<&Image>,
    canvas: Rect,
    req: &SongRequest,
) {
    draw_card(kit, background, canvas, main_card(), "", "");
    draw_cover(kit, root, &req.song_id);
    draw_cover_shade(kit);
    let chapter = req.chapter.trim();
    if !chapter.is_empty() {
        let style = sketch::text_style(sketch::FONT_BODY, COVER_NOTE_SIZE, NOTE_INK);
        let text = sketch::ellipsize(kit, &style, chapter, cover_title_w());
        let (x, baseline) = cover_chapter_at();
        sketch::draw_text(kit, &style, &text, x, baseline, NOTE_INK);
    }
    let title = req.title.trim();
    if !title.is_empty() {
        let (text, size) = fit_title(kit, title, cover_title_w());
        let style = sketch::text_style(sketch::FONT_DISPLAY, size, SCORE_INK);
        let (x, baseline) = cover_title_at();
        sketch::draw_text(kit, &style, &text, x, baseline, SCORE_INK);
    }
    draw_rank_rows(kit, root, req);
}

fn draw_note_bar(kit: &CanvasKit<'_>, area: Rect, counts: [i32; 4]) {
    let total: i32 = counts.iter().sum();
    if total <= 0 {
        return;
    }
    let mut x = area.left();
    let mut walked = 0;
    for (index, value) in counts.iter().enumerate() {
        if *value <= 0 {
            continue;
        }
        walked += *value;
        let edge = area.left() + area.width() * walked as f32 / total as f32;
        let segment = sketch::ph_rect(x, area.top(), edge - x, area.height());
        sketch::ph_fill(kit, segment, NOTE_INKS[index]);
        x = edge;
    }
}

fn draw_matrix(kit: &CanvasKit<'_>, background: Option<&Image>, canvas: Rect, req: &SongRequest) {
    let count = rows(req);
    let content = draw_card(
        kit,
        background,
        canvas,
        matrix_card(count),
        MATRIX_TITLE,
        "",
    );
    let head = header_base(count);
    let head_style = sketch::text_style(sketch::FONT_BODY, HEAD_SIZE, PLACEHOLDER_INK);
    sketch::draw_text(
        kit,
        &head_style,
        "难度",
        content.left(),
        head,
        PLACEHOLDER_INK,
    );
    sketch::text_right(
        kit,
        &head_style,
        "定数",
        const_right(count),
        head,
        PLACEHOLDER_INK,
    );
    sketch::draw_text(
        kit,
        &head_style,
        "谱师",
        charter_x(count),
        head,
        PLACEHOLDER_INK,
    );
    for (index, kind) in NOTE_KINDS.iter().enumerate() {
        sketch::text_right(
            kit,
            &head_style,
            kind,
            note_col_right(count, index),
            head,
            NOTE_INKS[index],
        );
    }
    sketch::text_right(
        kit,
        &head_style,
        "合计",
        total_right(count),
        head,
        PLACEHOLDER_INK,
    );

    if req.levels.is_empty() {
        let style = sketch::text_style(sketch::FONT_BODY, CARD_TEXT_SIZE, PLACEHOLDER_INK);
        sketch::draw_text(
            kit,
            &style,
            MATRIX_EMPTY,
            content.left(),
            row_base(count, 0),
            PLACEHOLDER_INK,
        );
        return;
    }

    let value_style = sketch::text_style(sketch::FONT_DISPLAY, ROW_VALUE_SIZE, SCORE_INK);
    let text_style = sketch::text_style(sketch::FONT_BODY, ROW_TEXT_SIZE, LABEL_INK);
    for (index, level) in req.levels.iter().enumerate() {
        let ink = memo::difficulty_chip(&level.diff);
        let chip = diff_chip(count, index);
        let area = sketch::ph_rect(chip.left(), chip.top(), chip.width(), chip.height());
        let fill = Color::new(ink.r, ink.g, ink.b, CHIP_ALPHA);
        sketch::fill_round(kit, area, chip.height() / 2.0, fill);
        let (chip_x, chip_base) = diff_text_at(count, index);
        let chip_style = sketch::text_style(sketch::FONT_DISPLAY, CARD_SUB_SIZE, ink);
        sketch::text_center(kit, &chip_style, &level.diff, chip_x, chip_base, ink);
        let base = row_base(count, index);
        sketch::text_right(
            kit,
            &value_style,
            &const_text(level),
            const_right(count),
            base,
            SCORE_INK,
        );
        let charter = level.charter.trim();
        if !charter.is_empty() {
            let text = sketch::ellipsize(kit, &text_style, charter, charter_w(count));
            sketch::draw_text(kit, &text_style, &text, charter_x(count), base, LABEL_INK);
        }
        let counts = notes_of(level);
        for (kind, value) in counts.iter().enumerate() {
            sketch::text_right(
                kit,
                &text_style,
                &count_or_dash(*value),
                note_col_right(count, kind),
                base,
                LABEL_INK,
            );
        }
        sketch::text_right(
            kit,
            &value_style,
            &count_or_dash(level.notes.combo()),
            total_right(count),
            base,
            SCORE_INK,
        );
        draw_note_bar(kit, row_bar(count, index), counts);
    }
}

fn info_cells(req: &SongRequest, alias_lines: &[String]) -> [(&'static str, String); 8] {
    let last = if alias_lines.is_empty() {
        (INFO_ALIAS_LABEL, req.aliases.join(ALIAS_SEPARATOR))
    } else {
        (
            INFO_LEVEL_COUNT_LABEL,
            format!("{}{LEVEL_COUNT_SUFFIX}", req.levels.len()),
        )
    };
    [
        ("章节", req.chapter.clone()),
        ("曲师", req.composer.clone()),
        ("绘师", req.illustrator.clone()),
        ("收录版本", req.version.clone()),
        ("BPM", req.bpm.clone()),
        ("时长", req.length.clone()),
        ("曲目 ID", req.song_id.clone()),
        last,
    ]
}

fn draw_info(
    kit: &CanvasKit<'_>,
    background: Option<&Image>,
    canvas: Rect,
    req: &SongRequest,
    alias_lines: &[String],
) {
    let count = rows(req);
    let hist = hist_rows(req);
    let sub = if req.aliases.is_empty() {
        String::new()
    } else {
        format!("{}{INFO_ALIAS_SUFFIX}", req.aliases.len())
    };
    draw_card(
        kit,
        background,
        canvas,
        info_card(count, hist, alias_lines.len()),
        INFO_TITLE,
        &sub,
    );
    let cells = info_cells(req, alias_lines);
    let label_style = sketch::text_style(sketch::FONT_BODY, CARD_NOTE_SIZE, NOTE_INK);
    let value_style = sketch::text_style(sketch::FONT_DISPLAY, CARD_TEXT_SIZE, SCORE_INK);
    let width = info_value_w(count);
    for (index, (label, value)) in cells.iter().enumerate() {
        let col = index % INFO_COLS;
        let row = index / INFO_COLS;
        let (label_x, label_base) = info_label_at(count, col, row);
        sketch::draw_text(kit, &label_style, label, label_x, label_base, NOTE_INK);
        let raw = value.trim();
        let text = if raw.is_empty() { DASH } else { raw };
        let fitted = sketch::ellipsize(kit, &value_style, text, width);
        let (value_x, value_base) = info_value_at(count, col, row);
        sketch::draw_text(kit, &value_style, &fitted, value_x, value_base, SCORE_INK);
    }

    if !alias_lines.is_empty() {
        let (label_x, label_base) = alias_label_at(count);
        sketch::draw_text(
            kit,
            &label_style,
            INFO_ALIAS_LABEL,
            label_x,
            label_base,
            NOTE_INK,
        );
        let alias_w = alias_value_w(count);
        for (index, line) in alias_lines.iter().enumerate() {
            let (x, base) = alias_value_at(count, index);
            let fitted = sketch::ellipsize(kit, &value_style, line, alias_w);
            sketch::draw_text(kit, &value_style, &fitted, x, base, SCORE_INK);
        }
    }

    if hist == 0 {
        return;
    }
    let content = info_content(count);
    let (label_x, label_base) = hist_label_at(count, alias_lines.len());
    sketch::draw_text(
        kit,
        &label_style,
        INFO_HIST_LABEL,
        label_x,
        label_base,
        NOTE_INK,
    );
    let hist_style = sketch::text_style(sketch::FONT_BODY, CARD_TEXT_SIZE, LABEL_INK);
    let const_style = sketch::text_style(sketch::FONT_DISPLAY, CARD_TEXT_SIZE, SCORE_INK);
    for (index, change) in req.history.iter().enumerate() {
        let base = hist_row_base(count, alias_lines.len(), index);
        sketch::draw_text(
            kit,
            &hist_style,
            &history_label(change),
            content.left(),
            base,
            LABEL_INK,
        );
        sketch::text_right(
            kit,
            &const_style,
            &const_change_text(change),
            content.right(),
            base,
            SCORE_INK,
        );
    }
}

fn draw_notes(kit: &CanvasKit<'_>, req: &SongRequest, note_lines: &[String], alias: usize) {
    if note_lines.is_empty() {
        return;
    }
    let count = rows(req);
    let hist = hist_rows(req);
    let style = sketch::text_style(sketch::FONT_BODY, CARD_TEXT_SIZE, LABEL_INK);
    for (index, line) in note_lines.iter().enumerate() {
        let (x, base) = note_at(count, hist, alias, index);
        sketch::draw_text(kit, &style, line, x, base, LABEL_INK);
    }
}

fn draw_footer(kit: &CanvasKit<'_>, req: &SongRequest, alias: usize, note: usize) {
    let style = sketch::text_style(sketch::FONT_BODY, CARD_NOTE_SIZE, PLACEHOLDER_INK);
    let text = sketch::ellipsize(kit, &style, FOOTER_TEXT, CONTENT_W);
    sketch::draw_text(
        kit,
        &style,
        &text,
        MARGIN_X,
        tip_baseline(rows(req), hist_rows(req), alias, note),
        PLACEHOLDER_INK,
    );
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SongRequest) {
    let alias = alias_lines(req);
    let notes = note_lines(req);
    let height = canvas_height(rows(req), hist_rows(req), alias.len(), notes.len()).round();
    let canvas = Rect::from_xywh(0.0, 0.0, CANVAS_W, height);
    let background = super::songinfo::background_image(kit, root, &req.song_id, CANVAS_W, height);
    match background.as_ref() {
        Some(image) => sketch::image_nearest(kit, image, canvas),
        None => {
            sketch::ph_fill(kit, canvas, super::songinfo::BASE_INK);
            sketch::ph_fill(kit, canvas, super::songinfo::PAGE_SHADE);
        }
    }
    draw_head(kit);
    draw_main(kit, root, background.as_ref(), canvas, req);
    draw_matrix(kit, background.as_ref(), canvas, req);
    draw_info(kit, background.as_ref(), canvas, req, &alias);
    draw_notes(kit, req, &notes, alias.len());
    draw_footer(kit, req, alias.len(), notes.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_fonts() {
        let count = super::super::register_fonts(&crate::assets_root());
        assert!(count >= 1, "assets/font 下必须至少注册上一支字体");
    }

    fn level(diff: &str) -> SongLevel {
        SongLevel {
            diff: diff.to_string(),
            constant: 15.0,
            known: true,
            ..SongLevel::default()
        }
    }

    #[test]
    fn canvas_height_follows_difficulty_count() {
        assert_eq!(canvas_height(1, 0, 0, 0).round(), 1200.0);
        assert_eq!(canvas_height(2, 0, 0, 0).round(), 1272.0);
        assert_eq!(canvas_height(3, 0, 0, 0).round(), 1344.0);
        assert_eq!(canvas_height(4, 0, 0, 0).round(), 1416.0);
    }

    #[test]
    fn canvas_height_counts_history_alias_and_note_rows() {
        let base = canvas_height(4, 0, 0, 0);
        assert_eq!(alias_h(0), 0.0);
        assert_eq!(hist_h(0), 0.0);
        assert_eq!(
            info_card(4, 0, 0).bottom(),
            info_card(4, 2, 3).bottom() - hist_h(2) - alias_h(3)
        );
        assert!(canvas_height(4, 3, 0, 0) > base);
        assert!(canvas_height(4, 0, 2, 0) > base);
        assert!(canvas_height(4, 0, 0, 2) > base);
        assert!((hist_h(2) - 115.4).abs() < 0.01);
        assert!((alias_h(3) - 125.4).abs() < 0.01);
        assert!((alias_h(2) - 99.4).abs() < 0.01);
        assert!((info_card(4, 2, 2).bottom() - 1553.409).abs() < 0.01);
        assert!((canvas_height(4, 2, 2, 0) - 1630.409).abs() < 0.01);
        assert_eq!(canvas_height(4, 2, 2, 0).round(), 1630.0);
        assert_eq!(canvas_height(4, 2, 3, 1).round(), 1706.0);
        assert_eq!(
            hist_label_at(4, 0).1 + CARD_TIGHT_GAP + CARD_TEXT_SIZE,
            hist_row_base(4, 0, 0)
        );
        let alias_drop = alias_label_at(4).1 - info_content(4).top() - info_content_h();
        assert!((alias_drop - (INFO_ALIAS_LABEL_DROP + CARD_NOTE_SIZE)).abs() < 0.01);
        assert!(hist_label_at(4, 3).1 > hist_label_at(4, 0).1);
        assert!(note_at(4, 2, 3, 1).1 > note_at(4, 2, 3, 0).1);
        assert!(
            tip_baseline(4, 2, 3, 2) - note_at(4, 2, 3, 1).1 > NOTE_PITCH * 0.9,
            "带说明的曲子, Tip 必须让到说明最后一行之下"
        );
        let tip = tip_baseline(4, 2, 3, 2);
        assert!(tip + CARD_NOTE_SIZE * 0.6 <= canvas_height(4, 2, 3, 2));
    }

    #[test]
    fn note_body_joins_sp_and_variant() {
        let mut req = SongRequest::default();
        assert!(note_lines(&req).is_empty());
        req.variant_of = String::from("Spasmodic");
        assert_eq!(
            note_body(&req),
            "本曲是《Spasmodic》的曲绘变体，谱面与本体相同"
        );
        req.sp = String::from("SP\n说明");
        assert!(note_body(&req).starts_with("SP 说明"));
        assert!(!note_lines(&req).is_empty());
        assert!(note_lines(&req).len() <= NOTE_MAX_LINES);
    }

    #[test]
    fn alias_lines_appear_only_when_one_cell_overflows() {
        ensure_fonts();
        let mut req = SongRequest {
            levels: vec![level("IN")],
            ..SongRequest::default()
        };
        assert!(alias_lines(&req).is_empty());
        req.aliases = vec![String::from("A")];
        assert!(alias_lines(&req).is_empty());
        req.aliases = vec![String::from("很长的别名").repeat(40)];
        let lines = alias_lines(&req);
        assert!(!lines.is_empty());
        assert!(lines.len() <= INFO_ALIAS_MAX_LINES);
        assert_eq!(info_cells(&req, &lines)[7].0, INFO_LEVEL_COUNT_LABEL);
        assert_eq!(info_cells(&req, &[])[7].0, INFO_ALIAS_LABEL);
    }

    #[test]
    fn segments_stack_with_block_gap() {
        assert_eq!(BLOCKS_TOP, 127.0);
        assert_eq!(main_card().top(), 127.0);
        assert_eq!(main_card().height(), COVER_H + 56.0);
        assert_eq!(matrix_card(4).top(), main_card().bottom() + BLOCK_GAP);
        assert_eq!(
            info_card(4, 0, 0).top(),
            matrix_card(4).bottom() + BLOCK_GAP
        );
        assert_eq!(info_card(4, 3, 0).top(), info_card(4, 0, 0).top());
        assert!(info_card(4, 3, 0).bottom() > info_card(4, 0, 0).bottom());
        assert!(info_card(4, 0, 2).bottom() > info_card(4, 0, 0).bottom());
        assert_eq!(
            tip_baseline(4, 0, 0, 0),
            info_card(4, 0, 0).bottom() + TIP_DROP
        );
    }

    #[test]
    fn columns_share_the_content_width_by_weight() {
        let c = matrix_content(3);
        let total = COL_WEIGHTS.iter().sum::<f32>();
        assert_eq!(col_right(3, 0), c.left() + c.width() * 0.075 / total);
        assert!((col_right(3, 7) - c.right()).abs() < 0.01);
        assert_eq!(charter_x(3), col_right(3, 1) + COL_GAP);
        assert_eq!(charter_w(3), col_right(3, 2) - charter_x(3) - COL_GAP);
        assert_eq!(note_group_left(3), col_right(3, 2));
        assert_eq!(total_right(3), col_right(3, 7));
    }

    #[test]
    fn rank_rows_fill_the_cover_height() {
        let c = rank_content();
        let cover = cover_rect();
        assert_eq!(c.left(), cover.right() + RANK_GAP);
        assert_eq!(c.right(), main_content().right());
        assert_eq!(c.top(), cover.top());
        assert_eq!(c.bottom(), cover.bottom());
        assert_eq!(rank_rows_top(), c.top() + 38.0);
        assert!((rank_pitch() * RANK_ROWS as f32 + 38.0 - c.height()).abs() < 0.01);
        assert!(rank_row_base(0) > c.top());
        assert!(rank_row_base(RANK_ROWS - 1) < c.bottom());
        assert!(rank_row_base(1) > rank_row_base(0));
        assert_eq!(rank_diff_right(), c.left() + 26.0);
        assert_eq!(rank_index_right(), c.left() + 78.0);
        assert_eq!(rank_avatar(0).left(), c.left() + 90.0);
        assert_eq!(rank_avatar(0).width(), 28.0);
        assert_eq!(rank_badge(0).left(), rank_avatar(0).right() + 12.0);
        assert_eq!(rank_badge(0).width(), 61.0);
        assert_eq!(rank_name_left(), rank_badge(0).right() + 12.0);
        assert_eq!(rank_score_right(), c.right());
        assert_eq!(rank_acc_right(), c.right() - 144.0);
        assert!(rank_name_w() > 100.0);
        let (x, base, size) = rank_empty_at();
        assert_eq!(x, c.left() + c.width() / 2.0);
        assert!(base > rank_rows_top() && base < c.bottom());
        assert!(size > 0.0);
    }

    #[test]
    fn rank_texts_follow_go_wording() {
        let empty = SongRanking::default();
        assert_eq!(rank_note_text(&empty), RANK_EMPTY);
        let filled = SongRanking {
            total: 12_345,
            rows: Vec::new(),
        };
        assert_eq!(rank_note_text(&filled), "12'345 条成绩");
        let row = SongRankRow {
            diff: String::from("IN"),
            rank: 7,
            ..SongRankRow::default()
        };
        assert_eq!(rank_label(&row), "#7");
        assert_eq!(
            history_label(&SongConstChange {
                diff: String::from("IN"),
                version: String::from("3.20.0"),
                ..SongConstChange::default()
            }),
            "IN 3.20.0"
        );
        assert_eq!(
            const_change_text(&SongConstChange {
                from: 15.9,
                to: 16.0,
                ..SongConstChange::default()
            }),
            "15.9 ↑ 16.0"
        );
        assert_eq!(
            const_change_text(&SongConstChange {
                from: 16.0,
                to: 15.9,
                ..SongConstChange::default()
            }),
            "16.0 ↓ 15.9"
        );
    }

    #[test]
    fn texts_fall_back_to_dash_and_unknown() {
        let sample = SongLevel {
            diff: String::from("IN"),
            constant: 15.0,
            known: true,
            ..SongLevel::default()
        };
        assert_eq!(const_text(&sample), "15.0");
        let mut custom = sample.clone();
        custom.const_text = String::from("15.4");
        assert_eq!(const_text(&custom), "15.4");
        let mut unknown = sample.clone();
        unknown.known = false;
        assert_eq!(const_text(&unknown), UNKNOWN_CONST);
        let mut zero = sample;
        zero.constant = 0.0;
        assert_eq!(const_text(&zero), DASH);
        assert_eq!(count_or_dash(0), DASH);
        assert_eq!(count_or_dash(1200), "1200");
        assert_eq!(score_text(987123), "987'123");
    }

    #[test]
    fn canvas_size_grows_with_the_restored_blocks() {
        ensure_fonts();
        let mut req = SongRequest {
            levels: vec![level("EZ"), level("HD"), level("IN")],
            ..SongRequest::default()
        };
        let plain = canvas_size(&req);
        assert_eq!(plain.0, 1920);
        assert_eq!(plain.1, 1344);
        req.history = vec![
            SongConstChange {
                diff: String::from("IN"),
                version: String::from("3.20.0"),
                from: 15.9,
                to: 16.0,
            },
            SongConstChange {
                diff: String::from("HD"),
                version: String::from("3.21.0"),
                from: 13.0,
                to: 13.2,
            },
        ];
        req.variant_of = String::from("Spasmodic");
        let grown = canvas_size(&req);
        assert_eq!(
            grown.1,
            canvas_height(3, 2, 0, note_lines(&req).len()).round() as i32
        );
        assert!(grown.1 > plain.1);
    }
}
