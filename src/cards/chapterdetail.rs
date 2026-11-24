use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::settlement::{self, TextAt};
use super::{memo, player, sketch};

pub const MID_RATIO: f32 = settlement::MID_RATIO;

pub const CHAP_W: f32 = 1046.0;

pub const CHAP_X0: f32 = 36.0;

pub const CHAP_GRADE_COLS: usize = 9;

pub const CHAP_PLAYER_K: f32 = 0.82;

pub const CHAP_CANVAS_W: i32 = 1046;

pub const CHAP_MARGIN_X: f32 = CHAP_X0;

pub const CHAP_RIGHT: f32 = CHAP_W - CHAP_X0;

pub const CHAP_CONTENT: f32 = CHAP_RIGHT - CHAP_MARGIN_X;

pub const CHAP_TITLE_TOP: f32 = 32.0;

pub const CHAP_TITLE_SIZE: f32 = 52.0;

pub const CHAP_NOTE_SIZE: f32 = 18.0;

pub const CHAP_TITLE_GAP: f32 = 12.0;

pub const CHAP_HEAD_H: f32 = CHAP_TITLE_SIZE + CHAP_TITLE_GAP + CHAP_NOTE_SIZE;

pub const CHAP_CARD_TOP: f32 = CHAP_TITLE_TOP + CHAP_HEAD_H + 24.0;

pub const CHAP_PAD: f32 = 40.0;

pub const CHAP_SUM_H: f32 = 226.0;

pub const CHAP_KPI_W: f32 = 298.0;

pub const CHAP_AXIS_DROP: f32 = 155.0;

pub const CHAP_BAR_MAX: f32 = 44.0;

pub const CHAP_BAR_W: f32 = 34.0;

pub const CHAP_SECTION_GAP: f32 = 26.0;

pub const CHAP_HEAD_H2: f32 = 40.0;

pub const CHAP_ROW_H: f32 = 84.0;

pub const CHAP_TITLE_W: f32 = 220.0;

pub const CHAP_NUM_COL: f32 = 125.0;

pub const CHAP_CHIP_H: f32 = 34.0;

pub const CHAP_CHIP_PAD_X: f32 = 12.0;

pub const CHAP_BEST_W: f32 = 100.0;

pub const CHAP_COVER: f32 = 40.0;

pub const CHAP_COVER_R: f32 = 8.0;

pub const CHAP_COVER_GAP: f32 = 14.0;

pub const CHAP_TOTAL_GAP: f32 = 14.0;

pub const CHAP_TOTAL_H: f32 = 52.0;

pub const CHAP_TAIL_PAD: f32 = 40.0;

pub const CHAP_BOTTOM: f32 = 62.0;

pub const CHAP_KPI_SIZE: f32 = 44.0;

pub const CHAP_KPI_SUF: f32 = 20.0;

pub const CHAP_LABEL_SIZE: f32 = 14.0;

pub const CHAP_AXIS_SIZE: f32 = 14.0;

pub const CHAP_HEAD_NAME: f32 = 13.0;

pub const CHAP_ROW_TITLE: f32 = 16.0;

pub const CHAP_ROW_SUB: f32 = 12.5;

pub const CHAP_CELL_SCORE: f32 = 15.0;

pub const CHAP_CELL_CONST: f32 = 12.0;

pub const CHAP_CELL_STAT: f32 = 10.5;

pub const CHAP_BEST_SIZE: f32 = 16.0;

pub const CHAP_TOTAL_VAL: f32 = 16.0;

pub const CHAP_TOTAL_SUB: f32 = 11.5;

pub const CHAP_CARD_RADIUS: f32 = 14.0;

pub const CHAP_KPI_COLS: usize = 3;

pub const CHAP_KPI_BASE_DROP: f32 = 56.0;

pub const CHAP_KPI_LABEL_DROP: f32 = 22.0;

pub const CHAP_CHART_DROP: f32 = 96.0;

pub const CHAP_CHART_H: f32 = 130.0;

pub const CHAP_BAR_RADIUS: f32 = 3.0;

pub const CHAP_BAR_STUB_H: f32 = 2.0;

pub const CHAP_BAR_STUB_RADIUS: f32 = 1.0;

pub const CHAP_BAR_COUNT_DROP: f32 = 8.0;

pub const CHAP_BAR_LABEL_DROP: f32 = 22.0;

pub const CHAP_HEAD_TEXT_RATIO: f32 = 0.36;

pub const CHAP_ROW_LINE1: f32 = 37.5;

pub const CHAP_ROW_LINE2: f32 = 55.5;

pub const CHAP_GRADE_SIDE: f32 = 15.0;

pub const CHAP_GRADE_GAP: f32 = 10.0;

pub const CHAP_CELL_INSET: f32 = 12.0;

pub const CHAP_HEAD_BEST_INSET: f32 = 12.0;

pub const CHAP_BEST_INSET: f32 = 14.0;

