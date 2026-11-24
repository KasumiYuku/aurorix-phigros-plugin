use std::path::Path;

use aurorix_render::skia_safe::{
    canvas::SrcRectConstraint, ClipOp, Image, Paint, Path as SkiaPath, PathBuilder, Rect,
};
use aurorix_render::{CanvasKit, Color, GradientStop, LinearGradient, TextStyle};
use chrono::Datelike;

use super::memo;
use super::player;
use super::sketch;
use crate::domain::overview as archive;
use crate::repo::{RksDaily, RksPoint};

pub const CANVAS_W: f32 = 1440.0;

pub const MARGIN_X: f32 = 48.0;

pub const CONTENT_RIGHT: f32 = 1392.0;

pub const CONTENT_W: f32 = 1344.0;

pub const COL_GAP: f32 = 48.0;

pub const BLOCK_GAP: f32 = 24.0;

pub const COL_W: f32 = (CONTENT_W - COL_GAP) / 2.0;

pub const RIGHT_X: f32 = MARGIN_X + COL_W + COL_GAP;

pub const CARD_PAD: f32 = 28.0;

pub const TITLE_SIZE: f32 = 22.0;

pub const TITLE_GAP: f32 = 22.0;

pub const CARD_RADIUS: f32 = 16.0;

pub const CARD_FROST: f32 = 22.0;

pub const TEXT_SIZE: f32 = 18.0;

pub const SUB_SIZE: f32 = 16.0;

pub const NOTE_SIZE: f32 = 13.0;

pub const MID_RATIO: f32 = 0.36;

pub const TIGHT_GAP: f32 = 16.0;

pub const ROW_GAP: f32 = 28.0;

pub const HEAD_TOP: f32 = 24.0;

pub const HEAD_TITLE_SIZE: f32 = 52.0;

pub const HEAD_H: f32 = HEAD_TITLE_SIZE + 10.0 + NOTE_SIZE;

pub const HEAD_TITLE_BASELINE: f32 = HEAD_TOP + HEAD_TITLE_SIZE;

pub const HEAD_NOTE_BASELINE: f32 = HEAD_TOP + HEAD_H;

pub const BLOCKS_TOP: f32 = HEAD_TOP + HEAD_H + 22.0;

pub const PROFILE_CONTENT_H: f32 = 186.0;

pub const AVATAR: f32 = 150.0;

pub const NAME_SIZE: f32 = 34.0;

pub const RKS_SIZE: f32 = 56.0;

pub const PROFILE_COL_X: f32 = 182.0;

pub const PROFILE_RKS_W: f32 = 260.0;

pub const NAME_DROP: f32 = 38.0;

pub const NAME_MID: f32 = 0.72;

pub const BADGE_DX: f32 = 20.0;

pub const BADGE_W: f32 = 61.0;

pub const BADGE_H: f32 = 20.0;

pub const BADGE_TEXT_SIZE: f32 = 30.0;

pub const BADGE_TEXT_DROP: f32 = 13.8;

pub const RKS_LABEL_DROP: f32 = 58.0;

pub const RKS_DROP: f32 = 118.0;

pub const INTRO_DROP: f32 = 66.0;

pub const INTRO_LINES: usize = 3;

pub const INTRO_PITCH: f32 = TEXT_SIZE + TIGHT_GAP * 0.5;

pub const STATS_TOP: f32 = 138.0;

pub const STAT_RULE_DROP: f32 = TIGHT_GAP * 0.7;

pub const STAT_BLOCK_H: f32 = NOTE_SIZE + 8.0 + TEXT_SIZE;

pub const STAT_VALUE_DROP: f32 = 8.0;

pub const MIDDLE_CONTENT_H: f32 = 462.0;

pub const RANK_ROWS: usize = 10;

pub const RANK_NUMBER_SIZE: f32 = 44.0;

pub const RANK_TRACK_H: f32 = 10.0;

pub const RANK_AVATAR: f32 = 22.0;

pub const RANK_INDEX_W: f32 = 88.0;

pub const RANK_BADGE_W: f32 = 61.0;

pub const RANK_BADGE_GAP: f32 = 12.0;

pub const RANK_BADGE_H: f32 = RANK_BADGE_W * 20.0 / 61.0;

pub const RANK_BADGE_TEXT: f32 = 24.0 * RANK_BADGE_W / 100.0;

pub const RANK_OMIT_SHARE: f32 = 0.5;

pub const RANK_BAND_SIDE: f32 = 10.0;

pub const RANK_BAND_INSET: f32 = 3.0;

pub const RANK_BAND_RADIUS: f32 = 7.0;

pub const RANK_ME_BAR_W: f32 = 2.0;

pub const RANK_ME_BAR_INSET: f32 = 6.0;

pub const RANK_MARK_W: f32 = 3.0;

pub const RANK_MARK_DROP: f32 = 6.0;
pub const DIFF_ROWS: usize = 4;

pub const DIFF_LABEL_W: f32 = 48.0;

pub const DIFF_BAR_H: f32 = 14.0;

pub const DIFF_ROWS_H: f32 = 190.0;

pub const DIFF_RULE_Y: f32 = DIFF_ROWS_H + 8.0;

pub const COLLECT_TOP: f32 = DIFF_RULE_Y + 8.0;

pub const COLLECT_RULE_Y: f32 = 310.0;

pub const COLLECT_NUM_SIZE: f32 = 36.0;

pub const COLLECT_NUM_DROP: f32 = 8.0;

pub const COLLECT_BAR_H: f32 = 12.0;

pub const COLLECT_BAR_DROP: f32 = 12.0;

pub const STATS_COLS: usize = 3;

pub const STATS_ROWS: usize = 2;

pub const STATS_CELL_GAP_X: f32 = ROW_GAP * 0.9;

pub const STATS_CELL_GAP_Y: f32 = TIGHT_GAP * 0.6;

pub const CURVE_GUTTER: f32 = TIGHT_GAP + ROW_GAP;

pub const CURVE_PLOT_H: f32 = 240.0;

pub const CURVE_NOTE_DROP: f32 = TIGHT_GAP * 0.75;

pub const CURVE_DATE_DROP: f32 = 18.0;

pub const CURVE_STATS_DROP: f32 = TIGHT_GAP * 0.6;

pub const MILESTONE_COL_W: f32 = 360.0;

pub const MILESTONE_COL_GAP: f32 = 40.0;

pub const MILESTONE_ROWS: usize = 5;

pub const MILESTONE_DOT: f32 = 9.0;

pub const MILESTONE_RAIL: f32 = TIGHT_GAP + TIGHT_GAP * 0.5;

pub const MILESTONE_RAIL_W: f32 = 1.0;

pub const MILESTONE_LABEL_H: f32 = TIGHT_GAP * 1.2;

pub const MILESTONE_PITCH: f32 = TIGHT_GAP * 0.6;

pub const CURVE_AREA_FADE: f32 = 0.10;

pub const FOOTER_DROP: f32 = 32.0;

pub const FOOTER_H: f32 = TEXT_SIZE;

pub const BOTTOM_MARGIN: f32 = 32.0;

pub const SHEET_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x14);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const LABEL_INK: Color = Color::new(0xB4, 0xBA, 0xC6, 0xFF);

pub const NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const PLACEHOLDER_INK: Color = Color::new(0x8A, 0x92, 0xA2, 0xFF);

pub const RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const ACCENT_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

pub const PHI_INK: Color = Color::new(0xF0, 0xC7, 0x5E, 0xFF);

pub const CLEAR_INK: Color = Color::new(0xDD, 0xE3, 0xEE, 0xFF);

pub const TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1C);

pub const AREA_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0x3A);

pub const RANK_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1E);

pub const RANK_TRACK_FILL_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xB0);

pub const RANK_ROW_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x14);

pub const MILESTONE_FC_INK: Color = Color::new(0x9F, 0xD8, 0xE8, 0xFF);

pub const MILESTONE_MARK_ALPHA: u8 = 0x50;

pub const HEAD_TITLE: &str = "PLAYER ARCHIVE";

pub const HEAD_NOTE: &str = "玩家档案";

pub const PROFILE_TITLE: &str = "账号信息";

pub const RANK_TITLE: &str = "全服排名";

pub const RANK_HIDDEN: &str = "？";

pub const RANK_EMPTY: [&str; 2] = ["这次没拿到排行榜", "稍后再试"];

pub const DIFF_TITLE: &str = "完成度与记录";

pub const CURVE_TITLE: &str = "RKS 成长曲线";

pub const EMPTY_NAME: &str = "未命名玩家";

pub const EMPTY_INTRO: &str = "（还没有填写签名）";

pub const MILESTONE_TITLE: &str = "里程碑";

pub const MILESTONE_EMPTY: &str = "还没有可查的里程碑";

pub const EMPTY_CURVE: [&str; 2] = ["还没有 RKS 记录", "推分并同步之后, 这里会画出曲线"];

pub const DASH: &str = "—";

pub const SIGNATURE: &str = "Generated By Okamikko";

pub const SYNC_PREFIX: &str = "数据截至 ";

