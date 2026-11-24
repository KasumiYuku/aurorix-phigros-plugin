use std::path::Path;

use aurorix_render::skia_safe::{
    images, surfaces, AlphaType, ClipOp, ColorType, Data, Image, ImageInfo, Paint, Rect,
};
use aurorix_render::{Color, CanvasKit};
use chrono::{DateTime, Utc};

use super::memo;
use super::player;
use super::sketch;

pub const CARD_WIDTH: f32 = 1046.0;

pub const MARGIN: f32 = 36.0;

pub const FOOTER_HEIGHT: f32 = 66.0;

pub const TITLE_FONT_SIZE: f32 = 48.0;

pub const SUBTITLE_FONT_SIZE: f32 = 22.0;

pub const TIER_FONT_SIZE: f32 = 30.0;

pub const SUBVALUE_FONT_SIZE: f32 = 22.0;

pub const SCORE_FONT_SIZE: f32 = 44.0;

pub const ROW_FONT_SIZE: f32 = 17.0;

pub const DATE_FONT_SIZE: f32 = 14.0;

pub const HEAD_FONT_SIZE: f32 = 15.0;

pub const HINT_FONT_SIZE: f32 = 18.0;

pub const TITLE_BASELINE: f32 = 69.5;

pub const SUBTITLE_BASELINE: f32 = 106.0;

pub const CARD_TOP: f32 = 142.0;

pub const CARD_GAP: f32 = 20.0;

pub const PADDING: f32 = 24.0;

pub const BAND_PAD: f32 = 16.0;

pub const GRADE_BOX: f32 = 40.0;

pub const BAND_HEIGHT: f32 = 2.0 * BAND_PAD + GRADE_BOX;

pub const ROW_HEIGHT: f32 = 35.0;

pub const HEAD_HEIGHT: f32 = 36.0;

pub const HEAD_BASELINE_DROP: f32 = 11.0;

pub const CARD_RADIUS: f32 = 14.0;

pub const CARD_BLUR: f32 = 8.0;

pub const PLAYER_BAR_OFFSET: f32 = 5.2;

pub const GRADE_ROW_BOX: f32 = 20.0;

pub const COL_DELTA_RIGHT: f32 = 0.0;

pub const COL_RKS_RIGHT: f32 = 150.0;

pub const COL_ACC_RIGHT: f32 = 245.0;

pub const COL_GRADE_RIGHT: f32 = 350.0;

pub const COL_SCORE_RIGHT: f32 = 415.0;

pub const MID_RATIO: f32 = 0.36;

pub const TITLE_GAP: f32 = 16.0;

pub const TIER_GAP: f32 = 28.0;

pub const LATEST_GAP: f32 = 18.0;

pub const GRAY_BLUR: f32 = 16.0;

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const LABEL_INK: Color = Color::new(0xB4, 0xBA, 0xC6, 0xFF);

pub const PLACEHOLDER_INK: Color = Color::new(0x8A, 0x92, 0xA2, 0xFF);

pub const NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const HAIRLINE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x10);

pub const GAIN_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

pub const DROP_INK: Color = Color::new(0xD9, 0x7A, 0x7A, 0xFF);

pub const CARD_INK: Color = Color::new(0x0A, 0x0C, 0x12, 0x4C);

pub const BASE_INK: Color = Color::new(0x22, 0x26, 0x30, 0xFF);

pub const BASE_TINT: Color = Color::new(0x06, 0x08, 0x0E, 0xB0);

pub const PAGE_SHADE: Color = Color::new(0x03, 0x04, 0x08, 0x5A);

pub const EMPTY_CARD_TEXT: &str = "该首单曲暂未存有任何变化记录捏，可以尝试更新存档或者换一首曲子试试w";

pub const NO_HISTORY_TEXT: &str = "还没有变化记录";

pub const NOT_PLAYED_TEXT: &str = "尚无成绩";

pub const NO_CHART_TEXT: &str = "无此谱面";

pub const LATEST_TEXT: &str = "最新";

pub const FOOTER_TEXT: &str = "成绩来自上次同步记录 · 使用/p update 可更新";

#[derive(Clone, Debug)]
pub struct SongInfoRequest {
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub player: player::Player,
    pub levels: Vec<SongInfoLevel>,
}

