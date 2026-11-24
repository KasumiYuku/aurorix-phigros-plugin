use std::path::Path;
use std::sync::Arc;

use aurorix_render::skia_safe::{
    color_filters, image_filters, images, paint, surfaces, AlphaType, ColorType, Data, ImageInfo,
    Paint, PathBuilder, Rect, TileMode,
};
use aurorix_render::{Color, CanvasKit, GradientStop, LinearGradient, SharedImage};
use chrono::{DateTime, Datelike, TimeDelta, Utc};

use super::{memo, player, sketch};

pub const CANVAS_WIDTH: f32 = 1920.0;

pub const CANVAS_HEIGHT: f32 = 886.0;

pub const LEAN: f32 = player::SKEW;

pub const MID_RATIO: f32 = player::MID_RATIO;

pub const SIZE_TITLE: f32 = 56.0;

pub const SIZE_LEVEL: f32 = 29.0;

pub const SIZE_SCORE: f32 = 73.0;

pub const SIZE_HISTORY: f32 = 24.0;

pub const SIZE_VALUE: f32 = 44.0;

pub const SIZE_LABEL: f32 = 22.0;

pub const SIZE_SCORE_CURVE_LABEL: f32 = 14.0;

pub const PANEL_INK: Color = Color::new(0x08, 0x08, 0x0C, 0x89);

pub const COVER_FALLBACK: Color = Color::new(0x22, 0x26, 0x30, 0xFF);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const LABEL_INK: Color = Color::new(0xB4, 0xBA, 0xC6, 0xFF);

pub const TITLE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const PLACEHOLDER_INK: Color = Color::new(0x8A, 0x92, 0xA2, 0xFF);

pub const SHADE_INK: Color = Color::new(0x00, 0x00, 0x00, 0xFF);

pub const RANK_ACCENT: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

pub const NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const BACKDROP_TINT: Color = Color::new(0x0A, 0x0A, 0x10, 0x7F);

pub const BUTTON_INK: Color = Color::new(0x00, 0x00, 0x00, 0xFF);

pub const BUTTON_EDGE: Color = Color::new(0xFF, 0xFF, 0xFF, 0xF0);

pub const SCORE_CURVE_AREA_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0x2E);

pub const SCORE_CURVE_AREA_FADE_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0x00);

pub const NOT_PLAYED_TEXT: &str = "尚未游玩";

pub const SINGLE_RKS_SEP: &str = " > ";

pub const MISSING_TEXT: &str = "—";

pub const SEG_SEPARATOR: &str = "   ·   ";

pub const SCORE_HISTORY_GAIN_LABEL: &str = "累计";

pub const SCORE_HISTORY_NOTE_LABEL: &str = "成绩变化";

pub const NO_HISTORY_NOTE: &str = "还没有记录";

pub const TITLE_MAX_WIDTH: f32 = 600.0;

pub const STAT_CELL_PAD: f32 = 47.0;

pub const STAT_VALUE_LIFT: f32 = 2.0;

pub const STAT_LABEL_DROP: f32 = 27.5;

pub const SCORE_CURVE_TEXT_PAD: f32 = 6.0;

pub const SCORE_CURVE_LINE_W: f32 = 1.0;

pub const GAINED_PAD: f32 = 34.0;

pub const GAINED_COL_W: f32 = 120.0;

pub const GAINED_COL_GAP: f32 = 20.0;

pub const BUTTON_BLEED: f32 = 14.0;

pub const SCORE_HISTORY_DOT_MAX: usize = 24;

pub const SCORE_HISTORY_PAD_RATIO: f64 = 0.18;

pub const LINE_MIN_SPAN: f64 = 2.0;

pub const PANEL_EDGE_LEFT: f32 = 1160.5;

pub const PANEL_EDGE_RIGHT: f32 = 1806.8;

pub const BACKDROP_DIVISOR: f32 = 8.0;

pub const BACKDROP_BLUR: f32 = 16.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextAt {
    pub x: f32,
    pub baseline: f32,
    pub size: f32,
    pub align: Align,
    pub font: Option<&'static str>,
}

impl TextAt {
    pub const fn left(x: f32, baseline: f32, size: f32) -> Self {
        Self {
            x,
            baseline,
            size,
            align: Align::Left,
            font: None,
        }
    }

    pub const fn right(x: f32, baseline: f32, size: f32) -> Self {
        Self {
            x,
            baseline,
            size,
            align: Align::Right,
            font: None,
        }
    }

    pub const fn center(x: f32, baseline: f32, size: f32) -> Self {
        Self {
            x,
            baseline,
            size,
            align: Align::Center,
            font: None,
        }
    }