pub const DAY_SECONDS: i64 = 86_400;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurvePoint {
    pub at: i64,
    pub rks: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverviewDiff {
    pub label: String,
    pub played: i32,
    pub total: i32,
    pub cleared: i32,
    pub full_combo: i32,
    pub phi: i32,
    pub best: i32,
    pub worst: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverviewCollection {
    pub unlocked_charts: i32,
    pub total_charts: i32,
    pub unlocked_songs: i32,
    pub total_songs: i32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverviewRequest {
    pub backdrop: String,
    pub player: player::Player,
    pub server: String,
    pub self_intro: String,
    pub curve: Vec<CurvePoint>,
    pub diffs: Vec<OverviewDiff>,
    pub collection: OverviewCollection,
    pub rank: archive::RankBoard,
    pub milestones: Vec<archive::Milestone>,
    pub synced: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveStats {
    pub days: i32,
    pub span_days: i32,
    pub streak: i32,
    pub first: i64,
    pub best_gain: f64,
    pub best_day: i64,
    pub has_best: bool,
    pub max_gap: i32,
}

pub fn curve_from_points(points: &[RksPoint]) -> Vec<CurvePoint> {
    points
        .iter()
        .map(|point| CurvePoint {
            at: point.observed_at,
            rks: point.rks,
        })
        .collect()
}

pub fn curve_from_daily(daily: &[RksDaily]) -> Vec<CurvePoint> {
    daily
        .iter()
        .map(|point| CurvePoint {
            at: point.observed_at,
            rks: point.rks,
        })
        .collect()
}

pub fn card_height(content_h: f32) -> f32 {
    content_h + 2.0 * CARD_PAD + TITLE_SIZE + TITLE_GAP
}

pub fn card_content(rect: Rect) -> Rect {
    let top = CARD_PAD + TITLE_SIZE + TITLE_GAP;
    Rect::from_xywh(
        rect.left() + CARD_PAD,
        rect.top() + top,
        rect.width() - 2.0 * CARD_PAD,
        rect.height() - CARD_PAD - top,
    )
}

pub fn profile_card() -> Rect {
    Rect::from_xywh(MARGIN_X, BLOCKS_TOP, CONTENT_W, card_height(PROFILE_CONTENT_H))
}

pub fn middle_top() -> f32 {
    profile_card().bottom() + BLOCK_GAP
}

pub fn rank_card() -> Rect {
    Rect::from_xywh(MARGIN_X, middle_top(), COL_W, card_height(MIDDLE_CONTENT_H))
}

pub fn rank_number_at() -> (f32, f32) {
    let content = card_content(rank_card());
    (content.left(), content.top() + RANK_NUMBER_SIZE)
}

pub fn rank_note_at() -> (f32, f32) {
    let (x, baseline) = rank_number_at();
    (x, baseline + ROW_GAP * 0.7)
}

pub fn rank_track() -> Rect {
    let content = card_content(rank_card());
    Rect::from_xywh(
        content.left(),
        rank_note_at().1 + ROW_GAP * 0.95,
        content.width(),
        RANK_TRACK_H,
    )
}

pub fn rank_rows_top() -> f32 {
    rank_track().bottom() + ROW_GAP * 0.7
}

pub fn rank_shares(omitted: &[bool]) -> Vec<f32> {
    if omitted.is_empty() {
        return vec![1.0; RANK_ROWS];
    }
    omitted
        .iter()
        .map(|flag| if *flag { RANK_OMIT_SHARE } else { 1.0 })
        .collect()
}

pub fn rank_slot(shares: &[f32], row: usize) -> (f32, f32) {
    let total: f32 = shares.iter().sum();
    if shares.is_empty() || total <= 0.0 {
        let top = rank_rows_top();
        return (top, top);
    }
    let index = row.min(shares.len() - 1);
    let unit = (card_content(rank_card()).bottom() - rank_rows_top()) / total;
    let mut top = rank_rows_top();
    for share in &shares[..index] {
        top += unit * share;
    }
    (top, top + unit * shares[index])
}

pub fn rank_row_base(shares: &[f32], row: usize) -> f32 {
    let (top, bottom) = rank_slot(shares, row);
    (top + bottom) / 2.0 + TEXT_SIZE * MID_RATIO
}

pub fn rank_avatar_rect(shares: &[f32], row: usize) -> Rect {
    let (top, bottom) = rank_slot(shares, row);
    let content = card_content(rank_card());
    Rect::from_xywh(
        content.left(),
        (top + bottom) / 2.0 - RANK_AVATAR / 2.0,
        RANK_AVATAR,
        RANK_AVATAR,
    )
}

pub fn rank_index_left() -> f32 {
    card_content(rank_card()).left() + RANK_AVATAR + TIGHT_GAP
}

pub fn rank_badge_rect(shares: &[f32], row: usize) -> Rect {
    let (top, bottom) = rank_slot(shares, row);
    Rect::from_xywh(
        rank_index_left() + RANK_INDEX_W + RANK_BADGE_GAP,
        (top + bottom) / 2.0 - RANK_BADGE_H / 2.0,
        RANK_BADGE_W,
        RANK_BADGE_H,
    )
}

pub fn rank_name_left() -> f32 {
    rank_index_left() + RANK_INDEX_W + RANK_BADGE_GAP + RANK_BADGE_W + RANK_BADGE_GAP
}

pub fn rank_rks_right() -> f32 {
    card_content(rank_card()).right()
}

pub fn rank_band(top: f32, bottom: f32) -> Rect {
    let card = rank_card();
    Rect::from_xywh(
        card.left() + RANK_BAND_SIDE,
        top + RANK_BAND_INSET,
        card.width() - 2.0 * RANK_BAND_SIDE,
        (bottom - top) - 2.0 * RANK_BAND_INSET,
    )
}

pub fn rank_me_bar(band: Rect) -> Rect {
    Rect::from_xywh(
        band.left() + RANK_ME_BAR_INSET,
        band.top() + 2.0,
        RANK_ME_BAR_W,
        band.height() - 4.0,
    )
}

pub fn diff_card() -> Rect {
    Rect::from_xywh(RIGHT_X, middle_top(), COL_W, card_height(MIDDLE_CONTENT_H))
}

pub fn curve_card() -> Rect {
    let content_h = NOTE_SIZE
        + CURVE_NOTE_DROP
        + CURVE_PLOT_H
        + CURVE_DATE_DROP
        + NOTE_SIZE * 2.0
        + CURVE_STATS_DROP;
    Rect::from_xywh(
        MARGIN_X,
        rank_card().bottom() + BLOCK_GAP,
        CONTENT_W,
        card_height(content_h),
    )
}

pub fn profile_avatar(content: Rect) -> Rect {
    let baseline = profile_name_at(content).1;
    Rect::from_xywh(
        content.left(),
        baseline - NAME_SIZE * NAME_MID,
        AVATAR,
        AVATAR,
    )
}

pub fn profile_text_x(content: Rect) -> f32 {
    content.left() + PROFILE_COL_X
}

pub fn profile_name_at(content: Rect) -> (f32, f32) {
    (profile_text_x(content), content.top() + NAME_DROP)
}

pub fn profile_name_w(content: Rect) -> f32 {
    profile_intro_w(content) - 180.0
}

pub fn profile_badge(content: Rect, name_w: f32) -> Rect {
    let (x, baseline) = profile_name_at(content);
    Rect::from_xywh(
        x + name_w + BADGE_DX,
        baseline - NAME_SIZE * MID_RATIO - BADGE_H / 2.0,
        BADGE_W,
        BADGE_H,
    )
}

pub fn profile_rks_label_at(content: Rect) -> (f32, f32) {
    (content.right(), content.top() + RKS_LABEL_DROP)
}

pub fn profile_rks_at(content: Rect) -> (f32, f32) {
    (content.right(), content.top() + RKS_DROP)
}

pub fn profile_intro_at(content: Rect, line: usize) -> (f32, f32) {
    (
        profile_text_x(content),
        content.top() + INTRO_DROP + line as f32 * INTRO_PITCH,
    )
}

pub fn profile_intro_w(content: Rect) -> f32 {
    content.right() - PROFILE_RKS_W - profile_text_x(content)
}

pub fn profile_stat_rule_y(content: Rect) -> f32 {
    content.top() + STATS_TOP - STAT_RULE_DROP
}

pub fn profile_stat_cell(content: Rect, index: usize) -> Rect {
    let x0 = profile_text_x(content);
    let total = content.right() - x0;
    let gap = ROW_GAP * 0.8;
    let w = (total - 2.0 * gap) / 3.0;
    let y = content.top() + STATS_TOP;
    Rect::from_xywh(
        x0 + index as f32 * (w + gap),
        y,
        w,
        content.bottom() - y,
    )
}

pub fn profile_stat_label_at(cell: Rect) -> (f32, f32) {
    (
        cell.left(),
        cell.top() + (cell.height() - STAT_BLOCK_H) / 2.0 + NOTE_SIZE,
    )
}

pub fn profile_stat_value_at(cell: Rect) -> (f32, f32) {
    let (x, baseline) = profile_stat_label_at(cell);
    (x, baseline + STAT_VALUE_DROP + TEXT_SIZE)
}

pub fn diff_row_base(content: Rect, index: usize) -> f32 {
    let pitch = DIFF_ROWS_H / DIFF_ROWS as f32;
    content.top() + (index as f32 + 0.5) * pitch + TEXT_SIZE * MID_RATIO
}

pub fn diff_bar_rect(content: Rect, index: usize, nums_left: f32) -> Rect {
    let x = content.left() + DIFF_LABEL_W + TIGHT_GAP;
    let w = nums_left - TIGHT_GAP - x;
    let mid = diff_row_base(content, index) - TEXT_SIZE * MID_RATIO;
    Rect::from_xywh(x, mid - DIFF_BAR_H / 2.0, w.max(1.0), DIFF_BAR_H)
}

pub fn diff_rule_y(content: Rect) -> f32 {
    content.top() + DIFF_RULE_Y
}

pub fn collect_label_at(content: Rect) -> (f32, f32) {
    (content.left(), content.top() + COLLECT_TOP + NOTE_SIZE)
}

pub fn collect_number_at(content: Rect) -> (f32, f32) {
    let (x, baseline) = collect_label_at(content);
    (x, baseline + COLLECT_NUM_DROP + COLLECT_NUM_SIZE)
}

pub fn collect_bar_rect(content: Rect) -> Rect {
    let (_, baseline) = collect_number_at(content);
    Rect::from_xywh(
        content.left(),
        baseline + COLLECT_BAR_DROP,
        content.width(),
        COLLECT_BAR_H,
    )
}

pub fn stats_rule_y(content: Rect) -> f32 {
    content.top() + COLLECT_RULE_Y
}

pub fn stats_cell(content: Rect, index: usize) -> Rect {
    let w = (content.width() - (STATS_COLS as f32 - 1.0) * STATS_CELL_GAP_X) / STATS_COLS as f32;
    let top = content.top() + COLLECT_RULE_Y + STATS_CELL_GAP_Y;
    let h = (content.bottom() - top - (STATS_ROWS as f32 - 1.0) * STATS_CELL_GAP_Y)
        / STATS_ROWS as f32;
    let row = index / STATS_COLS;
    let col = index % STATS_COLS;
    Rect::from_xywh(
        content.left() + col as f32 * (w + STATS_CELL_GAP_X),
        top + row as f32 * (h + STATS_CELL_GAP_Y),
        w,
        h,
    )
}

pub fn stats_label_at(cell: Rect) -> (f32, f32) {
    (
        cell.left(),
        cell.top() + (cell.height() - STAT_BLOCK_H) / 2.0 + NOTE_SIZE,
    )
}

pub fn stats_value_at(cell: Rect) -> (f32, f32) {
    let (x, baseline) = stats_label_at(cell);
    (x, baseline + STAT_VALUE_DROP + TEXT_SIZE)
}

pub fn curve_plot() -> Rect {
    let content = card_content(curve_card());
    Rect::from_xywh(
        content.left() + CURVE_GUTTER,
        content.top() + NOTE_SIZE + CURVE_NOTE_DROP,
        content.width() - CURVE_GUTTER - MILESTONE_COL_W - MILESTONE_COL_GAP,
        CURVE_PLOT_H,
    )
}

pub fn curve_note_at() -> (f32, f32) {
    let plot = curve_plot();
    (plot.right(), plot.top() - TIGHT_GAP * 0.5)
}

pub fn curve_date_at() -> (f32, f32) {
    let plot = curve_plot();
    (plot.left(), plot.bottom() + CURVE_DATE_DROP)
}

pub fn curve_date_end_at() -> (f32, f32) {
    (curve_plot().right(), curve_date_at().1)
}

pub fn curve_tick_at(y: f32) -> (f32, f32) {
    (
        curve_plot().left() - TIGHT_GAP,
        y + NOTE_SIZE * MID_RATIO,
    )
}

pub fn curve_empty_at(line: usize) -> (f32, f32) {
    let plot = curve_plot();
    (
        plot.left(),
        plot.top() + plot.height() / 2.0 + ROW_GAP + line as f32 * ROW_GAP,
    )
}

pub fn milestone_col() -> Rect {
    let plot = curve_plot();
    let content = card_content(curve_card());
    Rect::from_xywh(
        plot.right() + MILESTONE_COL_GAP,
        plot.top(),
        MILESTONE_COL_W,
        content.bottom() - plot.top(),
    )
}

pub fn milestone_label_at() -> (f32, f32) {
    let col = milestone_col();
    (col.left(), col.top() - TIGHT_GAP * 0.5)
}

pub fn milestone_body() -> Rect {
    let col = milestone_col();
    Rect::from_xywh(
        col.left(),
        col.top() + MILESTONE_LABEL_H,
        col.width(),
        col.height() - MILESTONE_LABEL_H,
    )
}

pub fn milestone_slot(index: usize, rows: usize) -> (f32, f32) {
    let body = milestone_body();
    let pitch = body.height() / rows.max(1) as f32;
    let top = body.top() + index as f32 * pitch;
    (top, top + pitch)
}

pub fn milestone_kind_at(index: usize, rows: usize) -> (f32, f32) {
    let (top, _) = milestone_slot(index, rows);
    let col = milestone_col();
    (
        col.left() + MILESTONE_RAIL,
        top + MILESTONE_PITCH + SUB_SIZE,
    )
}

pub fn milestone_day_at(index: usize, rows: usize) -> (f32, f32) {
    let (x, baseline) = milestone_kind_at(index, rows);
    (
        x,
        baseline + TIGHT_GAP * 0.5 + NOTE_SIZE,
    )
}

pub fn milestone_dot_at(index: usize, rows: usize) -> Rect {
    let (_, baseline) = milestone_kind_at(index, rows);
    let cy = baseline - SUB_SIZE * MID_RATIO;
    let radius = MILESTONE_DOT / 2.0;
    let col = milestone_col();
    Rect::from_xywh(
        col.left() + MILESTONE_RAIL / 2.0 - radius,
        cy - radius,
        MILESTONE_DOT,
        MILESTONE_DOT,
    )
}

pub fn rows_bottom() -> f32 {
    curve_card().bottom()
}

pub fn footer_top() -> f32 {
    rows_bottom() + FOOTER_DROP
}

pub fn footer_left_at() -> (f32, f32) {
    (MARGIN_X, footer_top() + FOOTER_H)
}

pub fn footer_right_at() -> (f32, f32) {
    (CONTENT_RIGHT, footer_top() + FOOTER_H)
}

pub fn canvas_height() -> f32 {
    footer_top() + FOOTER_H + BOTTOM_MARGIN
}

pub fn canvas_size() -> (i32, i32) {
    (
        CANVAS_W.round() as i32,
        canvas_height().round() as i32,
    )
}

pub fn curve_bounds(points: &[CurvePoint]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    let mut lo = points[0].rks;
    let mut hi = points[0].rks;
    for point in points {
        lo = lo.min(point.rks);
        hi = hi.max(point.rks);
    }
    lo = (lo * 5.0).floor() / 5.0;
    hi = (hi * 5.0).ceil() / 5.0;
    if hi <= lo {
        hi = lo + 0.2;
    }
    (lo, hi)
}

pub fn tick_values(lo: f64, hi: f64) -> [f64; 3] {
    [lo, (lo + hi) / 2.0, hi]
}

pub fn tick_y(plot: Rect, value: f64, lo: f64, hi: f64) -> f32 {
    if hi <= lo {
        return plot.bottom();
    }
    plot.bottom() - ((value - lo) / (hi - lo)) as f32 * plot.height()
}

pub fn point_x(points: &[CurvePoint], index: usize, rect: Rect) -> f32 {
    if points.is_empty() {
        return rect.left();
    }
    let start = points[0].at;
    let span = points[points.len() - 1].at - start;
    if span > 0 {
        return rect.left() + (points[index].at - start) as f32 / span as f32 * rect.width();
    }
    if points.len() > 1 {
        return rect.left() + index as f32 / (points.len() - 1) as f32 * rect.width();
    }
    rect.left()
}

pub fn peak_index(points: &[CurvePoint]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        match best {
            Some(at) if points[at].rks >= point.rks => {}
            _ => best = Some(index),
        }
    }
    best
}

pub fn low_index(points: &[CurvePoint]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        match best {
            Some(at) if points[at].rks <= point.rks => {}
            _ => best = Some(index),
        }
    }
    best
}

pub fn curve_stats(points: &[CurvePoint]) -> CurveStats {
    let mut stats = CurveStats::default();
    if points.is_empty() {
        return stats;
    }
    let mut closes: Vec<(i64, f64)> = Vec::with_capacity(points.len());
    for point in points {
        let day = point.at.div_euclid(DAY_SECONDS);
        match closes.last_mut() {
            Some(last) if last.0 == day => last.1 = point.rks,
            _ => closes.push((day, point.rks)),
        }
    }
    stats.days = closes.len() as i32;
    stats.first = closes[0].0 * DAY_SECONDS;
    stats.span_days = (closes[closes.len() - 1].0 - closes[0].0) as i32 + 1;
    let mut run = 1;
    for pair in closes.windows(2) {
        if pair[1].0 - pair[0].0 == 1 {
            run += 1;
            stats.streak = stats.streak.max(run);
        } else {
            run = 1;
        }
    }
    if stats.streak == 0 {
        stats.streak = 1;
    }
    for pair in closes.windows(2) {
        let gain = pair[1].1 - pair[0].1;
        if gain > stats.best_gain {
            stats.best_gain = gain;
            stats.best_day = pair[1].0 * DAY_SECONDS;
            stats.has_best = true;
        }
        let gap = (pair[1].0 - pair[0].0) as i32;
        stats.max_gap = stats.max_gap.max(gap);
    }
    stats
}

pub fn recent_delta(points: &[CurvePoint], days: i64) -> Option<f64> {
    let last = points.last()?;
    let edge = last.at - days * DAY_SECONDS;
    let mut found: Option<f64> = None;
    for point in points.iter().rev() {
        if point.at <= edge {
            found = Some(point.rks);
            break;
        }
    }
    found.map(|value| last.rks - value)
}

pub fn sync_text(req: &OverviewRequest) -> String {
    if !req.synced.trim().is_empty() {
        return req.synced.trim().to_string();
    }
    match req.curve.last() {
        Some(point) => date_text(point.at, true),
        None => String::new(),
    }
}

pub fn date_text(at: i64, with_year: bool) -> String {
    let Some(stamp) = chrono::DateTime::from_timestamp(at, 0) else {
        return String::new();
    };
    if with_year {
        stamp.format("%Y-%m-%d").to_string()
    } else {
        stamp.format("%m-%d").to_string()
    }
}

pub fn date_window_text(at: i64) -> String {
    let Some(stamp) = chrono::DateTime::from_timestamp(at, 0) else {
        return String::new();
    };
    stamp.format("%Y-%m-%d %H:%M").to_string()
}

pub fn delta_text(value: f64) -> String {
    let text = format!("{value:+.4}");
    if text == "-0.0000" {
        return String::from("+0.0000");
    }
    text
}

pub fn delta_or(value: f64, ok: bool) -> String {
    if ok {
        delta_text(value)
    } else {
        DASH.to_string()
    }
}

pub fn number_or_dash(value: i32, unit: &str) -> String {
    if value <= 0 {
        return DASH.to_string();
    }
    if unit.is_empty() {
        return value.to_string();
    }
    format!("{value} {unit}")
}

pub fn percent_or(value: i32, total: i32) -> String {
    if total <= 0 {
        return DASH.to_string();
    }
    format!("{}%", ((value as f64) / (total as f64) * 100.0).round() as i32)
}

pub fn ratio(value: i32, total: i32) -> f32 {
    if total <= 0 {
        return 0.0;
    }
    (value as f32 / total as f32).clamp(0.0, 1.0)
}

fn fill_rect(kit: &CanvasKit<'_>, area: Rect, ink: Color) {
    sketch::ph_fill(kit, sketch::ph_rect(area.left(), area.top(), area.width(), area.height()), ink);
}

fn rule(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, ink: Color) {
    if x1 <= x0 {
        return;
    }
    fill_rect(
        kit,
        Rect::from_xywh(x0, y, x1 - x0, 1.0),
        ink,
    );
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
    let baseline = area.top() + CARD_PAD + TITLE_SIZE;
    if !title.is_empty() {
        let style = sketch::text_style(sketch::FONT_DISPLAY, TITLE_SIZE, SCORE_INK);
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
        let style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
        sketch::text_right(
            kit,
            &style,
            sub,
            area.right() - CARD_PAD,
            baseline,
            NOTE_INK,
        );
    }
    card_content(area)
}

fn wrap_text(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    width: f32,
    max_lines: usize,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for ch in value.chars() {
        if ch == '\n' || ch == '\r' {
            lines.push(current.clone());
            current.clear();
            continue;
        }
        let mut probe = current.clone();
        probe.push(ch);
        if !current.is_empty() && sketch::text_width(kit, style, &probe).round() > width.round() {
            lines.push(current.clone());
            current.clear();
            current.push(ch);
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            *last = sketch::ellipsize(kit, style, last, width);
        }
    }
    lines
}

fn draw_head(kit: &CanvasKit<'_>, req: &OverviewRequest) {
    let style = sketch::text_style(sketch::FONT_DISPLAY, HEAD_TITLE_SIZE, SCORE_INK);
    sketch::draw_text(
        kit,
        &style,
        HEAD_TITLE,
        MARGIN_X,
        HEAD_TITLE_BASELINE,
        SCORE_INK,
    );
    let note_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let mut parts: Vec<String> = vec![String::from(HEAD_NOTE)];
    if req.player.has_player && !req.player.name.trim().is_empty() {
        parts.push(req.player.name.trim().to_string());
    }
    if !req.server.trim().is_empty() {
        parts.push(req.server.trim().to_string());
    }
    let note = parts.join(" · ");
    let width = CONTENT_RIGHT - 400.0 - MARGIN_X;
    let text = sketch::ellipsize(kit, &note_style, &note, width);
    sketch::draw_text(
        kit,
        &note_style,
        &text,
        MARGIN_X,
        HEAD_NOTE_BASELINE,
        NOTE_INK,
    );
}

fn draw_avatar(kit: &CanvasKit<'_>, root: &Path, area: Rect, chosen: &str) {
    draw_avatar_round(kit, root, area, chosen, CARD_RADIUS * 0.6);
}

fn draw_avatar_round(kit: &CanvasKit<'_>, root: &Path, area: Rect, chosen: &str, radius: f32) {
    let mut keys: Vec<String> = Vec::with_capacity(2);
    if !chosen.trim().is_empty() {
        keys.push(super::avatar_asset(root, chosen.trim()));
    }
    keys.push(super::ui_asset(root, memo::DEFAULT_AVATAR));
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    for key in keys {
        let Some(texture) = memo::texture(kit, &key) else {
            continue;
        };
        let Some(image) = texture.image() else {
            continue;
        };
        let source = aurorix_render::cover_source_rect(image, frame);
        let sampling = aurorix_render::sampling_for((source.width(), source.height()), frame);
        sketch::clip_round(kit, frame, radius, || {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            kit.canvas().draw_image_rect_with_sampling_options(
                image,
                Some((&source, SrcRectConstraint::Fast)),
                frame,
                sampling,
                &paint,
            );
        });
        return;
    }
}

fn draw_badge(kit: &CanvasKit<'_>, root: &Path, area: Rect, progress: i32, level: i32) {
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    player::rank_frame(kit, root, frame, level);
    let style = sketch::text_style(sketch::FONT_BODY, BADGE_TEXT_SIZE, SCORE_INK);
    sketch::text_center(
        kit,
        &style,
        &progress.to_string(),
        frame.left() + frame.width() / 2.0,
        frame.top() + BADGE_TEXT_DROP,
        SCORE_INK,
    );
}

fn draw_profile(
    kit: &CanvasKit<'_>,
    root: &Path,
    background: Option<&Image>,
    canvas: Rect,
    req: &OverviewRequest,
) {
    let content = draw_card(
        kit,
        background,
        canvas,
        profile_card(),
        PROFILE_TITLE,
        "",
    );
    draw_avatar(
        kit,
        root,
        profile_avatar(content),
        req.player.avatar.as_str(),
    );
    let name_style = sketch::text_style(sketch::FONT_DISPLAY, NAME_SIZE, SCORE_INK);
    let raw = req.player.name.trim();
    let name = if raw.is_empty() {
        String::from(EMPTY_NAME)
    } else {
        sketch::ellipsize(kit, &name_style, raw, profile_name_w(content))
    };
    let (name_x, name_base) = profile_name_at(content);
    sketch::draw_text(kit, &name_style, &name, name_x, name_base, SCORE_INK);
    let name_w = sketch::text_width(kit, &name_style, &name).round();
    if req.player.has_player {
        draw_badge(
            kit,
            root,
            profile_badge(content, name_w),
            req.player.rank,
            req.player.rank_level,
        );
    }

    let (rks_x, rks_base) = profile_rks_at(content);
    let big_style = sketch::text_style(sketch::FONT_DISPLAY, RKS_SIZE, SCORE_INK);
    if req.player.rks > 0.0 {
        let (label_x, label_base) = profile_rks_label_at(content);
        let label_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
        sketch::text_right(kit, &label_style, "RKS", label_x, label_base, NOTE_INK);
        sketch::text_right(
            kit,
            &big_style,
            &player::four_decimals(req.player.rks),
            rks_x,
            rks_base,
            SCORE_INK,
        );
    } else {
        sketch::text_right(kit, &big_style, DASH, rks_x, rks_base, SCORE_INK);
    }

    let intro_style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, LABEL_INK);
    let intro = req.self_intro.trim();
    if intro.is_empty() {
        let (x, baseline) = profile_intro_at(content, 0);
        sketch::draw_text(
            kit,
            &intro_style,
            EMPTY_INTRO,
            x,
            baseline,
            PLACEHOLDER_INK,
        );
    } else {
        let lines = wrap_text(
            kit,
            &intro_style,
            intro,
            profile_intro_w(content),
            INTRO_LINES,
        );
        for (index, line) in lines.iter().enumerate() {
            let (x, baseline) = profile_intro_at(content, index);
            sketch::draw_text(kit, &intro_style, line, x, baseline, LABEL_INK);
        }
    }

    let stats = curve_stats(&req.curve);
    rule(
        kit,
        profile_text_x(content),
        content.right(),
        profile_stat_rule_y(content),
        RULE_INK,
    );
    let cells = [
        (
            "记录天数",
            number_or_dash(stats.days, ""),
            "天",
        ),
        (
            "出勤率",
            percent_or(stats.days, stats.span_days),
            "",
        ),
        (
            "最长连续",
            number_or_dash(stats.streak, ""),
            "天",
        ),
    ];
    let label_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, LABEL_INK);
    let value_style = sketch::text_style(sketch::FONT_DISPLAY, TEXT_SIZE, SCORE_INK);
    let unit_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, PLACEHOLDER_INK);
    for (index, (label, number, unit)) in cells.iter().enumerate() {
        let cell = profile_stat_cell(content, index);
        let (label_x, label_base) = profile_stat_label_at(cell);
        sketch::draw_text(kit, &label_style, label, label_x, label_base, LABEL_INK);
        let (value_x, value_base) = profile_stat_value_at(cell);
        sketch::draw_text(kit, &value_style, number, value_x, value_base, SCORE_INK);
        if !unit.is_empty() && number.as_str() != DASH {
            let width = sketch::text_width(kit, &value_style, number).round();
            sketch::draw_text(
                kit,
                &unit_style,
                unit,
                value_x + width + TIGHT_GAP * 0.35,
                value_base,
                PLACEHOLDER_INK,
            );
        }
    }
}