#[derive(Clone, Debug)]
pub struct SongInfoLevel {
    pub diff: String,
    pub constant: f64,
    pub has: bool,
    pub played: bool,
    pub score: i32,
    pub acc: f64,
    pub grade: String,
    pub rks: f64,
    pub combo: i32,
    pub history: Vec<SongInfoPoint>,
    pub changes: i32,
}

#[derive(Clone, Debug)]
pub struct SongInfoPoint {
    pub at: DateTime<Utc>,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
    pub grade: String,
    pub delta: i32,
    pub first: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub tops: Vec<f32>,
    pub heights: Vec<f32>,
    pub last_bottom: f32,
    pub total_height: f32,
}

pub fn content_area() -> Rect {
    Rect::from_xywh(MARGIN, 0.0, CARD_WIDTH - 2.0 * MARGIN, 0.0)
}

pub fn content_width() -> f32 {
    CARD_WIDTH - 2.0 * MARGIN
}

pub fn card_height(rows: usize) -> f32 {
    BAND_HEIGHT + HEAD_HEIGHT + rows as f32 * ROW_HEIGHT + PADDING
}

pub fn layout(rows: &[usize]) -> Layout {
    if rows.is_empty() {
        let height = BAND_HEIGHT + HEAD_HEIGHT * 2.0 + PADDING;
        return Layout {
            tops: vec![CARD_TOP],
            heights: vec![height],
            last_bottom: CARD_TOP + height,
            total_height: CARD_TOP + height + FOOTER_HEIGHT,
        };
    }
    let mut tops = Vec::with_capacity(rows.len());
    let mut heights = Vec::with_capacity(rows.len());
    let mut y = CARD_TOP;
    for row in rows {
        let height = card_height(*row);
        tops.push(y);
        heights.push(height);
        y += height + CARD_GAP;
    }
    y -= CARD_GAP;
    Layout {
        tops,
        heights,
        last_bottom: y,
        total_height: y + FOOTER_HEIGHT,
    }
}

pub fn canvas_size(req: &SongInfoRequest) -> (i32, i32) {
    let rows: Vec<usize> = req.levels.iter().map(|level| level.history.len()).collect();
    let plan = layout(&rows);
    (CARD_WIDTH.round() as i32, plan.total_height.round() as i32)
}

pub fn card_rect(plan: &Layout, index: usize) -> Rect {
    Rect::from_xywh(
        MARGIN,
        plan.tops.get(index).copied().unwrap_or(CARD_TOP),
        content_width(),
        plan.heights.get(index).copied().unwrap_or_else(|| card_height(0)),
    )
}

pub fn card_inner(card: Rect) -> Rect {
    Rect::from_xywh(card.left() + PADDING, card.top(), card.width() - 2.0 * PADDING, card.height())
}

pub fn band_center_y(card: Rect) -> f32 {
    card.top() + BAND_PAD + GRADE_BOX / 2.0
}

pub fn tier_name_pos(card: Rect) -> (f32, f32) {
    (card_inner(card).left(), band_center_y(card) + TIER_FONT_SIZE * MID_RATIO)
}

pub fn score_right(card: Rect) -> f32 {
    card_inner(card).right() - GRADE_BOX - PADDING
}

pub fn score_baseline(card: Rect) -> f32 {
    band_center_y(card) + SCORE_FONT_SIZE * MID_RATIO
}

pub fn grade_box(card: Rect) -> Rect {
    let center = band_center_y(card);
    Rect::from_xywh(
        card_inner(card).right() - GRADE_BOX,
        center - GRADE_BOX / 2.0,
        GRADE_BOX,
        GRADE_BOX,
    )
}

pub fn body_top(card: Rect) -> f32 {
    card.top() + BAND_HEIGHT
}

pub fn history_head_y(card: Rect) -> (f32, f32) {
    (card_inner(card).left(), body_top(card) + HEAD_HEIGHT - HEAD_BASELINE_DROP)
}

pub fn count_pos(card: Rect) -> (f32, f32) {
    (card_inner(card).right(), body_top(card) + HEAD_HEIGHT - HEAD_BASELINE_DROP)
}

pub fn row_rect(card: Rect, index: usize) -> Rect {
    Rect::from_xywh(
        card_inner(card).left(),
        body_top(card) + HEAD_HEIGHT + index as f32 * ROW_HEIGHT,
        card_inner(card).right() - card_inner(card).left(),
        ROW_HEIGHT,
    )
}

