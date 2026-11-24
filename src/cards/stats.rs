use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::player::{self, Player};
use super::settlement::{self, Align, TextAt};
use super::{memo, sketch};

pub const STATS_BASE_W: f32 = 1920.0;

pub const STATS_SHRINK: f32 = 1.0;

pub const STATS_MARGIN_X: f32 = 72.0;

pub const STATS_CONTENT_RIGHT: f32 = STATS_BASE_W - STATS_MARGIN_X;

pub const STATS_CONTENT_W: f32 = STATS_BASE_W - 2.0 * STATS_MARGIN_X;

pub const STATS_HEAD_NAME: &str = "STATS";

pub const STATS_HEAD_TITLE_BASE: f32 = 84.0;

pub const STATS_HEAD_NOTE_BASE: f32 = 128.0;

pub const STATS_HEAD_TITLE_SIZE: f32 = 54.0;

pub const STATS_HEAD_NOTE_SIZE: f32 = 26.0;

pub const STATS_HEAD_BAR_GAP: f32 = 36.0;

pub const STATS_SUMMARY_Y0: f32 = 152.0;

pub const STATS_SUMMARY_H: f32 = 200.0;

pub const STATS_SUMMARY_PAD: f32 = 24.0;

pub const STATS_SUMMARY_BAR: f32 = 8.0;

pub const STATS_SUMMARY_COLS: i32 = 4;

pub const STATS_SUMMARY_SIZE: f32 = 38.0;

pub const STATS_SUMMARY_LABEL_SIZE: f32 = 19.0;

pub const STATS_SECTION_TOP: f32 = 40.0;

pub const STATS_SECTION_RULE_GAP: f32 = 12.0;

pub const STATS_SECTION_GAP: f32 = 28.0;

pub const STATS_SECTION_SIZE: f32 = 30.0;

pub const STATS_SECTION_TITLE: &str = "完成度";

pub const STATS_CONST_TITLE: &str = "定数细分";

pub const STATS_GRADE_TITLE: &str = "评级构成";

pub const STATS_BAND_HEAD_H: f32 = 40.0;

pub const STATS_BAND_ROW_H: f32 = 88.0;

pub const STATS_BAND_GAP: f32 = 10.0;

pub const STATS_BAND_BAR_H: f32 = 26.0;

pub const STATS_BAND_BAR_X0: f32 = 175.0;

pub const STATS_BAND_BAR_X1: f32 = 1060.0;

pub const STATS_BAND_NAME_X: f32 = 84.0;

pub const STATS_BAND_PLAYED: f32 = 1190.0;

pub const STATS_BAND_CLEAR: f32 = 1350.0;

pub const STATS_BAND_FC: f32 = 1490.0;

pub const STATS_BAND_PHI: f32 = 1630.0;

pub const STATS_BAND_ACC: f32 = 1848.0;

pub const STATS_BAND_ROW_RADIUS: f32 = 10.0;

pub const STATS_BAND_NAME_SIZE: f32 = 30.0;

pub const STATS_BAND_VALUE_SIZE: f32 = 28.0;

pub const STATS_BAND_HEAD_SIZE: f32 = 20.0;

pub const STATS_CONST_PLOT_H: f32 = 220.0;

pub const STATS_CONST_BAR_W: f32 = 96.0;

pub const STATS_CONST_BAR_GAP: f32 = 10.0;

pub const STATS_CONST_LABEL_GAP: f32 = 10.0;

pub const STATS_CONST_PANEL_PAD: f32 = 16.0;

pub const STATS_CONST_AXIS_SIZE: f32 = 19.0;

pub const STATS_CONST_VALUE_SIZE: f32 = 19.0;

pub const STATS_CONST_VALUE_H: f32 = COMP_BAR_GAP + STATS_CONST_VALUE_SIZE;

pub const STATS_CONST_LEGEND: &str = "柱高 谱面数 · 青色 已玩";

pub const STATS_GRADE_LINE_H: f32 = 52.0;

pub const STATS_GRADE_NAME_SIZE: f32 = 24.0;

pub const STATS_GRADE_VALUE_SIZE: f32 = 26.0;

pub const STATS_GRADE_STEP: f32 = 190.0;

pub const STATS_GRADE_LABEL_GAP: f32 = 10.0;

pub const STATS_SHORT_ROW_H: f32 = 60.0;

pub const STATS_SHORT_ROW_GAP: f32 = 8.0;

pub const STATS_SHORT_NOTE_H: f32 = 52.0;

pub const STATS_SHORT_TITLE_X: f32 = 84.0;

pub const STATS_SHORT_TITLE_W: f32 = 760.0;

pub const STATS_SHORT_DIFF_X: f32 = 1000.0;

pub const STATS_SHORT_CONST_X: f32 = 1130.0;

pub const STATS_SHORT_SCORE_X: f32 = 1350.0;

pub const STATS_SHORT_ACC_X: f32 = 1500.0;

pub const STATS_SHORT_GRADE_X: f32 = 1600.0;

pub const STATS_SHORT_GATE_X: f32 = 1848.0;

pub const STATS_SHORT_GRADE_SLOT: f32 = 34.0;

pub const STATS_SHORT_RADIUS: f32 = 8.0;

pub const STATS_SHORT_SIZE: f32 = 24.0;

pub const STATS_FOOTER_H: f32 = 170.0;

pub const STATS_FOOT_RULE_TOP: f32 = 50.0;

pub const STATS_FOOT_TEXT_TOP: f32 = 118.0;

pub const STATS_SIGN_SIZE: f32 = 19.0;

pub const STATS_BOTTOM_SCRIM_TOP: f32 = 16.0;

pub const STATS_BOTTOM_SCRIM_RAMP_H: f32 = 70.0;

pub const STATS_LONG_H: f32 = 1200.0;

pub const STATS_CONST_MAX: usize = 30;