fn draw_tracked_bar(kit: &CanvasKit<'_>, area: Rect, value: f32, height: f32) {
    let track = sketch::ph_rect(area.left(), area.top(), area.width(), height);
    sketch::fill_round(kit, track, height / 2.0, TRACK_INK);
    let filled = area.width() * value.clamp(0.0, 1.0);
    if filled <= 0.0 {
        return;
    }
    let bar = sketch::ph_rect(area.left(), area.top(), filled, height);
    sketch::fill_round(kit, bar, height / 2.0, ACCENT_INK);
}

fn rank_place_text(row: &archive::RankRow) -> String {
    if row.hidden {
        return String::from(RANK_HIDDEN);
    }
    if row.place <= 0 {
        return DASH.to_string();
    }
    format!("#{}", sketch::group_digits(&row.place.to_string()))
}

fn rank_index_text(row: &archive::RankRow) -> String {
    if row.hidden {
        return String::from(RANK_HIDDEN);
    }
    format!("#{}", sketch::group_digits(&row.place.to_string()))
}

fn rank_lines(board: &archive::RankBoard) -> Vec<(Option<archive::RankRow>, bool)> {
    let mut lines: Vec<(Option<archive::RankRow>, bool)> = Vec::with_capacity(RANK_ROWS + 2);
    let mut in_top = false;
    for row in &board.top {
        if lines.len() >= RANK_ROWS {
            break;
        }
        let me = board.me.place > 0 && row.place == board.me.place;
        in_top |= me;
        lines.push((Some(row.clone()), me));
    }
    if board.me.place > 0 && !in_top {
        if board.omitted > 0 {
            lines.push((None, false));
        }
        lines.push((Some(board.me.clone()), true));
    }
    lines
}