pub fn head_line_y(card: Rect) -> f32 {
    body_top(card) + HEAD_HEIGHT
}

pub fn row_line_y(card: Rect, index: usize) -> f32 {
    row_rect(card, index).bottom()
}

#[derive(Clone, Copy, Debug)]
pub struct RowSlots {
    pub date: (f32, f32),
    pub score: (f32, f32),
    pub grade: Rect,
    pub acc_percent: (f32, f32),
    pub rks: (f32, f32),
    pub delta: (f32, f32),
}

pub fn row_slots(card: Rect, index: usize) -> RowSlots {
    let row = row_rect(card, index);
    let base = row.top() + row.height() * 0.66;
    let right = row.right();
    RowSlots {
        date: (row.left(), base),
        score: (right - COL_SCORE_RIGHT, base),
        grade: Rect::from_xywh(
            right - COL_GRADE_RIGHT,
            base - GRADE_ROW_BOX * 0.78,
            GRADE_ROW_BOX,
            GRADE_ROW_BOX,
        ),
        acc_percent: (right - COL_ACC_RIGHT, base),
        rks: (right - COL_RKS_RIGHT, base),
        delta: (right - COL_DELTA_RIGHT, base),
    }
}

pub fn head_grade_pos(card: Rect) -> (f32, f32) {
    let row = row_rect(card, 0);
    let base = history_head_y(card).1;
    (row.right() - COL_GRADE_RIGHT + GRADE_ROW_BOX / 2.0, base)
}

pub fn footer_baseline(plan: &Layout) -> f32 {
    plan.last_bottom + FOOTER_HEIGHT / 2.0 + HINT_FONT_SIZE * MID_RATIO
}

pub fn tier_name(level: &SongInfoLevel) -> String {
    if level.has {
        format!("{}  {}", level.diff, sketch::one_decimal(level.constant))
    } else {
        level.diff.clone()
    }
}

pub fn score_text(score: i32) -> String {
    sketch::group_digits(&score.to_string())
}

pub fn history_count_text(level: &SongInfoLevel) -> String {
    if level.changes as usize > level.history.len() {
        format!(
            "共 {} 次 · 列最近 {} 次",
            level.changes,
            level.history.len()
        )
    } else {
        format!("共 {} 次", level.changes)
    }
}

pub fn delta_slot(point: &SongInfoPoint) -> (String, Color) {
    if point.first {
        return (String::from("NEW"), PLACEHOLDER_INK);
    }
    let ink = if point.delta > 0 {
        GAIN_INK
    } else if point.delta < 0 {
        DROP_INK
    } else {
        NOTE_INK
    };
    (sketch::signed_score(point.delta), ink)
}

pub fn title_avail_width(kit: &CanvasKit<'_>, req: &SongInfoRequest) -> f32 {
    if !req.player.has_player {
        return content_width();
    }
    let width = player::nickname(kit, &req.player.name)
        .map(|(_, value)| value)
        .unwrap_or(0.0);
    let panel = player::panel_rect(width);
    let left = CARD_WIDTH - player::CALIB_WIDTH * player::PLAYER_BAR_SCALE
        + panel.left() * player::PLAYER_BAR_SCALE;
    left - TITLE_GAP - MARGIN
}

pub fn fit_one_line(
    kit: &CanvasKit<'_>,
    family: &str,
    size: f32,
    value: &str,
    max_width: f32,
) -> (String, f32) {
    if value.is_empty() {
        return (value.to_string(), size);
    }
    let style = sketch::text_style(family, size, SCORE_INK);
    if sketch::text_width(kit, &style, value) <= max_width {
        return (value.to_string(), size);
    }
    let mut k = 0.95_f32;
    while k >= 0.7 - 1e-4 {
        let small = size * k;
        let probe = sketch::text_style(family, small, SCORE_INK);
        if sketch::text_width(kit, &probe, value) <= max_width {
            return (value.to_string(), small);
        }
        k -= 0.05;
    }
    let small = size * 0.7;
    let probe = sketch::text_style(family, small, SCORE_INK);
    (sketch::ellipsize(kit, &probe, value, max_width), small)
}

