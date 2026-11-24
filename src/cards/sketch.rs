use aurorix_render::skia_safe::canvas::{SaveLayerRec, SrcRectConstraint};
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{
    color_filters, image_filters, AlphaType, BlendMode, Canvas, ClipOp, ColorType, Image,
    ImageInfo, Paint, Path, PathBuilder, RRect, Rect, TileMode, surfaces,
};
use std::sync::Arc;

use aurorix_render::{Border, CanvasKit, Color, FontChain, TextStyle, Texture};

pub const FONT_BODY: &str = "Source_Han_Sans_&_Saira_Hybrid-Regular_-_Hot";

pub const FONT_DISPLAY: &str = "SourceHanSansCN_&_SairaCondensed_Hybrid_Medium";

pub const FONT_SYMBOL: &str = "NotoSansSymbols2-Regular";

pub fn font_chain() -> FontChain {
    FontChain::new(FONT_BODY).with_fallbacks([FONT_SYMBOL, FONT_DISPLAY])
}

pub fn font_size(size: f32) -> u32 {
    size.round().max(1.0) as u32
}

pub fn text_style(family: &str, size: f32, color: Color) -> TextStyle {
    TextStyle::new(family, font_size(size)).with_color(color)
}

pub fn ph_rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_xywh(
        x.round(),
        y.round(),
        w.round().max(0.0),
        h.round().max(0.0),
    )
}

pub fn round_int(value: f32) -> f32 {
    value.round()
}

pub fn draw_text(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    x: f32,
    baseline: f32,
    color: Color,
) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    let mut style = style.clone();
    style.color = color;
    kit.draw_text(value, x.round(), baseline.round(), &style)
        .map(|metrics| metrics.width)
        .unwrap_or(0.0)
}

pub fn text_width(kit: &CanvasKit<'_>, style: &TextStyle, value: &str) -> f32 {
    if value.is_empty() {
        return 0.0;
    }
    kit.measure(value, style)
        .map(|metrics| metrics.width)
        .unwrap_or(0.0)
}

pub fn text_right(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    right: f32,
    baseline: f32,
    color: Color,
) -> f32 {
    let width = text_width(kit, style, value);
    draw_text(
        kit,
        style,
        value,
        right - width.round(),
        baseline,
        color,
    );
    width
}

pub fn text_center(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    center: f32,
    baseline: f32,
    color: Color,
) -> f32 {
    let width = text_width(kit, style, value);
    draw_text(
        kit,
        style,
        value,
        center - (width / 2.0).round(),
        baseline,
        color,
    );
    width
}

pub fn truncate_width(value: &str, max_width: f32, measure_width: impl Fn(&str) -> f32) -> String {
    if max_width <= 0.0 {
        return String::new();
    }
    let limit = max_width.round();
    if measure_width(value).round() <= limit {
        return value.to_string();
    }
    let chars: Vec<char> = value.chars().collect();
    let mut probe = String::new();
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        probe.clear();
        probe.extend(&chars[..mid]);
        probe.push('…');
        if measure_width(&probe).round() <= limit {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    if lo == 0 {
        return match measure_width("…").round() <= limit {
            true => String::from("…"),
            false => String::new(),
        };
    }
    let mut out: String = chars[..lo].iter().collect();
    out.push('…');
    out
}

pub fn ellipsize(
    kit: &CanvasKit<'_>,
    style: &TextStyle,
    value: &str,
    max_width: f32,
) -> String {
    truncate_width(value, max_width, |text| text_width(kit, style, text))
}

fn solid_paint(color: Color) -> Paint {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color.to_skia());
    paint
}

fn stroke_paint(color: Color, thick: f32) -> Paint {
    let mut paint = solid_paint(color);
    paint.set_style(aurorix_render::skia_safe::paint::Style::Stroke);
    paint.set_stroke_width(thick.max(0.0));
    paint
}