fn draw_rank_badge(
    kit: &CanvasKit<'_>,
    root: &Path,
    area: Rect,
    progress: i32,
    level: i32,
    size: f32,
) {
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    player::rank_frame(kit, root, frame, level);
    let style = sketch::text_style(sketch::FONT_BODY, size, SCORE_INK);
    sketch::text_center(
        kit,
        &style,
        &progress.to_string(),
        frame.left() + frame.width() / 2.0,
        frame.top() + frame.height() / 2.0 + size * MID_RATIO,
        SCORE_INK,
    );
}

fn draw_rank(
    kit: &CanvasKit<'_>,
    root: &Path,
    background: Option<&Image>,
    canvas: Rect,
    req: &OverviewRequest,
) {
    let board = &req.rank;
    let sub = if board.total > 0 {
        format!("共 {} 人", sketch::group_digits(&board.total.to_string()))
    } else {
        String::new()
    };
    let title = if board.scope.is_empty() {
        RANK_TITLE
    } else {
        board.scope.as_str()
    };
    let content = draw_card(kit, background, canvas, rank_card(), title, &sub);
    if !board.has_data {
        let style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
        let (x, baseline) = rank_number_at();
        for (index, line) in RANK_EMPTY.iter().enumerate() {
            let fitted = sketch::ellipsize(kit, &style, line, content.width());
            sketch::draw_text(
                kit,
                &style,
                &fitted,
                x,
                baseline + index as f32 * ROW_GAP,
                PLACEHOLDER_INK,
            );
        }
        return;
    }
    let me = &board.me;
    let number_style = sketch::text_style(sketch::FONT_DISPLAY, RANK_NUMBER_SIZE, SCORE_INK);
    let (number_x, number_base) = rank_number_at();
    sketch::draw_text(
        kit,
        &number_style,
        &rank_place_text(me),
        number_x,
        number_base,
        SCORE_INK,
    );
    let note_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let (note_x, note_base) = rank_note_at();
    let note = if !me.hidden && me.place > 0 && board.total > 1 {
        format!(
            "超过 {:.0}% 的玩家",
            100.0 * (1.0 - f64::from(me.place) / f64::from(board.total))
        )
    } else {
        DASH.to_string()
    };
    sketch::draw_text(kit, &note_style, &note, note_x, note_base, NOTE_INK);
    let track = rank_track();
    sketch::fill_round(
        kit,
        sketch::ph_rect(track.left(), track.top(), track.width(), track.height()),
        track.height() / 2.0,
        RANK_TRACK_INK,
    );
    if !me.hidden && me.place > 0 && board.total > 1 {
        let better = 1.0 - (f64::from(me.place) - 1.0) / (f64::from(board.total) - 1.0);
        let mark_x = track.left() + better as f32 * track.width();
        if mark_x > track.left() {
            sketch::fill_round(
                kit,
                sketch::ph_rect(
                    track.left(),
                    track.top(),
                    mark_x - track.left(),
                    track.height(),
                ),
                track.height() / 2.0,
                RANK_TRACK_FILL_INK,
            );
        }
        sketch::fill_round(
            kit,
            sketch::ph_rect(
                mark_x - RANK_MARK_W / 2.0,
                track.top() - RANK_MARK_DROP,
                RANK_MARK_W,
                track.height() + 2.0 * RANK_MARK_DROP,
            ),
            RANK_MARK_W / 2.0,
            SCORE_INK,
        );
    }
    let lines = rank_lines(board);
    let mut omitted: Vec<bool> = Vec::with_capacity(lines.len());
    for (row, _) in lines.iter() {
        omitted.push(row.is_none());
    }
    let shares = rank_shares(&omitted);
    let cell_style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, LABEL_INK);
    let hint_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, PLACEHOLDER_INK);
    for (index, (row, is_me)) in lines.iter().enumerate() {
        let (top, bottom) = rank_slot(&shares, index);
        let base = rank_row_base(&shares, index);
        let Some(row) = row.as_ref() else {
            let text = format!(
                "省略 {} 人…",
                sketch::group_digits(&board.omitted.to_string())
            );
            let fitted = sketch::ellipsize(kit, &hint_style, &text, content.width());
            sketch::draw_text(
                kit,
                &hint_style,
                &fitted,
                content.left(),
                base,
                PLACEHOLDER_INK,
            );
            continue;
        };
        if *is_me {
            let band = rank_band(top, bottom);
            if index > 0 {
                if let Some(previous) = lines[index - 1].0.as_ref() {
                    if previous.place > 0 && row.place > previous.place + 1 {
                        rule(kit, band.left(), band.right(), top - 1.0, RULE_INK);
                    }
                }
            }
            sketch::fill_round(
                kit,
                sketch::ph_rect(band.left(), band.top(), band.width(), band.height()),
                RANK_BAND_RADIUS,
                RANK_ROW_INK,
            );
            let bar = rank_me_bar(band);
            sketch::fill_round(
                kit,
                sketch::ph_rect(bar.left(), bar.top(), bar.width(), bar.height()),
                bar.width() / 2.0,
                ACCENT_INK,
            );
        }
        let ink = if *is_me { SCORE_INK } else { LABEL_INK };
        draw_avatar_round(
            kit,
            root,
            rank_avatar_rect(&shares, index),
            row.avatar.as_str(),
            CARD_RADIUS * 0.5,
        );
        sketch::draw_text(
            kit,
            &cell_style,
            &rank_index_text(row),
            rank_index_left(),
            base,
            ink,
        );
        draw_rank_badge(
            kit,
            root,
            rank_badge_rect(&shares, index),
            row.badge,
            row.badge_level,
            RANK_BADGE_TEXT,
        );
        let name_x = rank_name_left();
        let name = sketch::ellipsize(
            kit,
            &cell_style,
            row.nickname.as_str(),
            rank_rks_right() - name_x - ROW_GAP,
        );
        sketch::draw_text(kit, &cell_style, &name, name_x, base, ink);
        sketch::text_right(
            kit,
            &cell_style,
            &player::four_decimals(row.rks),
            rank_rks_right(),
            base,
            SCORE_INK,
        );
    }
}

