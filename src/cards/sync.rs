use std::path::Path;

use aurorix_render::skia_safe::{
    color_filters, ClipOp, Image, Paint,
    RRect, Rect, Vector,
};
use aurorix_render::{CanvasKit, Color, GradientStop, LinearGradient};
use chrono::{DateTime, Datelike, Local, Utc};

use super::player::{self, Player};
use super::settlement::{line, measure, Align, TextAt};
use super::{memo, sketch};

pub const CANVAS_W: f32 = 1440.0;

pub const MARGIN_X: f32 = 48.0;

pub const CONTENT_RIGHT: f32 = 1392.0;

pub const CONTENT_W: f32 = 1344.0;

pub const SY_COLS: usize = 5;

pub const SY_GRID_ROWS: usize = 2;

pub const SY_PER_GROUP: usize = SY_COLS * SY_GRID_ROWS;

pub const SY_CARD_RATIO: f32 = 1.75;

pub const SY_BAND_RATIO: f32 = 0.381;

pub const SY_BAND_PLATEAU_RATIO: f32 = 0.127;

pub const SY_SUM_H: f32 = 86.0;

pub const SY_SUM_PAD_X: f32 = 34.0;

pub const SY_SUM_KPI_SZ: f32 = 36.0;

pub const SY_SUM_LABEL_SZ: f32 = 14.0;

pub const SY_SUM_STATS_X: f32 = 270.0;

pub const SY_SUM_STATS_TOP: f32 = 14.0;

pub const SY_SUM_STATS_H: f32 = 56.0;

pub const PANEL_TINT: Color = Color::new(0x16, 0x12, 0x14, 0x62);

pub const SY_STAT_SZ: f32 = 14.0;

pub const SY_ROW_GAP: f32 = 16.0;

pub const SY_SEC_SZ: f32 = 16.0;

pub const SY_GROUP_GAP: f32 = 20.0;

pub const SY_SEC_GAP: f32 = 14.0;

pub const SY_SEC_RULE_GAP: f32 = 6.0;

pub const SY_CARD_GAP: f32 = 14.0;

pub const SY_CARD_PAD: f32 = 12.0;

pub const SY_BOTTOM_MARGIN: f32 = 36.0;

pub const SY_HINT_H: f32 = 72.0;

pub const SY_HEAD_TITLE_SZ: f32 = 40.0;

pub const SY_HEAD_NOTE_SZ: f32 = 15.0;

pub const SY_HEAD_NOTE_GAP: f32 = 10.0;

pub const SY_HEAD_BAR_GAP: f32 = 8.0;

pub const SY_PLAYER_K: f32 = 1500.0 / 1920.0;

pub const SY_PLAYER_SCALE: f32 = 1500.0 / 1920.0;

pub const SY_TITLE_SZ: f32 = 12.0;

pub const SY_META_SZ: f32 = 11.0;

pub const SY_SCORE_SZ: f32 = 15.0;

pub const SY_INDEX_SZ: f32 = 15.0;

pub const SY_GRADE_W: f32 = 20.0;

pub const SY_GRADE_H: f32 = 20.0;

pub const SY_SHADOW_DY: f32 = 1.0;

pub const SY_SHADOW_BLUR: f32 = 1.0;

pub const LEAN: f32 = 0.2679492;

pub const STAMP_INK: Color = Color::new(0xDD, 0xE2, 0xEC, 0xFF);

pub const DELTA_INK: Color = Color::new(0x7A, 0xE3, 0xE7, 0xFF);

pub const SHADOW_INK: Color = Color::new(0x06, 0x07, 0x0B, 0xB3);

pub const LABEL_SHADOW_INK: Color = Color::new(0x05, 0x06, 0x0A, 0xD0);

pub const META_INK: Color = Color::new(0xE4, 0xE8, 0xF0, 0xFF);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const PLACEHOLDER_INK: Color = Color::new(0x8A, 0x92, 0xA2, 0xFF);

pub const CARD_DEEP_INK: Color = Color::new(0x0A, 0x0C, 0x12, 0x4C);

pub const SHEET_SCRIM: Color = Color::new(0x03, 0x04, 0x08, 0x5A);

pub const COVER_FALLBACK: Color = Color::new(0x22, 0x26, 0x30, 0xFF);

pub const RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const ACCENT_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

pub const CARD_SHADE_INK: Color = Color::new(0x06, 0x07, 0x0B, 0x0E);

pub const CARD_TOP_SCRIM_INK: Color = Color::new(0x05, 0x06, 0x0A, 0x73);

pub const CARD_TOP_SCRIM_H: f32 = 0.52;

pub const CARD_BAND_BOTTOM: Color = Color::new(0x06, 0x07, 0x0B, 0xE6);

pub const CARD_BAND_MIX: f32 = 0.28;

pub const CARD_EDGE_W: f32 = 3.2;

pub const CARD_EDGE_ALPHA: u8 = 0xFF;

pub const CARD_RADIUS: f32 = 11.0;

pub const CARD_BIG_EXTRA: f32 = 10.0;

pub const CARD_SMALL_RATIO: f32 = 0.7;

pub const CARD_SHADOW_STEPS: usize = 6;

pub const CARD_SHADOW_STEP_W: f32 = 1.0;

pub const CARD_SHADOW_INK: Color = Color::new(0x04, 0x05, 0x09, 0x3C);

pub const GLOW_CARD_W: f32 = 340.8;

pub const GLOW_REACH_RATIO: f32 = 0.06;

pub const GLOW_SOFT_REACH_RATIO: f32 = 0.035;

pub const GLOW_CORE: f32 = 1.5;

pub const GLOW_SOFT_CORE: f32 = 1.2;

pub const GLOW_PHI_INK: Color = Color::new(0xF0, 0xC7, 0x5E, 0xFF);

pub const GLOW_FC_INK: Color = Color::new(0xF2, 0xF7, 0xFF, 0x6E);

pub const GLOW_PLAIN_INK: Color = Color::new(0xE6, 0xEE, 0xFF, 0x28);