pub fn round_rect(area: Rect, radius: f32) -> RRect {
    let radius = radius.max(0.0);
    RRect::new_rect_xy(area, radius, radius)
}

pub fn ph_fill(kit: &CanvasKit<'_>, area: Rect, color: Color) {
    kit.fill_rect(area, color);
}

pub fn fill_round(kit: &CanvasKit<'_>, area: Rect, radius: f32, color: Color) {
    kit.rrect_fill(area, radius, color);
}

pub fn stroke_round(kit: &CanvasKit<'_>, area: Rect, radius: f32, thick: f32, color: Color) {
    kit.stroke(area, radius, Border::new(color, thick));
}

pub fn clip_round<R>(
    kit: &CanvasKit<'_>,
    area: Rect,
    radius: f32,
    body: impl FnOnce() -> R,
) -> R {
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(round_rect(area, radius), ClipOp::Intersect, true);
    let out = body();
    canvas.restore();
    out
}

pub fn skew_path(area: Rect, lean: f32) -> Path {
    let cut = lean * area.height() / 2.0;
    let mut builder = PathBuilder::new();
    builder.move_to((area.left() + cut, area.top()));
    builder.line_to((area.right() + cut, area.top()));
    builder.line_to((area.right() - cut, area.bottom()));
    builder.line_to((area.left() - cut, area.bottom()));
    builder.close();
    builder.detach()
}

pub fn skew_offset(area: Rect, y: f32, lean: f32) -> f32 {
    lean * (area.top() + area.height() / 2.0 - y)
}

pub fn skew_bounds(area: Rect, lean: f32) -> Rect {
    *skew_path(area, lean).bounds()
}

pub fn skew_cut(area: Rect, lean: f32) -> f32 {
    let bounds = skew_bounds(area, lean);
    (area.left() - bounds.left()).abs()
}

pub fn fill_skew(kit: &CanvasKit<'_>, area: Rect, lean: f32, color: Color) {
    kit.canvas()
        .draw_path(&skew_path(area, lean), &solid_paint(color));
}

pub fn stroke_skew(
    kit: &CanvasKit<'_>,
    area: Rect,
    lean: f32,
    thick: f32,
    color: Color,
) {
    kit.canvas()
        .draw_path(&skew_path(area, lean), &stroke_paint(color, thick));
}

pub fn clip_skew<R>(
    kit: &CanvasKit<'_>,
    area: Rect,
    lean: f32,
    body: impl FnOnce() -> R,
) -> R {
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_path(&skew_path(area, lean), ClipOp::Intersect, true);
    let out = body();
    canvas.restore();
    out
}

pub fn image_cover(kit: &CanvasKit<'_>, image: &Image, area: Rect) {
    kit.cover_image(image, area);
}

pub fn image_cover_skew(
    kit: &CanvasKit<'_>,
    image: &Image,
    area: Rect,
    lean: f32,
) {
    let dest = skew_bounds(area, lean);
    clip_skew(kit, area, lean, || image_cover(kit, image, dest));
}