fn diff_nums(diff: &OverviewDiff) -> (String, String) {
    let head = format!("{} / {}", diff.played, diff.total);
    let mid = if diff.phi > 0 {
        format!("φ {}", diff.phi)
    } else {
        String::new()
    };
    (head, mid)
}

fn draw_diff_stack(kit: &CanvasKit<'_>, area: Rect, diff: &OverviewDiff) {
    let total = diff.total.max(0);
    let phi = diff.phi.clamp(0, total);
    let fc = diff.full_combo.clamp(phi, total);
    let cleared = diff.cleared.clamp(fc, total);
    let segments: [(i32, Color); 4] = [
        (phi, PHI_INK),
        (fc - phi, ACCENT_INK),
        (cleared - fc, CLEAR_INK),
        (total - cleared, TRACK_INK),
    ];
    let track = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    let top = track.top();
    let height = track.height();
    sketch::clip_round(kit, track, height / 2.0, || {
        sketch::ph_fill(kit, track, TRACK_INK);
        if total <= 0 {
            return;
        }
        let mut left = track.left();
        let mut walked = 0;
        for (weight, ink) in segments {
            if weight <= 0 {
                continue;
            }
            walked += weight;
            let edge = if walked >= total {
                track.right()
            } else {
                (track.left() + track.width() * walked as f32 / total as f32).round()
            };
            if edge <= left {
                continue;
            }
            sketch::ph_fill(kit, Rect::from_xywh(left, top, edge - left, height), ink);
            left = edge;
        }
    });
}

