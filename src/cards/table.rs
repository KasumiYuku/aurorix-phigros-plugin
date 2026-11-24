use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::settlement::{self, TextAt};
use super::{memo, sketch};

pub const TABLE_NAME: &str = "TABLE";

pub const TABLE_W: f32 = 1160.0;

pub const TABLE_X0: f32 = 44.0;

pub const TABLE_CANVAS_W: i32 = 1160;

pub const TABLE_MARGIN_X: f32 = 44.0;

pub const TABLE_RIGHT: f32 = 1116.0;

pub const TABLE_CONTENT: f32 = 1072.0;

pub const TABLE_HEAD_TOP: f32 = 36.0;

pub const TABLE_TITLE_SIZE: f32 = 40.0;

pub const TABLE_NOTE_SIZE: f32 = 14.0;

pub const TABLE_HEAD_NOTE_GAP: f32 = 10.0;

pub const TABLE_HEAD_H: f32 = 64.0;

pub const TABLE_HEAD_GAP: f32 = 30.0;

pub const TABLE_CARD_TOP: f32 = 130.0;

pub const TABLE_CARD_PAD: f32 = 32.0;

pub const TABLE_CARD_TOP1: f32 = 10.0;

pub const TABLE_CARD_BOTTOM: f32 = 14.0;

pub const TABLE_HEAD_H2: f32 = 44.0;

pub const TABLE_ROW_H: f32 = 60.0;

pub const TABLE_BAND_H: f32 = 40.0;

pub const TABLE_BAND_GAP: f32 = 14.0;

pub const TABLE_ROW_PAD: f32 = 8.0;

pub const TABLE_BOTTOM: f32 = 36.0;

pub const TABLE_NUM_COL: f32 = 124.0;

pub const TABLE_NUM_W: f32 = 52.0;

pub const TABLE_COVER: f32 = 44.0;

pub const TABLE_COVER_R: f32 = 8.0;

pub const TABLE_COVER_GAP: f32 = 14.0;

pub const TABLE_ROW_LEAD: f32 = 25.0;

pub const TABLE_ROW_SUB: f32 = 45.0;

pub const TABLE_BAND_COUNT_GAP: f32 = 14.0;

pub const TABLE_TITLE_SIZE2: f32 = 17.0;

pub const TABLE_SUB_SIZE: f32 = 14.0;

pub const TABLE_NUM_SIZE: f32 = 18.0;

pub const TABLE_HEAD_SIZE: f32 = 13.0;

pub const TABLE_BAND_SIZE: f32 = 22.0;

pub const TABLE_BAND_COUNT_SIZE: f32 = 12.5;

pub const MID_RATIO: f32 = 0.36;

pub const TABLE_HIT_BAND_INSET_X: f32 = 14.0;

pub const TABLE_HIT_BAND_INSET_Y: f32 = 5.0;

pub const TABLE_HIT_BAND_RADIUS: f32 = 10.0;

pub const SHEET_RADIUS: f32 = 14.0;

pub const RULE_THICK: f32 = 1.0;

pub const SHEET_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x0E);

pub const COVER_FALLBACK: Color = Color::new(0x22, 0x26, 0x30, 0xFF);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const LABEL_INK: Color = Color::new(0xB4, 0xBA, 0xC6, 0xFF);

pub const OV_NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const TABLE_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const TABLE_HIT_CELL_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1E);

pub const TABLE_BAND_LABEL_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const TABLE_BAND_COUNT_INK: Color = Color::new(0xF2, 0xF4, 0xF8, 0xDA);

pub const TABLE_HEAD_INK: Color = Color::new(0xF2, 0xF4, 0xF8, 0xC8);

pub const TABLE_GAP_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x30);

pub const HEAD_NAME_LABEL: &str = "曲名";

pub const COUNT_SUFFIX: &str = " 首";

pub const GAP_DOT: &str = "·";

pub const SEPARATOR: &str = " · ";

pub const DIFF_EZ_INK: Color = Color::new(0xA0, 0xE0, 0x57, 0xFF);

pub const DIFF_HD_INK: Color = Color::new(0x45, 0xE0, 0xA8, 0xFF);

pub const DIFF_IN_INK: Color = Color::new(0x7F, 0xA3, 0xD4, 0xFF);