pub fn cover_source_focus(image: &Image, area: Rect, focus_y: f32) -> Rect {
    let source_width = image.width() as f32;
    let source_height = image.height() as f32;
    if source_width <= 0.0 || source_height <= 0.0 {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let target_width = area.width().abs().max(1.0);
    let target_height = area.height().abs().max(1.0);
    let scale = (source_width / target_width).max(source_height / target_height);
    let width = (target_width * scale).min(source_width);
    let height = (target_height * scale).min(source_height);
    let left = (source_width - width) / 2.0;
    let free = source_height - height;
    let top = (free * focus_y.clamp(0.0, 1.0)).clamp(0.0, free.max(0.0));
    Rect::from_xywh(
        left.round(),
        top.round(),
        width.round().max(1.0).min(source_width),
        height.round().max(1.0).min(source_height),
    )
}

pub fn image_cover_focus(kit: &CanvasKit<'_>, image: &Image, area: Rect, focus_y: f32) {
    let source = cover_source_focus(image, area, focus_y);
    kit.image_rect_src(image, source, area);
}

pub fn image_cover_skew_focus(kit: &CanvasKit<'_>, image: &Image, area: Rect, lean: f32, focus_y: f32) {
    let dest = skew_bounds(area, lean);
    clip_skew(kit, area, lean, || image_cover_focus(kit, image, dest, focus_y));
}

pub fn image_sprite(kit: &CanvasKit<'_>, image: &Image, area: Rect) {
    let side = area.width().max(1.0).round();
    let dest = Rect::from_xywh(
        area.left(),
        (area.top() + (area.height() - side) / 2.0).round(),
        side,
        side,
    );
    let source = Rect::from_xywh(0.0, 0.0, image.width() as f32, image.height() as f32);
    image_part(kit, image, source, dest);
}

pub fn image_sprite_skew(kit: &CanvasKit<'_>, image: &Image, area: Rect, lean: f32) {
    let dest = skew_bounds(area, lean);
    clip_skew(kit, area, lean, || image_sprite(kit, image, dest));
}

pub fn image_fill(kit: &CanvasKit<'_>, image: &Image, area: Rect) {
    let sampling =
        aurorix_render::sampling_for((image.width() as f32, image.height() as f32), area);
    kit.image_rect_with_sampling(image, area, sampling);
}

pub fn image_nearest(kit: &CanvasKit<'_>, image: &Image, area: Rect) {
    kit.image_rect_with_sampling(image, area, aurorix_render::sampling_nearest());
}

pub fn image_part_paint(
    kit: &CanvasKit<'_>,
    image: &Image,
    source: Rect,
    area: Rect,
    paint: &Paint,
) {
    let sampling = aurorix_render::sampling_for((source.width(), source.height()), area);
    kit.canvas().draw_image_rect_with_sampling_options(
        image,
        Some((&source, SrcRectConstraint::Fast)),
        area,
        sampling,
        paint,
    );
}

pub fn image_part(kit: &CanvasKit<'_>, image: &Image, source: Rect, area: Rect) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    image_part_paint(kit, image, source, area, &paint);
}

pub fn image_cover_paint(kit: &CanvasKit<'_>, image: &Image, dest: Rect, paint: &Paint) {
    let source = aurorix_render::cover_source_rect(image, dest);
    image_part_paint(kit, image, source, dest, paint);
}

pub fn image_contain(kit: &CanvasKit<'_>, key: &str, area: Rect) -> bool {
    kit.draw_asset(key, area, 0.0).is_ok()
}

pub fn image_cover_key(kit: &CanvasKit<'_>, key: &str, area: Rect) -> bool {
    kit.draw_asset_cover(key, area, 0.0).is_ok()
}

pub fn blur_sigma(radius: f32) -> f32 {
    let window = radius * 2.0 + 1.0;
    (((window * window - 1.0) / 12.0).max(0.0)).sqrt()
}

pub fn blur_layer<R>(
    kit: &CanvasKit<'_>,
    bounds: Rect,
    radius: f32,
    body: impl FnOnce(&CanvasKit<'_>) -> R,
) -> R {
    kit.blur_layer_in(bounds, blur_sigma(radius), body)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackdropGrade {
    pub factor: f32,
    pub radius: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub tone: Color,
    pub tone_mix: f32,
    pub focus_y: f32,
}

pub const BG_CLEAR: Color = Color::new(10, 8, 16, 255);

pub const BG_OFF_SIDE: i32 = 900;

pub const BG_OFF_BLEED: f32 = 100.0;

pub const BG_SLICE_ALPHA: u8 = 210;

pub const BG_BLEED_RATIO: f32 = 0.08;

pub const BG_DILATE: f32 = 6.0;

pub const BG_SIGMA: f32 = 38.0;

pub const BG_FILTER_SCALE: f32 = 0.34;

pub const BG_CUSHION: Color = Color::new(22, 14, 24, 20);

pub const BG_LAMP: Color = Color::new(255, 192, 128, 106);

pub const BG_LAMP_RADIUS: f32 = 1.05;

pub const BG_LAMPS: [(f32, f32); 3] = [(0.25, 0.20), (0.75, 0.50), (0.30, 0.82)];

pub const BG_SLICE_TOP: [(f32, f32, f32, f32, f32, f32); 3] = [
    (-200.0, -50.0, 1300.0, 450.0, 0.0, 0.34),
    (-50.0, 250.0, 1300.0, 450.0, 0.33, 0.34),
    (-250.0, 525.0, 1300.0, 450.0, 0.66, 0.34),
];

pub const BG_SHIELD: Color = Color::new(10, 8, 16, 255);

pub const BG_SHIELD_TOP_ALPHA: u8 = 178;

pub const BG_SHIELD_BOTTOM_ALPHA: u8 = 166;

pub const BG_SHIELD_TOP_STOP: f32 = 0.11;

pub const BG_SHIELD_BOTTOM_STOP: f32 = 0.89;

#[rustfmt::skip]
pub const BG_WARM_VIBRANT: [f32; 20] = [
    1.836, -0.669, -0.068, 0.0, 0.05,
    -0.181, 1.242, -0.061, 0.0, 0.02,
    -0.174, -0.584, 1.717, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0, 0.0,
];

pub fn seamless_base(image: &Image) -> Option<Image> {
    let mut surface = surfaces::raster_n32_premul((BG_OFF_SIDE, BG_OFF_SIDE))?;
    let side = BG_OFF_SIDE as f32;
    {
        let canvas = surface.canvas();
        canvas.clear(BG_CLEAR.to_skia());
        canvas.draw_image_rect(
            image,
            None,
            Rect::from_xywh(
                -BG_OFF_BLEED,
                -BG_OFF_BLEED,
                side + BG_OFF_BLEED * 2.0,
                side + BG_OFF_BLEED * 2.0,
            ),
            &Paint::default(),
        );
        let mut slice = Paint::default();
        slice.set_alpha(BG_SLICE_ALPHA);
        for (x, y, w, h, sx, sw) in BG_SLICE_TOP {
            let src = Rect::from_xywh(
                (image.width() as f32 * sx).round(),
                0.0,
                (image.width() as f32 * sw).round(),
                image.height() as f32,
            );
            canvas.draw_image_rect(
                image,
                Some((&src, SrcRectConstraint::Fast)),
                Rect::from_xywh(x, y, w, h),
                &slice,
            );
        }
    }
    Some(surface.image_snapshot())
}

fn bleed_rect(area: Rect) -> Rect {
    Rect::from_xywh(
        area.left() - area.width() * BG_BLEED_RATIO,
        area.top() - area.height() * BG_BLEED_RATIO,
        area.width() * (1.0 + BG_BLEED_RATIO * 2.0),
        area.height() * (1.0 + BG_BLEED_RATIO * 2.0),
    )
}

fn warm_cushion(canvas: &Canvas, window: Rect) {
    let mut paint = Paint::default();
    paint.set_color(BG_CUSHION.to_skia());
    canvas.draw_rect(window, &paint);
}

fn warm_lamps(canvas: &Canvas, area: Rect) {
    for (cx, cy) in BG_LAMPS {
        let gradient = aurorix_render::RadialGradient::new([
            aurorix_render::GradientStop::new(0.0, BG_LAMP),
            aurorix_render::GradientStop::new(1.0, Color::new(0, 0, 0, 0)),
        ])
        .with_center((
            area.left() + area.width() * cx,
            area.top() + area.height() * cy,
        ))
        .with_radius(area.width() * BG_LAMP_RADIUS);
        let Ok(shader) = gradient.shader(area) else {
            continue;
        };
        let mut paint = Paint::default();
        paint.set_shader(shader);
        paint.set_blend_mode(BlendMode::Screen);
        canvas.draw_rect(area, &paint);
    }
}

fn text_shield(canvas: &Canvas, area: Rect) {
    let gradient = aurorix_render::LinearGradient::vertical([
        aurorix_render::GradientStop::new(0.0, BG_SHIELD.with_alpha(BG_SHIELD_TOP_ALPHA)),
        aurorix_render::GradientStop::new(BG_SHIELD_TOP_STOP, BG_SHIELD.with_alpha(0)),
        aurorix_render::GradientStop::new(BG_SHIELD_BOTTOM_STOP, BG_SHIELD.with_alpha(0)),
        aurorix_render::GradientStop::new(1.0, BG_SHIELD.with_alpha(BG_SHIELD_BOTTOM_ALPHA)),
    ]);
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_shader(shader);
    paint.set_blend_mode(BlendMode::SrcOver);
    canvas.draw_rect(area, &paint);
}

fn backdrop_buffer(image: Option<&Image>, area: Rect) -> Option<Image> {
    let side = BG_FILTER_SCALE;
    let width = (area.width() * side).round().max(1.0) as i32;
    let height = (area.height() * side).round().max(1.0) as i32;
    let mut surface = surfaces::raster_n32_premul((width, height))?;
    {
        let canvas = surface.canvas();
        canvas.scale((side, side));
        if let Some(base) = image.and_then(seamless_base) {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color_filter(color_filters::matrix_row_major(&BG_WARM_VIBRANT, None));
            let widen = image_filters::dilate((BG_DILATE * side, BG_DILATE * side), None, None);
            paint.set_image_filter(image_filters::blur(
                (BG_SIGMA, BG_SIGMA),
                TileMode::Clamp,
                widen,
                None,
            ));
            canvas.draw_image_rect(&base, None, bleed_rect(area), &paint);
        }
        warm_cushion(canvas, Rect::from_xywh(0.0, 0.0, area.width(), area.height()));
        let total = 1.0 + BG_BLEED_RATIO * 2.0;
        let local = Rect::from_xywh(
            (area.left() + area.width() * BG_BLEED_RATIO) / total,
            (area.top() + area.height() * BG_BLEED_RATIO) / total,
            area.width() / total,
            area.height() / total,
        );
        warm_lamps(canvas, local);
        text_shield(canvas, local);
    }
    Some(surface.image_snapshot())
}

pub fn dream_backdrop(kit: &CanvasKit<'_>, root: &std::path::Path, song_id: &str, area: Rect) {
    ph_fill(kit, area, BG_CLEAR);
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let texture = if song_id.is_empty() {
        None
    } else {
        backdrop_texture(kit, root, song_id)
    };
    let image = texture
        .as_ref()
        .and_then(|item| item.image())
        .filter(|item| item.width() > 0 && item.height() > 0);
    let Some(buffer) = backdrop_buffer(image, area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    kit.canvas().draw_image_rect(buffer, None, bleed_rect(area), &paint);
}

pub fn backdrop_texture(kit: &CanvasKit<'_>, root: &std::path::Path, song_id: &str) -> Option<Arc<Texture>> {
    super::memo::texture(kit, &super::thumb_key(root, song_id))
        .or_else(|| super::memo::texture(kit, &super::illust_key(root, song_id)))
        .or_else(|| super::memo::texture(kit, &super::blur_key(root, song_id)))
}

pub const BACKDROP_BLUR: f32 = 18.0;

pub const BACKDROP_BLUR_SCALE: f32 = 1.0 / 6.0;

pub const BACKDROP_BLEED_SIGMA: f32 = 3.0;

pub fn blur_backdrop(kit: &CanvasKit<'_>, image: &Image, area: Rect, radius: f32, tint: Color) -> bool {
    let sigma = blur_sigma(radius) * BACKDROP_BLUR_SCALE;
    let bleed = (sigma * BACKDROP_BLEED_SIGMA).ceil() + 2.0;
    let width = ((area.width() * BACKDROP_BLUR_SCALE).ceil() + bleed * 2.0).max(1.0) as i32;
    let height = ((area.height() * BACKDROP_BLUR_SCALE).ceil() + bleed * 2.0).max(1.0) as i32;
    let Some(mut surface) = surfaces::raster_n32_premul((width, height)) else {
        return false;
    };
    let bounds = Rect::from_xywh(0.0, 0.0, width as f32, height as f32);
    let source = aurorix_render::cover_source_rect(image, bounds);
    let cover_sampling = aurorix_render::sampling_for((source.width(), source.height()), bounds);
    let Some(filter) = image_filters::blur((sigma, sigma), TileMode::Decal, None, None) else {
        return false;
    };
    let mut paint = Paint::default();
    paint.set_image_filter(filter);
    let record = SaveLayerRec::default().bounds(&bounds).paint(&paint);
    surface.canvas().save_layer(&record);
    surface.canvas().draw_image_rect_with_sampling_options(
        image,
        Some((&source, SrcRectConstraint::Fast)),
        bounds,
        cover_sampling,
        &Paint::default(),
    );
    surface.canvas().restore();
    let mut tint_paint = Paint::default();
    tint_paint.set_anti_alias(false);
    tint_paint.set_color(tint.to_skia());
    surface.canvas().draw_rect(bounds, &tint_paint);
    let snapshot = surface.image_snapshot();
    let crop = Rect::from_xywh(
        bleed,
        bleed,
        (bounds.width() - bleed * 2.0).max(1.0),
        (bounds.height() - bleed * 2.0).max(1.0),
    );
    let blit_sampling = aurorix_render::sampling_for((crop.width(), crop.height()), area);
    let blit = Paint::default();
    kit.canvas().draw_image_rect_with_sampling_options(
        &snapshot,
        Some((&crop, SrcRectConstraint::Fast)),
        area,
        blit_sampling,
        &blit,
    );
    true
}

pub const BACKDROP_TINT: Color = Color::new(0x06, 0x08, 0x0E, 0xB8);

pub const BACKDROP_SCRIM: Color = Color::new(0x04, 0x06, 0x0C, 0x30);

pub const BACKDROP_SHEET_SCRIM: Color = Color::new(0x03, 0x04, 0x08, 0x5A);

pub fn backdrop(
    kit: &CanvasKit<'_>,
    root: &std::path::Path,
    song_id: &str,
    area: Rect,
    fallback: Color,
    tint: Color,
    radius: f32,
) {
    ph_fill(kit, area, fallback);
    if area.width() > 0.0 && area.height() > 0.0 && !song_id.is_empty() {
        let texture = backdrop_texture(kit, root, song_id);
        if let Some(image) = texture.as_ref().and_then(|item| item.image()) {
            if blur_backdrop(kit, image, area, radius, tint) {
                return;
            }
        }
    }
    ph_fill(kit, area, tint);
}

pub fn content_bounds(image: &Image) -> Option<Rect> {
    let (width, height) = (image.width(), image.height());
    if width <= 0 || height <= 0 {
        return None;
    }
    const PROBE_SIDE: i32 = 192;
    let scale = (PROBE_SIDE as f32 / width as f32)
        .min(PROBE_SIDE as f32 / height as f32)
        .min(1.0);
    let probe_w = ((width as f32 * scale).round() as i32).max(1);
    let probe_h = ((height as f32 * scale).round() as i32).max(1);
    let mut data = vec![0u8; probe_w as usize * probe_h as usize * 4];
    let row = probe_w as usize * 4;
    let scaled = aurorix_render::sampling_linear();
    let mut surface = surfaces::raster_n32_premul((probe_w, probe_h))?;
    {
        let canvas = surface.canvas();
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        canvas.draw_image_rect_with_sampling_options(
            image,
            None,
            Rect::from_xywh(0.0, 0.0, probe_w as f32, probe_h as f32),
            scaled,
            &paint,
        );
    }
    let probe = surface.image_snapshot();
    let info = ImageInfo::new(
        (probe_w, probe_h),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    if !probe.read_pixels(&info, &mut data, row, (0, 0), CachingHint::Allow) {
        return None;
    }
    let mut min_x = probe_w;
    let mut min_y = probe_h;
    let mut max_x = -1;
    let mut max_y = -1;
    for y in 0..probe_h {
        for x in 0..probe_w {
            let at = y as usize * row + x as usize * 4;
            if data[at + 3] < 128 {
                continue;
            }
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    if max_x < min_x || max_y < min_y {
        return None;
    }
    let back = 1.0 / scale;
    let left = (min_x as f32 * back).floor().min(width as f32 - 1.0);
    let top = (min_y as f32 * back).floor().min(height as f32 - 1.0);
    let right = ((max_x + 1) as f32 * back).ceil().min(width as f32);
    let bottom = ((max_y + 1) as f32 * back).ceil().min(height as f32);
    Some(Rect::from_xywh(
        left,
        top,
        (right - left).max(1.0),
        (bottom - top).max(1.0),
    ))
}

pub fn group_digits(value: &str) -> String {
    let digits = value;
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push('\'');
        }
        out.push(digit);
    }
    out
}

pub fn signed_score(delta: i32) -> String {
    let (sign, value) = if delta < 0 {
        ('-', -delta)
    } else {
        ('+', delta)
    };
    format!("{sign}{}", group_digits(&value.to_string()))
}

pub fn one_decimal(value: f64) -> String {
    format!("{value:.1}")
}

pub fn two_decimals(value: f64) -> String {
    format!("{:.2}", value.max(0.0))
}

pub fn acc_percent(value: f64) -> String {
    format!("{:.2}%", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measure_width(text: &str) -> f32 {
        text.chars().count() as f32 * 10.0
    }

    #[test]
    fn group_digits_inserts_apostrophes() {
        assert_eq!(group_digits("1000000"), "1'000'000");
        assert_eq!(group_digits("987123"), "987'123");
        assert_eq!(group_digits("123"), "123");
        assert_eq!(group_digits("12"), "12");
    }

    #[test]
    fn signed_score_keeps_sign_without_padding() {
        assert_eq!(signed_score(1200), "+1'200");
        assert_eq!(signed_score(-300), "-300");
        assert_eq!(signed_score(0), "+0");
    }

    #[test]
    fn constants_and_accuracy_use_two_decimals() {
        assert_eq!(one_decimal(16.0), "16.0");
        assert_eq!(one_decimal(13.93), "13.9");
        assert_eq!(two_decimals(15.6234), "15.62");
        assert_eq!(two_decimals(-1.0), "0.00");
        assert_eq!(acc_percent(98.5123), "98.51%");
    }

    #[test]
    fn truncate_width_adds_ellipsis() {
        let cut = truncate_width("abcdef", 50.0, measure_width);
        assert!(cut.ends_with('…'));
        assert!(measure_width(&cut) <= 50.0);
        assert_eq!(truncate_width("abcdef", 60.0, measure_width), "abcdef");
        assert_eq!(truncate_width("abcdef", 0.0, measure_width), "");
    }

    #[test]
    fn skew_offset_is_relative_to_vertical_center() {
        let area = ph_rect(10.0, 20.0, 100.0, 40.0);
        assert_eq!(skew_offset(area, 20.0, 0.25), 5.0);
        assert_eq!(skew_offset(area, 40.0, 0.25), 0.0);
        assert_eq!(skew_offset(area, 60.0, 0.25), -5.0);
    }
}