fn draw_diff_row(
    kit: &CanvasKit<'_>,
    content: Rect,
    index: usize,
    diff: &OverviewDiff,
    nums_w: f32,
) {
    let base = diff_row_base(content, index);
    let label_style = sketch::text_style(sketch::FONT_DISPLAY, TEXT_SIZE, LABEL_INK);
    sketch::draw_text(
        kit,
        &label_style,
        &diff.label,
        content.left(),
        base,
        memo::difficulty_chip(&diff.label),
    );
    draw_diff_stack(kit, diff_bar_rect(content, index, content.right() - nums_w), diff);
    let (head, mid) = diff_nums(diff);
    let nums_style = sketch::text_style(sketch::FONT_BODY, SUB_SIZE, LABEL_INK);
    let head_w = sketch::text_width(kit, &nums_style, &head).round();
    let x = content.right() - nums_w;
    sketch::draw_text(kit, &nums_style, &head, x, base, LABEL_INK);
    if !mid.is_empty() {
        sketch::draw_text(
            kit,
            &nums_style,
            &mid,
            x + head_w + TIGHT_GAP,
            base,
            PHI_INK,
        );
    }
}

fn total_charts(diffs: &[OverviewDiff]) -> i32 {
    diffs
        .iter()
        .filter(|diff| diff.total > 0)
        .map(|diff| diff.total)
        .sum()
}

fn draw_collect(kit: &CanvasKit<'_>, content: Rect, req: &OverviewRequest) {
    rule(kit, content.left(), content.right(), diff_rule_y(content), RULE_INK);
    let label_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, PLACEHOLDER_INK);
    let (label_x, label_base) = collect_label_at(content);
    sketch::draw_text(
        kit,
        &label_style,
        "解锁谱面",
        label_x,
        label_base,
        PLACEHOLDER_INK,
    );
    let number_style = sketch::text_style(sketch::FONT_DISPLAY, COLLECT_NUM_SIZE, SCORE_INK);
    let (number_x, number_base) = collect_number_at(content);
    let number = req.collection.unlocked_charts.max(0).to_string();
    sketch::draw_text(
        kit,
        &number_style,
        &number,
        number_x,
        number_base,
        SCORE_INK,
    );
    let denom_style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
    let num_w = sketch::text_width(kit, &number_style, &number).round();
    let denom = if req.collection.total_charts > 0 {
        format!(" / {} 张谱面", req.collection.total_charts)
    } else {
        format!(" / {DASH} 谱面")
    };
    let denom_x = number_x + num_w + TIGHT_GAP;
    let denom = sketch::ellipsize(
        kit,
        &denom_style,
        &denom,
        content.right() - denom_x,
    );
    sketch::draw_text(kit, &denom_style, &denom, denom_x, number_base, PLACEHOLDER_INK);
    let bar = collect_bar_rect(content);
    draw_tracked_bar(
        kit,
        bar,
        ratio(req.collection.unlocked_charts, req.collection.total_charts),
        COLLECT_BAR_H,
    );
    let counts = if req.collection.total_songs > 0 {
        format!(
            "曲目 {}/{} 首",
            req.collection.unlocked_songs, req.collection.total_songs
        )
    } else {
        format!("曲目 {} 首", req.collection.unlocked_songs)
    };
    let counts_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    sketch::text_right(
        kit,
        &counts_style,
        &counts,
        content.right(),
        label_base,
        NOTE_INK,
    );
}

fn draw_stats(kit: &CanvasKit<'_>, content: Rect, req: &OverviewRequest) {
    rule(kit, content.left(), content.right(), stats_rule_y(content), RULE_INK);
    let mut phi = 0;
    let mut fc = 0;
    let mut cleared = 0;
    for diff in &req.diffs {
        phi += diff.phi.max(0);
        fc += diff.full_combo.max(0);
        cleared += diff.cleared.max(0);
    }
    let stats = curve_stats(&req.curve);
    let recent = recent_delta(&req.curve, 30);
    let best = if stats.has_best {
        format!("{}（{}）", delta_text(stats.best_gain), date_text(stats.best_day, false))
    } else {
        DASH.to_string()
    };
    let cells: [(&str, String, &str); 6] = [
        ("φ 谱面", number_or_dash(phi, ""), "张"),
        ("全连谱面", number_or_dash(fc, ""), "张"),
        ("通关谱面", number_or_dash(cleared, ""), "张"),
        ("单日最大涨幅", best, ""),
        ("近 30 天", delta_or(recent.unwrap_or(0.0), recent.is_some()), ""),
        (
            "首次记录",
            if stats.first > 0 {
                date_text(stats.first, true)
            } else {
                DASH.to_string()
            },
            "",
        ),
    ];
    let label_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, LABEL_INK);
    let value_style = sketch::text_style(sketch::FONT_DISPLAY, TEXT_SIZE, SCORE_INK);
    let unit_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, PLACEHOLDER_INK);
    for (index, (label, number, unit)) in cells.iter().enumerate() {
        let cell = stats_cell(content, index);
        let (label_x, label_base) = stats_label_at(cell);
        sketch::draw_text(kit, &label_style, label, label_x, label_base, LABEL_INK);
        let (value_x, value_base) = stats_value_at(cell);
        let fitted = sketch::ellipsize(kit, &value_style, number, cell.width());
        sketch::draw_text(kit, &value_style, &fitted, value_x, value_base, SCORE_INK);
        if !unit.is_empty() && number.as_str() != DASH {
            let width = sketch::text_width(kit, &value_style, &fitted).round();
            sketch::draw_text(
                kit,
                &unit_style,
                unit,
                value_x + width + TIGHT_GAP * 0.35,
                value_base,
                PLACEHOLDER_INK,
            );
        }
    }
}

fn draw_diffs(
    kit: &CanvasKit<'_>,
    background: Option<&Image>,
    canvas: Rect,
    req: &OverviewRequest,
) {
    let total = total_charts(&req.diffs);
    let sub = if total > 0 {
        format!(
            "解锁进度 {:.1}%",
            100.0 * req.collection.unlocked_charts as f32 / total as f32
        )
    } else {
        String::new()
    };
    let content = draw_card(
        kit,
        background,
        canvas,
        diff_card(),
        DIFF_TITLE,
        &sub,
    );
    if total > 0 {
        let measure = sketch::text_style(sketch::FONT_BODY, SUB_SIZE, LABEL_INK);
        let mut nums_w = 0.0_f32;
        for diff in &req.diffs {
            if diff.total <= 0 {
                continue;
            }
            let (head, mid) = diff_nums(diff);
            let mut width = sketch::text_width(kit, &measure, &head).round();
            if !mid.is_empty() {
                width += TIGHT_GAP + sketch::text_width(kit, &measure, &mid).round();
            }
            nums_w = nums_w.max(width);
        }
        for (index, diff) in req.diffs.iter().enumerate() {
            if diff.total <= 0 {
                continue;
            }
            draw_diff_row(kit, content, index, diff, nums_w);
        }
    } else {
        let style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
        let (x, baseline) = (content.left() + DIFF_LABEL_W, diff_row_base(content, 0));
        sketch::draw_text(kit, &style, "曲库还没就绪", x, baseline, PLACEHOLDER_INK);
    }
    draw_collect(kit, content, req);
    draw_stats(kit, content, req);
}