    pub const fn with_font(self, font: &'static str) -> Self {
        Self {
            font: Some(font),
            ..self
        }
    }

    pub fn font_key(&self) -> &'static str {
        self.font.unwrap_or(sketch::FONT_BODY)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Standing {
    pub played: bool,
    pub rks: f64,
    pub rank: i32,
    pub total: i32,
    pub suggest: f64,
    pub suggest_ok: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DifficultyScore {
    pub diff: String,
    pub played: bool,
    pub score: i32,
    pub acc: f64,
    pub grade: String,
    pub fc: bool,
    pub phi: bool,
    pub rks: f64,
    pub rank: i32,
    pub total: i32,
}

impl DifficultyScore {
    pub fn combo_mark(&self) -> &'static str {
        if self.phi {
            "Phi"
        } else if self.fc {
            "FC"
        } else {
            ""
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryPoint {
    pub at: DateTime<Utc>,
    pub score: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChartHistory {
    pub points: Vec<HistoryPoint>,
    pub gain: i32,
}

#[derive(Clone, Debug, Default)]
pub struct SettlementRequest {
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub played: bool,
    pub score: i32,
    pub acc: f64,
    pub history: i32,
    pub fc: bool,
    pub grade: String,
    pub player: player::Player,
    pub standing: Standing,
    pub scores: Vec<DifficultyScore>,
    pub chart_history: ChartHistory,
    pub gained: String,
}

#[derive(Clone, Debug, Default)]
pub struct CardData {
    pub played: bool,
    pub acc: f64,
    pub grade: String,
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub standing: Standing,
    pub score_line: String,
    pub history_line: String,
}

impl SettlementRequest {
    pub fn cards(&self) -> CardData {
        CardData {
            played: self.played,
            acc: self.acc,
            grade: self.grade.clone(),
            diff: self.diff.clone(),
            constant: self.constant,
            known: self.known,
            standing: self.standing,
            score_line: String::new(),
            history_line: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cards {
    pub score: Rect,
    pub combo_and_acc: Rect,
    pub more_info: Rect,
    pub ratings: Rect,
    pub score_text: TextAt,
    pub history_text: TextAt,
    pub backs: [Rect; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatSlots {
    pub left_value: TextAt,
    pub left_label: TextAt,
    pub right_value: TextAt,
    pub right_label: TextAt,
}

pub fn canvas_size() -> (i32, i32) {
    (CANVAS_WIDTH as i32, CANVAS_HEIGHT as i32)
}

pub fn content() -> Rect {
    Rect::from_xywh(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT)
}

pub fn cover() -> Rect {
    player::from_scene(content(), player::CENTER, -357.0, 1.0, 832.0, 624.0)
}

pub fn score_box() -> Rect {
    player::from_scene(content(), player::CENTER, 480.8, 156.5, 782.0, 313.0)
}

pub fn combo_and_acc_box() -> Rect {
    player::from_scene(content(), player::CENTER, 412.3, -99.5, 728.0, 112.0)
}

pub fn more_info_box() -> Rect {
    player::from_scene(content(), player::CENTER, 370.5, -255.0, 728.0, 112.0)
}

pub fn panel(area: Rect) -> Rect {
    let cy = area.top() + area.height() / 2.0;
    Rect::from_xywh(
        PANEL_EDGE_LEFT - LEAN * cy,
        area.top(),
        PANEL_EDGE_RIGHT - PANEL_EDGE_LEFT,
        area.height(),
    )
}

pub fn score_panel() -> Rect {
    panel(score_box())
}

pub fn combo_and_acc_panel() -> Rect {
    panel(combo_and_acc_box())
}

pub fn more_info_panel() -> Rect {
    panel(more_info_box())
}

pub fn ratings() -> Rect {
    player::from_scene(
        score_box(),
        player::TOP_RIGHT_CENTERED,
        -180.1,
        -153.6,
        219.9,
        232.8,
    )
}

pub fn songs_name() -> TextAt {
    TextAt::left(219.0, 693.0, SIZE_TITLE)
}

pub fn level_at() -> TextAt {
    TextAt::right(926.0, 703.3, SIZE_LEVEL)
}

pub fn score_text_at() -> TextAt {
    TextAt::left(1106.2, 367.6, SIZE_SCORE)
}

pub fn history_text_at() -> TextAt {
    TextAt::left(1113.8, 406.8, SIZE_HISTORY)
}

pub fn cover_shade() -> Rect {
    player::from_scene(cover(), player::CENTER, 0.0, -302.0, 1183.3, 221.1)
}

pub fn settle_cards() -> Cards {
    Cards {
        score: score_panel(),
        combo_and_acc: combo_and_acc_panel(),
        more_info: more_info_panel(),
        ratings: ratings(),
        score_text: score_text_at(),
        history_text: history_text_at(),
        backs: [score_panel(), combo_and_acc_panel(), more_info_panel()],
    }
}

pub fn stat_text(area: Rect) -> StatSlots {
    let cy = area.top() + area.height() / 2.0;
    let value_base = cy + STAT_VALUE_LIFT;
    let label_base = cy + STAT_LABEL_DROP;
    let dy = value_base - cy;
    let left = area.left() - LEAN * dy + STAT_CELL_PAD;
    let right = area.right() - LEAN * dy - STAT_CELL_PAD;
    StatSlots {
        left_value: TextAt::left(left, value_base, SIZE_VALUE),
        left_label: TextAt::left(left, label_base, SIZE_LABEL),
        right_value: TextAt::right(right, value_base, SIZE_VALUE),
        right_label: TextAt::right(right, label_base, SIZE_LABEL),
    }
}

pub fn gained() -> Rect {
    player::from_scene(content(), player::BOTTOM_MID, 0.0, 80.0, 620.0, 84.0)
}

pub fn cover_image_rect(img_w: f32, img_h: f32) -> Rect {
    let win = cover();
    if img_w <= 0.0 || img_h <= 0.0 {
        return win;
    }
    let width = win.height() * img_w / img_h;
    Rect::from_xywh(
        win.left() + (win.width() - width) / 2.0,
        win.top(),
        width,
        win.height(),
    )
}

pub fn retry() -> Rect {
    Rect::from_xywh(0.0, 10.0, 140.0, 97.0)
}

pub fn back() -> Rect {
    Rect::from_xywh(
        CANVAS_WIDTH - 142.0,
        CANVAS_HEIGHT - 97.0 - 11.0,
        142.0,
        97.0,
    )
}

pub fn retry_icon() -> Rect {
    player::from_scene(retry(), player::TOP_LEFT_CENTERED, 70.0, -56.0, 48.0, 47.0)
}

pub fn back_icon() -> Rect {
    player::from_scene(
        back(),
        player::BOTTOM_RIGHT_CENTERED,
        -70.0,
        56.0,
        41.0,
        41.0,
    )
}

pub fn gained_curve_top() -> f32 {
    gained().top() + 8.0
}

pub fn gained_curve_bottom() -> f32 {
    gained().top() + 50.0
}

pub fn gained_note_base() -> f32 {
    gained().top() + 65.0
}

pub fn gained_gain_base() -> f32 {
    gained().top() + 36.0
}

pub fn gained_span_base() -> f32 {
    gained().top() + 56.0
}

pub fn gained_content_left(y: f32) -> f32 {
    let bar = gained();
    bar.left() + sketch::skew_offset(bar, y, LEAN) + GAINED_PAD
}

pub fn gained_content_right(y: f32) -> f32 {
    let bar = gained();
    bar.right() + sketch::skew_offset(bar, y, LEAN) - GAINED_PAD
}

pub fn gained_col_right() -> f32 {
    gained_content_right(gained_span_base())
}

pub fn gained_curve_rect() -> Rect {
    let top = gained_curve_top();
    let left = gained_content_left(top);
    Rect::from_ltrb(
        left,
        top,
        gained_col_right() - GAINED_COL_W - GAINED_COL_GAP,
        gained_curve_bottom(),
    )
}

pub fn gained_note_at() -> TextAt {
    TextAt::left(
        gained_curve_rect().left(),
        gained_note_base(),
        SIZE_SCORE_CURVE_LABEL,
    )
}

pub fn gained_gain_at() -> TextAt {
    TextAt::right(gained_col_right(), gained_gain_base(), SIZE_LABEL)
}

pub fn gained_span_at() -> TextAt {
    TextAt::right(
        gained_col_right(),
        gained_span_base(),
        SIZE_SCORE_CURVE_LABEL,
    )
}

pub fn pad_score(value: i32) -> String {
    format!("{:07}", value.max(0))
}

pub fn score_text(score: i32) -> String {
    pad_score(score)
}

pub fn history_text(score: i32, history: i32) -> String {
    let delta = score - history;
    let sign = if delta < 0 { "-" } else { "+" };
    format!(
        "NEW BEST {} {}{}",
        pad_score(history),
        sign,
        pad_score(delta.abs())
    )
}

pub fn acc_text(acc: f64) -> String {
    sketch::acc_percent(acc)
}

pub fn rks_text2(value: f64) -> String {
    sketch::two_decimals(value)
}

pub fn const_text1(value: f64) -> String {
    sketch::one_decimal(value)
}

pub fn const_text(value: f64) -> String {
    if value == value.trunc() && value.abs() < 1e9 {
        return format!("{value:.0}");
    }
    format!("{value:.1}")
}

pub fn level_text(diff: &str, constant: f64, known: bool) -> String {
    if !known {
        if diff.is_empty() {
            return String::from("未收录");
        }
        return format!("{diff} 未收录");
    }
    format!("{diff} Lv.{}", const_text(constant))
}

pub fn level_constant(data: &CardData) -> String {
    if !data.known {
        return String::from("未收录");
    }
    format!("{} {}", data.diff, const_text1(data.constant))
}

pub fn rank_place_text(place: i32, total: i32) -> String {
    if place <= 0 || total <= 0 {
        return String::new();
    }
    format!("#{place} / {total}")
}

pub fn rank_text(standing: Standing) -> String {
    rank_place_text(standing.rank, standing.total)
}

pub fn gain_text(gain: i32) -> String {
    format!("+{}", sketch::group_digits(&gain.max(0).to_string()))
}

pub fn played_text(ok: bool, value: &str) -> String {
    if ok {
        value.to_string()
    } else {
        String::from(MISSING_TEXT)
    }
}

pub fn stat_ink(ok: bool) -> Color {
    if ok {
        SCORE_INK
    } else {
        PLACEHOLDER_INK
    }
}

pub fn stat_label_ink(ok: bool) -> Color {
    if ok {
        LABEL_INK
    } else {
        PLACEHOLDER_INK
    }
}

pub fn history_days(points: &[HistoryPoint]) -> i64 {
    if points.len() < 2 {
        return 0;
    }
    let delta = points[points.len() - 1].at - points[0].at;
    if delta <= TimeDelta::zero() {
        return 0;
    }
    (delta.num_hours() as f64 / 24.0 + 0.5) as i64
}

pub fn history_span(points: &[HistoryPoint]) -> String {
    let Some(first) = points.first() else {
        return String::new();
    };
    let last = points[points.len() - 1];
    if points.len() == 1 {
        return first.at.format("%m-%d").to_string();
    }
    if first.at.year() != last.at.year() {
        return format!(
            "{} → {}",
            first.at.format("%Y-%m-%d"),
            last.at.format("%Y-%m-%d")
        );
    }
    format!("{} → {}", first.at.format("%m-%d"), last.at.format("%m-%d"))
}

pub fn history_note_text(history: &ChartHistory) -> (String, String) {
    if history.points.is_empty() {
        return (
            String::from(SCORE_HISTORY_NOTE_LABEL),
            String::from(NO_HISTORY_NOTE),
        );
    }
    let mut parts = vec![format!("共 {} 次", history.points.len())];
    let days = history_days(&history.points);
    if days > 0 {
        parts.push(format!("{days} 天"));
    }
    (
        String::from(SCORE_HISTORY_NOTE_LABEL),
        parts.join(SEG_SEPARATOR),
    )
}

pub fn measure(kit: &CanvasKit<'_>, at: TextAt, value: &str) -> f32 {
    let style = sketch::text_style(at.font_key(), at.size, SCORE_INK);
    sketch::text_width(kit, &style, value)
}

pub fn line(kit: &CanvasKit<'_>, at: TextAt, value: &str, color: Color) {
    if value.is_empty() {
        return;
    }
    let style = sketch::text_style(at.font_key(), at.size, color);
    match at.align {
        Align::Right => {
            sketch::text_right(kit, &style, value, at.x, at.baseline, color);
        }
        Align::Center => {
            sketch::text_center(kit, &style, value, at.x, at.baseline, color);
        }
        Align::Left => {
            sketch::draw_text(kit, &style, value, at.x, at.baseline, color);
        }
    }
}

pub fn grade(kit: &CanvasKit<'_>, root: &Path, frame: Rect, played: bool, grade: &str) {
    if !played || grade.is_empty() {
        return;
    }
    let Some(name) = memo::grade_name(grade) else {
        return;
    };
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let area = sketch::ph_rect(frame.left(), frame.top(), frame.width(), frame.height());
    let target = aurorix_render::contain_destination_rect(image, area);
    sketch::image_fill(kit, image, target);
}

pub fn avatar_image(kit: &CanvasKit<'_>, root: &Path, chosen: &str) -> Option<Arc<SharedImage>> {
    let mut keys: Vec<String> = Vec::with_capacity(2);
    if !chosen.is_empty() {
        keys.push(super::avatar_asset(root, chosen));
    }
    keys.push(super::ui_asset(root, memo::DEFAULT_AVATAR));
    for key in keys {
        if let Some(texture) = memo::texture(kit, &key) {
            if let Some(image) = texture.image() {
                return Some(Arc::new(SharedImage::from_image(image.clone())));
            }
        }
    }
    None
}

fn gray_matrix() -> [f32; 20] {
    let r = 0.7 * 0.299 + 0.3;
    let g = 0.7 * 0.587;
    let b = 0.7 * 0.114;
    [
        r, g, b, 0.0, 0.0, r, g, b, 0.0, 0.0, r, g, b, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]
}

fn shade_matrix() -> [f32; 20] {
    [
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        1.0, 0.0,
    ]
}

pub fn shade_paint() -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color_filter(color_filters::matrix_row_major(&shade_matrix(), None));
    paint
}
pub fn backdrop_key(song_id: &str) -> String {
    format!("backdrop/{song_id}")
}

fn backdrop_image(
    kit: &CanvasKit<'_>,
    root: &Path,
    song_id: &str,
) -> Option<Arc<SharedImage>> {
    let key = backdrop_key(song_id);
    if let Some(hit) = memo::image(&key) {
        return Some(hit);
    }
    let texture = memo::texture(kit, &super::blur_key(root, song_id))?;
    let image = texture.image()?;
    let small_w = (CANVAS_WIDTH / BACKDROP_DIVISOR).round().max(1.0) as i32;
    let small_h = (CANVAS_HEIGHT / BACKDROP_DIVISOR).round().max(1.0) as i32;
    let info = ImageInfo::new(
        (small_w, small_h),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface = surfaces::raster(&info, None, None)?;
    let canvas = surface.canvas();
    canvas.clear(COVER_FALLBACK.to_skia());
    let area = Rect::from_xywh(0.0, 0.0, small_w as f32, small_h as f32);
    let source = aurorix_render::cover_source_rect(image, area);
    let sampling = aurorix_render::sampling_for((source.width(), source.height()), area);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color_filter(color_filters::matrix_row_major(&gray_matrix(), None));
    let radius = (BACKDROP_BLUR / BACKDROP_DIVISOR).trunc();
    if let Some(filter) = image_filters::blur(
        (sketch::blur_sigma(radius), sketch::blur_sigma(radius)),
        TileMode::Decal,
        None,
        None,
    ) {
        paint.set_image_filter(filter);
    }
    canvas.draw_image_rect_with_sampling_options(
        image,
        Some((
            &source,
            aurorix_render::skia_safe::canvas::SrcRectConstraint::Fast,
        )),
        area,
        sampling,
        &paint,
    );
    let row = small_w as usize * 4;
    let mut data = vec![0u8; row * small_h as usize];
    if !surface.read_pixels(&info, &mut data, row, (0, 0)) {
        return None;
    }
    let built = images::raster_from_data(&info, Data::new_copy(&data), row)?;
    Some(memo::keep_image(&key, built))
}

pub fn backdrop(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    sketch::ph_fill(kit, content(), COVER_FALLBACK);
    if !song_id.is_empty() {
        if let Some(image) = backdrop_image(kit, root, song_id) {
            sketch::image_fill(kit, image.image(), content());
        }
    }
    sketch::ph_fill(kit, content(), BACKDROP_TINT);
}

pub fn cover_window(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    let win = cover();
    let dest = sketch::skew_bounds(win, LEAN);
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::illust_key(root, song_id))
    };
    let image = texture.as_ref().and_then(|texture| texture.image());
    match image {
        Some(image) => {
            sketch::clip_skew(kit, win, LEAN, || {
                sketch::image_cover(kit, image, dest);
            });
        }
        None => {
            sketch::fill_skew(kit, win, LEAN, COVER_FALLBACK);
            return;
        }
    }
    let shade_key = super::ui_asset(root, "bar_bottom_color");
    let Some(shade) = memo::texture(kit, &shade_key) else {
        return;
    };
    let Some(shade_image) = shade.image() else {
        return;
    };
    let band = cover_shade();
    let shade_dest = Rect::from_ltrb(dest.left(), band.top(), dest.right(), band.bottom());
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color_filter(color_filters::matrix_row_major(&shade_matrix(), None));
    sketch::clip_skew(kit, win, LEAN, || {
        sketch::image_cover_paint(kit, shade_image, shade_dest, &paint);
    });
}

pub fn texts(kit: &CanvasKit<'_>, req: &SettlementRequest) {
    let style = sketch::text_style(sketch::FONT_BODY, SIZE_TITLE, TITLE_INK);
    let title = sketch::ellipsize(kit, &style, &req.title, TITLE_MAX_WIDTH);
    line(kit, songs_name(), &title, TITLE_INK);
    let ink = if req.played { SCORE_INK } else { PLACEHOLDER_INK };
    line(
        kit,
        level_at(),
        &level_text(&req.diff, req.constant, req.known),
        ink,
    );
}

pub fn draw_cards(kit: &CanvasKit<'_>, root: &Path, g: &Cards, d: &CardData) {
    for back in g.backs.iter() {
        sketch::fill_skew(kit, *back, LEAN, PANEL_INK);
    }
    grade(kit, root, g.ratings, d.played, &d.grade);
    let value_ink = if d.played { SCORE_INK } else { PLACEHOLDER_INK };
    line(kit, g.score_text, &d.score_line, value_ink);
    if !d.history_line.is_empty() {
        line(kit, g.history_text, &d.history_line, LABEL_INK);
    }

    let up = stat_text(g.combo_and_acc);
    let label = format!("{}{}", level_constant(d), SINGLE_RKS_SEP);
    line(kit, up.left_value, &label, LABEL_INK);
    let mut rks_at = up.left_value;
    rks_at.x += measure(kit, up.left_value, &label).round();
    line(
        kit,
        rks_at,
        &played_text(d.standing.played, &rks_text2(d.standing.rks)),
        stat_ink(d.standing.played),
    );
    line(kit, up.left_label, "RKS", stat_label_ink(d.standing.played));
    line(
        kit,
        up.right_value,
        &played_text(d.played, &acc_text(d.acc)),
        stat_ink(d.played),
    );
    line(kit, up.right_label, "Accuracy", stat_label_ink(d.played));

    let down = stat_text(g.more_info);
    line(
        kit,
        down.left_value,
        &played_text(d.standing.suggest_ok, &acc_text(d.standing.suggest)),
        stat_ink(d.standing.suggest_ok),
    );
    line(
        kit,
        down.left_label,
        "Target ACC",
        stat_label_ink(d.standing.suggest_ok),
    );
    line(
        kit,
        down.right_value,
        &played_text(d.standing.rank > 0, &rank_text(d.standing)),
        stat_ink(d.standing.rank > 0),
    );
    line(
        kit,
        down.right_label,
        "Rank",
        stat_label_ink(d.standing.rank > 0),
    );
}

fn line_range(lo: f64, hi: f64) -> (f64, f64) {
    if hi > lo {
        return (lo, hi);
    }
    let mut mid = lo;
    if !mid.is_finite() {
        mid = 0.0;
    }
    (mid - LINE_MIN_SPAN / 2.0, mid + LINE_MIN_SPAN / 2.0)
}

fn curve_bounds(points: &[HistoryPoint]) -> (f64, f64) {
    let mut lo = points[0].score as f64;
    let mut hi = lo;
    for point in points {
        let value = point.score as f64;
        if value < lo {
            lo = value;
        }
        if value > hi {
            hi = value;
        }
    }
    let pad = (hi - lo) * SCORE_HISTORY_PAD_RATIO;
    (lo - pad, hi + pad)
}

fn curve_points(points: &[HistoryPoint], area: Rect, lo: f64, hi: f64) -> Vec<(f32, f32)> {
    let t0 = points[0].at;
    let span = points[points.len() - 1].at - t0;
    let mut out: Vec<(f32, f32)> = Vec::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        let mut x = area.left();
        if span > TimeDelta::zero() {
            let progress = (point.at - t0).num_seconds() as f64 / span.num_seconds() as f64;
            x += (progress * area.width() as f64) as f32;
        } else if points.len() > 1 {
            x += index as f32 / (points.len() - 1) as f32 * area.width();
        }
        let value = point.score as f64;
        let ratio = if hi > lo {
            (value - lo) / (hi - lo)
        } else {
            0.5
        };
        let y = area.bottom() - (ratio as f32) * area.height();
        out.push((x, y));
    }
    out
}

fn curve_path(points: &[(f32, f32)]) -> PathBuilder {
    let mut builder = PathBuilder::new();
    builder.move_to(points[0]);
    for point in points.iter().skip(1) {
        builder.line_to(*point);
    }
    builder
}

fn draw_curve(kit: &CanvasKit<'_>, area: Rect, points: &[HistoryPoint]) {
    let (lo, hi) = curve_bounds(points);
    let (lo, hi) = line_range(lo, hi);
    let mapped = curve_points(points, area, lo, hi);
    let first = mapped[0];
    let last = mapped[mapped.len() - 1];
    let mut area_builder = curve_path(&mapped);
    area_builder.line_to((last.0, area.bottom()));
    area_builder.line_to((first.0, area.bottom()));
    area_builder.close();
    let area_path = area_builder.detach();
    let mut fill = Paint::default();
    fill.set_anti_alias(true);
    match LinearGradient::vertical([
        GradientStop::new(0.0, SCORE_CURVE_AREA_INK),
        GradientStop::new(1.0, SCORE_CURVE_AREA_FADE_INK),
    ])
    .shader(area)
    {
        Ok(shader) => {
            fill.set_shader(shader);
        }
        Err(_) => {
            fill.set_color(SCORE_CURVE_AREA_INK.to_skia());
        }
    }
    sketch::clip_skew(kit, gained(), LEAN, || {
        kit.canvas().draw_path(&area_path, &fill);
    });

    let line_path = curve_path(&mapped).detach();
    let mut stroke = Paint::default();
    stroke.set_anti_alias(true);
    stroke.set_style(paint::Style::Stroke);
    stroke.set_stroke_width(SCORE_CURVE_LINE_W.max(1.0));
    stroke.set_color(RANK_ACCENT.to_skia());
    kit.canvas().draw_path(&line_path, &stroke);

    if mapped.len() <= SCORE_HISTORY_DOT_MAX {
        for (x, y) in &mapped {
            sketch::ph_fill(
                kit,
                sketch::ph_rect(x.round() - 2.0, y.round() - 2.0, 5.0, 5.0),
                RANK_ACCENT,
            );
        }
    }
}

fn gained_note(kit: &CanvasKit<'_>, history: &ChartHistory) {
    let at = gained_note_at();
    let (label, note) = history_note_text(history);
    line(kit, at, &label, LABEL_INK);
    if note.is_empty() {
        return;
    }
    let mut moved = at;
    moved.x += measure(kit, at, &label).round() + SCORE_CURVE_TEXT_PAD;
    line(kit, moved, &note, PLACEHOLDER_INK);
}

fn gained_curve(kit: &CanvasKit<'_>, history: &ChartHistory) {
    let area = gained_curve_rect();
    gained_note(kit, history);
    if history.points.len() >= 2 {
        draw_curve(kit, area, &history.points);
        return;
    }
    let thick = SCORE_CURVE_LINE_W.round().max(1.0);
    let y = area.top() + area.height() / 2.0;
    sketch::ph_fill(
        kit,
        sketch::ph_rect(area.left(), y, area.width(), thick),
        RANK_ACCENT,
    );
    if history.points.len() == 1 {
        sketch::ph_fill(
            kit,
            sketch::ph_rect(area.left() + area.width() / 2.0 - 2.0, y - 2.0, 5.0, 5.0),
            RANK_ACCENT,
        );
    }
}

fn gained_stats(kit: &CanvasKit<'_>, history: &ChartHistory) {
    let at = gained_gain_at();
    let value = gain_text(history.gain);
    let mut label = at;
    label.size = SIZE_SCORE_CURVE_LABEL;
    label.x -= measure(kit, at, &value).round() + SCORE_CURVE_TEXT_PAD;
    line(kit, label, SCORE_HISTORY_GAIN_LABEL, PLACEHOLDER_INK);
    line(kit, at, &value, SCORE_INK);
    let span = history_span(&history.points);
    if !span.is_empty() {
        line(kit, gained_span_at(), &span, PLACEHOLDER_INK);
    }
}

pub fn gained_bar(kit: &CanvasKit<'_>, history: &ChartHistory) {
    let bar = gained();
    sketch::fill_skew(kit, bar, LEAN, PANEL_INK);
    let edge = 6.0;
    let inset = 12.0;
    sketch::fill_skew(
        kit,
        Rect::from_xywh(
            bar.left() + inset - edge / 2.0,
            bar.top(),
            edge,
            bar.height(),
        ),
        LEAN,
        BUTTON_EDGE,
    );
    sketch::fill_skew(
        kit,
        Rect::from_xywh(
            bar.right() - inset - edge / 2.0,
            bar.top(),
            edge,
            bar.height(),
        ),
        LEAN,
        BUTTON_EDGE,
    );
    gained_curve(kit, history);
    gained_stats(kit, history);
}

pub fn buttons(kit: &CanvasKit<'_>, root: &Path) {
    let edge = 9.0;
    for (area, icon_area, key, left) in [
        (retry(), retry_icon(), "retry", true),
        (back(), back_icon(), "back_in_result", false),
    ] {
        let body = if left {
            Rect::from_ltrb(
                area.left() - BUTTON_BLEED,
                area.top(),
                area.right(),
                area.bottom(),
            )
        } else {
            Rect::from_ltrb(
                area.left(),
                area.top(),
                area.right() + BUTTON_BLEED,
                area.bottom(),
            )
        };
        sketch::fill_skew(kit, body, LEAN, BUTTON_INK);
        let stripe = if left {
            Rect::from_ltrb(area.right() - edge, area.top(), area.right(), area.bottom())
        } else {
            Rect::from_ltrb(area.left(), area.top(), area.left() + edge, area.bottom())
        };
        sketch::fill_skew(kit, stripe, LEAN, BUTTON_EDGE);
        let Some(texture) = memo::texture(kit, &super::ui_asset(root, key)) else {
            continue;
        };
        let Some(image) = texture.image() else {
            continue;
        };
        let slot = sketch::ph_rect(
            icon_area.left(),
            icon_area.top(),
            icon_area.width(),
            icon_area.height(),
        );
        let target = aurorix_render::contain_destination_rect(image, slot);
        sketch::image_fill(kit, image, target);
    }
}

pub fn player_bar(kit: &CanvasKit<'_>, root: &Path, p: &player::Player, dy: f32) {
    player::draw(kit, root, CANVAS_WIDTH, 1.0, dy, p);
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SettlementRequest) {
    backdrop(kit, root, &req.song_id);
    let mut data = req.cards();
    data.score_line = score_text(req.score);
    if !req.played {
        data.score_line = String::from(NOT_PLAYED_TEXT);
    }
    if req.history > 0 {
        data.history_line = history_text(req.score, req.history);
    }
    draw_cards(kit, root, &settle_cards(), &data);
    cover_window(kit, root, &req.song_id);
    texts(kit, req);
    player_bar(kit, root, &req.player, 0.0);
    gained_bar(kit, &req.chart_history);
    buttons(kit, root);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_matches_go_layout() {
        assert_eq!(canvas_size(), (1920, 886));
        assert!((LEAN - 0.2679492).abs() < 1e-6);
    }

    #[test]
    fn score_and_history_lines_are_zero_padded() {
        assert_eq!(score_text(987123), "0987123");
        assert_eq!(score_text(-5), "0000000");
        assert_eq!(history_text(995307, 990614), "NEW BEST 0990614 +0004693");
        assert_eq!(history_text(900000, 995307), "NEW BEST 0995307 -0095307");
    }

    #[test]
    fn level_line_follows_official_writing() {
        assert_eq!(level_text("HD", 11.0, true), "HD Lv.11");
        assert_eq!(level_text("IN", 15.7, true), "IN Lv.15.7");
        assert_eq!(level_text("IN", 0.0, false), "IN 未收录");
        assert_eq!(level_text("", 0.0, false), "未收录");
        assert_eq!(rank_place_text(3, 312), "#3 / 312");
        assert_eq!(rank_place_text(0, 312), "");
        assert_eq!(gain_text(-4), "+0");
        assert_eq!(gain_text(12345), "+12'345");
    }

    #[test]
    fn cards_geometry_is_shared_and_skewed() {
        let g = settle_cards();
        assert_eq!(g.backs.len(), 3);
        assert_eq!(g.score, score_panel());
        assert_eq!(g.ratings, ratings());
        let cy = g.score.top() + g.score.height() / 2.0;
        assert!((g.score.left() + LEAN * cy - PANEL_EDGE_LEFT).abs() < 1e-3);
        assert!((g.score.width() - (PANEL_EDGE_RIGHT - PANEL_EDGE_LEFT)).abs() < 1e-3);
        assert!(g.score_text.size > g.history_text.size);
    }

    #[test]
    fn stat_cells_sit_inside_the_panel_edges() {
        let panel = combo_and_acc_panel();
        let slots = stat_text(panel);
        assert!(slots.left_value.x > panel.left());
        assert!(slots.right_value.x < panel.right());
        assert!(slots.left_value.size > slots.left_label.size);
        assert_eq!(slots.left_value.align, Align::Left);
        assert_eq!(slots.right_value.align, Align::Right);
    }

    #[test]
    fn history_note_counts_points_and_days() {
        let at = |text: &str| {
            DateTime::parse_from_rfc3339(text)
                .expect("时间戳")
                .with_timezone(&Utc)
        };
        let mut history = ChartHistory::default();
        assert_eq!(history_note_text(&history).1, "还没有记录");
        history.points = vec![HistoryPoint {
            at: at("2026-01-01T00:00:00Z"),
            score: 900000,
        }];
        assert_eq!(history_note_text(&history).1, "共 1 次");
        assert_eq!(history_span(&history.points), "01-01");
        history.points.push(HistoryPoint {
            at: at("2026-01-11T00:00:00Z"),
            score: 910000,
        });
        assert_eq!(history_note_text(&history).1, "共 2 次   ·   10 天");
        assert_eq!(history_span(&history.points), "01-01 → 01-11");
        assert_eq!(history_days(&history.points), 10);
    }
}