pub const STATS_SHORT_MAX: usize = 20;

pub const COMP_BAR_GAP: f32 = 10.0;

pub const SHEET_CARD_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x26);

pub const CHAP_CARD_RADIUS: f32 = 14.0;

pub const STATS_HEAD_INK: Color = settlement::SCORE_INK;

pub const STATS_NOTE_INK: Color = settlement::LABEL_INK;

pub const STATS_SECTION_INK: Color = settlement::SCORE_INK;

pub const STATS_SECTION_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x28);

pub const STATS_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const STATS_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0C);

pub const STATS_SEG_UNCLEAR: Color = Color::new(0xFF, 0xFF, 0xFF, 0x33);

pub const STATS_SEG_CLEARED: Color = Color::new(0xFF, 0xFF, 0xFF, 0x70);

pub const STATS_SEG_FC: Color = Color::new(0xFF, 0xFF, 0xFF, 0xC4);

pub const STATS_SEG_PHI: Color = settlement::RANK_ACCENT;

pub const STATS_BAND_ROW_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0A);

pub const STATS_CONST_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const STATS_SHORT_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0E);

pub const STATS_BOTTOM_SCRIM: Color = Color::new(0x06, 0x07, 0x0B, 0xE6);

pub const B30_ACCENT_INK: Color = settlement::RANK_ACCENT;

pub const SIGN_SHADOW_INK: Color = Color::new(0x04, 0x05, 0x09, 0xB4);

pub const SIGN_SHADOW_OFFSET: f32 = 1.0;

pub const SIGN_TRACKING: f32 = 1.4;

pub const SIGNATURE_PREFIX: &str = "Generated By ";

pub const SIGNATURE_MARK: &str = "Okamikko";

pub const SEPARATOR: &str = " · ";

pub const DASH: &str = "—";

pub const FULL_MARK_TEXT: &str = "满分";