fn curve_area_path(points: &[CurvePoint], plot: Rect, lo: f64, hi: f64) -> SkiaPath {
    let mut builder = PathBuilder::new();
    let first = point_x(points, 0, plot);
    builder.move_to((first, plot.bottom()));
    for (index, point) in points.iter().enumerate() {
        let x = point_x(points, index, plot);
        let y = tick_y(plot, point.rks, lo, hi);
        builder.line_to((x, y));
    }
    let last = point_x(points, points.len() - 1, plot);
    builder.line_to((last, plot.bottom()));
    builder.close();
    builder.detach()
}

fn curve_line_path(points: &[CurvePoint], plot: Rect, lo: f64, hi: f64) -> SkiaPath {
    let mut builder = PathBuilder::new();
    for (index, point) in points.iter().enumerate() {
        let x = point_x(points, index, plot);
        let y = tick_y(plot, point.rks, lo, hi);
        if index == 0 {
            builder.move_to((x, y));
        } else {
            builder.line_to((x, y));
        }
    }
    builder.detach()
}

fn draw_curve_area(kit: &CanvasKit<'_>, points: &[CurvePoint], plot: Rect, lo: f64, hi: f64) {
    let path = curve_area_path(points, plot, lo, hi);
    let fade = Color::new(
        AREA_INK.r,
        AREA_INK.g,
        AREA_INK.b,
        (AREA_INK.a as f32 * CURVE_AREA_FADE).round() as u8,
    );
    let gradient = LinearGradient::vertical([
        GradientStop::new(0.0, AREA_INK),
        GradientStop::new(1.0, fade),
    ]);
    let Ok(shader) = gradient.shader(plot) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas().draw_path(&path, &paint);
}

fn draw_curve_line(kit: &CanvasKit<'_>, points: &[CurvePoint], plot: Rect, lo: f64, hi: f64) {
    let path = curve_line_path(points, plot, lo, hi);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(aurorix_render::skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(1.0);
    paint.set_color(ACCENT_INK.to_skia());
    kit.canvas().draw_path(&path, &paint);
}

fn draw_curve_ticks(kit: &CanvasKit<'_>, plot: Rect, lo: f64, hi: f64) {
    if hi <= lo {
        return;
    }
    let style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    for value in tick_values(lo, hi) {
        let y = tick_y(plot, value, lo, hi);
        rule(kit, plot.left(), plot.right(), y, RULE_INK);
        let (x, baseline) = curve_tick_at(y);
        sketch::text_right(
            kit,
            &style,
            &format!("{value:.1}"),
            x,
            baseline,
            NOTE_INK,
        );
    }
}

fn draw_curve_empty(kit: &CanvasKit<'_>, plot: Rect, count: usize) {
    rule(kit, plot.left(), plot.right(), plot.bottom(), RULE_INK);
    let title = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, LABEL_INK);
    let note = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
    let lines = if count == 1 {
        [String::from("只有 1 次记录"), String::from("再推一次就能连成曲线")]
    } else {
        [
            String::from(EMPTY_CURVE[0]),
            String::from(EMPTY_CURVE[1]),
        ]
    };
    let (x0, y0) = curve_empty_at(0);
    sketch::draw_text(kit, &title, &lines[0], x0, y0, LABEL_INK);
    let (x1, y1) = curve_empty_at(1);
    sketch::draw_text(kit, &note, &lines[1], x1, y1, PLACEHOLDER_INK);
}

fn draw_curve_peak(kit: &CanvasKit<'_>, points: &[CurvePoint], plot: Rect, lo: f64, hi: f64) {
    let Some(index) = peak_index(points) else {
        return;
    };
    let x = point_x(points, index, plot);
    let y = tick_y(plot, points[index].rks, lo, hi);
    kit.circle((x, y), TIGHT_GAP / 2.0, SCORE_INK);
    let (note_x, note_base) = curve_note_at();
    let style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let text = format!(
        "最高 RKS {} · {}",
        player::four_decimals(points[index].rks),
        date_text(points[index].at, true)
    );
    sketch::text_right(kit, &style, &text, note_x, note_base, NOTE_INK);
}

fn draw_curve_axis(kit: &CanvasKit<'_>, points: &[CurvePoint]) {
    let first = points[0].at;
    let last = points[points.len() - 1].at;
    let with_year = chrono::DateTime::from_timestamp(first, 0)
        .zip(chrono::DateTime::from_timestamp(last, 0))
        .map(|(a, b)| a.year() != b.year())
        .unwrap_or(false);
    let style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let (left_x, left_base) = curve_date_at();
    sketch::draw_text(
        kit,
        &style,
        &date_text(first, with_year),
        left_x,
        left_base,
        NOTE_INK,
    );
    let (right_x, right_base) = curve_date_end_at();
    sketch::text_right(
        kit,
        &style,
        &date_text(last, with_year),
        right_x,
        right_base,
        NOTE_INK,
    );
}

fn milestone_note(items: &[archive::Milestone]) -> String {
    if items.is_empty() {
        return String::new();
    }
    if items.len() <= MILESTONE_ROWS {
        return format!("共 {} 条", items.len());
    }
    format!("最近 {MILESTONE_ROWS} 条 · 共 {} 条", items.len())
}

fn milestone_ink(kind: archive::MilestoneKind) -> Color {
    match kind {
        archive::MilestoneKind::FirstPhi => PHI_INK,
        archive::MilestoneKind::FirstFc => MILESTONE_FC_INK,
        archive::MilestoneKind::Rks => ACCENT_INK,
        archive::MilestoneKind::Peak => SCORE_INK,
        archive::MilestoneKind::Recent => LABEL_INK,
        archive::MilestoneKind::Joined => PLACEHOLDER_INK,
    }
}

fn draw_curve_marks(
    kit: &CanvasKit<'_>,
    plot: Rect,
    items: &[archive::Milestone],
    points: &[CurvePoint],
) {
    if points.len() < 2 {
        return;
    }
    let start = points[0].at;
    let span = points[points.len() - 1].at - start;
    if span <= 0 {
        return;
    }
    for item in items {
        if item.at <= 0 {
            continue;
        }
        let x = plot.left() + (item.at - start) as f32 / span as f32 * plot.width();
        if x <= plot.left() || x >= plot.right() {
            continue;
        }
        let ink = milestone_ink(item.kind);
        let dash = Color::new(ink.r, ink.g, ink.b, MILESTONE_MARK_ALPHA);
        let mut y = plot.top();
        while y < plot.bottom() {
            fill_rect(kit, Rect::from_xywh(x, y, 1.0, 1.0), dash);
            y += TIGHT_GAP;
        }
        kit.circle((x, plot.bottom()), TIGHT_GAP / 8.0, ink);
    }
}

fn draw_milestones(kit: &CanvasKit<'_>, items: &[archive::Milestone]) {
    let label_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let (label_x, label_base) = milestone_label_at();
    sketch::draw_text(
        kit,
        &label_style,
        MILESTONE_TITLE,
        label_x,
        label_base,
        PLACEHOLDER_INK,
    );
    let note = milestone_note(items);
    if !note.is_empty() {
        sketch::text_right(
            kit,
            &label_style,
            &note,
            milestone_col().right(),
            label_base,
            NOTE_INK,
        );
    }
    let rows = archive::pick_milestones(items, MILESTONE_ROWS);
    if rows.is_empty() {
        let body = milestone_body();
        let style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
        sketch::draw_text(
            kit,
            &style,
            MILESTONE_EMPTY,
            body.left(),
            body.top() + TEXT_SIZE,
            PLACEHOLDER_INK,
        );
        return;
    }
    let count = rows.len();
    let first = milestone_dot_at(0, count);
    let last = milestone_dot_at(count - 1, count);
    let rail_x = milestone_col().left() + MILESTONE_RAIL / 2.0;
    let rail_top = first.top() + first.height() / 2.0;
    let rail_bottom = last.top() + last.height() / 2.0;
    fill_rect(
        kit,
        Rect::from_xywh(
            rail_x - MILESTONE_RAIL_W / 2.0,
            rail_top,
            MILESTONE_RAIL_W,
            rail_bottom - rail_top,
        ),
        RULE_INK,
    );
    let kind_style = sketch::text_style(sketch::FONT_DISPLAY, SUB_SIZE, SCORE_INK);
    let day_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    for (index, item) in rows.iter().enumerate() {
        let ink = milestone_ink(item.kind);
        let dot = milestone_dot_at(index, count);
        kit.circle(
            (
                dot.left() + dot.width() / 2.0,
                dot.top() + dot.height() / 2.0,
            ),
            MILESTONE_DOT / 2.0,
            ink,
        );
        let (kind_x, kind_base) = milestone_kind_at(index, count);
        let kind = item.kind.label();
        sketch::draw_text(kit, &kind_style, kind, kind_x, kind_base, ink);
        if !item.day.is_empty() {
            let (day_x, day_base) = milestone_day_at(index, count);
            sketch::draw_text(kit, &day_style, &item.day, day_x, day_base, NOTE_INK);
        }
        if !item.text.is_empty() {
            let value = item.text.trim_start_matches("RKS ");
            let kind_w = sketch::text_width(kit, &kind_style, kind).round();
            let value_x = kind_x + kind_w + TIGHT_GAP;
            let fitted =
                sketch::ellipsize(kit, &kind_style, value, milestone_col().right() - value_x);
            sketch::draw_text(kit, &kind_style, &fitted, value_x, kind_base, SCORE_INK);
        }
    }
}