pub const DIFF_EZ_INK: Color = Color::new(0xA0, 0xE0, 0x57, 0xFF);

pub const DIFF_HD_INK: Color = Color::new(0x45, 0xE0, 0xA8, 0xFF);

pub const DIFF_IN_INK: Color = Color::new(0x7F, 0xA3, 0xD4, 0xFF);

pub const DIFF_AT_INK: Color = Color::new(0xFF, 0x5F, 0x68, 0xFF);

pub const DIFF_UNKNOWN_INK: Color = Color::new(0xB9, 0xC0, 0xCA, 0xFF);

pub const BACKDROP_DIVISOR: f32 = 8.0;

pub const PLATE_BAND_BOTTOM: f32 = 161.0;

pub const LABEL_RKS: &str = "RKS";

pub const EMPTY_LINE: &str = "没有发现新的成绩";

pub const STAT_RECORDS: &str = "成绩";

pub const STAT_CHARTS: &str = "谱面";

pub const STAT_RECENT: &str = "近 7 天";

pub const STAT_RANK: &str = "进榜";

pub const DASH: &str = "—";

pub const NONE_TEXT: &str = "无";

pub const TITLE_FIRST: &str = "已接入存档";

pub const TITLE_UPDATED: &str = "更新完成";

pub const HINT_FIRST: &str = "成绩已入库 · 发 /p me 看总览, 发 /p b30 看榜面";

pub const HINT_LATEST: &str = "本地数据已是最新 · 打一首再发 /p update 就有明细";

pub const VERDICT_IMPORT_PREFIX: &str = "已导入 ";

pub const VERDICT_IMPORT_SUFFIX: &str = " 条成绩";

pub const VERDICT_ADDED: &str = "新增 ";

pub const VERDICT_IMPROVED: &str = "提分 ";

pub const VERDICT_DROPPED: &str = "掉分 ";

pub const SEPARATOR: &str = " · ";

pub const NEW_LABEL: &str = "NEW";

pub const COUNT_PREFIX: &str = "共 ";

pub const COUNT_SUFFIX: &str = " 条";