pub const CHAP_DIFF_GAP: f32 = 20.0;

pub const CHAP_ROWS_TOP_DROP: f32 = 8.0;

pub const CHAP_TOTAL_LABEL_DROP: f32 = 20.0;

pub const CHAP_TOTAL_ACC_DROP: f32 = 40.0;

pub const CHAP_TOTAL_FC_DROP: f32 = 19.0;

pub const CHAP_FOOTER_DROP: f32 = 40.0;

pub const SHEET_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0E);

pub const SUG_PHI_INK: Color = Color::new(0xF0, 0xC7, 0x5E, 0xFF);

pub const RANK_ACCENT: Color = settlement::RANK_ACCENT;

pub const OV_NOTE_INK: Color = settlement::NOTE_INK;

pub const CHAP_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1E);

pub const CHAP_HEAD_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x24);

pub const CHAP_AXIS_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x26);

pub const CHAP_ZERO_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x3C);

pub const CHAP_BAR_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x18);

pub const CHAP_RULE_THICK: f32 = 1.0;

pub const CHAP_HEAD_TITLE: &str = "曲目";

pub const CHAP_HEAD_BEST: &str = "Best ACC";

pub const CHAP_TOTAL_LABEL: &str = "合计";

pub const CHAP_KPI_PLAYED: &str = "已打谱面";

pub const CHAP_KPI_ACC: &str = "平均 ACC";

pub const CHAP_KPI_SONGS: &str = "曲目";

pub const CHAP_KPI_TOTAL_SEP: &str = "/ ";

pub const CHAP_KPI_SONG_SUFFIX: &str = " 首";

pub const CHAP_FC_SUFFIX: &str = " FC";

pub const CHAP_PLAYED_SEP: &str = " / ";

pub const CHAP_CELL_SEP: &str = " · ";

pub const CHAP_DASH: &str = settlement::MISSING_TEXT;

pub const CHAP_FOOTER: &str = "数据来自你上次拉取的存档，/p pull 可更新";