fn draw_curve(
    kit: &CanvasKit<'_>,
    background: Option<&Image>,
    canvas: Rect,
    req: &OverviewRequest,
) {
    let sub = format!(
        "近 7 天 {}",
        delta_or(
            recent_delta(&req.curve, 7).unwrap_or(0.0),
            recent_delta(&req.curve, 7).is_some()
        )
    );
    draw_card(
        kit,
        background,
        canvas,
        curve_card(),
        CURVE_TITLE,
        &sub,
    );
    let plot = curve_plot();
    if req.curve.len() < 2 {
        draw_curve_empty(kit, plot, req.curve.len());
        draw_milestones(kit, &req.milestones);
        return;
    }
    let (lo, hi) = curve_bounds(&req.curve);
    draw_curve_ticks(kit, plot, lo, hi);
    kit.canvas()
        .save();
    kit.canvas()
        .clip_rect(plot, ClipOp::Intersect, true);
    draw_curve_area(kit, &req.curve, plot, lo, hi);
    draw_curve_line(kit, &req.curve, plot, lo, hi);
    kit.canvas().restore();
    let last = &req.curve[req.curve.len() - 1];
    kit.circle(
        (
            point_x(&req.curve, req.curve.len() - 1, plot),
            tick_y(plot, last.rks, lo, hi),
        ),
        (TIGHT_GAP / 4.0) / 2.0,
        ACCENT_INK,
    );
    draw_curve_peak(kit, &req.curve, plot, lo, hi);
    draw_curve_axis(kit, &req.curve);
    draw_curve_marks(kit, plot, &req.milestones, &req.curve);
    draw_milestones(kit, &req.milestones);
    let date_style = sketch::text_style(sketch::FONT_BODY, NOTE_SIZE, NOTE_INK);
    let (date_x, date_base) = curve_date_at();
    let hint = format!("{} 起", date_text(req.curve[0].at, true));
    sketch::draw_text(
        kit,
        &date_style,
        &hint,
        date_x,
        date_base + NOTE_SIZE + CURVE_STATS_DROP,
        NOTE_INK,
    );
}

fn draw_footer(kit: &CanvasKit<'_>, req: &OverviewRequest) {
    let style = sketch::text_style(sketch::FONT_BODY, TEXT_SIZE, PLACEHOLDER_INK);
    let synced = sync_text(req);
    if !synced.is_empty() {
        let (x, baseline) = footer_left_at();
        let text = format!("{SYNC_PREFIX}{synced}");
        sketch::draw_text(kit, &style, &text, x, baseline, PLACEHOLDER_INK);
    }
    let (right_x, right_base) = footer_right_at();
    sketch::text_right(
        kit,
        &style,
        SIGNATURE,
        right_x,
        right_base,
        PLACEHOLDER_INK,
    );
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &OverviewRequest) {
    let (_, raw_height) = canvas_size();
    let height = raw_height as f32;
    let canvas = Rect::from_xywh(0.0, 0.0, CANVAS_W, height);
    let background =
        super::songinfo::background_image(kit, root, &req.backdrop, CANVAS_W, height);
    match background.as_ref() {
        Some(image) => sketch::image_nearest(kit, image, canvas),
        None => {
            sketch::ph_fill(kit, canvas, super::songinfo::BASE_INK);
            sketch::ph_fill(kit, canvas, super::songinfo::PAGE_SHADE);
        }
    }
    draw_head(kit, req);
    draw_profile(kit, root, background.as_ref(), canvas, req);
    draw_rank(kit, root, background.as_ref(), canvas, req);
    draw_diffs(kit, background.as_ref(), canvas, req);
    draw_curve(kit, background.as_ref(), canvas, req);
    draw_footer(kit, req);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(left: f32, right: f32) -> bool {
        (left - right).abs() < 0.05
    }

    #[test]
    fn canvas_is_1440_by_1518() {
        assert_eq!(canvas_size(), (1440, 1518));
    }

    #[test]
    fn segments_stack_from_head_to_footer() {
        assert!(near(BLOCKS_TOP, 121.0));
        assert!(near(profile_card().top(), 121.0));
        assert!(near(profile_card().height(), 286.0));
        assert!(near(middle_top(), 431.0));
        assert!(near(rank_card().height(), 562.0));
        assert!(near(diff_card().left(), RIGHT_X));
        assert!(near(rank_card().bottom(), 993.0));
        assert!(near(curve_card().top(), 1017.0));
        assert!(near(rows_bottom(), 1435.6));
        assert!(near(footer_top(), 1467.6));
    }

    #[test]
    fn plot_and_columns_follow_the_curve_card() {
        let plot = curve_plot();
        assert!(near(plot.left(), 120.0));
        assert!(near(plot.width(), 844.0));
        assert!(near(plot.height(), CURVE_PLOT_H));
        assert!(near(milestone_col().left(), plot.right() + MILESTONE_COL_GAP));
        assert!(near(milestone_col().width(), MILESTONE_COL_W));
        let content = card_content(diff_card());
        assert!(near(diff_row_base(content, 0), content.top() + 30.23));
        assert!(near(diff_rule_y(content), content.top() + DIFF_RULE_Y));
        assert!(near(stats_rule_y(content), content.top() + COLLECT_RULE_Y));
    }

    #[test]
    fn rank_slots_and_stats_cells_fill_the_middle_row() {
        let content = card_content(rank_card());
        assert!(near(content.top(), middle_top() + CARD_PAD + TITLE_SIZE + TITLE_GAP));
        assert!(near(rank_number_at().1, content.top() + RANK_NUMBER_SIZE));
        assert!(near(rank_track().left(), content.left()));
        assert!(near(rank_track().height(), RANK_TRACK_H));
        let shares = rank_shares(&[false, false, true, false]);
        let first = rank_slot(&shares, 0);
        assert!(near(first.0, rank_rows_top()));
        let omitted = rank_slot(&shares, 2);
        assert!(near(omitted.1 - omitted.0, (first.1 - first.0) * 0.5));
        let last = rank_slot(&shares, 3);
        assert!(near(last.1, content.bottom()));
        assert!(near(rank_avatar_rect(&shares, 0).width(), RANK_AVATAR));
        assert!(near(rank_avatar_rect(&shares, 0).left(), content.left()));
        assert!(rank_name_left() > rank_badge_rect(&shares, 0).right());
        let cell = stats_cell(card_content(diff_card()), 5);
        assert!(cell.right() <= card_content(diff_card()).right() + 0.01);
        assert!(cell.bottom() <= card_content(diff_card()).bottom() + 0.01);
        assert!(near(profile_avatar(content_with_profile()).width(), AVATAR));
    }

    fn content_with_profile() -> Rect {
        card_content(profile_card())
    }

    #[test]
    fn curve_bounds_and_ticks_snap_to_the_data() {
        let points = [
            CurvePoint {
                at: 0,
                rks: 15.23,
            },
            CurvePoint {
                at: DAY_SECONDS * 9,
                rks: 15.62,
            },
        ];
        let (lo, hi) = curve_bounds(&points);
        assert_eq!(lo, 15.2);
        assert_eq!(hi, 15.8);
        assert_eq!(tick_values(lo, hi), [15.2, 15.5, 15.8]);
        assert!((recent_delta(&points, 7).unwrap_or(0.0) - 0.39).abs() < 1e-6);
        assert!(recent_delta(&points, 30).is_none());
        let stats = curve_stats(&points);
        assert_eq!(stats.days, 2);
        assert_eq!(stats.span_days, 10);
        assert_eq!(stats.streak, 1);
        assert_eq!(stats.max_gap, 9);
        assert!(stats.has_best);
    }

    #[test]
    fn point_x_spans_the_plot_by_time() {
        let plot = Rect::from_xywh(100.0, 0.0, 600.0, 200.0);
        let points = [
            CurvePoint {
                at: 0,
                rks: 15.0,
            },
            CurvePoint {
                at: 300,
                rks: 15.1,
            },
            CurvePoint {
                at: 600,
                rks: 15.2,
            },
        ];
        assert_eq!(point_x(&points, 0, plot), 100.0);
        assert_eq!(point_x(&points, 1, plot), 400.0);
        assert_eq!(point_x(&points, 2, plot), 700.0);
        assert_eq!(tick_y(plot, 15.2, 15.0, 15.2), 0.0);
        assert_eq!(peak_index(&points), Some(2));
        assert_eq!(low_index(&points), Some(0));
    }
}