pub const DIFF_AT_INK: Color = Color::new(0xFF, 0x5F, 0x68, 0xFF);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableCell {
    pub constant: f64,
    pub hit: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableRow {
    pub song_id: String,
    pub title: String,
    pub sub: String,
    pub cells: Vec<TableCell>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableSection {
    pub label: String,
    pub rows: Vec<TableRow>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableRequest {
    pub name: String,
    pub title: String,
    pub note: String,
    pub backdrop: String,
    pub cols: Vec<String>,
    pub sections: Vec<TableSection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableSectionIn {
    pub label: String,
    pub rows: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TableBand {
    pub label: String,
    pub count: usize,
    pub area: Rect,
    pub base: f32,
    pub rows: Vec<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TablePlan {
    pub w: i32,
    pub cols: usize,
    pub columns: usize,
    pub column_w: f32,
    pub column_left: [f32; 2],
    pub card: Rect,
    pub row_tops: Vec<f32>,
    pub row_col: Vec<u8>,
    pub bands: Vec<TableBand>,
    pub height: f32,
}

pub fn px(value: f32) -> i32 {
    (value + 0.5) as i32
}

pub const TABLE_COLUMN_GAP: f32 = 28.0;

pub const TABLE_TWO_COL_ROWS: usize = 40;

pub const TABLE_TWO_COL_W: i32 = 1680;

pub const TABLE_MAX_BANDS: usize = 12;

pub fn plan(req: &TableRequest) -> TablePlan {
    let sections: Vec<TableSectionIn> = req
        .sections
        .iter()
        .map(|section| TableSectionIn {
            label: section.label.clone(),
            rows: section.rows.len(),
        })
        .collect();
    plan_for(&sections, req.cols.len())
}

pub fn plan_for(sections: &[TableSectionIn], cols: usize) -> TablePlan {
    let cols = cols.max(1);
    let total: usize = sections.iter().map(|item| item.rows).sum();
    let merged: Vec<TableSectionIn> = match sections.len() > TABLE_MAX_BANDS {
        true => vec![TableSectionIn {
            label: String::new(),
            rows: total,
        }],
        false => sections.to_vec(),
    };
    let sections = merged.as_slice();
    let columns = match total > TABLE_TWO_COL_ROWS {
        true => 2,
        false => 1,
    };
    let width = match columns {
        2 => TABLE_TWO_COL_W,
        _ => TABLE_CANVAS_W,
    };
    let mut out = TablePlan {
        w: width,
        cols,
        columns,
        ..TablePlan::default()
    };
    let content = content_rect(&out);
    out.column_w = (content.width() - TABLE_COLUMN_GAP * (columns as f32 - 1.0)) / columns as f32;
    out.column_left = [
        content.left(),
        content.left() + out.column_w + TABLE_COLUMN_GAP,
    ];
    let mut y = TABLE_CARD_TOP + TABLE_CARD_TOP1 + TABLE_HEAD_H2;
    let mut row = 0_usize;
    for (index, item) in sections.iter().enumerate() {
        if index > 0 {
            y += TABLE_BAND_GAP;
        }
        let mut band = TableBand {
            label: item.label.clone(),
            count: item.rows,
            area: Rect::from_xywh(content.left(), y, content.width(), TABLE_BAND_H),
            base: y + TABLE_BAND_H / 2.0 + TABLE_BAND_SIZE * MID_RATIO,
            rows: Vec::with_capacity(item.rows),
        };
        for slot in 0..item.rows {
            band.rows.push(row);
            let line = (slot / columns) as f32;
            out.row_tops
                .push(y + TABLE_BAND_H + TABLE_ROW_PAD + line * TABLE_ROW_H);
            out.row_col.push((slot % columns) as u8);
            row += 1;
        }
        let lines = (item.rows as f32 / columns as f32).ceil();
        y = band.area.bottom() + TABLE_ROW_PAD + lines * TABLE_ROW_H;
        out.bands.push(band);
    }
    out.card = Rect::from_xywh(
        content.left() - TABLE_CARD_PAD,
        TABLE_CARD_TOP,
        content.width() + 2.0 * TABLE_CARD_PAD,
        y + TABLE_CARD_BOTTOM - TABLE_CARD_TOP,
    );
    out.height = out.card.bottom() + TABLE_BOTTOM;
    out
}

pub fn canvas_size(req: &TableRequest) -> (i32, i32) {
    let plan = plan(req);
    (plan.w, (plan.height + 0.5).floor() as i32)
}

pub fn content_rect(plan: &TablePlan) -> Rect {
    let pad = TABLE_MARGIN_X + TABLE_CARD_PAD;
    Rect::from_xywh(pad, TABLE_CARD_TOP, plan.w as f32 - 2.0 * pad, 0.0)
}

pub fn head_col_rect(plan: &TablePlan, column: usize) -> Rect {
    Rect::from_xywh(
        plan.column_left[column.min(1)],
        TABLE_CARD_TOP,
        plan.column_w,
        0.0,
    )
}

pub fn column_rect(plan: &TablePlan, index: usize) -> Rect {
    let column = plan.row_col.get(index).copied().unwrap_or_default() as usize;
    head_col_rect(plan, column)
}

pub fn head_row_base() -> f32 {
    TABLE_CARD_TOP + TABLE_CARD_TOP1 + TABLE_HEAD_H2 / 2.0 + TABLE_HEAD_SIZE * MID_RATIO
}

pub fn head_rule_y() -> f32 {
    TABLE_CARD_TOP + TABLE_CARD_TOP1 + TABLE_HEAD_H2
}

pub fn title_left(plan: &TablePlan, index: usize) -> f32 {
    column_rect(plan, index).left() + TABLE_COVER + TABLE_COVER_GAP
}

pub fn diff_span(plan: &TablePlan, index: usize, k: usize) -> (f32, f32) {
    let content = column_rect(plan, index);
    let right = content.right() - (plan.cols as f32 - 1.0 - k as f32) * TABLE_NUM_COL;
    (right - TABLE_NUM_COL, right)
}

pub fn diff_left(plan: &TablePlan, index: usize, k: usize) -> f32 {
    diff_span(plan, index, k).0
}

pub fn diff_right(plan: &TablePlan, index: usize, k: usize) -> f32 {
    diff_span(plan, index, k).1
}

pub fn col_mid(plan: &TablePlan, index: usize, k: usize) -> f32 {
    (diff_left(plan, index, k) + diff_right(plan, index, k)) / 2.0
}

pub fn col_mid_in(content: Rect, plan: &TablePlan, k: usize) -> f32 {
    let right = content.right() - (plan.cols as f32 - 1.0 - k as f32) * TABLE_NUM_COL;
    right - TABLE_NUM_COL / 2.0
}

pub fn title_right(plan: &TablePlan, index: usize) -> f32 {
    diff_left(plan, index, 0) - 20.0
}

pub fn title_width(plan: &TablePlan, index: usize) -> i32 {
    (title_right(plan, index) - title_left(plan, index)) as i32
}

pub fn row_base(plan: &TablePlan, i: usize) -> f32 {
    plan.row_tops[i] + TABLE_ROW_LEAD
}

pub fn row_num_base(plan: &TablePlan, i: usize) -> f32 {
    plan.row_tops[i] + TABLE_ROW_H / 2.0 + TABLE_NUM_SIZE * MID_RATIO
}

pub fn row_sub_base(plan: &TablePlan, i: usize) -> f32 {
    plan.row_tops[i] + TABLE_ROW_SUB
}

pub fn cover_rect(plan: &TablePlan, i: usize) -> Rect {
    Rect::from_xywh(
        column_rect(plan, i).left(),
        plan.row_tops[i] + (TABLE_ROW_H - TABLE_COVER) / 2.0,
        TABLE_COVER,
        TABLE_COVER,
    )
}

pub fn cell_rect(plan: &TablePlan, i: usize, k: usize) -> Rect {
    Rect::from_xywh(
        diff_left(plan, i, k),
        plan.row_tops[i],
        TABLE_NUM_COL,
        TABLE_ROW_H,
    )
}

pub fn row_rule_y(plan: &TablePlan, i: usize) -> Option<f32> {
    for band in &plan.bands {
        if band.rows.last() == Some(&i) {
            return None;
        }
    }
    Some(plan.row_tops[i] + TABLE_ROW_H)
}

pub fn band_rule_y(band: &TableBand) -> f32 {
    band.area.top()
}

pub fn title_at() -> TextAt {
    TextAt::left(
        TABLE_MARGIN_X,
        TABLE_HEAD_TOP + TABLE_TITLE_SIZE,
        TABLE_TITLE_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn note_at() -> TextAt {
    let mut at = title_at();
    at.baseline += TABLE_HEAD_NOTE_GAP + TABLE_NOTE_SIZE;
    at.size = TABLE_NOTE_SIZE;
    at.font = Some(sketch::FONT_BODY);
    at
}

pub fn diff_ink(label: &str) -> Color {
    match label {
        "EZ" => DIFF_EZ_INK,
        "HD" => DIFF_HD_INK,
        "IN" => DIFF_IN_INK,
        "AT" => DIFF_AT_INK,
        _ => LABEL_INK,
    }
}

pub fn head_note(parts: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::with_capacity(parts.len());
    for part in parts {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            out.push(trimmed);
        }
    }
    out.join(SEPARATOR)
}

pub fn text_width(kit: &CanvasKit<'_>, font: &str, size: f32, value: &str) -> f32 {
    let style = sketch::text_style(font, size, SCORE_INK);
    sketch::text_width(kit, &style, value)
}

pub fn ellipsize_line(
    kit: &CanvasKit<'_>,
    font: &str,
    width: i32,
    size: f32,
    value: &str,
) -> String {
    if value.trim().is_empty() {
        return String::new();
    }
    let style = sketch::text_style(font, size, SCORE_INK);
    sketch::ellipsize(kit, &style, value, width as f32)
}

pub fn cover_key(root: &Path, song_id: &str) -> String {
    super::thumb_key(root, song_id)
}

pub fn backdrop_cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str, canvas: Rect, tint: Color) {
    sketch::backdrop(
        kit,
        root,
        song_id,
        canvas,
        COVER_FALLBACK,
        tint,
        sketch::BACKDROP_BLUR,
    );
}

pub fn prefetch(kit: &CanvasKit<'_>, root: &Path, req: &TableRequest) {
    let mut seen: Vec<&str> = Vec::new();
    for section in &req.sections {
        for row in &section.rows {
            if row.song_id.is_empty() || seen.contains(&row.song_id.as_str()) {
                continue;
            }
            seen.push(&row.song_id);
            memo::texture(kit, &cover_key(root, &row.song_id));
        }
    }
}

pub fn sheet(kit: &CanvasKit<'_>, card: Rect) {
    let frame = sketch::ph_rect(card.left(), card.top(), card.width(), card.height());
    sketch::fill_round(kit, frame, SHEET_RADIUS, SHEET_INK);
}

pub fn head(kit: &CanvasKit<'_>, req: &TableRequest) {
    let name = if req.name.is_empty() {
        String::from(TABLE_NAME)
    } else {
        req.name.clone()
    };
    let at = title_at();
    let title = ellipsize_line(kit, at.font_key(), TABLE_CONTENT as i32, at.size, &name);
    settlement::line(kit, at, &title, SCORE_INK);
    let rest = head_note(&[&req.title, &req.note]);
    if rest.is_empty() {
        return;
    }
    let at = note_at();
    let note = ellipsize_line(kit, at.font_key(), TABLE_CONTENT as i32, at.size, &rest);
    settlement::line(kit, at, &note, OV_NOTE_INK);
}

pub fn rule_line(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32, ink: Color) {
    if x1 <= x0 {
        return;
    }
    let thick = px(RULE_THICK).max(1) as f32;
    sketch::ph_fill(
        kit,
        sketch::ph_rect(x0, y, x1 - x0, thick),
        ink,
    );
}

pub fn head_row(kit: &CanvasKit<'_>, req: &TableRequest, plan: &TablePlan) {
    let content = content_rect(plan);
    for column in 0..plan.columns {
        let cell = head_col_rect(plan, column);
        settlement::line(
            kit,
            TextAt::left(cell.left(), head_row_base(), TABLE_HEAD_SIZE)
                .with_font(sketch::FONT_BODY),
            HEAD_NAME_LABEL,
            TABLE_HEAD_INK,
        );
        for (k, label) in req.cols.iter().enumerate() {
            if k >= plan.cols {
                break;
            }
            settlement::line(
                kit,
                TextAt::center(col_mid_in(cell, plan, k), head_row_base(), TABLE_HEAD_SIZE)
                    .with_font(sketch::FONT_DISPLAY),
                label,
                diff_ink(label),
            );
        }
    }
    rule_line(
        kit,
        content.left(),
        content.right(),
        head_rule_y(),
        TABLE_RULE_INK,
    );
}

pub fn row_at<'a>(req: &'a TableRequest, plan: &TablePlan, index: usize) -> Option<&'a TableRow> {
    if index >= plan.row_tops.len() {
        return None;
    }
    let mut seen = 0;
    for section in &req.sections {
        if index < seen + section.rows.len() {
            return section.rows.get(index - seen);
        }
        seen += section.rows.len();
    }
    None
}

pub fn hit_bands(_kit: &CanvasKit<'_>, _req: &TableRequest, _plan: &TablePlan) {}

pub fn band_label_at(plan: &TablePlan, band: &TableBand) -> TextAt {
    TextAt::left(content_rect(plan).left(), band.base, TABLE_BAND_SIZE)
        .with_font(sketch::FONT_DISPLAY)
}

pub fn band_count(band: &TableBand) -> String {
    format!("{}{COUNT_SUFFIX}", band.count)
}

pub fn bands(kit: &CanvasKit<'_>, root: &Path, req: &TableRequest, plan: &TablePlan) {
    let content = content_rect(plan);
    for band in &plan.bands {
        rule_line(
            kit,
            content.left(),
            content.right(),
            band_rule_y(band),
            TABLE_RULE_INK,
        );
        let at = band_label_at(plan, band);
        sketch::ph_fill(
            kit,
            Rect::from_xywh(
                content.left(),
                band.area.top() + TABLE_BAND_H * 0.30,
                4.0,
                TABLE_BAND_H * 0.40,
            ),
            TABLE_BAND_LABEL_INK,
        );
        settlement::line(
            kit,
            TextAt {
                x: content.left() + 14.0,
                ..at
            },
            &band.label,
            TABLE_BAND_LABEL_INK,
        );
        let width = text_width(kit, at.font_key(), at.size, &band.label);
        settlement::line(
            kit,
            TextAt::left(
                content.left() + width + 14.0 + TABLE_BAND_COUNT_GAP,
                band.base,
                TABLE_BAND_COUNT_SIZE,
            )
            .with_font(sketch::FONT_BODY),
            &band_count(band),
            TABLE_BAND_COUNT_INK,
        );
        for index in &band.rows {
            if *index < plan.row_tops.len() {
                row(kit, root, req, plan, *index);
            }
        }
    }
}

pub fn cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str, area: Rect) {
    let frame = sketch::ph_rect(area.left(), area.top(), area.width(), area.height());
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::thumb_key(root, song_id))
            .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
    };
    match texture.as_ref().and_then(|item| item.image()) {
        Some(image) => sketch::clip_round(kit, frame, TABLE_COVER_R, || {
            sketch::image_cover(kit, image, frame);
        }),
        None => sketch::fill_round(kit, frame, TABLE_COVER_R, COVER_FALLBACK),
    }
}

pub fn row(kit: &CanvasKit<'_>, root: &Path, req: &TableRequest, plan: &TablePlan, index: usize) {
    let Some(item) = row_at(req, plan, index) else {
        return;
    };
    let content = column_rect(plan, index);
    if let Some(y) = row_rule_y(plan, index) {
        rule_line(kit, content.left(), content.right(), y, TABLE_RULE_INK);
    }
    cover(kit, root, &item.song_id, cover_rect(plan, index));
    let title = ellipsize_line(
        kit,
        sketch::FONT_BODY,
        title_width(plan, index),
        TABLE_TITLE_SIZE2,
        &item.title,
    );
    settlement::line(
        kit,
        TextAt::left(title_left(plan, index), row_base(plan, index), TABLE_TITLE_SIZE2)
            .with_font(sketch::FONT_BODY),
        &title,
        SCORE_INK,
    );
    if !item.sub.is_empty() {
        let sub = ellipsize_line(
            kit,
            sketch::FONT_BODY,
            title_width(plan, index),
            TABLE_SUB_SIZE,
            &item.sub,
        );
        settlement::line(
            kit,
            TextAt::left(title_left(plan, index), row_sub_base(plan, index), TABLE_SUB_SIZE)
                .with_font(sketch::FONT_BODY),
            &sub,
            TABLE_BAND_COUNT_INK,
        );
    }
    for (k, _) in req.cols.iter().enumerate() {
        if k >= item.cells.len() || k >= plan.cols {
            break;
        }
        if item.cells[k].constant <= 0.0 {
            settlement::line(
                kit,
                TextAt::center(col_mid(plan, index, k), row_num_base(plan, index), TABLE_NUM_SIZE)
                    .with_font(sketch::FONT_BODY),
                GAP_DOT,
                TABLE_GAP_INK,
            );
            continue;
        }
        settlement::line(
            kit,
            TextAt::center(col_mid(plan, index, k), row_num_base(plan, index), TABLE_NUM_SIZE)
                .with_font(sketch::FONT_DISPLAY),
            &settlement::const_text1(item.cells[k].constant),
            SCORE_INK,
        );
    }
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &TableRequest) {
    let plan = plan(req);
    let height = (plan.height + 0.5).floor();
    let canvas = sketch::ph_rect(0.0, 0.0, plan.w as f32, height);
    backdrop_cover(kit, root, &req.backdrop, canvas, sketch::BACKDROP_TINT);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SCRIM);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SHEET_SCRIM);
    prefetch(kit, root, req);
    sheet(kit, plan.card);
    head(kit, req);
    head_row(kit, req, &plan);
    hit_bands(kit, req, &plan);
    bands(kit, root, req, &plan);
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_render::skia_safe::Image;
    use std::time::Instant;

    const OUT: &str = "/tmp/phigros-verify/11-table.png";

    fn cell(constant: f64, hit: bool) -> TableCell {
        TableCell { constant, hit }
    }

    fn row_data(index: usize) -> TableRow {
        let ids = [
            "000AinSophAur.Yumeji.0",
            "70MinutesFighters.かたぎり.0",
            "996.李化禹.0",
        ];
        let id = ids[index % ids.len()];
        TableRow {
            song_id: String::from(id),
            title: format!("Song {index}"),
            sub: String::from("谱师 · BPM 185  ·  2:34"),
            cells: vec![
                cell(8.6, false),
                cell(9.5, false),
                cell(15.9 - index as f64 * 0.1, index == 0),
                cell(0.0, false),
            ],
        }
    }

    fn request() -> TableRequest {
        TableRequest {
            name: String::new(),
            title: String::from("定数 14+ · IN"),
            note: String::from("命中 3 首 · 4 张谱面"),
            backdrop: String::from("000AinSophAur.Yumeji.0"),
            cols: vec![
                String::from("EZ"),
                String::from("HD"),
                String::from("IN"),
                String::from("AT"),
            ],
            sections: vec![
                TableSection {
                    label: String::from("15.9"),
                    rows: vec![row_data(0), row_data(1)],
                },
                TableSection {
                    label: String::from("15.8"),
                    rows: vec![row_data(2)],
                },
            ],
        }
    }

    #[test]
    fn plan_stacks_bands_and_rows() {
        let plan = plan(&request());
        assert_eq!(plan.w, TABLE_CANVAS_W);
        assert_eq!(plan.cols, 4);
        assert_eq!(plan.row_tops.len(), 3);
        assert_eq!(plan.bands.len(), 2);
        assert!((plan.bands[0].area.top() - 184.0).abs() < 1e-3);
        assert!((plan.row_tops[0] - (184.0 + TABLE_BAND_H + TABLE_ROW_PAD)).abs() < 1e-3);
        assert!((plan.row_tops[1] - (plan.row_tops[0] + TABLE_ROW_H)).abs() < 1e-3);
        assert!(plan.bands[1].area.top() > plan.row_tops[1] + TABLE_ROW_H);
        assert_eq!(row_rule_y(&plan, 1), None);
        assert!(row_rule_y(&plan, 0).is_some());
    }

    #[test]
    fn columns_are_pinned_to_the_right_edge() {
        let plan = plan(&request());
        let content = content_rect(&plan);
        assert!((diff_right(&plan, 0, 3) - content.right()).abs() < 1e-3);
        assert!((diff_left(&plan, 0, 0) - (content.right() - 4.0 * TABLE_NUM_COL)).abs() < 1e-3);
        assert!(title_left(&plan, 0) > content.left());
        assert!(title_right(&plan, 0) > title_left(&plan, 0));
        assert!(title_width(&plan, 0) > 0);
    }

    #[test]
    fn head_note_skips_empty_parts() {
        assert_eq!(head_note(&["", "  ", "a"]), "a");
        assert_eq!(head_note(&["a", "b"]), "a · b");
        assert_eq!(head_note(&[""]), "");
    }

    #[test]
    #[ignore]
    fn render_table() {
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
        let request = aurorix_render::RenderRequest::new(width, height);
        let start = Instant::now();
        let bytes = context
            .draw(request, |canvas, ctx| {
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
        println!(
            "定数表渲染耗时 {} ms ({} 字节, {width}x{height})",
            elapsed.as_millis(),
            bytes.len()
        );
        let image = Image::from_encoded(aurorix_render::skia_safe::Data::new_copy(&bytes))
            .expect("解码头");
        assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
        assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
        assert!(bytes.len() > 1000, "字节数必须大于 1000");
    }
}