pub const CHAP_GRADE_LABELS: [&str; CHAP_GRADE_COLS] =
    ["φ", "FC", "V", "S", "A", "B", "C", "F", "未打"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterDetailLevel {
    pub diff: String,
    pub played: usize,
    pub total: usize,
    pub acc: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterDetailCell {
    pub constant: f64,
    pub grade: String,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterDetailSong {
    pub song_id: String,
    pub title: String,
    pub sub: String,
    pub cells: Vec<ChapterDetailCell>,
    pub best_acc: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChapterDetailRequest {
    pub title: String,
    pub note: String,
    pub player: player::Player,
    pub backdrop: String,
    pub banner: String,
    pub total: usize,
    pub played: usize,
    pub full_combo: usize,
    pub phi: usize,
    pub avg_acc: f64,
    pub grades: [usize; CHAP_GRADE_COLS],
    pub levels: Vec<ChapterDetailLevel>,
    pub songs: Vec<ChapterDetailSong>,
}

pub fn chap_inner() -> Rect {
    Rect::from_xywh(
        CHAP_MARGIN_X + CHAP_PAD,
        CHAP_CARD_TOP,
        CHAP_CONTENT - 2.0 * CHAP_PAD,
        0.0,
    )
}

pub fn chap_sum_top() -> f32 {
    CHAP_CARD_TOP + CHAP_PAD
}

pub fn chap_kpi_at(index: usize) -> (TextAt, TextAt) {
    let inner = chap_inner();
    let x = inner.left() + (index as f32 + 0.5) * CHAP_KPI_W;
    let base = chap_sum_top() + CHAP_KPI_BASE_DROP;
    let value = TextAt::center(x, base, CHAP_KPI_SIZE).with_font(sketch::FONT_DISPLAY);
    let label = TextAt::center(x, base + CHAP_KPI_LABEL_DROP, CHAP_LABEL_SIZE);
    (value, label)
}

pub fn chap_chart_top() -> f32 {
    chap_sum_top() + CHAP_CHART_DROP
}

pub fn chap_chart_rect() -> Rect {
    let inner = chap_inner();
    Rect::from_xywh(inner.left(), chap_chart_top(), inner.width(), CHAP_CHART_H)
}

pub fn chap_chart_col(index: usize) -> Rect {
    let area = chap_chart_rect();
    let width = area.width() / CHAP_GRADE_COLS as f32;
    Rect::from_xywh(
        area.left() + index as f32 * width,
        area.top(),
        width,
        area.height(),
    )
}

pub fn chap_axis_y() -> f32 {
    chap_sum_top() + CHAP_AXIS_DROP
}

pub fn chap_bar_track(index: usize) -> Rect {
    let col = chap_chart_col(index);
    Rect::from_xywh(
        col.left() + (col.width() - CHAP_BAR_W) / 2.0,
        chap_axis_y() - CHAP_BAR_MAX,
        CHAP_BAR_W,
        CHAP_BAR_MAX,
    )
}

pub fn chap_bar_rect(index: usize, count: usize, peak: usize) -> Rect {
    let h = if count > 0 && peak > 0 {
        CHAP_BAR_MAX * count as f32 / peak as f32
    } else {
        0.0
    };
    let col = chap_chart_col(index);
    Rect::from_xywh(
        col.left() + (col.width() - CHAP_BAR_W) / 2.0,
        chap_axis_y() - h,
        CHAP_BAR_W,
        h,
    )
}

pub fn chap_bar_count_at(index: usize) -> TextAt {
    let col = chap_chart_col(index);
    TextAt::center(
        col.left() + col.width() / 2.0,
        chap_axis_y() - CHAP_BAR_MAX - CHAP_BAR_COUNT_DROP,
        CHAP_AXIS_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_bar_label_at(index: usize) -> TextAt {
    let col = chap_chart_col(index);
    TextAt::center(
        col.left() + col.width() / 2.0,
        chap_axis_y() + CHAP_BAR_LABEL_DROP,
        CHAP_AXIS_SIZE,
    )
}

pub fn chap_table_top() -> f32 {
    chap_sum_top() + CHAP_SUM_H + CHAP_SECTION_GAP
}

pub fn chap_table_rule_y() -> f32 {
    chap_sum_top() + CHAP_SUM_H + CHAP_SECTION_GAP / 2.0
}

pub fn chap_head_rule_y() -> f32 {
    chap_table_top() + CHAP_HEAD_H2
}

pub fn chap_head_base() -> f32 {
    let top = chap_table_rule_y();
    let bottom = chap_head_rule_y();
    top + (bottom - top) / 2.0 + CHAP_HEAD_NAME * CHAP_HEAD_TEXT_RATIO
}

pub fn chap_head_title_at() -> TextAt {
    TextAt::left(chap_inner().left(), chap_head_base(), CHAP_HEAD_NAME)
}

pub fn chap_head_best_at() -> TextAt {
    TextAt::right(
        chap_inner().right() - CHAP_HEAD_BEST_INSET,
        chap_head_base(),
        CHAP_HEAD_NAME,
    )
}

pub fn chap_head_diff_at(cols: usize, index: usize) -> TextAt {
    TextAt::right(chap_cell_right(cols, index), chap_head_base(), CHAP_HEAD_NAME)
        .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_cell_right(cols: usize, index: usize) -> f32 {
    chap_diff_right(cols, index) - CHAP_CELL_INSET
}

pub fn chap_rows_top() -> f32 {
    chap_head_rule_y() + CHAP_ROWS_TOP_DROP
}

pub fn chap_row_top(index: usize) -> f32 {
    chap_rows_top() + index as f32 * CHAP_ROW_H
}

pub fn chap_row_rule_y(index: usize) -> f32 {
    chap_row_top(index) + CHAP_ROW_H
}

pub fn chap_total_top(rows: usize) -> f32 {
    chap_rows_top() + rows as f32 * CHAP_ROW_H + CHAP_TOTAL_GAP
}

pub fn chap_total_rule_y(rows: usize) -> f32 {
    chap_rows_top() + rows as f32 * CHAP_ROW_H + CHAP_TOTAL_GAP / 2.0
}

pub fn chap_total_label_at(rows: usize) -> TextAt {
    TextAt::left(
        chap_title_x(),
        chap_total_top(rows) + CHAP_TOTAL_H / 2.0 + CHAP_HEAD_NAME * CHAP_HEAD_TEXT_RATIO,
        CHAP_HEAD_NAME,
    )
}

pub fn chap_diff_left(_cols: usize, index: usize) -> f32 {
    chap_title_x() + CHAP_TITLE_W + CHAP_DIFF_GAP + index as f32 * CHAP_NUM_COL
}

pub fn chap_diff_right(cols: usize, index: usize) -> f32 {
    chap_diff_left(cols, index) + CHAP_NUM_COL
}

pub fn chap_title_x() -> f32 {
    chap_inner().left() + CHAP_COVER + CHAP_COVER_GAP
}

pub fn chap_title_width(_cols: usize) -> f32 {
    CHAP_TITLE_W
}

pub fn chap_cover_rect(index: usize) -> Rect {
    Rect::from_xywh(
        chap_inner().left(),
        chap_row_top(index) + (CHAP_ROW_H - CHAP_COVER) / 2.0,
        CHAP_COVER,
        CHAP_COVER,
    )
}

pub fn chap_row_title_at(index: usize) -> TextAt {
    TextAt::left(
        chap_title_x(),
        chap_row_top(index) + CHAP_ROW_LINE1,
        CHAP_ROW_TITLE,
    )
}

pub fn chap_row_sub_at(index: usize) -> TextAt {
    TextAt::left(
        chap_title_x(),
        chap_row_top(index) + CHAP_ROW_LINE2,
        CHAP_ROW_SUB,
    )
}

pub fn chap_best_at(_cols: usize, index: usize) -> TextAt {
    TextAt::right(
        chap_inner().right() - CHAP_BEST_INSET,
        chap_row_top(index) + CHAP_ROW_H / 2.0 + CHAP_BEST_SIZE * MID_RATIO,
        CHAP_BEST_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_cell_score_at(cols: usize, index: usize, row: usize) -> TextAt {
    TextAt::right(
        chap_cell_right(cols, index) - CHAP_GRADE_SIDE - CHAP_GRADE_GAP,
        chap_row_top(row) + CHAP_ROW_LINE1,
        CHAP_CELL_SCORE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_grade_box(cols: usize, index: usize, row: usize) -> Rect {
    Rect::from_xywh(
        chap_cell_right(cols, index) - CHAP_GRADE_SIDE,
        chap_row_top(row) + CHAP_ROW_LINE1
            - CHAP_CELL_SCORE * MID_RATIO
            - CHAP_GRADE_SIDE / 2.0,
        CHAP_GRADE_SIDE,
        CHAP_GRADE_SIDE,
    )
}

pub fn chap_cell_stat_at(cols: usize, index: usize, row: usize) -> TextAt {
    TextAt::right(
        chap_cell_right(cols, index),
        chap_row_top(row) + CHAP_ROW_LINE2,
        CHAP_CELL_STAT,
    )
}

pub fn chap_cell_none_at(cols: usize, index: usize, row: usize) -> TextAt {
    TextAt::right(
        chap_cell_right(cols, index),
        chap_row_top(row) + CHAP_ROW_LINE1,
        CHAP_CELL_CONST,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_total_count_at(cols: usize, index: usize, rows: usize) -> TextAt {
    TextAt::right(
        chap_cell_right(cols, index),
        chap_total_top(rows) + CHAP_TOTAL_LABEL_DROP,
        CHAP_TOTAL_VAL,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_total_acc_at(cols: usize, index: usize, rows: usize) -> TextAt {
    TextAt::right(
        chap_cell_right(cols, index),
        chap_total_top(rows) + CHAP_TOTAL_ACC_DROP,
        CHAP_TOTAL_SUB,
    )
}

pub fn chap_total_best_label_at(rows: usize) -> TextAt {
    TextAt::right(
        chap_inner().right(),
        chap_total_top(rows) + CHAP_TOTAL_FC_DROP,
        CHAP_TOTAL_SUB,
    )
}

pub fn chap_total_best_at(rows: usize) -> TextAt {
    TextAt::right(
        chap_inner().right(),
        chap_total_top(rows) + CHAP_TOTAL_ACC_DROP,
        CHAP_TOTAL_VAL,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn chap_card(rows: usize) -> Rect {
    let h = chap_total_top(rows) + CHAP_TOTAL_H - CHAP_CARD_TOP + CHAP_TAIL_PAD;
    Rect::from_xywh(CHAP_MARGIN_X, CHAP_CARD_TOP, CHAP_CONTENT, h)
}

pub fn chap_height(rows: usize) -> f32 {
    chap_card(rows).bottom() + CHAP_BOTTOM
}

pub fn chap_footer_at(rows: usize) -> TextAt {
    TextAt::left(
        chap_inner().left(),
        chap_card(rows).bottom() + CHAP_FOOTER_DROP,
        CHAP_ROW_SUB,
    )
}

pub fn grade_ink(index: usize) -> Color {
    match index {
        0 => SUG_PHI_INK,
        1 => RANK_ACCENT,
        2 | 3 => Color::new(0xF2, 0xF4, 0xF8, 0xE8),
        4 | 5 => Color::new(0xC6, 0xCC, 0xD8, 0x9E),
        6 | 7 => Color::new(0xC6, 0xCC, 0xD8, 0x62),
        _ => Color::new(0xFF, 0xFF, 0xFF, 0x2E),
    }
}

pub fn columns(req: &ChapterDetailRequest) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(req.levels.len());
    for (index, level) in req.levels.iter().enumerate() {
        if level.total > 0 {
            out.push(index);
        }
    }
    out
}

pub fn canvas_size(req: &ChapterDetailRequest) -> (i32, i32) {
    (CHAP_CANVAS_W, chap_height(req.songs.len()).round() as i32)
}

fn line(kit: &CanvasKit<'_>, at: TextAt, value: &str, ink: Color) {
    settlement::line(kit, at, value, ink);
}

fn width_of(kit: &CanvasKit<'_>, at: TextAt, value: &str) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
    sketch::text_width(kit, &style, value).round()
}

fn ellipsize(kit: &CanvasKit<'_>, at: TextAt, value: &str, max_width: f32) -> String {
    if value.is_empty() || max_width <= 0.0 {
        return value.to_string();
    }
    let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
    sketch::ellipsize(kit, &style, value, max_width)
}

fn rule(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, ink: Color) {
    if x1 <= x0 {
        return;
    }
    sketch::ph_fill(
        kit,
        sketch::ph_rect(x0, y, x1 - x0, CHAP_RULE_THICK.max(1.0)),
        ink,
    );
}

fn backdrop(kit: &CanvasKit<'_>, root: &Path, req: &ChapterDetailRequest, height: f32) {
    let canvas = sketch::ph_rect(0.0, 0.0, CHAP_W, height);
    let mut drawn = false;
    if !req.banner.is_empty() {
        let key = super::locate(root, &format!("chapter/{}", req.banner));
        if let Some(texture) = memo::texture(kit, &key) {
            if let Some(image) = texture.image() {
                sketch::ph_fill(kit, canvas, settlement::COVER_FALLBACK);
                drawn = sketch::blur_backdrop(
                    kit,
                    image,
                    canvas,
                    sketch::BACKDROP_BLUR,
                    sketch::BACKDROP_TINT,
                );
            }
        }
    }
    if !drawn {
        sketch::backdrop(
            kit,
            root,
            &req.backdrop,
            canvas,
            settlement::COVER_FALLBACK,
            sketch::BACKDROP_TINT,
            sketch::BACKDROP_BLUR,
        );
    }
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SCRIM);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SHEET_SCRIM);
}

fn head(kit: &CanvasKit<'_>, req: &ChapterDetailRequest) {
    let title_at = TextAt::left(
        CHAP_MARGIN_X,
        CHAP_TITLE_TOP + CHAP_TITLE_SIZE,
        CHAP_TITLE_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY);
    let title = ellipsize(kit, title_at, &req.title, CHAP_CONTENT);
    line(kit, title_at, &title, settlement::SCORE_INK);
    if !req.note.is_empty() {
        let note_at = TextAt::left(
            CHAP_MARGIN_X,
            CHAP_TITLE_TOP + CHAP_TITLE_SIZE + CHAP_TITLE_GAP + CHAP_NOTE_SIZE,
            CHAP_NOTE_SIZE,
        );
        let note = ellipsize(kit, note_at, &req.note, CHAP_CONTENT);
        line(kit, note_at, &note, settlement::LABEL_INK);
    }
}

fn kpi(
    kit: &CanvasKit<'_>,
    index: usize,
    number: &str,
    suffix: &str,
    label: &str,
) {
    if number.is_empty() {
        return;
    }
    let (value, label_at) = chap_kpi_at(index);
    let small = TextAt {
        size: value.size * 0.5,
        ..value
    };
    let gap = value.size * 0.20;
    let body_w = width_of(kit, value, number);
    let suffix_w = width_of(kit, small, suffix);
    let suffix_gap = if suffix.is_empty() { 0.0 } else { gap };
    let total = body_w + suffix_gap + suffix_w;
    let start = match value.align {
        settlement::Align::Right => value.x - total,
        settlement::Align::Center => value.x - total / 2.0,
        settlement::Align::Left => value.x,
    };
    line(
        kit,
        TextAt {
            x: start.round(),
            align: settlement::Align::Left,
            ..value
        },
        number,
        settlement::SCORE_INK,
    );
    if !suffix.is_empty() {
        line(
            kit,
            TextAt {
                x: (start + body_w + gap).round(),
                align: settlement::Align::Left,
                ..small
            },
            suffix,
            OV_NOTE_INK,
        );
    }
    line(kit, label_at, label, OV_NOTE_INK);
}

fn summary(kit: &CanvasKit<'_>, req: &ChapterDetailRequest) {
    let played = format!("{}", req.played);
    let total = format!("{}{}", CHAP_KPI_TOTAL_SEP, req.total);
    kpi(kit, 0, &played, &total, CHAP_KPI_PLAYED);
    let avg = if req.played > 0 {
        settlement::acc_text(req.avg_acc)
    } else {
        String::from(CHAP_DASH)
    };
    kpi(kit, 1, &avg, "", CHAP_KPI_ACC);
    let songs = format!("{}", req.songs.len());
    kpi(kit, 2, &songs, CHAP_KPI_SONG_SUFFIX, CHAP_KPI_SONGS);
    grade_bars(kit, req);
}

fn grade_bars(kit: &CanvasKit<'_>, req: &ChapterDetailRequest) {
    let chart = chap_chart_rect();
    rule(
        kit,
        chart.left(),
        chap_inner().right(),
        chap_axis_y(),
        CHAP_AXIS_INK,
    );
    let peak = req.grades.iter().copied().max().unwrap_or(0);
    for index in 0..CHAP_GRADE_COLS {
        let count = req.grades[index];
        let mut ink = grade_ink(index);
        let track = chap_bar_track(index);
        sketch::fill_round(kit, track, CHAP_BAR_RADIUS, CHAP_BAR_TRACK_INK);
        let bar = chap_bar_rect(index, count, peak);
        if bar.height() > 0.0 {
            sketch::fill_round(kit, bar, CHAP_BAR_RADIUS, ink);
        } else {
            let stub = chap_bar_rect(index, 1, 1);
            sketch::fill_round(
                kit,
                Rect::from_xywh(
                    stub.left(),
                    chap_axis_y() - CHAP_BAR_STUB_H,
                    stub.width(),
                    CHAP_BAR_STUB_H,
                ),
                CHAP_BAR_STUB_RADIUS,
                CHAP_ZERO_INK,
            );
        }
        if count == 0 || index == CHAP_GRADE_COLS - 1 {
            ink = settlement::PLACEHOLDER_INK;
        }
        line(
            kit,
            chap_bar_count_at(index),
            &count.to_string(),
            ink,
        );
        line(
            kit,
            chap_bar_label_at(index),
            CHAP_GRADE_LABELS[index],
            OV_NOTE_INK,
        );
    }
}

fn head_row(kit: &CanvasKit<'_>, req: &ChapterDetailRequest, cols: &[usize]) {
    line(
        kit,
        chap_head_title_at(),
        CHAP_HEAD_TITLE,
        super::table::TABLE_HEAD_INK,
    );
    line(
        kit,
        chap_head_best_at(),
        CHAP_HEAD_BEST,
        super::table::TABLE_HEAD_INK,
    );
    for (slot, index) in cols.iter().enumerate() {
        let level = &req.levels[*index];
        line(
            kit,
            chap_head_diff_at(cols.len(), slot),
            &level.diff,
            super::table::diff_ink(&level.diff),
        );
    }
}

fn cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str, area: Rect) {
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::thumb_key(root, song_id))
            .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
    };
    match texture.as_ref().and_then(|item| item.image()) {
        Some(image) => sketch::clip_round(kit, frame, CHAP_COVER_R, || {
            sketch::image_cover(kit, image, frame);
        }),
        None => sketch::fill_round(kit, frame, CHAP_COVER_R, settlement::COVER_FALLBACK),
    }
}

fn cell_of(song: &ChapterDetailSong, index: usize) -> Option<&ChapterDetailCell> {
    song.cells.get(index)
}

fn song_rows(
    kit: &CanvasKit<'_>,
    root: &Path,
    req: &ChapterDetailRequest,
    cols: &[usize],
) {
    let inner = chap_inner();
    let width = chap_title_width(cols.len());
    let count = req.songs.len();
    for (row, song) in req.songs.iter().enumerate() {
        if row + 1 < count {
            rule(
                kit,
                inner.left(),
                inner.right(),
                chap_row_rule_y(row),
                CHAP_RULE_INK,
            );
        }
        cover(kit, root, &song.song_id, chap_cover_rect(row));
        let title_at = chap_row_title_at(row);
        let title = ellipsize(kit, title_at, &song.title, width);
        line(kit, title_at, &title, settlement::SCORE_INK);
        if !song.sub.is_empty() {
            let sub_at = chap_row_sub_at(row);
            let sub = ellipsize(kit, sub_at, &song.sub, width);
            line(kit, sub_at, &sub, OV_NOTE_INK);
        }
        let (best, best_ink) = if song.best_acc > 0.0 {
            (settlement::acc_text(song.best_acc), OV_NOTE_INK)
        } else {
            (String::from(CHAP_DASH), settlement::PLACEHOLDER_INK)
        };
        line(kit, chap_best_at(cols.len(), row), &best, best_ink);
        for (slot, index) in cols.iter().enumerate() {
            let Some(cell) = cell_of(song, *index) else {
                line(
                    kit,
                    chap_cell_none_at(cols.len(), slot, row),
                    CHAP_DASH,
                    settlement::PLACEHOLDER_INK,
                );
                continue;
            };
            if cell.constant <= 0.0 || cell.grade.is_empty() {
                line(
                    kit,
                    chap_cell_none_at(cols.len(), slot, row),
                    CHAP_DASH,
                    settlement::PLACEHOLDER_INK,
                );
                continue;
            }
            settlement::grade(
                kit,
                root,
                chap_grade_box(cols.len(), slot, row),
                true,
                &cell.grade,
            );
            line(
                kit,
                chap_cell_score_at(cols.len(), slot, row),
                &sketch::group_digits(&cell.score.to_string()),
                settlement::SCORE_INK,
            );
            let stat = format!(
                "{}{}{}",
                settlement::const_text1(cell.constant),
                CHAP_CELL_SEP,
                settlement::acc_text(cell.acc)
            );
            line(
                kit,
                chap_cell_stat_at(cols.len(), slot, row),
                &stat,
                OV_NOTE_INK,
            );
        }
    }
}

fn total_row(kit: &CanvasKit<'_>, req: &ChapterDetailRequest, cols: &[usize]) {
    let rows = req.songs.len();
    let inner = chap_inner();
    rule(
        kit,
        inner.left(),
        inner.right(),
        chap_total_rule_y(rows),
        CHAP_RULE_INK,
    );
    line(
        kit,
        chap_total_label_at(rows),
        CHAP_TOTAL_LABEL,
        settlement::SCORE_INK,
    );
    let mut top = 0.0_f64;
    for song in &req.songs {
        if song.best_acc > top {
            top = song.best_acc;
        }
    }
    if req.full_combo > 0 {
        line(
            kit,
            chap_total_best_label_at(rows),
            &format!("{}{}", req.full_combo, CHAP_FC_SUFFIX),
            settlement::PLACEHOLDER_INK,
        );
    }
    let top_text = if top > 0.0 {
        settlement::acc_text(top)
    } else {
        String::from(CHAP_DASH)
    };
    line(
        kit,
        chap_total_best_at(rows),
        &top_text,
        OV_NOTE_INK,
    );
    for (slot, index) in cols.iter().enumerate() {
        let level = &req.levels[*index];
        line(
            kit,
            chap_total_count_at(cols.len(), slot, rows),
            &format!("{}{}{}", level.played, CHAP_PLAYED_SEP, level.total),
            settlement::SCORE_INK,
        );
        let acc = if level.played > 0 {
            settlement::acc_text(level.acc)
        } else {
            String::from(CHAP_DASH)
        };
        line(
            kit,
            chap_total_acc_at(cols.len(), slot, rows),
            &acc,
            OV_NOTE_INK,
        );
    }
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &ChapterDetailRequest) {
    let rows = req.songs.len();
    let height = chap_height(rows).round();
    backdrop(kit, root, req, height);
    head(kit, req);
    player::draw(kit, root, CHAP_RIGHT, CHAP_PLAYER_K, 0.0, &req.player);
    sketch::fill_round(kit, chap_card(rows), CHAP_CARD_RADIUS, SHEET_INK);
    let inner = chap_inner();
    let cols = columns(req);
    summary(kit, req);
    rule(
        kit,
        inner.left(),
        inner.right(),
        chap_table_rule_y(),
        CHAP_RULE_INK,
    );
    head_row(kit, req, &cols);
    rule(
        kit,
        inner.left(),
        inner.right(),
        chap_head_rule_y(),
        CHAP_HEAD_RULE_INK,
    );
    song_rows(kit, root, req, &cols);
    total_row(kit, req, &cols);
    line(
        kit,
        chap_footer_at(rows),
        CHAP_FOOTER,
        settlement::LABEL_INK,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_render::skia_safe::image::CachingHint;
    use aurorix_render::skia_safe::{AlphaType, ColorType, Data, ImageInfo};
    use std::time::Instant;

    const OUT: &str = "/tmp/phigros-verify/15-chapterdetail.png";

    fn library() -> Vec<crate::model::Song> {
        let root = crate::assets_root();
        let Ok(catalog) = crate::domain::catalog::Catalog::open(root.join("../data/phi.db")) else {
            return Vec::new();
        };
        catalog.songs().to_vec()
    }

    fn grades_of(chart: &crate::model::Chart) -> (String, i32, f64) {
        if !chart.known() {
            return (String::new(), 0, 0.0);
        }
        let ratio = (chart.constant / 17.0).clamp(0.0, 1.0);
        let acc = 100.0 - 6.0 * (1.0 - ratio);
        let score = 1_000_000 - (60_000.0 * (1.0 - ratio)) as i32;
        let grade = if acc >= 99.0 {
            String::from("V")
        } else if acc >= 95.0 {
            String::from("S")
        } else if acc >= 90.0 {
            String::from("A")
        } else {
            String::from("B")
        };
        (grade, score, acc)
    }

    fn request() -> ChapterDetailRequest {
        let all = library();
        let labels = ["EZ", "HD", "IN", "AT"];
        let picked: Vec<crate::model::Song> = all.iter().take(12).cloned().collect();
        let mut songs: Vec<ChapterDetailSong> = Vec::with_capacity(picked.len());
        let mut best_all = 0.0_f64;
        for song in &picked {
            let mut cells: Vec<ChapterDetailCell> = Vec::with_capacity(4);
            let mut best = 0.0_f64;
            for chart in &song.charts {
                let (grade, score, acc) = grades_of(chart);
                if acc > best {
                    best = acc;
                }
                cells.push(ChapterDetailCell {
                    constant: chart.constant,
                    grade,
                    score,
                    acc,
                    rks: chart.constant * 0.7,
                });
            }
            if best > best_all {
                best_all = best;
            }
            songs.push(ChapterDetailSong {
                song_id: song.id.clone(),
                title: song.title.clone(),
                sub: song.charts[2].charter.clone(),
                cells,
                best_acc: best,
            });
        }
        let levels: Vec<ChapterDetailLevel> = labels
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let total = picked
                    .iter()
                    .filter(|song| song.charts[index].known())
                    .count();
                ChapterDetailLevel {
                    diff: String::from(*label),
                    played: total.saturating_sub(1),
                    total,
                    acc: 96.0 + index as f64 * 0.4,
                }
            })
            .collect();
        let total = levels.iter().map(|level| level.total).sum();
        let played = levels.iter().map(|level| level.played).sum();
        let player = player::Player {
            has_player: true,
            name: String::from("Okamikko"),
            avatar: String::from("avatar_1"),
            rank: 3,
            rank_level: 4,
            rks: best_all - 80.0,
        };
        ChapterDetailRequest {
            title: String::from("霓虹灯牌"),
            note: String::from("第 5 章 · 12 首 · 48 张谱面"),
            player,
            backdrop: picked
                .first()
                .map(|song| song.id.clone())
                .unwrap_or_default(),
            banner: String::from("Chapter 5 霓虹灯牌"),
            total,
            played,
            full_combo: 3,
            phi: 1,
            avg_acc: 96.8,
            grades: [1, 3, 5, 6, 7, 4, 2, 1, 2],
            levels,
            songs,
        }
    }

    fn luma_variance(image: &aurorix_render::skia_safe::Image) -> f64 {
        let width = image.width();
        let height = image.height();
        if width <= 0 || height <= 0 {
            return 0.0;
        }
        let info = ImageInfo::new(
            (width, height),
            ColorType::RGBA8888,
            AlphaType::Unpremul,
            None,
        );
        let row = width as usize * 4;
        let mut data = vec![0u8; row * height as usize];
        if !image.read_pixels(&info, &mut data, row, (0, 0), CachingHint::Allow) {
            return 0.0;
        }
        let step = 8_usize;
        let mut sum = 0.0;
        let mut square = 0.0;
        let mut count = 0.0;
        let mut y = 0;
        while y < height as usize {
            let mut x = 0;
            while x < width as usize {
                let at = y * row + x * 4;
                let luma = 0.299 * f64::from(data[at])
                    + 0.587 * f64::from(data[at + 1])
                    + 0.114 * f64::from(data[at + 2]);
                sum += luma;
                square += luma * luma;
                count += 1.0;
                x += step;
            }
            y += step;
        }
        if count <= 0.0 {
            return 0.0;
        }
        let mean = sum / count;
        square / count - mean * mean
    }

    #[test]
    fn canvas_height_follows_the_row_count() {
        let req = request();
        let (width, height) = canvas_size(&req);
        assert_eq!(width, CHAP_CANVAS_W);
        assert_eq!(width, 1046);
        assert_eq!(height, 1654);
        assert_eq!(chap_height(0).round() as i32, 646);
        assert_eq!(chap_height(12).round() as i32, height);
    }

    #[test]
    fn columns_and_cells_line_up() {
        let req = request();
        let cols = columns(&req);
        assert_eq!(cols.len(), 4);
        assert_eq!(cols[0], 0);
        assert!((chap_diff_left(4, 0) - (chap_title_x() + CHAP_TITLE_W + 20.0)).abs() < 0.001);
        assert!((chap_cell_right(4, 0) - (chap_diff_right(4, 0) - 12.0)).abs() < 0.001);
        assert!(chap_cell_right(4, 3) < chap_inner().right());
        assert!((chap_card(12).bottom() + CHAP_BOTTOM - chap_height(12)).abs() < 0.001);
        assert_eq!(chap_grade_box(4, 3, 11).width(), CHAP_GRADE_SIDE);
        assert!(chap_bar_rect(0, 5, 5).height() > chap_bar_rect(0, 1, 5).height());
        let empty = chap_bar_rect(0, 0, 5);
        assert_eq!(empty.height(), 0.0);
    }

    #[test]
    #[ignore]
    fn render_chapterdetail() {
        let root = crate::assets_root();
        crate::cards::register_fonts(&root);
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context = aurorix_render::RenderContext::new(
            aurorix_render::SkiaBackend::handle(),
            64,
            256 * 1024 * 1024,
        )
        .with_asset_loader(assets.loader());
        let req = request();
        let (width, height) = canvas_size(&req);
        let render = aurorix_render::RenderRequest::new(width, height);
        let start = Instant::now();
        let bytes = context
            .draw(render, |canvas, ctx| {
                let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
                draw(&kit, &root, &req);
            })
            .expect("渲染失败")
            .to_vec();
        let elapsed = start.elapsed();
        if let Some(parent) = std::path::Path::new(OUT).parent() {
            std::fs::create_dir_all(parent).expect("建出图目录");
        }
        std::fs::write(OUT, &bytes).expect("写出 PNG");
        let image = aurorix_render::skia_safe::Image::from_encoded(Data::new_copy(&bytes))
            .expect("解码头");
        println!(
            "单章详情渲染耗时 {} ms ({} 字节, {}x{})",
            elapsed.as_millis(),
            bytes.len(),
            image.width(),
            image.height()
        );
        assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
        assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
        assert!(bytes.len() > 20000, "字节数必须大于 20000");
        let variance = luma_variance(&image);
        println!("单章详情 luma 方差 {variance:.2}");
        assert!(variance > 100.0, "luma 方差必须大于 100, 实际 {variance:.2}");
    }
}