fn gray_matrix() -> [f32; 20] {
    let r = 0.7 * 0.299 + 0.3;
    let g = 0.7 * 0.587;
    let b = 0.7 * 0.114;
    [
        r, g, b, 0.0, 0.0, r, g, b, 0.0, 0.0, r, g, b, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

fn small_background(
    kit: &CanvasKit<'_>,
    root: &Path,
    song_id: &str,
    width: f32,
    height: f32,
) -> Option<Image> {
    let small_w = (width / 8.0).max(1.0).round() as i32;
    let small_h = (height / 8.0).max(1.0).round() as i32;
    let info = ImageInfo::new(
        (small_w, small_h),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface = surfaces::raster(&info, None, None)?;
    let canvas = surface.canvas();
    canvas.clear(BASE_INK.to_skia());
    let key = super::blur_key(root, song_id);
    if let Some(texture) = memo::texture(kit, &key) {
        if let Some(image) = texture.image() {
            let area = Rect::from_xywh(0.0, 0.0, small_w as f32, small_h as f32);
            let source = aurorix_render::cover_source_rect(image, area);
            let sampling =
                aurorix_render::sampling_for((source.width(), source.height()), area);
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            let filter = aurorix_render::skia_safe::color_filters::matrix_row_major(&gray_matrix(), None);
            paint.set_color_filter(filter);
            let radius = (GRAY_BLUR / 8.0).trunc();
            if let Some(filter) = aurorix_render::skia_safe::image_filters::blur(
                (sketch::blur_sigma(radius), sketch::blur_sigma(radius)),
                aurorix_render::skia_safe::TileMode::Decal,
                None,
                None,
            ) {
                paint.set_image_filter(filter);
            }
            canvas.draw_image_rect_with_sampling_options(
                image,
                Some((&source, aurorix_render::skia_safe::canvas::SrcRectConstraint::Fast)),
                area,
                sampling,
                &paint,
            );
        }
    }
    let row = small_w as usize * 4;
    let mut data = vec![0u8; row * small_h as usize];
    if !surface.read_pixels(&info, &mut data, row, (0, 0)) {
        return None;
    }
    images::raster_from_data(&info, Data::new_copy(&data), row)
}

pub fn background_image(
    kit: &CanvasKit<'_>,
    root: &Path,
    song_id: &str,
    width: f32,
    height: f32,
) -> Option<Image> {
    let w = width.round().max(1.0) as i32;
    let h = height.round().max(1.0) as i32;
    let info = ImageInfo::new((w, h), ColorType::RGBA8888, AlphaType::Premul, None);
    let mut surface = surfaces::raster(&info, None, None)?;
    let canvas = surface.canvas();
    canvas.clear(BASE_INK.to_skia());
    if !song_id.is_empty() {
        if let Some(small) = small_background(kit, root, song_id, width, height) {
            let area = Rect::from_xywh(0.0, 0.0, w as f32, h as f32);
            canvas.draw_image_rect_with_sampling_options(
                &small,
                None,
                area,
                aurorix_render::sampling_nearest(),
                &Paint::default(),
            );
        }
    }
    let area = Rect::from_xywh(0.0, 0.0, w as f32, h as f32);
    canvas.draw_rect(area, &solid_paint(BASE_TINT));
    canvas.draw_rect(area, &solid_paint(PAGE_SHADE));
    let row = w as usize * 4;
    let mut data = vec![0u8; row * h as usize];
    if !surface.read_pixels(&info, &mut data, row, (0, 0)) {
        return None;
    }
    images::raster_from_data(&info, Data::new_copy(&data), row)
}

fn solid_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint
}

fn card_backdrop(kit: &CanvasKit<'_>, background: Option<&Image>, canvas: Rect, card: Rect) {
    let area = sketch::ph_rect(card.left(), card.top(), card.width(), card.height());
    if let Some(image) = background {
        let bounds = Rect::from_ltrb(
            area.left() - 24.0,
            area.top() - 24.0,
            area.right() + 24.0,
            area.bottom() + 24.0,
        );
        kit.canvas().save();
        kit.canvas()
            .clip_rrect(sketch::round_rect(area, CARD_RADIUS), ClipOp::Intersect, true);
        sketch::blur_layer(kit, bounds, CARD_BLUR, |inner| {
            sketch::image_nearest(inner, image, canvas);
        });
        kit.canvas().restore();
    }
    sketch::fill_round(kit, area, CARD_RADIUS, CARD_INK);
}

fn grade_image(kit: &CanvasKit<'_>, root: &Path, frame: Rect, played: bool, grade: &str) {
    if !played || grade.is_empty() {
        return;
    }
    let Some(name) = memo::grade_name(grade) else {
        return;
    };
    let key = super::ui_asset(root, name);
    let Some(texture) = memo::texture(kit, &key) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let area = sketch::ph_rect(frame.left(), frame.top(), frame.width(), frame.height());
    let target = aurorix_render::contain_destination_rect(image, area);
    sketch::image_fill(kit, image, target);
}

fn draw_line(kit: &CanvasKit<'_>, x0: f32, x1: f32, y: f32) {
    if x1 <= x0 {
        return;
    }
    let line = sketch::ph_rect(x0, y, x1 - x0, 1.0);
    sketch::ph_fill(kit, line, HAIRLINE_INK);
}

fn draw_tier(kit: &CanvasKit<'_>, root: &Path, card: Rect, level: &SongInfoLevel) {
    let name = tier_name(level);
    let (name_x, name_base) = tier_name_pos(card);
    let name_style = sketch::text_style(sketch::FONT_DISPLAY, TIER_FONT_SIZE, SCORE_INK);
    sketch::draw_text(kit, &name_style, &name, name_x, name_base, SCORE_INK);
    if level.played {
        let width = sketch::text_width(kit, &name_style, &name).round();
        let meta = format!(
            "RKS {}  ·  {}",
            sketch::two_decimals(level.rks),
            sketch::acc_percent(level.acc)
        );
        let meta_style = sketch::text_style(sketch::FONT_BODY, SUBVALUE_FONT_SIZE, NOTE_INK);
        sketch::draw_text(
            kit,
            &meta_style,
            &meta,
            name_x + width + TIER_GAP,
            name_base,
            NOTE_INK,
        );
        let score_style = sketch::text_style(sketch::FONT_DISPLAY, SCORE_FONT_SIZE, SCORE_INK);
        sketch::text_right(
            kit,
            &score_style,
            &score_text(level.score),
            score_right(card),
            score_baseline(card),
            SCORE_INK,
        );
        grade_image(kit, root, grade_box(card), true, &level.grade);
    } else {
        let note = if level.has { NOT_PLAYED_TEXT } else { NO_CHART_TEXT };
        let note_style = sketch::text_style(sketch::FONT_BODY, HINT_FONT_SIZE, PLACEHOLDER_INK);
        sketch::text_right(
            kit,
            &note_style,
            note,
            score_right(card),
            score_baseline(card),
            PLACEHOLDER_INK,
        );
    }

    let (note_x, note_base) = history_head_y(card);
    if level.history.is_empty() {
        let style = sketch::text_style(sketch::FONT_BODY, HINT_FONT_SIZE, PLACEHOLDER_INK);
        sketch::draw_text(kit, &style, NO_HISTORY_TEXT, note_x, note_base, PLACEHOLDER_INK);
        return;
    }
    let head_style = sketch::text_style(sketch::FONT_BODY, HEAD_FONT_SIZE, PLACEHOLDER_INK);
    let head = row_slots(card, 0);
    sketch::draw_text(kit, &head_style, "时间", head.date.0, note_base, PLACEHOLDER_INK);
    sketch::text_right(kit, &head_style, "得分", head.score.0, note_base, PLACEHOLDER_INK);
    let grade_at = head_grade_pos(card);
    sketch::text_center(kit, &head_style, "评级", grade_at.0, note_base, PLACEHOLDER_INK);
    sketch::text_right(kit, &head_style, "ACC", head.acc_percent.0, note_base, PLACEHOLDER_INK);
    sketch::text_right(kit, &head_style, "RKS", head.rks.0, note_base, PLACEHOLDER_INK);
    let first = row_rect(card, 0);
    draw_line(kit, first.left(), first.right(), head_line_y(card));
    let count = history_count_text(level);
    let (count_x, count_base) = count_pos(card);
    let count_style = sketch::text_style(sketch::FONT_BODY, HEAD_FONT_SIZE, LABEL_INK);
    sketch::text_right(kit, &count_style, &count, count_x, count_base, LABEL_INK);

    for (index, point) in level.history.iter().enumerate() {
        let at = row_slots(card, index);
        let stamp = point.at.format("%Y-%m-%d %H:%M").to_string();
        let date_style = sketch::text_style(sketch::FONT_BODY, DATE_FONT_SIZE, NOTE_INK);
        sketch::draw_text(kit, &date_style, &stamp, at.date.0, at.date.1, NOTE_INK);
        if index == 0 {
            let width = sketch::text_width(kit, &date_style, &stamp).round();
            sketch::draw_text(
                kit,
                &date_style,
                LATEST_TEXT,
                at.date.0 + width + LATEST_GAP,
                at.date.1,
                PLACEHOLDER_INK,
            );
        }
        let score_style = sketch::text_style(sketch::FONT_DISPLAY, ROW_FONT_SIZE, SCORE_INK);
        sketch::text_right(
            kit,
            &score_style,
            &score_text(point.score),
            at.score.0,
            at.score.1,
            SCORE_INK,
        );
        grade_image(kit, root, at.grade, true, &point.grade);
        let small_style = sketch::text_style(sketch::FONT_BODY, DATE_FONT_SIZE, NOTE_INK);
        let acc = sketch::acc_percent(point.acc);
        sketch::text_right(kit, &small_style, &acc, at.acc_percent.0, at.acc_percent.1, NOTE_INK);
        let rks = sketch::two_decimals(point.rks);
        sketch::text_right(kit, &small_style, &rks, at.rks.0, at.rks.1, LABEL_INK);
        let (delta, ink) = delta_slot(point);
        sketch::text_right(kit, &score_style, &delta, at.delta.0, at.delta.1, ink);
        if index + 1 < level.history.len() {
            let row = row_rect(card, index);
            draw_line(kit, row.left(), row.right(), row_line_y(card, index));
        }
    }
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SongInfoRequest) {
    let rows: Vec<usize> = req.levels.iter().map(|level| level.history.len()).collect();
    let plan = layout(&rows);
    let height = plan.total_height.round();
    let canvas = Rect::from_xywh(0.0, 0.0, CARD_WIDTH, height);
    let background = background_image(kit, root, &req.song_id, CARD_WIDTH, height);
    match background.as_ref() {
        Some(image) => sketch::image_nearest(kit, image, canvas),
        None => {
            sketch::ph_fill(kit, canvas, BASE_INK);
            sketch::ph_fill(kit, canvas, PAGE_SHADE);
        }
    }

    let limit = title_avail_width(kit, req);
    let (title, title_size) =
        fit_one_line(kit, sketch::FONT_DISPLAY, TITLE_FONT_SIZE, &req.title, limit);
    let title_style = sketch::text_style(sketch::FONT_DISPLAY, title_size, SCORE_INK);
    sketch::draw_text(kit, &title_style, &title, MARGIN, TITLE_BASELINE, SCORE_INK);
    if !req.composer.is_empty() {
        let (composer, composer_size) =
            fit_one_line(kit, sketch::FONT_BODY, SUBTITLE_FONT_SIZE, &req.composer, limit);
        let composer_style = sketch::text_style(sketch::FONT_BODY, composer_size, LABEL_INK);
        sketch::draw_text(kit, &composer_style, &composer, MARGIN, SUBTITLE_BASELINE, LABEL_INK);
    }

    player::draw(
        kit,
        root,
        CARD_WIDTH,
        player::PLAYER_BAR_SCALE,
        PLAYER_BAR_OFFSET,
        &req.player,
    );

    if req.levels.is_empty() {
        let card = card_rect(&plan, 0);
        card_backdrop(kit, background.as_ref(), canvas, card);
        let (x, base) = history_head_y(card);
        let style = sketch::text_style(sketch::FONT_BODY, HINT_FONT_SIZE, PLACEHOLDER_INK);
        sketch::draw_text(kit, &style, EMPTY_CARD_TEXT, x, base, PLACEHOLDER_INK);
    }
    for (index, level) in req.levels.iter().enumerate() {
        let card = card_rect(&plan, index);
        card_backdrop(kit, background.as_ref(), canvas, card);
        draw_tier(kit, root, card, level);
    }

    let footer_style = sketch::text_style(sketch::FONT_BODY, HINT_FONT_SIZE, PLACEHOLDER_INK);
    sketch::draw_text(
        kit,
        &footer_style,
        FOOTER_TEXT,
        MARGIN,
        footer_baseline(&plan),
        PLACEHOLDER_INK,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_height_scales_with_rows() {
        assert_eq!(card_height(0), 132.0);
        assert_eq!(card_height(1), 167.0);
        assert_eq!(card_height(3), 237.0);
    }

    #[test]
    fn layout_stacks_tiers_and_drops_last_gap() {
        let plan = layout(&[1, 2, 0, 4]);
        assert_eq!(plan.tops, vec![142.0, 329.0, 551.0, 703.0]);
        assert_eq!(plan.heights, vec![167.0, 202.0, 132.0, 272.0]);
        assert_eq!(plan.last_bottom, 975.0);
        assert_eq!(plan.total_height, 1041.0);
    }

    #[test]
    fn empty_levels_still_render_one_card() {
        let plan = layout(&[]);
        assert_eq!(plan.tops, vec![142.0]);
        assert_eq!(plan.heights, vec![168.0]);
        assert_eq!(plan.total_height, 142.0 + 168.0 + 66.0);
    }

    #[test]
    fn tier_name_includes_constant_with_one_decimal() {
        let level = SongInfoLevel {
            diff: String::from("IN"),
            constant: 14.9,
            has: true,
            played: true,
            score: 1000000,
            acc: 100.0,
            grade: String::from("PHI"),
            rks: 16.1,
            combo: 1200,
            history: Vec::new(),
            changes: 0,
        };
        assert_eq!(tier_name(&level), "IN  14.9");
        let missing = SongInfoLevel {
            diff: String::from("AT"),
            has: false,
            ..level
        };
        assert_eq!(tier_name(&missing), "AT");
    }

    #[test]
    fn history_count_and_score_text() {
        let level = SongInfoLevel {
            diff: String::from("IN"),
            constant: 15.0,
            has: true,
            played: true,
            score: 987123,
            acc: 99.12,
            grade: String::from("V"),
            rks: 15.2,
            combo: 900,
            history: vec![
                SongInfoPoint {
                    at: Utc::now(),
                    score: 1,
                    acc: 1.0,
                    rks: 1.0,
                    grade: String::from("V"),
                    delta: 1,
                    first: false,
                },
                SongInfoPoint {
                    at: Utc::now(),
                    score: 2,
                    acc: 1.0,
                    rks: 1.0,
                    grade: String::from("V"),
                    delta: -5,
                    first: false,
                },
            ],
            changes: 12,
        };
        assert_eq!(score_text(987123), "987'123");
        assert_eq!(history_count_text(&level), "共 12 次 · 列最近 2 次");
        let mut short = level.clone();
        short.changes = 2;
        assert_eq!(history_count_text(&short), "共 2 次");
    }

    #[test]
    fn delta_column_marks_new_and_colors() {
        let point = SongInfoPoint {
            at: Utc::now(),
            score: 100,
            acc: 100.0,
            rks: 1.0,
            grade: String::from("V"),
            delta: 0,
            first: true,
        };
        assert_eq!(delta_slot(&point).0, "NEW");
        let gain = SongInfoPoint {
            first: false,
            delta: 1200,
            ..point.clone()
        };
        assert_eq!(delta_slot(&gain), (String::from("+1'200"), GAIN_INK));
        let drop = SongInfoPoint {
            first: false,
            delta: -300,
            ..point
        };
        assert_eq!(delta_slot(&drop), (String::from("-300"), DROP_INK));
    }

    #[test]
    fn head_and_row_slots_align_right() {
        let card = card_rect(&layout(&[2]), 0);
        let row = row_rect(card, 1);
        let at = row_slots(card, 1);
        assert_eq!(at.score.0, row.right() - 415.0);
        assert_eq!(at.acc_percent.0, row.right() - 245.0);
        assert_eq!(at.rks.0, row.right() - 150.0);
        assert_eq!(at.delta.0, row.right());
        assert_eq!(at.grade.left(), row.right() - 350.0);
        assert_eq!(at.date.0, card_inner(card).left());
        assert_eq!(at.score.1, row.top() + row.height() * 0.66);
        assert_eq!(head_line_y(card), card.top() + BAND_HEIGHT + HEAD_HEIGHT);
        assert_eq!(row_line_y(card, 0), row.top());
    }
}