pub const GAP_PREFIX: &str = "差 ";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListStat {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsBand {
    pub name: String,
    pub total: i32,
    pub played: i32,
    pub cleared: i32,
    pub full_combo: i32,
    pub phi: i32,
    pub avg_acc: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsConst {
    pub constant: f64,
    pub total: i32,
    pub played: i32,
    pub avg_acc: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatsGrade {
    pub name: String,
    pub count: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsRow {
    pub song_id: String,
    pub title: String,
    pub diff: String,
    pub constant: f64,
    pub score: i32,
    pub acc: f64,
    pub grade: String,
    pub gap: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StatsRequest {
    pub player: Player,
    pub head: String,
    pub note: String,
    pub backdrop: String,
    pub stats: Vec<ListStat>,
    pub bands: Vec<StatsBand>,
    pub consts: Vec<StatsConst>,
    pub const_note: String,
    pub grades: Vec<StatsGrade>,
    pub short_title: String,
    pub short: Vec<StatsRow>,
    pub short_note: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Section {
    pub head: f32,
    pub rule_y: f32,
    pub top: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub w: f32,
    pub h: f32,
    pub summary: Rect,
    pub bands: Section,
    pub band_rows: Vec<Rect>,
    pub consts: Section,
    pub const_plot: Rect,
    pub const_panel: Rect,
    pub grades: Section,
    pub short: Section,
    pub short_rows: Vec<Rect>,
    pub last_bottom: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlanIn {
    pub bands: usize,
    pub consts: usize,
    pub grades: bool,
    pub short: usize,
    pub short_note: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstBar {
    pub value: f32,
    pub fill: f32,
    pub text: String,
}

pub const BAND_HEADS: [(f32, &str); 5] = [
    (STATS_BAND_PLAYED, "已玩/总"),
    (STATS_BAND_CLEAR, "通关"),
    (STATS_BAND_FC, "FC"),
    (STATS_BAND_PHI, "φ"),
    (STATS_BAND_ACC, "均 ACC"),
];

fn rows_in(n: usize, cols: usize) -> usize {
    if n == 0 || cols == 0 {
        return 0;
    }
    n.div_ceil(cols)
}

pub fn summary_rect() -> Rect {
    Rect::from_xywh(
        STATS_MARGIN_X,
        STATS_SUMMARY_Y0,
        STATS_CONTENT_W,
        STATS_SUMMARY_H,
    )
}

pub fn summary_content() -> Rect {
    let area = summary_rect();
    Rect::from_ltrb(
        area.left() + STATS_SUMMARY_PAD,
        area.top() + STATS_SUMMARY_PAD,
        area.right() - STATS_SUMMARY_PAD,
        area.bottom() - STATS_SUMMARY_PAD,
    )
}

pub fn summary_kpi_at(content: Rect, index: usize, count: usize) -> (TextAt, TextAt) {
    let empty = TextAt::left(0.0, 0.0, 0.0);
    if content.width() <= 0.0 || count == 0 {
        return (empty, empty);
    }
    let cols = STATS_SUMMARY_COLS as usize;
    let (col, row) = (index % cols, index / cols);
    let cell_w = content.width() / STATS_SUMMARY_COLS as f32;
    let center = content.left() + (col as f32 + 0.5) * cell_w;
    let rows = rows_in(count, cols).max(1) as f32;
    let row_h = content.height() / rows;
    let top = content.top() + row as f32 * row_h;
    let value = TextAt::center(center, top + row_h * 0.42, STATS_SUMMARY_SIZE)
        .with_font(sketch::FONT_DISPLAY);
    let label = TextAt::center(center, top + row_h * 0.72, STATS_SUMMARY_LABEL_SIZE);
    (value, label)
}

pub fn head_title_at() -> TextAt {
    TextAt::left(
        STATS_MARGIN_X,
        STATS_HEAD_TITLE_BASE,
        STATS_HEAD_TITLE_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn head_note_at() -> TextAt {
    TextAt::left(STATS_MARGIN_X, STATS_HEAD_NOTE_BASE, STATS_HEAD_NOTE_SIZE)
}

pub fn head_max_text_w() -> f32 {
    STATS_CONTENT_W
}

pub fn section_title_at(sec: Section) -> TextAt {
    TextAt::left(STATS_MARGIN_X, sec.head, STATS_SECTION_SIZE).with_font(sketch::FONT_DISPLAY)
}

pub fn section_rule(sec: Section) -> Rect {
    Rect::from_ltrb(
        STATS_MARGIN_X,
        sec.rule_y,
        STATS_CONTENT_RIGHT,
        sec.rule_y + 1.0,
    )
}

fn col_at(row: Rect, right: f32, size: f32, font: Option<&'static str>) -> TextAt {
    let at = TextAt::right(right, row.top() + row.height() / 2.0 + size * 0.36, size);
    match font {
        Some(key) => at.with_font(key),
        None => at,
    }
}

pub fn band_name_at(row: Rect) -> TextAt {
    TextAt::left(
        STATS_BAND_NAME_X,
        row.top() + row.height() / 2.0 + STATS_BAND_NAME_SIZE * 0.36,
        STATS_BAND_NAME_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn band_bar_rect(row: Rect) -> Rect {
    Rect::from_xywh(
        STATS_BAND_BAR_X0,
        row.top() + (row.height() - STATS_BAND_BAR_H) / 2.0,
        STATS_BAND_BAR_X1 - STATS_BAND_BAR_X0,
        STATS_BAND_BAR_H,
    )
}

pub fn band_played_at(row: Rect) -> TextAt {
    col_at(row, STATS_BAND_PLAYED, STATS_BAND_VALUE_SIZE, Some(sketch::FONT_DISPLAY))
}

pub fn band_clear_at(row: Rect) -> TextAt {
    col_at(row, STATS_BAND_CLEAR, STATS_BAND_VALUE_SIZE, None)
}

pub fn band_fc_at(row: Rect) -> TextAt {
    col_at(row, STATS_BAND_FC, STATS_BAND_VALUE_SIZE, None)
}

pub fn band_phi_at(row: Rect) -> TextAt {
    col_at(row, STATS_BAND_PHI, STATS_BAND_VALUE_SIZE, None)
}

pub fn band_acc_at(row: Rect) -> TextAt {
    col_at(row, STATS_BAND_ACC, STATS_BAND_VALUE_SIZE, None)
}

pub fn band_head_at(right: f32, top: f32) -> TextAt {
    TextAt::right(right, top + STATS_BAND_HEAD_SIZE, STATS_BAND_HEAD_SIZE)
}

pub fn const_label_at(plot: Rect, cell: Rect) -> TextAt {
    TextAt::center(
        cell.left() + cell.width() / 2.0,
        plot.bottom() + STATS_CONST_LABEL_GAP + STATS_CONST_AXIS_SIZE,
        STATS_CONST_AXIS_SIZE,
    )
}

pub fn grade_at(sec: Section, index: usize) -> TextAt {
    TextAt::left(
        STATS_MARGIN_X + index as f32 * STATS_GRADE_STEP,
        sec.top + STATS_GRADE_VALUE_SIZE,
        STATS_GRADE_NAME_SIZE,
    )
}

pub fn short_title_at(row: Rect) -> TextAt {
    TextAt::left(
        STATS_SHORT_TITLE_X,
        row.top() + row.height() / 2.0 + STATS_SHORT_SIZE * 0.36,
        STATS_SHORT_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn short_title_max_w() -> f32 {
    STATS_SHORT_TITLE_W
}

fn short_col_at(row: Rect, right: f32, font: Option<&'static str>) -> TextAt {
    col_at(row, right, STATS_SHORT_SIZE, font)
}

pub fn short_diff_at(row: Rect) -> TextAt {
    short_col_at(row, STATS_SHORT_DIFF_X, None)
}

pub fn short_const_at(row: Rect) -> TextAt {
    short_col_at(row, STATS_SHORT_CONST_X, None)
}

pub fn short_score_at(row: Rect) -> TextAt {
    short_col_at(row, STATS_SHORT_SCORE_X, Some(sketch::FONT_DISPLAY))
}

pub fn short_acc_at(row: Rect) -> TextAt {
    short_col_at(row, STATS_SHORT_ACC_X, None)
}

pub fn short_gate_at(row: Rect) -> TextAt {
    short_col_at(row, STATS_SHORT_GATE_X, None)
}

pub fn short_grade_rect(row: Rect) -> Rect {
    Rect::from_xywh(
        STATS_SHORT_GRADE_X - STATS_SHORT_GRADE_SLOT / 2.0,
        row.top() + (row.height() - STATS_SHORT_GRADE_SLOT) / 2.0,
        STATS_SHORT_GRADE_SLOT,
        STATS_SHORT_GRADE_SLOT,
    )
}

pub fn short_note_at(sec: Section) -> TextAt {
    TextAt::left(
        STATS_MARGIN_X,
        sec.top + STATS_SHORT_NOTE_H / 2.0 + STATS_SHORT_SIZE * 0.36,
        STATS_SHORT_SIZE,
    )
}

pub fn foot_rule(plan: &Plan) -> Rect {
    Rect::from_xywh(
        STATS_MARGIN_X,
        plan.last_bottom + STATS_FOOT_RULE_TOP,
        STATS_CONTENT_W,
        1.0,
    )
}

pub fn sign_at(plan: &Plan) -> TextAt {
    TextAt::left(
        STATS_MARGIN_X,
        plan.last_bottom + STATS_FOOT_TEXT_TOP,
        STATS_SIGN_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn is_long(plan: &Plan) -> bool {
    plan.h > STATS_LONG_H
}

fn section_from(y: f32) -> Section {
    let head = y + STATS_SECTION_TOP + STATS_SECTION_SIZE;
    let rule = head + STATS_SECTION_RULE_GAP;
    Section {
        head,
        rule_y: rule,
        top: rule + 1.0 + STATS_SECTION_GAP,
    }
}

fn row_rect(top: f32, index: usize, height: f32, gap: f32) -> Rect {
    Rect::from_xywh(
        STATS_MARGIN_X,
        top + index as f32 * (height + gap),
        STATS_CONTENT_W,
        height,
    )
}

fn rows_bottom(top: f32, count: usize, height: f32, gap: f32) -> f32 {
    if count == 0 {
        return top;
    }
    top + count as f32 * (height + gap) - gap
}

pub fn const_width(count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let avail = STATS_CONTENT_W - 2.0 * STATS_CONST_PANEL_PAD;
    let mut slot = avail / count as f32;
    let cap = STATS_CONST_BAR_W + STATS_CONST_BAR_GAP;
    if slot > cap {
        slot = cap;
    }
    if slot <= STATS_CONST_BAR_GAP {
        return 0.0;
    }
    slot * count as f32
}

pub fn const_panel(top: f32) -> Rect {
    Rect::from_xywh(
        STATS_MARGIN_X,
        top - STATS_CONST_PANEL_PAD - STATS_CONST_VALUE_H,
        STATS_CONTENT_W,
        STATS_CONST_PLOT_H
            + STATS_CONST_VALUE_H
            + STATS_CONST_LABEL_GAP
            + STATS_CONST_AXIS_SIZE
            + 2.0 * STATS_CONST_PANEL_PAD,
    )
}

pub fn const_plot(top: f32, count: usize) -> Rect {
    let width = const_width(count);
    if width <= 0.0 {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let panel = const_panel(top);
    Rect::from_xywh(
        panel.left() + (panel.width() - width) / 2.0,
        top,
        width,
        STATS_CONST_PLOT_H,
    )
}

pub fn plan_for(input: PlanIn) -> Plan {
    let summary = summary_rect();
    let mut y = summary.bottom();
    let mut plan = Plan {
        w: STATS_BASE_W,
        summary,
        ..Plan::default()
    };

    if input.bands > 0 {
        plan.bands = section_from(y);
        let top = plan.bands.top + STATS_BAND_HEAD_H;
        for index in 0..input.bands {
            plan.band_rows
                .push(row_rect(top, index, STATS_BAND_ROW_H, STATS_BAND_GAP));
        }
        y = rows_bottom(top, input.bands, STATS_BAND_ROW_H, STATS_BAND_GAP);
    }

    if input.consts > 0 {
        let mut sec = section_from(y);
        sec.top += STATS_CONST_VALUE_H + STATS_CONST_PANEL_PAD;
        let plot = const_plot(sec.top, input.consts);
        if plot.width() > 0.0 {
            plan.consts = sec;
            plan.const_plot = plot;
            plan.const_panel = const_panel(sec.top);
            y = plan.const_panel.bottom();
        }
    }

    if input.grades {
        plan.grades = section_from(y);
        y = plan.grades.top + STATS_GRADE_LINE_H;
    }

    if input.short > 0 || input.short_note {
        plan.short = section_from(y);
        if input.short > 0 {
            for index in 0..input.short {
                plan.short_rows.push(row_rect(
                    plan.short.top,
                    index,
                    STATS_SHORT_ROW_H,
                    STATS_SHORT_ROW_GAP,
                ));
            }
            y = rows_bottom(
                plan.short.top,
                input.short,
                STATS_SHORT_ROW_H,
                STATS_SHORT_ROW_GAP,
            );
        } else {
            y = plan.short.top + STATS_SHORT_NOTE_H;
        }
    }

    plan.last_bottom = y;
    plan.h = y + STATS_FOOTER_H;
    plan
}

pub fn plan(req: &StatsRequest) -> Plan {
    plan_for(PlanIn {
        bands: req.bands.len(),
        consts: req.consts.len().min(STATS_CONST_MAX),
        grades: !req.grades.is_empty(),
        short: req.short.len().min(STATS_SHORT_MAX),
        short_note: !req.short_title.is_empty() && req.short.is_empty(),
    })
}

pub fn canvas_size(req: &StatsRequest) -> (i32, i32) {
    let plan = plan(req);
    (plan.w.round() as i32, plan.h.round() as i32)
}

pub fn head_note(parts: &[&str]) -> String {
    let mut kept: Vec<&str> = Vec::with_capacity(parts.len());
    for part in parts {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            kept.push(trimmed);
        }
    }
    kept.join(SEPARATOR)
}

pub fn band_segments(band: &StatsBand) -> Vec<(f32, Color)> {
    let total = band.total.max(0);
    let phi = band.phi.max(0).min(total);
    let fc = band.full_combo.max(phi).min(total);
    let cleared = band.cleared.max(fc).min(total);
    let played = band.played.max(cleared).min(total);
    vec![
        ((total - played) as f32, STATS_TRACK_INK),
        ((played - cleared) as f32, STATS_SEG_UNCLEAR),
        ((cleared - fc) as f32, STATS_SEG_CLEARED),
        ((fc - phi) as f32, STATS_SEG_FC),
        (phi as f32, STATS_SEG_PHI),
    ]
}

pub fn band_acc_text(band: &StatsBand) -> String {
    if band.played <= 0 {
        return String::from(DASH);
    }
    settlement::acc_text(band.avg_acc)
}

pub fn const_bars(slots: &[StatsConst]) -> Vec<ConstBar> {
    let total = slots.iter().map(|slot| slot.total).max().unwrap_or(0);
    if total <= 0 {
        return Vec::new();
    }
    slots
        .iter()
        .map(|slot| ConstBar {
            value: slot.total as f32,
            fill: const_ratio(slot),
            text: format!("{}/{}", slot.played, slot.total),
        })
        .collect()
}

pub fn const_ratio(slot: &StatsConst) -> f32 {
    if slot.total <= 0 {
        return 0.0;
    }
    slot.played as f32 / slot.total as f32
}

pub fn gate_text(gap: i32) -> String {
    if gap <= 0 {
        return String::from(FULL_MARK_TEXT);
    }
    format!("{GAP_PREFIX}{}", sketch::group_digits(&gap.to_string()))
}

pub fn band_played_text(band: &StatsBand) -> String {
    format!("{}/{}", band.played, band.total)
}

fn fill_rect(kit: &CanvasKit<'_>, area: Rect, ink: Color) -> Rect {
    let frame = sketch::ph_rect(
        area.left(),
        area.top(),
        area.width().max(0.0),
        area.height().max(0.0),
    );
    if frame.width() <= 0.0 || frame.height() <= 0.0 {
        return frame;
    }
    sketch::ph_fill(kit, frame, ink);
    frame
}

fn fill_gradient_rows(kit: &CanvasKit<'_>, area: Rect, top: Color, bottom: Color) {
    let frame = sketch::ph_rect(
        area.left(),
        area.top(),
        area.width().max(0.0),
        area.height().max(0.0),
    );
    let height = frame.height() as i32;
    if height <= 0 {
        return;
    }
    let span = (height - 1).max(1) as f32;
    for row in 0..height {
        let t = row as f32 / span;
        let lerp = |a: u8, b: u8| -> u8 {
            (f32::from(a) + (f32::from(b) - f32::from(a)) * t) as u8
        };
        let ink = Color::new(
            lerp(top.r, bottom.r),
            lerp(top.g, bottom.g),
            lerp(top.b, bottom.b),
            lerp(top.a, bottom.a),
        );
        sketch::ph_fill(
            kit,
            Rect::from_ltrb(
                frame.left(),
                frame.top() + row as f32,
                frame.right(),
                frame.top() + row as f32 + 1.0,
            ),
            ink,
        );
    }
}

fn signature_line(kit: &CanvasKit<'_>, at: TextAt, tracking: f32, segments: &[&str]) {
    let base = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK)
        .with_letter_spacing(tracking);
    let shadow = base.clone().with_color(SIGN_SHADOW_INK);
    let mut widths = Vec::with_capacity(segments.len());
    let mut total = 0.0;
    for segment in segments {
        let width = sketch::text_width(kit, &base, segment);
        widths.push(width);
        total += width + tracking;
    }
    let start = match at.align {
        Align::Right => at.x - total,
        Align::Center => at.x - total / 2.0,
        Align::Left => at.x,
    };
    for (index, segment) in segments.iter().enumerate() {
        let x = start + widths.iter().take(index).sum::<f32>() + index as f32 * tracking;
        sketch::draw_text(
            kit,
            &shadow,
            segment,
            x + SIGN_SHADOW_OFFSET,
            at.baseline + SIGN_SHADOW_OFFSET,
            SIGN_SHADOW_INK,
        );
    }
    for (index, segment) in segments.iter().enumerate() {
        let x = start + widths.iter().take(index).sum::<f32>() + index as f32 * tracking;
        sketch::draw_text(kit, &base, segment, x, at.baseline, settlement::SCORE_INK);
    }
}

fn backdrop_cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str, area: Rect, tint: Color) {
    sketch::backdrop(
        kit,
        root,
        song_id,
        area,
        settlement::COVER_FALLBACK,
        tint,
        sketch::BACKDROP_BLUR,
    );
}

fn bottom_scrim(kit: &CanvasKit<'_>, canvas: Rect, last_bottom: f32) {
    let top = last_bottom + STATS_BOTTOM_SCRIM_TOP;
    let mut end = last_bottom + STATS_BOTTOM_SCRIM_TOP + STATS_BOTTOM_SCRIM_RAMP_H;
    if top >= canvas.bottom() {
        return;
    }
    if end > canvas.bottom() {
        end = canvas.bottom();
    }
    if end > top {
        let clear = Color::new(
            STATS_BOTTOM_SCRIM.r,
            STATS_BOTTOM_SCRIM.g,
            STATS_BOTTOM_SCRIM.b,
            0,
        );
        fill_gradient_rows(
            kit,
            Rect::from_ltrb(0.0, top, canvas.right(), end),
            clear,
            STATS_BOTTOM_SCRIM,
        );
    }
    if end < canvas.bottom() {
        fill_rect(
            kit,
            Rect::from_ltrb(0.0, end, canvas.right(), canvas.bottom()),
            STATS_BOTTOM_SCRIM,
        );
    }
}

fn head(kit: &CanvasKit<'_>, req: &StatsRequest) {
    let avail = head_avail(kit, req);
    if avail < 1.0 {
        return;
    }
    settlement::line(kit, head_title_at(), STATS_HEAD_NAME, STATS_HEAD_INK);
    let rest = head_note(&[&req.head, &req.note]);
    if rest.is_empty() {
        return;
    }
    let at = head_note_at();
    let style = sketch::text_style(at.font_key(), at.size, STATS_NOTE_INK);
    let cut = sketch::ellipsize(kit, &style, &rest, avail);
    settlement::line(kit, at, &cut, STATS_NOTE_INK);
}

fn player_left(kit: &CanvasKit<'_>, player: &Player) -> f32 {
    if !player.has_player {
        return STATS_CONTENT_RIGHT;
    }
    let width = player::nickname(kit, &player.name)
        .map(|(_, width)| width)
        .unwrap_or(0.0);
    player::panel_rect(width).left()
}

fn head_avail(kit: &CanvasKit<'_>, req: &StatsRequest) -> f32 {
    let full = head_max_text_w();
    if !req.player.has_player {
        return full;
    }
    let left = player_left(kit, &req.player);
    let room = left - STATS_MARGIN_X - STATS_HEAD_BAR_GAP;
    room.clamp(0.0, full)
}

fn section_head(kit: &CanvasKit<'_>, sec: Section, title: &str, note: &str) {
    if title.is_empty() {
        return;
    }
    settlement::line(kit, section_title_at(sec), title, STATS_SECTION_INK);
    fill_rect(kit, section_rule(sec), STATS_SECTION_RULE_INK);
    if note.is_empty() {
        return;
    }
    let at = TextAt::right(STATS_CONTENT_RIGHT, sec.head, STATS_HEAD_NOTE_SIZE);
    settlement::line(kit, at, note, STATS_NOTE_INK);
}

fn kpi_number(kit: &CanvasKit<'_>, at: TextAt, number: &str, ink: Color) -> f32 {
    if number.is_empty() || at.size <= 0.0 {
        return 0.0;
    }
    let width = settlement::measure(kit, at, number);
    let style = sketch::text_style(at.font_key(), at.size, ink);
    sketch::draw_text(
        kit,
        &style,
        number,
        at.x - (width / 2.0).round(),
        at.baseline,
        ink,
    );
    width
}

fn summary(kit: &CanvasKit<'_>, req: &StatsRequest) {
    if req.stats.is_empty() {
        return;
    }
    let area = summary_rect();
    card(kit, area, CHAP_CARD_RADIUS, STATS_SUMMARY_BAR);
    let content = summary_content();
    for (index, stat) in req.stats.iter().enumerate() {
        let (value, label) = summary_kpi_at(content, index, req.stats.len());
        kpi_number(kit, value, &stat.value, settlement::SCORE_INK);
        if !stat.label.is_empty() {
            settlement::line(kit, label, &stat.label, settlement::LABEL_INK);
        }
    }
}

fn card(kit: &CanvasKit<'_>, area: Rect, radius: f32, bar: f32) {
    let frame = sketch::ph_rect(
        area.left(),
        area.top(),
        area.width().max(0.0),
        area.height().max(0.0),
    );
    if frame.width() <= 0.0 || frame.height() <= 0.0 {
        return;
    }
    sketch::fill_round(kit, frame, radius, SHEET_CARD_INK);
    if bar <= 0.0 {
        return;
    }
    sketch::clip_round(kit, frame, radius, || {
        sketch::ph_fill(
            kit,
            Rect::from_xywh(frame.left(), frame.top(), bar, frame.height()),
            B30_ACCENT_INK,
        );
    });
}

fn stack(kit: &CanvasKit<'_>, area: Rect, segments: &[(f32, Color)]) {
    let mut total = 0.0;
    for (weight, _) in segments {
        if *weight > 0.0 {
            total += *weight;
        }
    }
    if total <= 0.0 || area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let mut acc = 0.0;
    let mut previous = 0.0;
    for (weight, ink) in segments {
        if *weight > 0.0 {
            acc += *weight;
        }
        let next = acc / total;
        if next > previous {
            let piece = Rect::from_ltrb(
                area.left() + previous * area.width(),
                area.top(),
                area.left() + next * area.width(),
                area.bottom(),
            );
            sketch::fill_skew(kit, piece, settlement::LEAN, *ink);
        }
        previous = next;
    }
}

fn band_row(kit: &CanvasKit<'_>, row: Rect, band: &StatsBand) {
    if row.width() <= 0.0 || row.height() <= 0.0 {
        return;
    }
    let frame = sketch::ph_rect(row.left(), row.top(), row.width(), row.height());
    sketch::fill_round(kit, frame, STATS_BAND_ROW_RADIUS, STATS_BAND_ROW_INK);
    settlement::line(kit, band_name_at(row), &band.name, settlement::SCORE_INK);
    stack(kit, band_bar_rect(row), &band_segments(band));
    let played = band_played_text(band);
    settlement::line(kit, band_played_at(row), &played, settlement::SCORE_INK);
    for (at, value) in [
        (band_clear_at(row), band.cleared),
        (band_fc_at(row), band.full_combo),
        (band_phi_at(row), band.phi),
    ] {
        settlement::line(kit, at, &value.to_string(), settlement::SCORE_INK);
    }
    let acc = band_acc_text(band);
    settlement::line(kit, band_acc_at(row), &acc, settlement::SCORE_INK);
}

fn bands(kit: &CanvasKit<'_>, req: &StatsRequest, plan: &Plan) {
    if req.bands.is_empty() || plan.band_rows.is_empty() {
        return;
    }
    section_head(kit, plan.bands, STATS_SECTION_TITLE, "");
    for (right, name) in BAND_HEADS {
        settlement::line(
            kit,
            band_head_at(right, plan.bands.top),
            name,
            STATS_NOTE_INK,
        );
    }
    for (index, band) in req.bands.iter().enumerate() {
        let Some(row) = plan.band_rows.get(index) else {
            break;
        };
        band_row(kit, *row, band);
    }
}

fn bar_height(value: f32, max: f32, height: f32) -> f32 {
    if !(value > 0.0 && max > 0.0) || height <= 0.0 {
        return 0.0;
    }
    if value >= max {
        return height;
    }
    height * value / max
}

fn bar_at(area: Rect, count: usize, index: usize, gap: f32) -> Rect {
    if count == 0 || index >= count {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let slot = area.width() / count as f32;
    let mut width = slot - gap;
    if width <= 0.0 {
        width = slot;
    }
    Rect::from_xywh(
        area.left() + index as f32 * slot + gap / 2.0,
        area.top(),
        width,
        area.height(),
    )
}

fn histogram(kit: &CanvasKit<'_>, plot: Rect, bars: &[ConstBar], gap: f32) {
    let frame = sketch::ph_rect(
        plot.left(),
        plot.top(),
        plot.width().max(0.0),
        plot.height().max(0.0),
    );
    if frame.width() <= 0.0 || frame.height() <= 0.0 || bars.is_empty() {
        return;
    }
    let axis_y = frame.bottom() - 1.0;
    fill_rect(
        kit,
        Rect::from_ltrb(frame.left(), axis_y, frame.right(), frame.bottom()),
        STATS_RULE_INK,
    );
    let max = bars.iter().map(|bar| bar.value).fold(0.0_f32, f32::max);
    let area_h = axis_y - frame.top();
    for (index, bar) in bars.iter().enumerate() {
        let cell = bar_at(plot, bars.len(), index, gap);
        if cell.width() <= 0.0 {
            continue;
        }
        let height = bar_height(bar.value, max, area_h);
        let top = axis_y - height;
        if height > 0.0 {
            fill_rect(
                kit,
                Rect::from_ltrb(cell.left(), top, cell.right(), axis_y),
                STATS_CONST_TRACK_INK,
            );
            let filled = bar_height(bar.fill, 1.0, height);
            if filled > 0.0 {
                fill_rect(
                    kit,
                    Rect::from_ltrb(cell.left(), axis_y - filled, cell.right(), axis_y),
                    STATS_SEG_PHI,
                );
            }
        }
        if bar.text.is_empty() {
            continue;
        }
        let at = TextAt::center(
            cell.left() + cell.width() / 2.0,
            top - COMP_BAR_GAP,
            STATS_CONST_VALUE_SIZE,
        );
        settlement::line(kit, at, &bar.text, settlement::SCORE_INK);
    }
}

fn consts(kit: &CanvasKit<'_>, req: &StatsRequest, plan: &Plan) {
    if req.consts.is_empty() || plan.const_plot.width() <= 0.0 {
        return;
    }
    let frame = sketch::ph_rect(
        plan.const_panel.left(),
        plan.const_panel.top(),
        plan.const_panel.width(),
        plan.const_panel.height(),
    );
    sketch::fill_round(kit, frame, CHAP_CARD_RADIUS, SHEET_CARD_INK);
    let mut note = String::from(STATS_CONST_LEGEND);
    if !req.const_note.is_empty() {
        note.push_str(SEPARATOR);
        note.push_str(&req.const_note);
    }
    section_head(kit, plan.consts, STATS_CONST_TITLE, &note);
    let plot = plan.const_plot;
    histogram(kit, plot, &const_bars(&req.consts), STATS_CONST_BAR_GAP);
    for (index, slot) in req.consts.iter().enumerate() {
        let cell = bar_at(plot, req.consts.len(), index, STATS_CONST_BAR_GAP);
        if cell.width() <= 0.0 {
            break;
        }
        settlement::line(
            kit,
            const_label_at(plot, cell),
            &settlement::const_text1(slot.constant),
            STATS_NOTE_INK,
        );
    }
}

fn grades(kit: &CanvasKit<'_>, req: &StatsRequest, plan: &Plan) {
    if req.grades.is_empty() {
        return;
    }
    section_head(kit, plan.grades, STATS_GRADE_TITLE, "");
    for (index, grade) in req.grades.iter().enumerate() {
        let at = grade_at(plan.grades, index);
        settlement::line(kit, at, &grade.name, settlement::LABEL_INK);
        let width = settlement::measure(kit, at, &grade.name);
        let value = TextAt {
            x: at.x + width.round() + STATS_GRADE_LABEL_GAP,
            size: STATS_GRADE_VALUE_SIZE,
            font: Some(sketch::FONT_DISPLAY),
            ..at
        };
        settlement::line(kit, value, &grade.count.to_string(), settlement::SCORE_INK);
    }
}

fn grade_sprite(kit: &CanvasKit<'_>, root: &Path, row: Rect, grade: &str) {
    let Some(name) = memo::grade_name(grade) else {
        return;
    };
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let Some(ink) = sketch::content_bounds(image) else {
        return;
    };
    let slot = short_grade_rect(row);
    if slot.width() <= 0.0 || slot.height() <= 0.0 {
        return;
    }
    let frame = Rect::from_xywh(0.0, 0.0, slot.width(), slot.height());
    let fitted = aurorix_render::contain_destination_rect(image, frame);
    let scale = fitted.width() / (image.width() as f32).max(1.0);
    let x = (slot.left() + slot.width() / 2.0 - (ink.left() + ink.right()) * scale / 2.0).round();
    let y = (slot.top() + slot.height() / 2.0 - (ink.top() + ink.bottom()) * scale / 2.0).round();
    let dest = Rect::from_xywh(x, y, fitted.width(), fitted.height());
    sketch::image_fill(kit, image, dest);
}

fn short_row(kit: &CanvasKit<'_>, root: &Path, row: Rect, item: &StatsRow, zebra: bool) {
    if row.width() <= 0.0 || row.height() <= 0.0 {
        return;
    }
    let frame = sketch::ph_rect(row.left(), row.top(), row.width(), row.height());
    if zebra {
        sketch::fill_round(kit, frame, STATS_SHORT_RADIUS, STATS_SHORT_RULE_INK);
    }
    if !item.title.is_empty() {
        let at = short_title_at(row);
        let style = sketch::text_style(at.font_key(), at.size, settlement::TITLE_INK);
        let cut = sketch::ellipsize(kit, &style, &item.title, short_title_max_w());
        settlement::line(kit, at, &cut, settlement::TITLE_INK);
    }
    settlement::line(kit, short_diff_at(row), &item.diff, settlement::LABEL_INK);
    settlement::line(
        kit,
        short_const_at(row),
        &settlement::const_text(item.constant),
        settlement::LABEL_INK,
    );
    settlement::line(
        kit,
        short_score_at(row),
        &sketch::group_digits(&settlement::score_text(item.score)),
        settlement::SCORE_INK,
    );
    settlement::line(
        kit,
        short_acc_at(row),
        &settlement::acc_text(item.acc),
        settlement::SCORE_INK,
    );
    grade_sprite(kit, root, row, &item.grade);
    settlement::line(
        kit,
        short_gate_at(row),
        &gate_text(item.gap),
        STATS_NOTE_INK,
    );
}

fn short(kit: &CanvasKit<'_>, root: &Path, req: &StatsRequest, plan: &Plan) {
    if req.short_title.is_empty() {
        return;
    }
    section_head(kit, plan.short, &req.short_title, "");
    if req.short.is_empty() {
        settlement::line(kit, short_note_at(plan.short), &req.short_note, STATS_NOTE_INK);
        return;
    }
    for (index, item) in req.short.iter().enumerate() {
        if index >= plan.short_rows.len() {
            break;
        }
        short_row(kit, root, plan.short_rows[index], item, index % 2 == 1);
    }
}
fn foot(kit: &CanvasKit<'_>, plan: &Plan) {
    let rule = foot_rule(plan);
    fill_rect(
        kit,
        Rect::from_xywh(
            rule.left().round(),
            rule.top().round(),
            rule.width(),
            rule.height(),
        ),
        STATS_RULE_INK,
    );
    signature_line(
        kit,
        sign_at(plan),
        SIGN_TRACKING,
        &[SIGNATURE_PREFIX, SIGNATURE_MARK],
    );
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &StatsRequest) {
    let plan = plan(req);
    let canvas = Rect::from_xywh(0.0, 0.0, STATS_BASE_W, plan.h);
    backdrop_cover(kit, root, &req.backdrop, canvas, sketch::BACKDROP_TINT);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SCRIM);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SHEET_SCRIM);
    bottom_scrim(kit, canvas, plan.last_bottom);
    head(kit, req);
    player::draw(kit, root, STATS_BASE_W, 1.0, 0.0, &req.player);
    summary(kit, req);
    bands(kit, req, &plan);
    consts(kit, req, &plan);
    grades(kit, req, &plan);
    short(kit, root, req, &plan);
    foot(kit, &plan);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Instant;

    use aurorix_render::skia_safe::{Data, Image};
    use aurorix_render::{RenderContext, RenderRequest, SkiaBackend};

    use super::*;

    const OUT_DIR: &str = "/tmp/phigros-verify";

    const OUT_STATS: &str = "/tmp/phigros-verify/12-stats.png";

    fn request() -> StatsRequest {
        let bands: Vec<StatsBand> = ["EZ", "HD", "IN", "AT"]
            .iter()
            .enumerate()
            .map(|(index, name)| StatsBand {
                name: String::from(*name),
                total: 120 + index as i32 * 8,
                played: 96 + index as i32 * 6,
                cleared: 88 + index as i32 * 5,
                full_combo: 64 + index as i32 * 4,
                phi: 12 + index as i32,
                avg_acc: 97.42,
            })
            .collect();
        let consts: Vec<StatsConst> = (0..12)
            .map(|index| StatsConst {
                constant: 12.0 + index as f64 * 0.5,
                total: 40 + index * 3,
                played: 22 + index * 2,
                avg_acc: 96.18,
            })
            .collect();
        let short: Vec<StatsRow> = (0..6)
            .map(|index| StatsRow {
                song_id: String::from("Spasmodic.姜米條颶風元力上人.0"),
                title: String::from("Spasmodic"),
                diff: String::from("IN"),
                constant: 15.7,
                score: 952_000 + index * 1100,
                acc: 98.42,
                grade: String::from("V"),
                gap: 48_000 - index * 700,
            })
            .collect();
        StatsRequest {
            player: Player {
                has_player: true,
                name: String::from("Okamikko"),
                avatar: String::new(),
                rank: 213,
                rank_level: 4,
                rks: 15.6234,
            },
            head: String::from("15.0-15.9 · IN"),
            note: String::from("命中 42 条 · 按定数从高到低"),
            backdrop: String::from("Spasmodic.姜米條颶風元力上人.0"),
            stats: vec![
                ListStat {
                    value: String::from("128"),
                    label: String::from("谱面"),
                },
                ListStat {
                    value: String::from("96"),
                    label: String::from("已玩"),
                },
                ListStat {
                    value: String::from("15.6234"),
                    label: String::from("平均 RKS"),
                },
                ListStat {
                    value: String::from("98.42%"),
                    label: String::from("平均 ACC"),
                },
            ],
            bands,
            consts,
            const_note: String::from("已截到最高 12 档"),
            grades: vec![
                StatsGrade {
                    name: String::from("φ"),
                    count: 12,
                },
                StatsGrade {
                    name: String::from("FC"),
                    count: 30,
                },
                StatsGrade {
                    name: String::from("V"),
                    count: 41,
                },
            ],
            short_title: String::from("短板清单"),
            short,
            short_note: String::from("没有条目"),
        }
    }

    #[test]
    #[ignore]
    fn render_stats() {
        let root = crate::assets_root();
        let _ = crate::cards::register_fonts(&root);
        let assets = Arc::new(crate::assets_in(&root));
        let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
            .with_asset_loader(assets.loader());
        let req = request();
        let (width, height) = canvas_size(&req);
        let request = RenderRequest::new(width, height);
        let start = Instant::now();
        let bytes = context
            .draw(request, |canvas, ctx| {
                let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
                draw(&kit, &root, &req);
            })
            .expect("渲染失败")
            .to_vec();
        let elapsed = start.elapsed();
        std::fs::create_dir_all(OUT_DIR).expect("建出图目录");
        std::fs::write(OUT_STATS, &bytes).expect("写出 PNG");
        let image = Image::from_encoded(Data::new_copy(&bytes)).expect("解码 PNG");
        println!(
            "stats 渲染耗时 {} ms ({} 字节)",
            elapsed.as_millis(),
            bytes.len()
        );
        assert_eq!(
            (image.width(), image.height()),
            (width, height),
            "PNG 尺寸必须等于 canvas_size {width}x{height}"
        );
        assert!(bytes.len() > 1000, "字节数 {} 必须大于 1000", bytes.len());
    }
}