pub const SINGLE_RKS_SEP: &str = "  >  ";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncItem {
    pub song_id: String,
    pub title: String,
    pub difficulty: String,
    pub constant: f64,
    pub known: bool,
    pub old_score: i32,
    pub new_score: i32,
    pub old_acc: f64,
    pub new_acc: f64,
    pub delta: i32,
    pub rks: f64,
    pub grade: String,
    pub in_best: bool,
    pub rks_delta: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncGroup {
    pub at: Option<DateTime<Utc>>,
    pub label: String,
    pub items: Vec<SyncItem>,
    pub total: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncRequest {
    pub player: Player,
    pub backdrop: String,
    pub at: Option<DateTime<Utc>>,
    pub save_modified: Option<DateTime<Utc>>,
    pub first_sync: bool,
    pub changed: bool,
    pub rks_before: f64,
    pub rks_after: f64,
    pub groups: Vec<SyncGroup>,
    pub total: i32,
    pub played: i32,
    pub charts: i32,
    pub recent7: f64,
    pub has_recent: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub shown: Vec<usize>,
    pub hint: bool,
}

pub fn plan(req: &SyncRequest) -> Plan {
    let shown: Vec<usize> = req
        .groups
        .iter()
        .map(|group| rows_shown(group.items.len()))
        .collect();
    Plan {
        hint: shown.is_empty(),
        shown,
    }
}

pub fn ink(size: f32) -> f32 {
    size * 0.84
}

pub fn num_ink(size: f32) -> f32 {
    size * 0.72
}

pub fn rows_shown(items: usize) -> usize {
    items.min(SY_PER_GROUP)
}

pub fn sec_h() -> f32 {
    num_ink(SY_SEC_SZ) + SY_SEC_RULE_GAP + SY_SEC_GAP
}

pub fn card_w() -> f32 {
    (CONTENT_W - (SY_COLS as f32 - 1.0) * SY_CARD_GAP) / SY_COLS as f32
}

pub fn card_h() -> f32 {
    card_w() / SY_CARD_RATIO
}

pub fn grid_h(count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let lines = count.div_ceil(SY_COLS);
    lines as f32 * card_h() + (lines as f32 - 1.0) * SY_ROW_GAP
}

pub fn player_bar_bottom() -> f32 {
    let bar = player::player_bar();
    let dy = PLATE_BAND_BOTTOM * 0.5 - (bar.top() + bar.bottom()) / 2.0;
    let panel = player::panel_rect(0.0);
    let avatar = player::avatar_window();
    dy + panel.bottom().max(avatar.bottom())
}

pub fn player_bar_dy() -> f32 {
    let bar = player::player_bar();
    PLATE_BAND_BOTTOM * 0.5 - (bar.top() + bar.bottom()) / 2.0
}

pub fn blocks_top() -> f32 {
    player_bar_bottom() * SY_PLAYER_SCALE + SY_HEAD_BAR_GAP
}

pub fn head_title_at() -> TextAt {
    let top = (blocks_top() - ink(SY_HEAD_TITLE_SZ) - SY_HEAD_NOTE_GAP - ink(SY_HEAD_NOTE_SZ)) / 2.0;
    TextAt::left(MARGIN_X, top + ink(SY_HEAD_TITLE_SZ), SY_HEAD_TITLE_SZ)
        .with_font(sketch::FONT_DISPLAY)
}

pub fn head_note_at() -> TextAt {
    let at = head_title_at();
    TextAt::left(
        at.x,
        at.baseline + SY_HEAD_NOTE_GAP + ink(SY_HEAD_NOTE_SZ),
        SY_HEAD_NOTE_SZ,
    )
    .with_font(sketch::FONT_BODY)
}

pub fn sum_rect() -> Rect {
    Rect::from_xywh(MARGIN_X, blocks_top(), CONTENT_W, SY_SUM_H)
}

pub fn sum_kpi_at() -> TextAt {
    let area = sum_rect();
    TextAt::left(
        area.left() + SY_SUM_PAD_X,
        area.top() + 42.0,
        SY_SUM_KPI_SZ,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn sum_label_at() -> TextAt {
    let area = sum_rect();
    TextAt::left(
        area.left() + SY_SUM_PAD_X,
        area.top() + 70.0,
        SY_SUM_LABEL_SZ,
    )
    .with_font(sketch::FONT_BODY)
}

pub fn sum_stats() -> Rect {
    let area = sum_rect();
    Rect::from_xywh(
        area.left() + SY_SUM_STATS_X,
        area.top() + SY_SUM_STATS_TOP,
        area.width() - SY_SUM_STATS_X - SY_SUM_PAD_X,
        SY_SUM_STATS_H,
    )
}

pub fn groups_top() -> f32 {
    sum_rect().bottom() + SY_GROUP_GAP
}

pub fn group_top(plan: &Plan, index: usize) -> f32 {
    let mut y = groups_top();
    for shown in plan.shown.iter().take(index) {
        y += sec_h() + grid_h(*shown) + SY_GROUP_GAP;
    }
    y
}

pub fn sec_at(plan: &Plan, index: usize) -> TextAt {
    TextAt::left(
        MARGIN_X,
        group_top(plan, index) + num_ink(SY_SEC_SZ),
        SY_SEC_SZ,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn sec_count_at(plan: &Plan, index: usize) -> TextAt {
    let at = sec_at(plan, index);
    TextAt::right(CONTENT_RIGHT, at.baseline, 11.0).with_font(sketch::FONT_BODY)
}

pub fn sec_rule_y(plan: &Plan, index: usize) -> f32 {
    sec_at(plan, index).baseline + SY_SEC_RULE_GAP
}

pub fn card_at(plan: &Plan, group: usize, index: usize) -> Rect {
    let (w, h) = (card_w(), card_h());
    let col = index % SY_COLS;
    let row = index / SY_COLS;
    Rect::from_xywh(
        MARGIN_X + col as f32 * (w + SY_CARD_GAP),
        group_top(plan, group) + sec_h() + row as f32 * (h + SY_ROW_GAP),
        w,
        h,
    )
}

pub fn canvas_h(plan: &Plan) -> f32 {
    let mut height = groups_top();
    for shown in &plan.shown {
        height += sec_h() + grid_h(*shown) + SY_GROUP_GAP;
    }
    if !plan.shown.is_empty() {
        return height - SY_GROUP_GAP + SY_BOTTOM_MARGIN;
    }
    if plan.hint {
        return height + SY_HINT_H;
    }
    height + SY_BOTTOM_MARGIN
}

pub fn hint_at() -> TextAt {
    TextAt::left(MARGIN_X, groups_top() + SY_HINT_H / 2.0 + 8.0, 13.0)
        .with_font(sketch::FONT_BODY)
}

pub fn card_index_at(card: Rect) -> TextAt {
    TextAt::left(
        card.left() + SY_CARD_PAD * 0.55,
        card.top() + SY_CARD_PAD * 0.7 + SY_INDEX_SZ,
        SY_INDEX_SZ,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn card_name_at(card: Rect) -> TextAt {
    TextAt::left(
        card.left() + SY_CARD_PAD,
        card.bottom() - SY_CARD_PAD - 15.0,
        SY_TITLE_SZ,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn card_meta_at(card: Rect) -> TextAt {
    TextAt::left(
        card.left() + SY_CARD_PAD,
        card.bottom() - SY_CARD_PAD,
        SY_META_SZ,
    )
    .with_font(sketch::FONT_BODY)
}

pub fn card_score_at(card: Rect) -> TextAt {
    TextAt::right(
        card.right() - SY_CARD_PAD,
        card.bottom() - SY_CARD_PAD - 15.0,
        SY_SCORE_SZ,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn card_tail_at(card: Rect) -> TextAt {
    TextAt::right(
        card.right() - SY_CARD_PAD,
        card.bottom() - SY_CARD_PAD,
        SY_META_SZ,
    )
    .with_font(sketch::FONT_BODY)
}

pub fn card_top_strip(card: Rect) -> Rect {
    Rect::from_xywh(
        card.left(),
        card.top(),
        card.width(),
        SY_CARD_PAD + SY_GRADE_H + SY_CARD_PAD / 2.0,
    )
}

pub fn card_grade_right(card: Rect) -> f32 {
    card.right() - SY_CARD_PAD
}

pub fn skew_at(area: Rect, y: f32) -> f32 {
    LEAN * (area.top() + area.height() / 2.0 - y)
}

pub fn rks_text(value: f64) -> String {
    format!("{:.4}", value.max(0.0))
}

pub fn delta_text(delta: f64) -> String {
    let text = format!("{delta:+.4}");
    if text == "-0.0000" {
        return String::from("+0.0000");
    }
    text
}

pub fn signed_percent(value: f64) -> String {
    let text = format!("{value:.2}");
    if value >= 0.0 {
        return format!("+{text}%");
    }
    format!("{text}%")
}

pub fn delta_ink(delta: f64) -> Color {
    if delta > 0.0 {
        return ACCENT_INK;
    }
    if delta < 0.0 {
        return SCORE_INK;
    }
    NOTE_INK
}

pub fn diff_hue(diff: &str) -> Color {
    match diff {
        "EZ" => DIFF_EZ_INK,
        "HD" => DIFF_HD_INK,
        "IN" => DIFF_IN_INK,
        "AT" => DIFF_AT_INK,
        _ => DIFF_UNKNOWN_INK,
    }
}

pub fn edge_ink(diff: &str) -> Color {
    diff_hue(diff).with_alpha(CARD_EDGE_ALPHA)
}

pub fn band_ink(diff: &str) -> Color {
    let base = CARD_BAND_BOTTOM;
    let hue = match diff {
        "EZ" => DIFF_EZ_INK,
        "HD" => DIFF_HD_INK,
        "IN" => DIFF_IN_INK,
        "AT" => DIFF_AT_INK,
        _ => return base,
    };
    let blend = |b: u8, h: u8| -> u8 {
        (f64::from(b) * f64::from(1.0 - CARD_BAND_MIX)
            + f64::from(h) * f64::from(CARD_BAND_MIX)
            + 0.5) as u8
    };
    Color::new(
        blend(base.r, hue.r),
        blend(base.g, hue.g),
        blend(base.b, hue.b),
        base.a,
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub ink: Color,
    pub reach: f32,
    pub core: f32,
}

pub fn glow_of(grade: &str) -> Glow {
    if grade.eq_ignore_ascii_case("PHI") {
        return Glow {
            ink: GLOW_PHI_INK,
            reach: GLOW_CARD_W * GLOW_REACH_RATIO,
            core: GLOW_CORE,
        };
    }
    if grade.eq_ignore_ascii_case("FC") {
        return Glow {
            ink: GLOW_FC_INK,
            reach: GLOW_CARD_W * GLOW_REACH_RATIO,
            core: GLOW_CORE,
        };
    }
    Glow {
        ink: GLOW_PLAIN_INK,
        reach: GLOW_CARD_W * GLOW_SOFT_REACH_RATIO,
        core: GLOW_SOFT_CORE,
    }
}

pub fn count_kinds(groups: &[SyncGroup]) -> (i32, i32, i32) {
    let mut added = 0;
    let mut improved = 0;
    let mut dropped = 0;
    for group in groups {
        for item in &group.items {
            if item.old_score <= 0 {
                added += 1;
            } else if item.delta > 0 {
                improved += 1;
            } else if item.delta < 0 {
                dropped += 1;
            }
        }
    }
    (added, improved, dropped)
}

pub fn head_title(req: &SyncRequest) -> &'static str {
    if req.first_sync {
        return TITLE_FIRST;
    }
    TITLE_UPDATED
}

pub fn stamp_text(t: DateTime<Utc>) -> String {
    t.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
}

pub fn short_stamp(t: DateTime<Utc>, reference: Option<DateTime<Utc>>) -> String {
    let local = t.with_timezone(&Local);
    match reference {
        Some(other) if other.with_timezone(&Local).year() != local.year() => {
            local.format("%Y-%m-%d %H:%M").to_string()
        }
        _ => local.format("%m-%d %H:%M").to_string(),
    }
}

pub fn verdict_text(req: &SyncRequest) -> String {
    if req.first_sync {
        return format!("{VERDICT_IMPORT_PREFIX}{}{VERDICT_IMPORT_SUFFIX}", req.total);
    }
    let (added, improved, dropped) = count_kinds(&req.groups);
    if !req.changed || added + improved + dropped == 0 {
        return String::from(EMPTY_LINE);
    }
    let mut parts: Vec<String> = Vec::with_capacity(3);
    if added > 0 {
        parts.push(format!("{VERDICT_ADDED}{added}"));
    }
    if improved > 0 {
        parts.push(format!("{VERDICT_IMPROVED}{improved}"));
    }
    if dropped > 0 {
        parts.push(format!("{VERDICT_DROPPED}{dropped}"));
    }
    parts.join(SEPARATOR)
}

pub fn head_note(req: &SyncRequest) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(3);
    if let Some(at) = req.at {
        parts.push(format!("{} 同步", stamp_text(at)));
    }
    if let Some(modified) = req.save_modified {
        parts.push(format!("存档 {}", short_stamp(modified, req.at)));
    }
    parts.push(verdict_text(req));
    parts.join(SEPARATOR)
}

pub fn group_title(group: &SyncGroup) -> String {
    let mut label = match group.at {
        Some(at) => stamp_text(at),
        None => String::from("0001-01-01 00:00"),
    };
    if !group.label.is_empty() {
        label.push_str(SEPARATOR);
        label.push_str(&group.label);
    }
    label
}

pub fn hint_text(req: &SyncRequest) -> &'static str {
    if req.first_sync {
        return HINT_FIRST;
    }
    HINT_LATEST
}

pub fn stat_cells(req: &SyncRequest) -> Vec<(String, String)> {
    let value = |text: String| -> String {
        if text.is_empty() {
            return String::from(DASH);
        }
        text
    };
    let records = if req.total > 0 {
        sketch::group_digits(&req.total.to_string())
    } else {
        String::new()
    };
    let charts = if req.played > 0 && req.charts > 0 {
        format!("{}/{}", req.played, req.charts)
    } else {
        String::new()
    };
    let recent = if req.has_recent {
        delta_text(req.recent7)
    } else {
        String::new()
    };
    let mut bested = 0;
    for group in &req.groups {
        for item in &group.items {
            if item.in_best {
                bested += 1;
            }
        }
    }
    let rank = if bested > 0 {
        format!("{bested}{COUNT_SUFFIX}")
    } else {
        String::from(NONE_TEXT)
    };
    vec![
        (String::from(STAT_RECORDS), value(records)),
        (String::from(STAT_CHARTS), value(charts)),
        (String::from(STAT_RECENT), value(recent)),
        (String::from(STAT_RANK), value(rank)),
    ]
}

pub fn solid_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint
}

pub fn stroke_paint(color: Color, width: f32) -> Paint {
    let mut paint = solid_paint(color);
    paint.set_style(aurorix_render::skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(width.max(0.0));
    paint
}

pub fn card_rrect(card: Rect, grow: f32) -> RRect {
    let big = (CARD_RADIUS + CARD_BIG_EXTRA + grow).max(0.0);
    let small = (CARD_RADIUS * CARD_SMALL_RATIO + grow).max(0.0);
    let area = Rect::from_ltrb(
        card.left() - grow,
        card.top() - grow,
        card.right() + grow,
        card.bottom() + grow,
    );
    RRect::new_rect_radii(
        area,
        &[
            Vector::new(big, big),
            Vector::new(small, small),
            Vector::new(big, big),
            Vector::new(small, small),
        ],
    )
}

pub fn expand(area: Rect, by: f32) -> Rect {
    Rect::from_ltrb(
        area.left() - by,
        area.top() - by,
        area.right() + by,
        area.bottom() + by,
    )
}

pub fn clip_card<R>(kit: &CanvasKit<'_>, card: Rect, grow: f32, body: impl FnOnce() -> R) -> R {
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(card_rrect(card, grow), ClipOp::Intersect, true);
    let out = body();
    canvas.restore();
    out
}

pub fn fill_gradient(kit: &CanvasKit<'_>, area: Rect, stops: Vec<GradientStop>) {
    let gradient = LinearGradient::vertical(stops);
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas().draw_rect(area, &paint);
}

pub const CARD_FOCUS_Y: f32 = 0.38;

pub fn backdrop(kit: &CanvasKit<'_>, root: &Path, song_id: &str, canvas: Rect) {
    sketch::dream_backdrop(kit, root, song_id, canvas);
}

pub fn frost_panel(kit: &CanvasKit<'_>, area: Rect, canvas: Rect) {
    let plate = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    sketch::clip_skew(kit, plate, LEAN, || {
        sketch::ph_fill(kit, canvas, PANEL_TINT);
    });
    sketch::fill_skew(kit, plate, LEAN, CARD_DEEP_INK);
}

pub fn player_left(kit: &CanvasKit<'_>, req: &SyncRequest) -> f32 {
    if !req.player.has_player {
        return CONTENT_RIGHT;
    }
    let width = player::nickname(kit, &req.player.name)
        .map(|(_, width)| width)
        .unwrap_or(0.0);
    let panel = player::panel_rect(width);
    CANVAS_W - (player::CALIB_WIDTH - panel.left()) * SY_PLAYER_K
}

pub fn head(kit: &CanvasKit<'_>, req: &SyncRequest) {
    line(kit, head_title_at(), head_title(req), SCORE_INK);
    let note = head_note(req);
    if note.is_empty() {
        return;
    }
    let at = head_note_at();
    let limit = player_left(kit, req) - 16.0 - at.x;
    let style = sketch::text_style(at.font_key(), at.size, STAMP_INK);
    let cut = sketch::ellipsize(kit, &style, &note, limit);
    line(kit, at, &cut, STAMP_INK);
}

pub fn stat_tile(kit: &CanvasKit<'_>, area: Rect, gap: f32, label: &str, value: &str) {
    let base = area.top() + SY_STAT_SZ;
    let mut at = TextAt::left(area.left(), base, SY_STAT_SZ).with_font(sketch::FONT_BODY);
    line(kit, at, label, NOTE_INK);
    if !label.is_empty() {
        at.x += measure(kit, at, label) + gap;
    }
    line(kit, at, value, SCORE_INK);
}

pub fn stat_strip(kit: &CanvasKit<'_>, req: &SyncRequest) {
    let cells = stat_cells(req);
    if cells.is_empty() {
        return;
    }
    let area = sum_stats();
    let probe = TextAt::left(0.0, 0.0, SY_STAT_SZ).with_font(sketch::FONT_BODY);
    let gap = 10.0_f32;
    let mut sep = 30.0_f32;
    let widths: Vec<f32> = cells
        .iter()
        .map(|(label, value)| measure(kit, probe, label) + gap + measure(kit, probe, value))
        .collect();
    let content: f32 = widths.iter().sum();
    if cells.len() > 1 {
        let slack = (area.width() - content) / (cells.len() - 1) as f32;
        if slack > sep {
            sep = slack;
        }
    }
    let mut x = area.left() + skew_at(sum_rect(), area.top() + area.height() / 2.0);
    let top = area.top() + area.height() / 2.0 - SY_STAT_SZ * 0.62;
    for (index, (label, value)) in cells.iter().enumerate() {
        let cell = Rect::from_xywh(x, top, widths[index], SY_STAT_SZ * 1.4);
        stat_tile(kit, cell, gap, label, value);
        x += widths[index] + sep;
    }
}

pub fn summary(kit: &CanvasKit<'_>, canvas: Rect, req: &SyncRequest) {
    let plate = sum_rect();
    frost_panel(kit, plate, canvas);
    let mut at = sum_kpi_at();
    at.x += skew_at(plate, at.baseline);
    let delta = req.rks_after - req.rks_before;
    let (number, ink) = if req.first_sync || req.rks_before <= 0.0 {
        (rks_text(req.rks_after), SCORE_INK)
    } else {
        (delta_text(delta), delta_ink(delta))
    };
    line(kit, at, &number, ink);
    line(kit, sum_label_at(), LABEL_RKS, NOTE_INK);
    stat_strip(kit, req);
}

pub fn section(kit: &CanvasKit<'_>, at: TextAt, title: &str, rule_y: f32) {
    line(kit, at, title, STAMP_INK);
    if CONTENT_RIGHT <= at.x {
        return;
    }
    let top = rule_y.trunc();
    sketch::ph_fill(
        kit,
        Rect::from_ltrb(at.x.trunc(), top, CONTENT_RIGHT.trunc(), top + 1.0),
        RULE_INK,
    );
}

pub fn art(kit: &CanvasKit<'_>, root: &Path, song_id: &str, card: Rect) {
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::thumb_key(root, song_id))
            .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
    };
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(card_rrect(card, -CARD_EDGE_W), ClipOp::Intersect, true);
    match texture.as_ref().and_then(|item| item.image()) {
        Some(image) => sketch::image_cover_focus(kit, image, card, CARD_FOCUS_Y),
        None => sketch::ph_fill(kit, card, COVER_FALLBACK),
    }
    canvas.restore();
}

pub fn shade(kit: &CanvasKit<'_>, card: Rect) {
    clip_card(kit, card, 0.0, || sketch::ph_fill(kit, card, CARD_SHADE_INK));
}

pub fn top_scrim(kit: &CanvasKit<'_>, card: Rect) {
    let bottom = card.top() + card.height() * CARD_TOP_SCRIM_H;
    if bottom <= card.top() {
        return;
    }
    let area = Rect::from_ltrb(card.left(), card.top(), card.right(), bottom);
    clip_card(kit, card, 0.0, || {
        fill_gradient(
            kit,
            area,
            vec![
                GradientStop::new(0.0, CARD_TOP_SCRIM_INK),
                GradientStop::new(1.0, CARD_TOP_SCRIM_INK.with_alpha(0)),
            ],
        );
    });
}

pub fn band(kit: &CanvasKit<'_>, card: Rect, ink: Color, height: f32, plateau: f32) {
    let bottom = card.bottom();
    let mid = bottom - plateau;
    let top = bottom - height;
    if mid <= top {
        return;
    }
    let area = Rect::from_ltrb(card.left(), top, card.right(), mid);
    let steps = 8;
    let stops: Vec<GradientStop> = (0..=steps)
        .map(|index| {
            let t = index as f32 / steps as f32;
            let alpha = 1.0 - (1.0 - t).powf(2.8);
            GradientStop::new(t, ink.with_alpha((f32::from(ink.a) * alpha).round() as u8))
        })
        .collect();
    clip_card(kit, card, 0.0, || fill_gradient(kit, area, stops));
    clip_card(kit, card, 0.0, || {
        sketch::ph_fill(
            kit,
            Rect::from_ltrb(card.left(), mid, card.right(), bottom),
            ink,
        );
    });
}

pub fn card_edge(kit: &CanvasKit<'_>, card: Rect, ink: Color) {
    let shape = card_rrect(card, -CARD_EDGE_W / 2.0);
    kit.canvas()
        .draw_rrect(shape, &stroke_paint(ink, CARD_EDGE_W));
}

pub fn card_shadow(kit: &CanvasKit<'_>, card: Rect) {
    let steps = CARD_SHADOW_STEPS;
    let dy = (steps - 1) as f32 * CARD_SHADOW_STEP_W;
    for layer in (0..steps).rev() {
        let delta = layer as f32 * CARD_SHADOW_STEP_W;
        let alpha =
            (f32::from(CARD_SHADOW_INK.a) * (steps - layer) as f32 / steps as f32).round() as u8;
        if alpha == 0 {
            continue;
        }
        let ink = CARD_SHADOW_INK.with_alpha(alpha);
        let canvas = kit.canvas();
        canvas.save();
        canvas.translate(Vector::new(0.0, dy));
        canvas.draw_rrect(card_rrect(card, delta), &solid_paint(ink));
        canvas.restore();
    }
}

pub fn card_glow(kit: &CanvasKit<'_>, card: Rect, glow: Glow) {
    if glow.ink.a == 0 || glow.reach <= 0.0 {
        return;
    }
    let core = glow.core.clamp(0.0, glow.reach);
    let spread = glow.reach - core;
    let steps = glow.reach.ceil().max(1.0) as i32;
    for step in 0..steps {
        let distance = step as f32 + 0.5;
        let mut coverage = 1.0_f32;
        if distance > core && spread > 0.0 {
            let t = (distance - core) / spread;
            coverage = (1.0 - t) * (1.0 - t);
        }
        if coverage <= 0.0 {
            continue;
        }
        let alpha = (f32::from(glow.ink.a) * coverage).round() as u8;
        if alpha == 0 {
            continue;
        }
        let ink = glow.ink.with_alpha(alpha);
        kit.canvas()
            .draw_rrect(card_rrect(card, distance), &stroke_paint(ink, 1.0));
    }
}

pub fn card_chrome(kit: &CanvasKit<'_>, root: &Path, item: &SyncItem, card: Rect) {
    card_shadow(kit, card);
    card_glow(kit, card, glow_of(&item.grade));
    art(kit, root, &item.song_id, card);
    shade(kit, card);
    top_scrim(kit, card);
    band(
        kit,
        card,
        band_ink(&item.difficulty),
        card.height() * SY_BAND_RATIO,
        card.height() * SY_BAND_PLATEAU_RATIO,
    );
    card_edge(kit, card, edge_ink(&item.difficulty));
}

pub fn silhouette(kit: &CanvasKit<'_>, image: &Image, dest: Rect, ink: Color, radius: f32) {
    let alpha = f32::from(ink.a) / 255.0;
    let matrix = [
        0.0, 0.0, 0.0, 0.0, f32::from(ink.r) / 255.0,
        0.0, 0.0, 0.0, 0.0, f32::from(ink.g) / 255.0,
        0.0, 0.0, 0.0, 0.0, f32::from(ink.b) / 255.0,
        0.0, 0.0, 0.0, alpha, 0.0,
    ];
    let filter = color_filters::matrix_row_major(&matrix, None);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color_filter(filter);
    let bounds = expand(dest, radius * 3.0 + 4.0);
    sketch::blur_layer(kit, bounds, radius, |inner| {
        inner.canvas().draw_image_rect_with_sampling_options(
            image,
            None,
            dest,
            aurorix_render::sampling_for(
                (image.width() as f32, image.height() as f32),
                dest,
            ),
            &paint,
        );
    });
}

pub fn card_grade(kit: &CanvasKit<'_>, root: &Path, item: &SyncItem, card: Rect) {
    if item.grade.is_empty() {
        return;
    }
    let Some(name) = memo::grade_name(&item.grade) else {
        return;
    };
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let frame = Rect::from_xywh(0.0, 0.0, SY_GRADE_W.round(), SY_GRADE_H.round());
    let target = aurorix_render::contain_destination_rect(image, frame);
    let Some(solid) = sketch::content_bounds(image) else {
        return;
    };
    let scale_x = target.width() / image.width() as f32;
    let scale_y = target.height() / image.height() as f32;
    let x = (card_grade_right(card) - solid.right() * scale_x + 0.5).floor();
    let y = (card.top() + SY_CARD_PAD).floor() - (solid.top() * scale_y).round();
    let dest = Rect::from_xywh(x, y, target.width(), target.height());
    let drop = Rect::from_xywh(x, y + SY_SHADOW_DY, target.width(), target.height());
    silhouette(kit, image, drop, SHADOW_INK, SY_SHADOW_BLUR);
    sketch::image_fill(kit, image, dest);
}

pub fn shadow_text(kit: &CanvasKit<'_>, at: TextAt, value: &str, ink: Color) {
    if value.is_empty() {
        return;
    }
    let mut shadow = at;
    shadow.baseline += SY_SHADOW_DY;
    line(kit, shadow, value, LABEL_SHADOW_INK);
    line(kit, at, value, ink);
}

pub fn card_meta(kit: &CanvasKit<'_>, item: &SyncItem, card: Rect) {
    let at = card_meta_at(card);
    let mut lead = item.difficulty.clone();
    if item.known {
        lead.push_str("  ");
        lead.push_str(&sketch::one_decimal(item.constant));
    }
    if item.rks > 0.0 {
        lead.push_str(SINGLE_RKS_SEP);
    }
    line(kit, at, &lead, META_INK);
    if item.rks > 0.0 {
        let mut next = at;
        next.x += measure(kit, at, &lead);
        line(kit, next, &rks_text(item.rks), SCORE_INK);
    }
}

pub fn card_tail(kit: &CanvasKit<'_>, item: &SyncItem, card: Rect) {
    let tail = card_tail_at(card);
    let acc = format!("{}%", sketch::two_decimals(item.new_acc));
    let mut acc_at = tail;
    let change = item.new_acc - item.old_acc;
    if item.old_score > 0 && change.abs() >= 0.005 {
        let delta = signed_percent(change);
        line(kit, tail, &delta, delta_ink(change));
        acc_at.align = Align::Left;
        acc_at.x = tail.x - measure(kit, tail, &delta) - 12.0 - measure(kit, tail, &acc);
    }
    line(kit, acc_at, &acc, META_INK);
}

pub fn card(kit: &CanvasKit<'_>, root: &Path, item: &SyncItem, area: Rect) {
    card_chrome(kit, root, item, area);
    card_grade(kit, root, item, area);
    let delta = if item.old_score > 0 {
        sketch::signed_score(item.delta)
    } else {
        String::from(NEW_LABEL)
    };
    shadow_text(kit, card_index_at(area), &delta, DELTA_INK);
    let name_at = card_name_at(area);
    let score_at = card_score_at(area);
    let score = sketch::group_digits(&item.new_score.to_string());
    let score_w = measure(kit, score_at, &score);
    let max_w = (score_at.x - score_w - 2.0 * SY_CARD_GAP - name_at.x).max(0.0);
    let style = sketch::text_style(name_at.font_key(), name_at.size, SCORE_INK);
    let title = sketch::ellipsize(kit, &style, &item.title, max_w);
    line(kit, name_at, &title, SCORE_INK);
    line(kit, score_at, &score, SCORE_INK);
    card_meta(kit, item, area);
    card_tail(kit, item, area);
}

pub fn groups(kit: &CanvasKit<'_>, root: &Path, req: &SyncRequest, plan: &Plan) {
    for (index, group) in req.groups.iter().enumerate() {
        if index >= plan.shown.len() {
            break;
        }
        section(
            kit,
            sec_at(plan, index),
            &group_title(group),
            sec_rule_y(plan, index),
        );
        if group.total > 0 {
            line(
                kit,
                sec_count_at(plan, index),
                &format!("{COUNT_PREFIX}{}{COUNT_SUFFIX}", group.total),
                NOTE_INK,
            );
        }
        for (item_index, item) in group
            .items
            .iter()
            .enumerate()
            .take(plan.shown[index].min(group.items.len()))
        {
            card(kit, root, item, card_at(plan, index, item_index));
        }
    }
}

pub fn canvas_size(req: &SyncRequest) -> (i32, i32) {
    let height = canvas_h(&plan(req)).round() as i32;
    (CANVAS_W as i32, height)
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SyncRequest) {
    let plan = plan(req);
    let height = canvas_h(&plan).round();
    let canvas = Rect::from_xywh(0.0, 0.0, CANVAS_W, height);
    backdrop(kit, root, &req.backdrop, canvas);
    player::draw(
        kit,
        root,
        CANVAS_W,
        SY_PLAYER_K,
        player_bar_dy() * SY_PLAYER_SCALE,
        &req.player,
    );
    head(kit, req);
    summary(kit, canvas, req);
    if plan.hint {
        line(kit, hint_at(), hint_text(req), PLACEHOLDER_INK);
        return;
    }
    groups(kit, root, req, &plan);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(left: f32, right: f32) -> bool {
        (left - right).abs() < 0.01
    }

    fn item(old_score: i32, new_score: i32) -> SyncItem {
        SyncItem {
            song_id: String::from("Spasmodic"),
            title: String::from("Spasmodic"),
            difficulty: String::from("IN"),
            constant: 15.7,
            known: true,
            old_score,
            new_score,
            old_acc: 98.1,
            new_acc: 99.2,
            delta: new_score - old_score,
            rks: 15.6,
            grade: String::from("V"),
            in_best: true,
            rks_delta: 0.02,
        }
    }

    fn group(count: usize) -> SyncGroup {
        SyncGroup {
            at: None,
            label: String::new(),
            items: (0..count).map(|index| item(index as i32, 900_000)).collect(),
            total: count as i32,
        }
    }

    #[test]
    fn canvas_height_grows_with_group_count() {
        let empty = SyncRequest::default();
        let one = SyncRequest {
            groups: vec![group(2)],
            ..SyncRequest::default()
        };
        let two = SyncRequest {
            groups: vec![group(2), group(2)],
            ..SyncRequest::default()
        };
        let hint = canvas_size(&empty);
        let first = canvas_size(&one);
        let second = canvas_size(&two);
        assert_eq!(hint.0, 1440);
        assert_eq!(first.0, 1440);
        assert_eq!(second.0, 1440);
        assert!(hint.1 < first.1, "{hint:?} < {first:?}");
        assert!(first.1 < second.1, "{first:?} < {second:?}");
        assert_eq!(hint.1, 292);
        assert_eq!(second.1, 634);
    }

    #[test]
    fn grid_caps_at_ten_cards_per_group() {
        assert_eq!(SY_PER_GROUP, 10);
        assert_eq!(rows_shown(0), 0);
        assert_eq!(rows_shown(3), 3);
        assert_eq!(rows_shown(10), 10);
        assert_eq!(rows_shown(12), 10);
        let plan = Plan {
            shown: vec![10],
            hint: false,
        };
        let last = card_at(&plan, 0, 9);
        assert!(near(last.top(), group_top(&plan, 0) + sec_h() + card_h() + SY_ROW_GAP));
        let capped = SyncRequest {
            groups: vec![group(10), group(10)],
            ..SyncRequest::default()
        };
        assert_eq!(canvas_size(&capped).1, 960);
    }

    #[test]
    fn card_keeps_the_declared_aspect_ratio() {
        assert!(near(card_w(), 257.6));
        assert!(near(card_h(), 147.2));
        assert!(near(card_w() / card_h(), SY_CARD_RATIO));
        assert!(near(card_w() * SY_COLS as f32 + SY_CARD_GAP * (SY_COLS as f32 - 1.0), CONTENT_W));
    }

    #[test]
    fn section_gaps_are_strictly_ordered() {
        assert!(std::hint::black_box(SY_GROUP_GAP) > std::hint::black_box(SY_ROW_GAP));
        let plan = Plan {
            shown: vec![10],
            hint: false,
        };
        let first = card_at(&plan, 0, 0);
        let second = card_at(&plan, 0, 5);
        assert!(near(second.top() - first.bottom(), SY_ROW_GAP));
        let baseline = sec_at(&plan, 0).baseline;
        let rule = sec_rule_y(&plan, 0);
        assert!(baseline < rule);
        assert!(rule < first.top());
        assert!(near(rule - baseline, SY_SEC_RULE_GAP));
        assert!(near(first.top() - rule, SY_SEC_GAP));
        let plan_two = Plan {
            shown: vec![10, 1],
            hint: false,
        };
        let group_one = group_top(&plan_two, 1);
        let bottom = card_at(&plan_two, 0, 9).bottom();
        assert!(near(group_one - bottom, SY_GROUP_GAP));
        assert!(near(sec_at(&plan_two, 1).baseline, group_one + num_ink(SY_SEC_SZ)));
    }

    #[test]
    fn summary_plate_spans_the_content_width() {
        let area = sum_rect();
        assert!(near(area.left(), MARGIN_X));
        assert!(near(area.width(), CONTENT_W));
        assert!(near(area.right(), CONTENT_RIGHT));
        assert!(near(area.height(), SY_SUM_H));
        assert!(near(area.top(), blocks_top()));
        assert!(near(groups_top() - area.bottom(), SY_GROUP_GAP));
        let stats = sum_stats();
        assert!(stats.right() <= area.right() + 0.01);
        assert!(stats.left() > area.left() + SY_SUM_STATS_X - 0.01);
    }

    #[test]
    fn delta_text_is_signed_and_four_decimals() {
        assert_eq!(delta_text(0.0), "+0.0000");
        assert_eq!(delta_text(-0.00001), "+0.0000");
        assert_eq!(delta_text(0.0123), "+0.0123");
        assert_eq!(delta_text(-0.5), "-0.5000");
        assert_eq!(signed_percent(0.53), "+0.53%");
        assert_eq!(signed_percent(-0.53), "-0.53%");
        assert_eq!(sketch::signed_score(1200), "+1'200");
        assert_eq!(sketch::signed_score(-300), "-300");
        assert_eq!(delta_ink(0.1), ACCENT_INK);
        assert_eq!(delta_ink(-0.1), SCORE_INK);
        assert_eq!(delta_ink(0.0), NOTE_INK);
    }

    #[test]
    fn stats_fall_back_to_dash_and_none() {
        let empty = SyncRequest::default();
        let cells = stat_cells(&empty);
        assert_eq!(cells.len(), 4);
        for (_, value) in &cells[..3] {
            assert_eq!(value, DASH);
        }
        assert_eq!(cells[3].1, NONE_TEXT);
        let full = SyncRequest {
            total: 12345,
            played: 286,
            charts: 349,
            recent7: 0.0234,
            has_recent: true,
            groups: vec![group(3)],
            ..SyncRequest::default()
        };
        let cells = stat_cells(&full);
        assert_eq!(cells[0].1, "12'345");
        assert_eq!(cells[1].1, "286/349");
        assert_eq!(cells[2].1, "+0.0234");
        assert_eq!(cells[3].1, format!("3{COUNT_SUFFIX}"));
    }

    #[test]
    fn head_and_verdict_lines_follow_the_request() {
        let first = SyncRequest {
            first_sync: true,
            total: 349,
            ..SyncRequest::default()
        };
        assert_eq!(head_title(&first), TITLE_FIRST);
        assert_eq!(verdict_text(&first), "已导入 349 条成绩");
        assert_eq!(hint_text(&first), HINT_FIRST);
        let idle = SyncRequest::default();
        assert_eq!(head_title(&idle), TITLE_UPDATED);
        assert_eq!(verdict_text(&idle), EMPTY_LINE);
        assert_eq!(hint_text(&idle), HINT_LATEST);
        let changed = SyncRequest {
            changed: true,
            groups: vec![SyncGroup {
                at: None,
                label: String::new(),
                items: vec![item(0, 900_000), item(800_000, 900_000), item(900_000, 800_000)],
                total: 3,
            }],
            ..SyncRequest::default()
        };
        assert_eq!(verdict_text(&changed), "新增 1 · 提分 1 · 掉分 1");
    }
}
