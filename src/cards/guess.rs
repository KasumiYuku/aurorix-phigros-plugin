use std::path::Path;

use aurorix_render::skia_safe::canvas::SrcRectConstraint;
use aurorix_render::skia_safe::image::CachingHint;
use aurorix_render::skia_safe::{
    images, surfaces, AlphaType, ColorType, Data, Image, ImageInfo, Paint, Rect,
};
use aurorix_render::{CanvasKit, Color, GradientStop, LinearGradient};

use super::settlement::{self, TextAt};
use super::sketch;

pub const GUESS_TITLE: &str = "SIGNAL RECOVERY";

pub const GUESS_W: f32 = 1180.0;

pub const GUESS_ASPECT: f32 = 2048.0 / 1080.0;

pub const GUESS_PAD: f32 = 64.0;

pub const GUESS_IMAGE_TOP: f32 = GUESS_PAD + 56.0 + 28.0;

pub const GUESS_IMAGE_W: f32 = GUESS_W - 2.0 * GUESS_PAD;

pub const GUESS_IMAGE_H: f32 = 555.0;

pub const GUESS_H: f32 = GUESS_IMAGE_TOP + GUESS_IMAGE_H + GUESS_PAD;

pub const GUESS_SHRINK: f32 = 0.78;

pub fn gp(v: f32) -> f32 {
    v * GUESS_SHRINK
}

pub const GUESS_TITLE_SIZE: f32 = 56.0;

pub const GUESS_CODE_SIZE: f32 = 26.0;

pub const GUESS_TITLE_LIFT: f32 = 56.0;

pub const GUESS_CODE_LIFT: f32 = 40.0;

pub const GUESS_BAR_GAP: f32 = 52.0;

pub const GUESS_BAR_H: f32 = 14.0;

pub const GUESS_FOOT_GAP: f32 = 46.0;

pub const GUESS_FOOT_SIZE: f32 = 23.0;

pub const GUESS_EDGE: f32 = 2.0;

pub const GUESS_FRAME_LINE: f32 = 8.0;

pub const GUESS_FRAME_TICK: f32 = 64.0;

pub const GUESS_FRAME_THICK: f32 = 22.0;

pub const GUESS_FRAME_MIN: f32 = 12.0;

pub const GUESS_RESAMPLE: f64 = 16.0;

pub const GUESS_BLUR: i32 = 14;

pub const GUESS_COMPANION: f64 = 0.90;

pub const GUESS_CHROMA: f64 = 0.5;

pub const GUESS_SEED_PATTERN: u64 = 0x9E37_79B9_7F4A_7C15;

pub const FX_SEED_PATTERN: u64 = 0x2545_F491_4F6C_DD1D;

pub const CHART_KEEP: f64 = 0.16;

pub const CHART_REVEAL: f64 = 0.62;

pub const CHART_TINT: u8 = 110;

pub const GUESS_PROBE: i32 = 96;

pub const PALETTE_SAMPLES: usize = 6000;

pub const PALETTE_ROUNDS: usize = 6;

pub const PALETTE_SEED: u32 = 2654435761;

pub const PALETTE_PATTERN: u64 = 0x9E37_79B9_7F4A_7C15 + 1;

pub const VARIANT_GLITCH: &str = "glitch";

pub const VARIANT_WARP: &str = "warp";

pub const VARIANT_COLOR: &str = "color";

pub const VARIANT_POLAR: &str = "polar";

pub const VARIANT_OUTLINE: &str = "outline";

pub const VARIANT_HUE: &str = "hue";

pub const VARIANT_CHART: &str = "chart";

pub const GUESS_VARIANTS: [&str; 6] = [
    VARIANT_GLITCH,
    VARIANT_WARP,
    VARIANT_COLOR,
    VARIANT_POLAR,
    VARIANT_OUTLINE,
    VARIANT_HUE,
];

pub const VARIANT_NAME_GLITCH: &str = "信号干扰";

pub const VARIANT_NAME_WARP: &str = "空间扭曲";

pub const VARIANT_NAME_COLOR: &str = "色卡解剖";

pub const VARIANT_NAME_POLAR: &str = "极坐标";

pub const VARIANT_NAME_OUTLINE: &str = "线稿";

pub const VARIANT_NAME_HUE: &str = "色相";

pub const VARIANT_NAME_CHART: &str = "谱面遮挡";

pub const GUESS_GROUND_TOP: Color = Color::new(0x0B, 0x0D, 0x1A, 0xFF);

pub const GUESS_GROUND_BOTTOM: Color = Color::new(0x04, 0x05, 0x0A, 0xFF);

pub const GUESS_INK: Color = Color::new(0xE8, 0xEF, 0xFF, 0xFF);

pub const GUESS_DIM: Color = Color::new(0x8A, 0x93, 0xB0, 0xFF);

pub const GUESS_ACCENT: Color = Color::new(0x7C, 0xE0, 0xFF, 0xFF);

pub const GUESS_BAR_TRACK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const GUESS_FRAME_DIM: Color = Color::new(0x04, 0x05, 0x0A, 0x78);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GuessRequest {
    pub song_id: String,
    pub variant: String,
    pub level: f64,
    pub zoom: f64,
    pub interference: f64,
    pub seed: i64,
    pub code: String,
    pub r#final: bool,
    pub frame_zoom: f64,
}

pub fn guess_variants() -> [&'static str; 6] {
    GUESS_VARIANTS
}

pub fn guess_last_variant() -> &'static str {
    VARIANT_CHART
}

pub fn guess_variant_name(variant: &str) -> &'static str {
    match variant {
        VARIANT_GLITCH => VARIANT_NAME_GLITCH,
        VARIANT_WARP => VARIANT_NAME_WARP,
        VARIANT_COLOR => VARIANT_NAME_COLOR,
        VARIANT_POLAR => VARIANT_NAME_POLAR,
        VARIANT_OUTLINE => VARIANT_NAME_OUTLINE,
        VARIANT_HUE => VARIANT_NAME_HUE,
        VARIANT_CHART => VARIANT_NAME_CHART,
        _ => "",
    }
}

pub fn guess_title_at() -> TextAt {
    TextAt::left(
        gp(GUESS_PAD).round(),
        gp(GUESS_PAD + GUESS_TITLE_LIFT).round(),
        gp(GUESS_TITLE_SIZE),
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn guess_code_at() -> TextAt {
    TextAt::right(
        gp(GUESS_W - GUESS_PAD).round(),
        gp(GUESS_PAD + GUESS_CODE_LIFT).round(),
        gp(GUESS_CODE_SIZE),
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn guess_image_rect() -> Rect {
    Rect::from_ltrb(
        gp(GUESS_PAD).round(),
        gp(GUESS_IMAGE_TOP).round(),
        gp(GUESS_W - GUESS_PAD).round(),
        gp(GUESS_IMAGE_TOP + GUESS_IMAGE_H).round(),
    )
}

pub fn guess_bar_rect() -> Rect {
    let image = guess_image_rect();
    Rect::from_ltrb(
        image.left(),
        image.bottom() + gp(GUESS_BAR_GAP).round(),
        image.right(),
        image.bottom() + gp(GUESS_BAR_GAP + GUESS_BAR_H).round(),
    )
}

pub fn guess_foot_at() -> TextAt {
    TextAt::left(
        gp(GUESS_PAD).round(),
        guess_bar_rect().bottom() + gp(GUESS_FOOT_GAP).round(),
        gp(GUESS_FOOT_SIZE),
    )
    .with_font(sketch::FONT_BODY)
}

pub fn guess_page_rect() -> Rect {
    let (width, height) = guess_out_size();
    Rect::from_xywh(0.0, 0.0, width as f32, height as f32)
}

pub fn guess_out_size() -> (i32, i32) {
    (gp(GUESS_W).round() as i32, gp(GUESS_H).round() as i32)
}

pub fn canvas_size(_req: &GuessRequest) -> (i32, i32) {
    guess_out_size()
}

pub fn fragment_center(seed: i64) -> (f64, f64) {
    let mut rng = Rand::new(seed as u64, GUESS_SEED_PATTERN);
    (
        0.28 + 0.44 * rng.unit(),
        0.28 + 0.44 * rng.unit(),
    )
}

pub fn guess_window(area: Rect, seed: i64, reveal: f64) -> Option<Rect> {
    if reveal <= 0.0 || reveal >= 1.0 || area.width() <= 0.0 || area.height() <= 0.0 {
        return None;
    }
    let (cx, cy) = fragment_center(seed);
    let left = (cx - reveal / 2.0).clamp(0.0, 1.0 - reveal);
    let top = (cy - reveal / 2.0).clamp(0.0, 1.0 - reveal);
    let width = f64::from(area.width());
    let height = f64::from(area.height());
    let x0 = area.left() + (left * width).trunc() as f32;
    let y0 = area.top() + (top * height).trunc() as f32;
    let x1 = area.left() + ((left + reveal) * width).trunc() as f32;
    let y1 = area.top() + ((top + reveal) * height).trunc() as f32;
    if x1 - x0 < gp(GUESS_FRAME_MIN) || y1 - y0 < gp(GUESS_FRAME_MIN) {
        return None;
    }
    Some(Rect::from_ltrb(x0, y0, x1, y1))
}

const PCG_MUL_HI: u64 = 2_549_297_995_355_413_924;

const PCG_MUL_LO: u64 = 4_865_540_595_714_422_341;

const PCG_INC_HI: u64 = 6_364_136_223_846_793_005;

const PCG_INC_LO: u64 = 1_442_695_040_888_963_407;

struct Rand {
    lo: u64,
    hi: u64,
}

impl Rand {
    fn new(seed: u64, pattern: u64) -> Self {
        Self {
            lo: seed,
            hi: pattern | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let wide = u128::from(self.lo) * u128::from(PCG_MUL_LO);
        let mut hi = (wide >> 64) as u64;
        let mut lo = wide as u64;
        hi = hi
            .wrapping_add(self.hi.wrapping_mul(PCG_MUL_LO))
            .wrapping_add(self.lo.wrapping_mul(PCG_MUL_HI));
        let (next, carry) = lo.overflowing_add(PCG_INC_LO);
        lo = next;
        hi = hi.wrapping_add(PCG_INC_HI).wrapping_add(u64::from(carry));
        self.lo = lo;
        self.hi = hi;
        (lo ^ (lo >> 58)).rotate_right(5) ^ hi
    }

    fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1_u64 << 53) as f64)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        let wide = u128::from(self.next_u64()) * (n as u128);
        ((wide >> 64) as usize).min(n - 1)
    }

    fn perm(&mut self, n: usize) -> Vec<usize> {
        let mut out: Vec<usize> = (0..n).collect();
        for i in 1..n {
            let j = self.below(i + 1);
            out.swap(i, j);
        }
        out
    }
}

#[derive(Clone)]
struct Plate {
    w: usize,
    h: usize,
    px: Vec<u8>,
}

impl Plate {
    fn new(w: usize, h: usize) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        Self {
            w,
            h,
            px: vec![0_u8; w * h * 4],
        }
    }

    fn at(&self, x: usize, y: usize) -> [u8; 4] {
        let base = (y * self.w + x) * 4;
        [
            self.px[base],
            self.px[base + 1],
            self.px[base + 2],
            self.px[base + 3],
        ]
    }

    fn put(&mut self, x: usize, y: usize, value: [u8; 4]) {
        let base = (y * self.w + x) * 4;
        self.px[base] = value[0];
        self.px[base + 1] = value[1];
        self.px[base + 2] = value[2];
        self.px[base + 3] = value[3];
    }

    fn put_rgb(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let base = (y * self.w + x) * 4;
        self.px[base] = r;
        self.px[base + 1] = g;
        self.px[base + 2] = b;
    }

    fn sample(&self, fx: f64, fy: f64) -> [u8; 4] {
        let max_x = self.w - 1;
        let max_y = self.h - 1;
        let fx = fx.clamp(0.0, max_x as f64);
        let fy = fy.clamp(0.0, max_y as f64);
        let x0 = fx as usize;
        let y0 = fy as usize;
        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;
        let x1 = (x0 + 1).min(max_x);
        let y1 = (y0 + 1).min(max_y);
        let c00 = self.at(x0, y0);
        let c10 = self.at(x1, y0);
        let c01 = self.at(x0, y1);
        let c11 = self.at(x1, y1);
        let mut out = [255_u8; 4];
        for channel in 0..3 {
            let top = mix8(c00[channel], c10[channel], tx);
            let bottom = mix8(c01[channel], c11[channel], tx);
            out[channel] = mix8(top, bottom, ty);
        }
        out
    }
}

fn mix8(a: u8, b: u8, t: f64) -> u8 {
    clamp8(f64::from(a) * (1.0 - t) + f64::from(b) * t)
}

fn clamp8(value: f64) -> u8 {
    if value <= 0.0 {
        0
    } else if value >= 255.0 {
        255
    } else {
        (value + 0.5) as u8
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn scale_int(value: i64, amount: f64) -> i64 {
    (value as f64 * amount).round() as i64
}

fn blend(d: [u8; 4], s: [u8; 4], t: f64) -> [u8; 4] {
    [
        mix8(d[0], s[0], t),
        mix8(d[1], s[1], t),
        mix8(d[2], s[2], t),
        255,
    ]
}

fn resize_area(src: &Plate, w: usize, h: usize) -> Plate {
    let mut out = Plate::new(w, h);
    let sx = src.w as f64 / w as f64;
    let sy = src.h as f64 / h as f64;
    for y in 0..h {
        let y0 = ((y as f64) * sy) as usize;
        let y1 = (((y + 1) as f64) * sy) as usize;
        let y1 = y1.max(y0 + 1).min(src.h);
        for x in 0..w {
            let x0 = ((x as f64) * sx) as usize;
            let x1 = (((x + 1) as f64) * sx) as usize;
            let x1 = x1.max(x0 + 1).min(src.w);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let c = src.at(xx, yy);
                    let a = f64::from(c[3]);
                    sr += f64::from(c[0]) * a;
                    sg += f64::from(c[1]) * a;
                    sb += f64::from(c[2]) * a;
                    sa += a;
                    n += 1.0;
                }
            }
            if n > 0.0 && sa > 0.0 {
                out.put(
                    x,
                    y,
                    [
                        clamp8(sr / sa + 0.5),
                        clamp8(sg / sa + 0.5),
                        clamp8(sb / sa + 0.5),
                        clamp8(sa / n + 0.5),
                    ],
                );
            }
        }
    }
    out
}

fn resize_bilinear(src: &Plate, w: usize, h: usize) -> Plate {
    let mut out = Plate::new(w, h);
    let sx = src.w as f64 / w as f64;
    let sy = src.h as f64 / h as f64;
    for y in 0..h {
        let fy = (y as f64 + 0.5) * sy - 0.5;
        for x in 0..w {
            let fx = (x as f64 + 0.5) * sx - 0.5;
            out.put(x, y, src.sample(fx, fy));
        }
    }
    out
}

fn resample(src: &Plate, ratio: f64, amount: f64) -> Plate {
    if amount <= 0.0 || ratio <= 0.0 {
        return src.clone();
    }
    let scale = 1.0 / (1.0 + ratio * amount);
    let sw = (((src.w as f64 * scale) + 0.5) as usize).max(8);
    let sh = (((src.h as f64 * scale) + 0.5) as usize).max(8);
    if sw >= src.w && sh >= src.h {
        return src.clone();
    }
    let small = resize_area(src, sw.min(src.w), sh.min(src.h));
    resize_bilinear(&small, src.w, src.h)
}

fn box_blur(img: &mut Plate, radius: usize) {
    if radius < 1 || img.w == 0 || img.h == 0 {
        return;
    }
    let rad = radius + 1;
    let w = img.w;
    let h = img.h;
    let top_v = radius.min(h - 1);
    let mut ring = vec![[0_u8; 3]; rad];
    for x in 0..w {
        let (mut s0, mut s1, mut s2, mut n) = (0_i64, 0_i64, 0_i64, 0_i64);
        for y in 0..=top_v {
            let c = img.at(x, y);
            s0 += i64::from(c[0]);
            s1 += i64::from(c[1]);
            s2 += i64::from(c[2]);
            n += 1;
        }
        for y in 0..h {
            let c = img.at(x, y);
            ring[y % rad] = [c[0], c[1], c[2]];
            img.put_rgb(
                x,
                y,
                (s0 / n) as u8,
                (s1 / n) as u8,
                (s2 / n) as u8,
            );
            if y >= radius {
                let d = ring[(y - radius) % rad];
                s0 -= i64::from(d[0]);
                s1 -= i64::from(d[1]);
                s2 -= i64::from(d[2]);
                n -= 1;
            }
            if y + radius + 1 < h {
                let c = img.at(x, y + radius + 1);
                s0 += i64::from(c[0]);
                s1 += i64::from(c[1]);
                s2 += i64::from(c[2]);
                n += 1;
            }
        }
    }
    let top_h = radius.min(w - 1);
    for y in 0..h {
        let (mut s0, mut s1, mut s2, mut n) = (0_i64, 0_i64, 0_i64, 0_i64);
        for x in 0..=top_h {
            let c = img.at(x, y);
            s0 += i64::from(c[0]);
            s1 += i64::from(c[1]);
            s2 += i64::from(c[2]);
            n += 1;
        }
        for x in 0..w {
            let c = img.at(x, y);
            ring[x % rad] = [c[0], c[1], c[2]];
            img.put_rgb(
                x,
                y,
                (s0 / n) as u8,
                (s1 / n) as u8,
                (s2 / n) as u8,
            );
            if x >= radius {
                let d = ring[(x - radius) % rad];
                s0 -= i64::from(d[0]);
                s1 -= i64::from(d[1]);
                s2 -= i64::from(d[2]);
                n -= 1;
            }
            if x + radius + 1 < w {
                let c = img.at(x + radius + 1, y);
                s0 += i64::from(c[0]);
                s1 += i64::from(c[1]);
                s2 += i64::from(c[2]);
                n += 1;
            }
        }
    }
}

fn blur(src: &Plate, radius: i32, amount: f64) -> Plate {
    let r = (f64::from(radius) * amount + 0.5) as usize;
    if r < 2 || src.w == 0 || src.h == 0 {
        return src.clone();
    }
    let mut out = src.clone();
    box_blur(&mut out, r);
    out
}

fn zoom(src: &Plate, reveal: f64, cx: f64, cy: f64) -> Plate {
    let reveal = reveal.clamp(0.02, 1.0);
    if reveal >= 1.0 || src.w < 2 || src.h < 2 {
        return src.clone();
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let span_x = w * reveal;
    let span_y = h * reveal;
    let left = (cx * w - span_x / 2.0).clamp(0.0, w - span_x);
    let top = (cy * h - span_y / 2.0).clamp(0.0, h - span_y);
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let sy = top + (y as f64 + 0.5) / h * span_y;
        for x in 0..src.w {
            let sx = left + (x as f64 + 0.5) / w * span_x;
            out.put(x, y, src.sample(sx, sy));
        }
    }
    out
}

fn channel_shift(img: &mut Plate, dr: i64, dg: i64, db: i64) {
    let src = img.clone();
    let w = src.w;
    for y in 0..src.h {
        for x in 0..w {
            let xr = shift_index(x, dr, w);
            let xg = shift_index(x, dg, w);
            let xb = shift_index(x, db, w);
            let r = src.at(xr, y)[0];
            let g = src.at(xg, y)[1];
            let b = src.at(xb, y)[2];
            img.put_rgb(x, y, r, g, b);
        }
    }
}

fn shift_index(x: usize, delta: i64, limit: usize) -> usize {
    let value = x as i64 + delta;
    value.clamp(0, limit as i64 - 1) as usize
}

fn band_shift(img: &mut Plate, y0: usize, rows: usize, dx: i64) {
    if dx == 0 {
        return;
    }
    let src = img.clone();
    let w = src.w;
    let h = src.h;
    let last = (y0 + rows).min(h);
    for y in y0..last {
        for x in 0..w {
            let c = src.at(shift_index(x, dx, w), y);
            img.put_rgb(x, y, c[0], c[1], c[2]);
        }
    }
}

fn cross_fade(a: &Plate, b: &Plate, t: f64) -> Plate {
    let mut out = a.clone();
    let len = out.px.len().min(b.px.len());
    let mut i = 0;
    while i + 3 < len {
        out.px[i] = mix8(out.px[i], b.px[i], t);
        out.px[i + 1] = mix8(out.px[i + 1], b.px[i + 1], t);
        out.px[i + 2] = mix8(out.px[i + 2], b.px[i + 2], t);
        i += 4;
    }
    out
}

fn nearest(c: [u8; 3], centers: &[[u8; 3]]) -> usize {
    let mut best = 0;
    let mut best_d = f64::MAX;
    for (index, m) in centers.iter().enumerate() {
        let dr = f64::from(c[0]) - f64::from(m[0]);
        let dg = f64::from(c[1]) - f64::from(m[1]);
        let db = f64::from(c[2]) - f64::from(m[2]);
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best = index;
            best_d = d;
        }
    }
    best
}

fn dominant(src: &Plate, k: usize) -> Vec<[u8; 3]> {
    if src.w == 0 || src.h == 0 || k == 0 {
        return Vec::new();
    }
    let total = src.w * src.h;
    let step = if total > PALETTE_SAMPLES {
        total / PALETTE_SAMPLES + 1
    } else {
        1
    };
    let mut pool: Vec<[u8; 3]> = Vec::with_capacity(PALETTE_SAMPLES.min(total));
    let mut index = 0;
    while index < total {
        let c = src.at(index % src.w, index / src.w);
        pool.push([c[0], c[1], c[2]]);
        index += step;
    }
    if pool.is_empty() {
        return Vec::new();
    }
    let k = k.min(pool.len());
    let seed = (pool.len() as u64)
        .wrapping_mul(u64::from(PALETTE_SEED))
        .wrapping_add(17);
    let mut rng = Rand::new(seed, PALETTE_PATTERN);
    let mut centers: Vec<[u8; 3]> = (0..k).map(|_| pool[rng.below(pool.len())]).collect();
    let mut counts = vec![0_usize; k];
    let mut sums = vec![[0_u64; 3]; k];
    for _ in 0..PALETTE_ROUNDS {
        counts.fill(0);
        sums.fill([0; 3]);
        for sample in &pool {
            let hit = nearest(*sample, &centers);
            sums[hit][0] += u64::from(sample[0]);
            sums[hit][1] += u64::from(sample[1]);
            sums[hit][2] += u64::from(sample[2]);
            counts[hit] += 1;
        }
        for (slot, count) in counts.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            let n = *count as u64;
            centers[slot] = [
                (sums[slot][0] / n) as u8,
                (sums[slot][1] / n) as u8,
                (sums[slot][2] / n) as u8,
            ];
        }
    }
    let mut order: Vec<usize> = (0..k).filter(|slot| counts[*slot] > 0).collect();
    order.sort_by(|a, b| counts[*b].cmp(&counts[*a]));
    order.into_iter().map(|slot| centers[slot]).collect()
}

fn quantize(src: &Plate, k: usize) -> Plate {
    let centers = dominant(src, k);
    if centers.is_empty() {
        return src.clone();
    }
    let mut out = src.clone();
    let mut i = 0;
    let len = out.px.len();
    while i + 3 < len {
        let c = [out.px[i], out.px[i + 1], out.px[i + 2]];
        let hit = centers[nearest(c, &centers)];
        out.px[i] = hit[0];
        out.px[i + 1] = hit[1];
        out.px[i + 2] = hit[2];
        i += 4;
    }
    out
}

fn variant_glitch(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let amp = 2 + (34.0 * rng.unit()) as i64;
    let count = 2 + rng.below(8);
    let mut bands: Vec<(usize, usize, i64)> = Vec::with_capacity(count);
    for _ in 0..count {
        let rows = 4 + rng.below((4_usize).max((h as f64 * 0.06) as usize));
        let y0 = rng.below((h.saturating_sub(rows)).max(1));
        let dx = ((rng.unit() * 2.0 - 1.0) * w as f64 * 0.22) as i64;
        bands.push((y0, rows, dx));
    }
    let mut out = src.clone();
    let dr = scale_int(rng.below((2 * amp + 1) as usize) as i64 - amp, amount);
    let db = -scale_int(rng.below((2 * amp + 1) as usize) as i64 - amp, amount);
    channel_shift(&mut out, dr, 0, db);
    for (y0, rows, dx) in bands {
        band_shift(&mut out, y0, rows, scale_int(dx, amount));
    }
    out
}

fn variant_ripple(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.35 + 0.30 * rng.unit());
    let cy = h * (0.35 + 0.30 * rng.unit());
    let amp = (0.05 + 0.09 * rng.unit()) * w * amount;
    let lambda = (0.15 + 0.25 * rng.unit()) * w;
    let bend = (0.5 + 1.1 * rng.unit()) * amount;
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let fy = y as f64;
        for x in 0..src.w {
            let fx = x as f64;
            let mut sx = fx + amp * (std::f64::consts::TAU * fy / lambda).sin();
            let mut sy = fy + amp * (std::f64::consts::TAU * fx / lambda).cos();
            let (dx, dy) = (sx - cx, sy - cy);
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.0 {
                let factor = (dist / (w / 2.0)).powf(2.0 * bend);
                sx = cx + dx * factor;
                sy = cy + dy * factor;
            }
            out.put(x, y, src.sample(sx, sy));
        }
    }
    out
}

fn variant_kaleido(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let cx = w * (0.5 + (rng.unit() - 0.5) * 0.20);
    let cy = h * (0.5 + (rng.unit() - 0.5) * 0.20);
    let lobes = (3 + rng.below(6)) as f64;
    let seg = std::f64::consts::TAU / lobes;
    let r_max = (w - cx).max(cx).hypot((h - cy).max(cy));
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        let fy = y as f64;
        for x in 0..src.w {
            let fx = x as f64;
            let (dx, dy) = (fx - cx, fy - cy);
            let mut angle = dy.atan2(dx).rem_euclid(seg);
            if angle > seg / 2.0 {
                angle = seg - angle;
            }
            let u = (angle * lobes / std::f64::consts::TAU + 0.5) * w;
            let v = (dx * dx + dy * dy).sqrt() / r_max * h;
            out.put(
                x,
                y,
                src.sample(lerp(fx, u, amount), lerp(fy, v, amount)),
            );
        }
    }
    out
}

fn variant_shuffle(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let width = w as f64 / (8 + rng.below(9)) as f64;
    if width <= 0.0 {
        return src.clone();
    }
    let n = ((w as f64 / width).ceil() as usize).max(2);
    let perm = rng.perm(n);
    let mut out = Plate::new(w, h);
    for (dst, from) in perm.iter().enumerate() {
        let x0 = (dst as f64 * width) as usize;
        let x1 = (((dst + 1) as f64 * width) as usize).min(w);
        let origin = (*from as f64 * width) as usize;
        for x in x0..x1 {
            let target = (origin + x - x0).min(w - 1);
            let sx = lerp(x as f64, target as f64, amount);
            for y in 0..h {
                out.put(x, y, src.sample(sx, y as f64));
            }
        }
    }
    out
}

fn variant_outline(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let gain = 4.0 + 14.0 * rng.unit();
    let w = src.w;
    let h = src.h;
    let mut gray = vec![0_f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let c = src.at(x, y);
            gray[y * w + x] = 0.299 * f64::from(c[0]) + 0.587 * f64::from(c[1]) + 0.114 * f64::from(c[2]);
        }
    }
    let mut lines = Plate::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let gx = gray[y * w + (x + 1).min(w - 1)] - gray[y * w + x.saturating_sub(1)];
            let gy = gray[(y + 1).min(h - 1) * w + x] - gray[y.saturating_sub(1) * w + x];
            let line = 255 - i32::from(clamp8((gx * gx + gy * gy).sqrt() * gain));
            let value = line.clamp(0, 255) as u8;
            lines.put(x, y, [value, value, value, 255]);
        }
    }
    cross_fade(src, &lines, amount)
}

fn variant_hue(src: &Plate, amount: f64) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let cos = (amount * std::f64::consts::PI).cos();
    let sin = (amount * std::f64::consts::PI).sin();
    let m: [f64; 9] = [
        0.213 + cos * 0.787 - sin * 0.213,
        0.715 - cos * 0.715 - sin * 0.715,
        0.072 - cos * 0.072 + sin * 0.928,
        0.213 - cos * 0.213 + sin * 0.143,
        0.715 + cos * 0.285 + sin * 0.140,
        0.072 - cos * 0.072 - sin * 0.283,
        0.213 - cos * 0.213 - sin * 0.787,
        0.715 - cos * 0.715 + sin * 0.715,
        0.072 + cos * 0.928 + sin * 0.072,
    ];
    let mut out = src.clone();
    let mut i = 0;
    let len = out.px.len();
    while i + 3 < len {
        let r = f64::from(out.px[i]);
        let g = f64::from(out.px[i + 1]);
        let b = f64::from(out.px[i + 2]);
        out.px[i] = clamp8(m[0] * r + m[1] * g + m[2] * b);
        out.px[i + 1] = clamp8(m[3] * r + m[4] * g + m[5] * b);
        out.px[i + 2] = clamp8(m[6] * r + m[7] * g + m[8] * b);
        i += 4;
    }
    out
}

#[derive(Clone, Copy)]
enum Shape {
    Skew {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        lean: f64,
    },
    Round {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        r: f64,
    },
}

impl Shape {
    fn bounds(&self) -> (f64, f64, f64, f64) {
        match *self {
            Shape::Skew { x, y, w, h, lean } => {
                let cut = lean.abs() * h / 2.0;
                (x - cut, y, x + w + cut, y + h)
            }
            Shape::Round { x, y, w, h, .. } => (x, y, x + w, y + h),
        }
    }

    fn contains(&self, px: f64, py: f64) -> bool {
        match *self {
            Shape::Skew { x, y, w, h, lean } => {
                if py < y || py >= y + h {
                    return false;
                }
                let shift = lean * (y + h / 2.0 - py);
                px >= x + shift && px < x + w + shift
            }
            Shape::Round { x, y, w, h, r } => {
                let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
                let nx = px.clamp(x + r, x + w - r);
                let ny = py.clamp(y + r, y + h - r);
                let dx = px - nx;
                let dy = py - ny;
                dx * dx + dy * dy <= r * r
            }
        }
    }
}

fn window(dst: &mut Plate, src: &Plate, amount: f64, tint: u8, shapes: &[Shape]) {
    if amount <= 0.0 {
        return;
    }
    let tint_mix = f64::from(tint) / 255.0;
    let w = dst.w;
    let h = dst.h;
    for shape in shapes {
        let (x0, y0, x1, y1) = shape.bounds();
        let xa = (x0.floor().max(0.0)) as usize;
        let ya = (y0.floor().max(0.0)) as usize;
        let xb = (x1.ceil().min(w as f64)) as usize;
        let yb = (y1.ceil().min(h as f64)) as usize;
        for y in ya..yb {
            for x in xa..xb {
                if !shape.contains(x as f64, y as f64) {
                    continue;
                }
                let mut c = src.at(x, y);
                if tint_mix > 0.0 {
                    for slot in c.iter_mut().take(3) {
                        *slot = mix8(*slot, 255, tint_mix);
                    }
                }
                let d = dst.at(x, y);
                dst.put(
                    x,
                    y,
                    [
                        mix8(d[0], c[0], amount),
                        mix8(d[1], c[1], amount),
                        mix8(d[2], c[2], amount),
                        d[3],
                    ],
                );
            }
        }
    }
}

fn variant_chart(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    let w = src.w as f64;
    let h = src.h as f64;
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        for x in 0..src.w {
            let c = src.at(x, y);
            out.put(
                x,
                y,
                [
                    (f64::from(c[0]) * CHART_KEEP) as u8,
                    (f64::from(c[1]) * CHART_KEEP) as u8,
                    (f64::from(c[2]) * CHART_KEEP) as u8,
                    c[3],
                ],
            );
        }
    }
    let mut shapes: Vec<Shape> = Vec::new();
    let slabs = 2 + (2.0 * (1.0 - amount)).round() as usize;
    for _ in 0..slabs {
        let cx = w * (0.2 + 0.6 * rng.unit());
        let cy = h * (0.2 + 0.6 * rng.unit());
        let angle = std::f64::consts::PI * (0.15 + 0.7 * rng.unit());
        let length = w * (0.5 + 0.5 * rng.unit());
        let lean = angle.tan().clamp(-2.5, 2.5);
        shapes.push(Shape::Skew {
            x: cx - length / 2.0,
            y: cy,
            w: length,
            h: h * 0.004,
            lean,
        });
        let notes = 4 + (10.0 * (1.0 - amount)).round() as usize;
        for note in 0..notes {
            let t = (note as f64 + 0.5) / notes as f64;
            let px = cx + angle.cos() * (t - 0.5) * length;
            let py = cy + angle.sin() * (t - 0.5) * length;
            shapes.push(Shape::Round {
                x: px - w * 0.003,
                y: py - h * 0.015,
                w: w * 0.006,
                h: h * 0.030,
                r: 2.0,
            });
        }
    }
    let dots = 8 + (26.0 * (1.0 - amount)).round() as usize;
    for _ in 0..dots {
        let px = rng.unit() * w;
        let py = rng.unit() * h;
        let size = w * (0.005 + 0.006 * rng.unit());
        shapes.push(Shape::Round {
            x: px,
            y: py,
            w: size * 2.0,
            h: size * 2.0,
            r: size,
        });
    }
    window(&mut out, src, CHART_REVEAL, CHART_TINT, &shapes);
    out
}

fn wash_then_shuffle(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let colors = 4 + rng.below(8);
    let washed = cross_fade(src, &quantize(src, colors), amount);
    if amount > 0.35 {
        return variant_shuffle(&washed, amount, rng);
    }
    washed
}

fn variant_step(src: &Plate, variant: &str, amount: f64, rng: &mut Rand) -> Plate {
    match variant {
        VARIANT_GLITCH => variant_glitch(src, amount, rng),
        VARIANT_WARP => variant_ripple(src, amount, rng),
        VARIANT_COLOR => wash_then_shuffle(src, amount, rng),
        VARIANT_POLAR => variant_kaleido(src, amount, rng),
        VARIANT_OUTLINE => variant_outline(src, amount, rng),
        VARIANT_HUE => variant_hue(src, amount),
        _ => variant_chart(src, amount, rng),
    }
}

fn luma(c: [u8; 4]) -> f64 {
    0.299 * f64::from(c[0]) + 0.587 * f64::from(c[1]) + 0.114 * f64::from(c[2])
}

fn fx_scanlines(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 2 || src.h < 2 {
        return src.clone();
    }
    let period = 2 + rng.below(3);
    let depth = (0.30 + 0.45 * rng.unit()) * amount;
    let wobble = (0.03 + 0.06 * rng.unit()) * amount;
    let phase = rng.unit() * std::f64::consts::TAU;
    let freq = 0.25 + 0.85 * rng.unit();
    let mut out = src.clone();
    for y in 0..src.h {
        let mut k = 1.0 + wobble * ((y as f64) * freq + phase).sin();
        if y % period == 0 {
            k *= 1.0 - depth;
        }
        for x in 0..src.w {
            let c = out.at(x, y);
            out.put_rgb(
                x,
                y,
                clamp8(f64::from(c[0]) * k),
                clamp8(f64::from(c[1]) * k),
                clamp8(f64::from(c[2]) * k),
            );
        }
    }
    out
}

fn fx_chroma(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 4 {
        return src.clone();
    }
    let w = src.w;
    let amp = 1 + (amount * w as f64 * 0.03) as i64;
    let mut dr = rng.below((2 * amp + 1) as usize) as i64 - amp;
    let db = rng.below((2 * amp + 1) as usize) as i64 - amp;
    if dr == 0 && db == 0 {
        dr = amp;
    }
    let mut out = src.clone();
    for y in 0..src.h {
        for x in 0..w {
            let r = src.at(shift_index(x, dr, w), y)[0];
            let g = src.at(x, y)[1];
            let b = src.at(shift_index(x, db, w), y)[2];
            out.put_rgb(x, y, r, g, b);
        }
    }
    out
}

fn fx_slice(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 8 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let mut shift = vec![0_i64; h];
    let bands = 4 + rng.below(7);
    for _ in 0..bands {
        let rows = 2 + rng.below((2_usize).max(h / 10));
        let y0 = rng.below(h.saturating_sub(rows).max(1));
        let dx = ((rng.unit() * 2.0 - 1.0) * w as f64 * 0.32 * amount) as i64;
        for slot in shift
            .iter_mut()
            .take((y0 + rows).min(h))
            .skip(y0)
        {
            *slot = dx;
        }
    }
    let mut out = src.clone();
    for (y, &offset) in shift.iter().enumerate() {
        if offset == 0 {
            continue;
        }
        for x in 0..w {
            let sx = (x as i64 - offset).rem_euclid(w as i64) as usize;
            let c = src.at(sx, y);
            out.put_rgb(x, y, c[0], c[1], c[2]);
        }
    }
    out
}

fn fx_pixel_sort(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 4 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let threshold = 60.0 + 120.0 * rng.unit();
    let mut out = src.clone();
    let mut px: Vec<[u8; 4]> = vec![[0; 4]; w];
    let mut keys: Vec<f64> = vec![0.0; w];
    for y in 0..h {
        let mut run = 0_usize;
        for x in 0..=w {
            let mut lit = false;
            if x < w {
                let c = src.at(x, y);
                if luma(c) > threshold {
                    px[run] = c;
                    keys[run] = luma(c);
                    run += 1;
                    lit = true;
                }
            }
            if lit || run == 0 {
                continue;
            }
            if run >= 3 && run <= w / 2 {
                let mut order: Vec<usize> = (0..run).collect();
                order.sort_by(|a, b| {
                    keys[*a]
                        .partial_cmp(&keys[*b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let x0 = x - run;
                for (offset, slot) in order.iter().enumerate() {
                    let d = out.at(x0 + offset, y);
                    out.put(x0 + offset, y, blend(d, px[*slot], amount));
                }
            }
            run = 0;
        }
    }
    out
}

fn fx_halftone(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 8 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let cell = 3 + (9.0 * amount) as usize;
    let cols = w.div_ceil(cell);
    let rows = h.div_ceil(cell);
    let mut avg = vec![[0_u8; 4]; cols * rows];
    for by in 0..rows {
        for bx in 0..cols {
            avg[by * cols + bx] = block_avg(src, bx * cell, by * cell, cell, cell);
        }
    }
    let gain = 0.8 + 0.6 * rng.unit();
    let mut out = Plate::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let c = avg[(y / cell) * cols + x / cell];
            let centre_x = ((x / cell) * cell) as f64 + cell as f64 / 2.0 - 0.5;
            let centre_y = ((y / cell) * cell) as f64 + cell as f64 / 2.0 - 0.5;
            let lum = (luma(c) / 255.0 * gain).min(1.0);
            let rad = cell as f64 * 0.55 * lum.sqrt();
            let dist = (x as f64 + 0.5 - centre_x).hypot(y as f64 + 0.5 - centre_y);
            let cov = (rad - dist + 0.5).clamp(0.0, 1.0);
            let dot = [
                clamp8(f64::from(c[0]) * cov),
                clamp8(f64::from(c[1]) * cov),
                clamp8(f64::from(c[2]) * cov),
                255,
            ];
            let d = src.at(x, y);
            out.put(x, y, blend(d, dot, amount));
        }
    }
    out
}

fn block_avg(src: &Plate, x0: usize, y0: usize, bw: usize, bh: usize) -> [u8; 4] {
    let mut sr = 0.0;
    let mut sg = 0.0;
    let mut sb = 0.0;
    let mut n = 0.0;
    for y in y0..(y0 + bh).min(src.h) {
        for x in x0..(x0 + bw).min(src.w) {
            let c = src.at(x, y);
            sr += f64::from(c[0]);
            sg += f64::from(c[1]);
            sb += f64::from(c[2]);
            n += 1.0;
        }
    }
    if n == 0.0 {
        return [0, 0, 0, 255];
    }
    [
        clamp8(sr / n),
        clamp8(sg / n),
        clamp8(sb / n),
        255,
    ]
}

fn fx_posterize(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let levels = (9.0 - 7.0 * amount) as i32;
    let levels = levels.max(2);
    let step = 255.0 / f64::from(levels - 1);
    let dither = (0.35 + 0.4 * rng.unit()) * step * amount;
    const BAYER: [[f64; 4]; 4] = [
        [0.0, 8.0, 2.0, 10.0],
        [12.0, 4.0, 14.0, 6.0],
        [3.0, 11.0, 1.0, 9.0],
        [15.0, 7.0, 13.0, 5.0],
    ];
    let mut out = src.clone();
    for y in 0..src.h {
        for x in 0..src.w {
            let c = out.at(x, y);
            let bias = (BAYER[y % 4][x % 4] / 16.0 - 0.5) * dither;
            out.put(
                x,
                y,
                [
                    clamp8(((f64::from(c[0]) + bias) / step).round() * step),
                    clamp8(((f64::from(c[1]) + bias) / step).round() * step),
                    clamp8(((f64::from(c[2]) + bias) / step).round() * step),
                    255,
                ],
            );
        }
    }
    out
}

fn fx_mosaic(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 8 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let base = 3 + (13.0 * amount) as usize;
    let mut xs: Vec<usize> = Vec::new();
    let mut x = 0;
    while x < w {
        xs.push(x);
        x += base + rng.below(base);
    }
    let mut ys: Vec<usize> = Vec::new();
    let mut y = 0;
    while y < h {
        ys.push(y);
        y += base + rng.below(base);
    }
    if xs.is_empty() || ys.is_empty() {
        return src.clone();
    }
    let cols = xs.len();
    let mut avg = vec![[0_u8; 4]; cols * ys.len()];
    for (j, y0) in ys.iter().enumerate() {
        let next_y = ys.get(j + 1).copied().unwrap_or(h);
        for (i, x0) in xs.iter().enumerate() {
            let next_x = xs.get(i + 1).copied().unwrap_or(w);
            avg[j * cols + i] = block_avg(src, *x0, *y0, next_x - x0, next_y - y0);
        }
    }
    let mut col_of = vec![0_usize; w];
    for (i, x0) in xs.iter().enumerate() {
        let next = xs.get(i + 1).copied().unwrap_or(w);
        for slot in col_of.iter_mut().take(next).skip(*x0) {
            *slot = i;
        }
    }
    let mut row_of = vec![0_usize; h];
    for (j, y0) in ys.iter().enumerate() {
        let next = ys.get(j + 1).copied().unwrap_or(h);
        for slot in row_of.iter_mut().take(next).skip(*y0) {
            *slot = j;
        }
    }
    let mut out = Plate::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let d = src.at(x, y);
            out.put(x, y, blend(d, avg[row_of[y] * cols + col_of[x]], amount));
        }
    }
    out
}

fn fx_twirl(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 8 || src.h < 8 {
        return src.clone();
    }
    let w = src.w as f64;
    let h = src.h as f64;
    let centre_x = w * (0.3 + 0.4 * rng.unit());
    let centre_y = h * (0.3 + 0.4 * rng.unit());
    let radius = w.hypot(h) * (0.3 + 0.5 * rng.unit());
    let mut spin = (1.1 + 2.8 * rng.unit()) * amount;
    if rng.below(2) == 0 {
        spin = -spin;
    }
    const TURNS: usize = 1024;
    let mut out = Plate::new(src.w, src.h);
    for y in 0..src.h {
        for x in 0..src.w {
            let dx = x as f64 - centre_x;
            let dy = y as f64 - centre_y;
            let t = 1.0 - dx.hypot(dy) / radius;
            if t <= 0.0 {
                out.put(x, y, src.at(x, y));
                continue;
            }
            let slot = ((t * t * TURNS as f64) as usize).min(TURNS);
            let u = slot as f64 / TURNS as f64;
            let angle = spin * u * u;
            let (sin, cos) = angle.sin_cos();
            out.put(
                x,
                y,
                src.sample(
                    centre_x + dx * cos - dy * sin,
                    centre_y + dx * sin + dy * cos,
                ),
            );
        }
    }
    out
}

fn fx_bloom(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 32 || src.h < 32 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let sw = (w / 10).max(8);
    let sh = (h / 10).max(8);
    let mut buf = Vec::with_capacity(sw * sh);
    for y in 0..sh {
        let y0 = y * h / sh;
        let y1 = (y * h / sh + 1).max((y + 1) * h / sh);
        for x in 0..sw {
            let x0 = x * w / sw;
            let x1 = (x * w / sw + 1).max((x + 1) * w / sw);
            buf.push(block_avg(src, x0, y0, x1 - x0, y1 - y0));
        }
    }
    let threshold = 110.0 + 70.0 * rng.unit();
    for c in &mut buf {
        let lum = luma(*c);
        let k = if lum > threshold {
            ((lum - threshold) / threshold * 2.0).min(1.0)
        } else {
            0.0
        };
        *c = [
            clamp8(f64::from(c[0]) * k),
            clamp8(f64::from(c[1]) * k),
            clamp8(f64::from(c[2]) * k),
            255,
        ];
    }
    box3(&mut buf, sw, sh);
    box3(&mut buf, sw, sh);
    let gain = (0.9 + 0.9 * rng.unit()) * amount;
    let mut out = Plate::new(w, h);
    for y in 0..h {
        let fy = (y as f64 + 0.5) / h as f64 * sh as f64 - 0.5;
        for x in 0..w {
            let fx = (x as f64 + 0.5) / w as f64 * sw as f64 - 0.5;
            let g = buf_sample(&buf, sw, sh, fx, fy);
            let d = src.at(x, y);
            out.put(
                x,
                y,
                [
                    clamp8(f64::from(d[0]) + f64::from(g[0]) * gain),
                    clamp8(f64::from(d[1]) + f64::from(g[1]) * gain),
                    clamp8(f64::from(d[2]) + f64::from(g[2]) * gain),
                    255,
                ],
            );
        }
    }
    out
}

fn box3(buf: &mut [[u8; 4]], w: usize, h: usize) {
    if w < 3 || h < 3 {
        return;
    }
    let src = buf.to_vec();
    for y in 0..h {
        for x in 0..w {
            let (mut sr, mut sg, mut sb) = (0.0, 0.0, 0.0);
            for dy in -1_i64..=1 {
                for dx in -1_i64..=1 {
                    let px = (x as i64 + dx).clamp(0, w as i64 - 1) as usize;
                    let py = (y as i64 + dy).clamp(0, h as i64 - 1) as usize;
                    let c = src[py * w + px];
                    sr += f64::from(c[0]);
                    sg += f64::from(c[1]);
                    sb += f64::from(c[2]);
                }
            }
            buf[y * w + x] = [clamp8(sr / 9.0), clamp8(sg / 9.0), clamp8(sb / 9.0), 255];
        }
    }
}

fn buf_sample(buf: &[[u8; 4]], w: usize, h: usize, fx: f64, fy: f64) -> [u8; 4] {
    let fx = fx.clamp(0.0, w as f64 - 1.0);
    let fy = fy.clamp(0.0, h as f64 - 1.0);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let mut out = [255_u8; 4];
    for channel in 0..3 {
        let lo = mix8(buf[y0 * w + x0][channel], buf[y0 * w + x1][channel], tx);
        let hi = mix8(buf[y1 * w + x0][channel], buf[y1 * w + x1][channel], tx);
        out[channel] = mix8(lo, hi, ty);
    }
    out
}

fn fx_mirror(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 || src.w < 16 || src.h < 16 {
        return src.clone();
    }
    let w = src.w;
    let h = src.h;
    let vertical = rng.below(2) == 0;
    let axis = if vertical {
        w / 4 + rng.below((w / 2).max(1))
    } else {
        h / 4 + rng.below((h / 2).max(1))
    };
    let mut out = src.clone();
    for y in 0..h {
        for x in 0..w {
            let sx = if vertical {
                axis as i64 * 2 - x as i64
            } else {
                x as i64
            };
            let sy = if vertical {
                y as i64
            } else {
                axis as i64 * 2 - y as i64
            };
            if sx < 0 || sx >= w as i64 || sy < 0 || sy >= h as i64 {
                continue;
            }
            let d = out.at(x, y);
            let s = src.at(sx as usize, sy as usize);
            out.put(x, y, blend(d, s, amount));
        }
    }
    out
}

fn fx_structural(index: usize, src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    match index {
        0 => fx_twirl(src, amount, rng),
        1 => fx_mosaic(src, amount, rng),
        2 => fx_halftone(src, amount, rng),
        _ => fx_mirror(src, amount, rng),
    }
}

fn fx_texture(index: usize, src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    match index {
        0 => fx_scanlines(src, amount, rng),
        1 => fx_chroma(src, amount, rng),
        2 => fx_slice(src, amount, rng),
        3 => fx_pixel_sort(src, amount, rng),
        4 => fx_posterize(src, amount, rng),
        _ => fx_bloom(src, amount, rng),
    }
}

pub fn fx_picks(seed: i64) -> (usize, [usize; 2]) {
    let mut rng = Rand::new(seed as u64, FX_SEED_PATTERN);
    let structural = rng.below(4);
    let perm = rng.perm(6);
    (structural, [perm[0], perm[1]])
}

fn clarity_chroma(src: &Plate, amount: f64, k: f64) -> Plate {
    if k <= 0.0 || amount <= 0.0 {
        return src.clone();
    }
    let keep = (1.0 - k * amount).max(0.0);
    let mut out = src.clone();
    let mut i = 0;
    let len = out.px.len();
    while i + 3 < len {
        let r = f64::from(out.px[i]);
        let g = f64::from(out.px[i + 1]);
        let b = f64::from(out.px[i + 2]);
        let l = 0.299 * r + 0.587 * g + 0.114 * b;
        out.px[i] = clamp8(l + (r - l) * keep);
        out.px[i + 1] = clamp8(l + (g - l) * keep);
        out.px[i + 2] = clamp8(l + (b - l) * keep);
        i += 4;
    }
    out
}

fn mottle(img: &mut Plate, cell: usize, amp: f64, rng: &mut Rand) {
    if cell == 0 {
        return;
    }
    let sw = img.w.div_ceil(cell) + 2;
    let sh = img.h.div_ceil(cell) + 2;
    let mut stain = vec![0_u8; sw * sh];
    for value in &mut stain {
        *value = clamp8(128.0 + (rng.unit() * 2.0 - 1.0) * amp);
    }
    for y in 0..img.h {
        let fy = (y as f64 + 0.5) / cell as f64;
        for x in 0..img.w {
            let fx = (x as f64 + 0.5) / cell as f64;
            let d = f64::from(sample_gray(&stain, sw, sh, fx, fy)) - 128.0;
            let c = img.at(x, y);
            img.put_rgb(
                x,
                y,
                clamp8(f64::from(c[0]) + d),
                clamp8(f64::from(c[1]) + d),
                clamp8(f64::from(c[2]) + d),
            );
        }
    }
}

fn sample_gray(buf: &[u8], w: usize, h: usize, fx: f64, fy: f64) -> u8 {
    let fx = fx.clamp(0.0, w as f64 - 1.0);
    let fy = fy.clamp(0.0, h as f64 - 1.0);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let lo = mix8(buf[y0 * w + x0], buf[y0 * w + x1], tx);
    let hi = mix8(buf[y1 * w + x0], buf[y1 * w + x1], tx);
    mix8(lo, hi, ty)
}

fn streaks(img: &mut Plate, amount: f64, rng: &mut Rand) {
    if amount <= 0.0 || img.w == 0 || img.h == 0 {
        return;
    }
    let w = img.w;
    let h = img.h;
    let count = 3 + rng.below(4);
    for _ in 0..count {
        let y0 = rng.below(h);
        let rows = 1 + rng.below((h / 50).max(1));
        let x0 = rng.below(w);
        let x1 = x0 + w / 4 + rng.below((w / 2).max(1));
        let d = (rng.unit() * 2.0 - 1.0) * amount * 90.0;
        for y in y0..(y0 + rows).min(h) {
            for x in x0..x1.min(w) {
                let c = img.at(x, y);
                img.put_rgb(
                    x,
                    y,
                    clamp8(f64::from(c[0]) + d),
                    clamp8(f64::from(c[1]) + d),
                    clamp8(f64::from(c[2]) + d),
                );
            }
        }
    }
}

fn grain(src: &Plate, amount: f64, rng: &mut Rand) -> Plate {
    if amount <= 0.0 {
        return src.clone();
    }
    let mut out = src.clone();
    mottle(&mut out, 24, amount * 31.0, rng);
    streaks(&mut out, amount * 0.22, rng);
    out
}

fn degrade_plan(art: &Plate, req: &GuessRequest) -> Plate {
    let amount = 1.0 - req.level.clamp(0.0, 1.0);
    if amount <= 0.0 {
        return art.clone();
    }
    let mut rng = Rand::new(req.seed as u64, GUESS_SEED_PATTERN);
    let (cx, cy) = fragment_center(req.seed);
    let mut out = zoom(art, req.zoom, cx, cy);
    out = resample(&out, GUESS_RESAMPLE, amount);
    out = blur(&out, GUESS_BLUR, amount);
    let interference = req.interference.clamp(0.0, 1.0);
    out = variant_step(&out, &req.variant, amount * interference, &mut rng);
    if interference > 0.0 {
        let damp = interference * GUESS_COMPANION;
        let (structural, textures) = fx_picks(req.seed);
        out = fx_structural(structural, &out, damp, &mut rng);
        for index in textures {
            out = fx_texture(index, &out, damp, &mut rng);
        }
    }
    out = clarity_chroma(&out, amount, GUESS_CHROMA);
    grain(&out, amount, &mut rng)
}

fn plate_image(plate: &Plate) -> Option<Image> {
    let info = ImageInfo::new(
        (plate.w as i32, plate.h as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    images::raster_from_data(&info, Data::new_copy(&plate.px), plate.w * 4)
}

fn paint_plate(kit: &CanvasKit<'_>, plate: &Plate, area: Rect) {
    let Some(image) = plate_image(plate) else {
        return;
    };
    let source = Rect::from_xywh(0.0, 0.0, plate.w as f32, plate.h as f32);
    sketch::image_part(kit, &image, source, area);
}

fn read_plate(image: &Image, w: usize, h: usize) -> Option<Plate> {
    if w == 0 || h == 0 {
        return None;
    }
    let info = ImageInfo::new(
        (w as i32, h as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row = w * 4;
    let mut px = vec![0_u8; row * h];
    if !image.read_pixels(&info, &mut px, row, (0, 0), CachingHint::Allow) {
        return None;
    }
    Some(Plate { w, h, px })
}

fn cover_plate(kit: &CanvasKit<'_>, root: &Path, song_id: &str, w: usize, h: usize) -> Option<Plate> {
    if song_id.is_empty() || w == 0 || h == 0 {
        return None;
    }
    let texture = super::memo::texture(kit, &super::illust_key(root, song_id))?;
    let image = texture.image()?;
    if image.width() <= 0 || image.height() <= 0 {
        return None;
    }
    let mut surface = surfaces::raster_n32_premul((w as i32, h as i32))?;
    {
        let canvas = surface.canvas();
        canvas.clear(GUESS_GROUND_TOP.to_skia());
        let dest = Rect::from_xywh(0.0, 0.0, w as f32, h as f32);
        let source = aurorix_render::cover_source_rect(image, dest);
        let sampling = aurorix_render::sampling_for((source.width(), source.height()), dest);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        canvas.draw_image_rect_with_sampling_options(
            image,
            Some((&source, SrcRectConstraint::Fast)),
            dest,
            sampling,
            &paint,
        );
    }
    let snapshot = surface.image_snapshot();
    read_plate(&snapshot, w, h)
}

fn ground(kit: &CanvasKit<'_>) {
    let page = guess_page_rect();
    sketch::ph_fill(kit, page, GUESS_GROUND_TOP);
    let gradient = LinearGradient::vertical([
        GradientStop::new(0.0, GUESS_GROUND_TOP),
        GradientStop::new(1.0, GUESS_GROUND_BOTTOM),
    ]);
    let Ok(shader) = gradient.shader(page) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_shader(shader);
    kit.canvas().draw_rect(page, &paint);
}

fn frame(kit: &CanvasKit<'_>, area: Rect, req: &GuessRequest) {
    let Some(window) = guess_window(area, req.seed, req.frame_zoom) else {
        return;
    };
    let x0 = window.left();
    let y0 = window.top();
    let x1 = window.right();
    let y1 = window.bottom();
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left(), area.top(), area.width(), y0 - area.top()),
        GUESS_FRAME_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left(), y1, area.width(), area.bottom() - y1),
        GUESS_FRAME_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left(), y0, x0 - area.left(), y1 - y0),
        GUESS_FRAME_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(x1, y0, area.right() - x1, y1 - y0),
        GUESS_FRAME_DIM,
    );
    let line = gp(GUESS_FRAME_LINE).round();
    sketch::ph_fill(kit, Rect::from_xywh(x0, y0, x1 - x0, line), GUESS_ACCENT);
    sketch::ph_fill(kit, Rect::from_xywh(x0, y1 - line, x1 - x0, line), GUESS_ACCENT);
    sketch::ph_fill(kit, Rect::from_xywh(x0, y0, line, y1 - y0), GUESS_ACCENT);
    sketch::ph_fill(kit, Rect::from_xywh(x1 - line, y0, line, y1 - y0), GUESS_ACCENT);
    let tick = gp(GUESS_FRAME_TICK).round();
    let thick = gp(GUESS_FRAME_THICK).round();
    let corners = [(x0, y0, 1.0, 1.0), (x1, y1, -1.0, -1.0), (x0, y1, 1.0, -1.0), (x1, y0, -1.0, 1.0)];
    for (cx, cy, sx, sy) in corners {
        let ax = cx.min(cx + sx * tick);
        let ay = cy.min(cy + sy * thick);
        sketch::ph_fill(kit, Rect::from_xywh(ax, ay, tick, thick), GUESS_ACCENT);
        let bx = cx.min(cx + sx * thick);
        let by = cy.min(cy + sy * tick);
        sketch::ph_fill(kit, Rect::from_xywh(bx, by, thick, tick), GUESS_ACCENT);
    }
}

fn puzzle(kit: &CanvasKit<'_>, root: &Path, req: &GuessRequest) -> bool {
    let area = guess_image_rect();
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return false;
    }
    let w = GUESS_IMAGE_W as usize;
    let h = GUESS_IMAGE_H as usize;
    let art = cover_plate(kit, root, &req.song_id, w, h);
    let shown = art.is_some();
    match art {
        None => sketch::ph_fill(kit, area, settlement::COVER_FALLBACK),
        Some(plate) => {
            let shown = if req.r#final {
                plate
            } else {
                degrade_plan(&plate, req)
            };
            paint_plate(kit, &shown, area);
        }
    }
    let edge = gp(GUESS_EDGE).round();
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left() - edge, area.top() - edge, area.width() + edge * 2.0, edge),
        GUESS_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left() - edge, area.bottom(), area.width() + edge * 2.0, edge),
        GUESS_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.left() - edge, area.top(), edge, area.height()),
        GUESS_DIM,
    );
    sketch::ph_fill(
        kit,
        Rect::from_xywh(area.right(), area.top(), edge, area.height()),
        GUESS_DIM,
    );
    shown
}

fn head(kit: &CanvasKit<'_>, req: &GuessRequest) {
    settlement::line(kit, guess_title_at(), GUESS_TITLE, GUESS_INK);
    if !req.code.is_empty() {
        settlement::line(kit, guess_code_at(), &req.code, GUESS_ACCENT);
    }
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &GuessRequest) {
    ground(kit);
    if puzzle(kit, root, req) && req.r#final {
        let area = guess_image_rect();
        if area.width() > 0.0 && area.height() > 0.0 {
            frame(kit, area, req);
        }
    }
    head(kit, req);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GuessRequest {
        GuessRequest {
            song_id: String::from("000AinSophAur.Yumeji.0"),
            variant: String::from(VARIANT_GLITCH),
            level: 0.34,
            zoom: 0.42,
            interference: 0.85,
            seed: 20260117,
            code: String::from("GX-71F3"),
            r#final: false,
            frame_zoom: 0.42,
        }
    }

    fn final_request() -> GuessRequest {
        GuessRequest {
            r#final: true,
            ..request()
        }
    }

    #[test]
    fn layout_follows_the_go_truth() {
        let image = guess_image_rect();
        assert_eq!(image.left(), gp(GUESS_PAD).round());
        assert_eq!(image.top(), gp(148.0).round());
        assert_eq!(image.right(), gp(1116.0).round());
        assert_eq!(image.bottom(), gp(703.0).round());
        assert!((GUESS_IMAGE_TOP - 148.0).abs() < 1e-6);
        assert!((GUESS_IMAGE_H - 555.0).abs() < 1e-6);
        assert!((GUESS_H - 767.0).abs() < 1e-6);
        assert!((GUESS_ASPECT - 2048.0 / 1080.0).abs() < 1e-6);
        let bar = guess_bar_rect();
        assert_eq!(bar.top(), image.bottom() + gp(52.0).round());
        assert!((bar.height() - gp(GUESS_BAR_H)).abs() <= 1.0);
        assert_eq!(
            guess_foot_at().baseline,
            bar.bottom() + gp(GUESS_FOOT_GAP).round()
        );
        assert_eq!(guess_title_at().align, settlement::Align::Left);
        assert_eq!(guess_code_at().align, settlement::Align::Right);
        assert_eq!(guess_code_at().x, gp(1116.0).round());
        assert_eq!(guess_title_at().baseline, gp(120.0).round());
        assert_eq!(guess_code_at().baseline, gp(104.0).round());
    }

    #[test]
    fn out_size_applies_the_go_shrink() {
        let (width, height) = canvas_size(&request());
        assert_eq!(width, 920);
        assert_eq!(height, 598);
        assert_eq!((width, height), ((GUESS_W * 0.78).round() as i32, (767.0_f32 * 0.78).round() as i32));
    }

    #[test]
    fn variants_match_the_go_names() {
        assert_eq!(guess_variants().len(), 6);
        assert_eq!(guess_last_variant(), "chart");
        assert_eq!(guess_variant_name("glitch"), "信号干扰");
        assert_eq!(guess_variant_name("chart"), "谱面遮挡");
        assert_eq!(guess_variant_name("nope"), "");
        assert_eq!(GUESS_RESAMPLE, 16.0);
        assert_eq!(GUESS_BLUR, 14);
        assert_eq!(GUESS_CHROMA, 0.5);
        assert_eq!(GUESS_COMPANION, 0.90);
        assert_eq!(GUESS_SEED_PATTERN, 0x9E37_79B9_7F4A_7C15);
        assert_eq!(FX_SEED_PATTERN, 0x2545_F491_4F6C_DD1D);
        assert_eq!(GUESS_PROBE, 96);
        assert_eq!(CHART_KEEP, 0.16);
        assert_eq!(CHART_REVEAL, 0.62);
        assert_eq!(CHART_TINT, 110);
    }

    #[test]
    fn window_stays_inside_the_frame() {
        let area = guess_image_rect();
        assert!(guess_window(area, 20260117, 1.0).is_none());
        assert!(guess_window(area, 20260117, 0.0).is_none());
        let window = guess_window(area, 20260117, 0.42).expect("窗口");
        assert!(window.left() >= area.left());
        assert!(window.top() >= area.top());
        assert!(window.right() <= area.right());
        assert!(window.bottom() <= area.bottom());
        let (cx, cy) = fragment_center(20260117);
        assert!((0.28..=0.72).contains(&cx));
        assert!((0.28..=0.72).contains(&cy));
    }

    #[test]
    fn degrade_plan_keeps_the_frame_size() {
        let plate = Plate::new(64, 32);
        let mut filled = plate.clone();
        for y in 0..32 {
            for x in 0..64 {
                filled.put(x, y, [(x * 4) as u8, (y * 8) as u8, 96, 255]);
            }
        }
        let req = request();
        let out = degrade_plan(&filled, &req);
        assert_eq!((out.w, out.h), (64, 32));
        let final_req = final_request();
        let clean = degrade_plan(&filled, &final_req);
        assert_eq!((clean.w, clean.h), (64, 32));
        let (structural, textures) = fx_picks(req.seed);
        assert!(structural < 4);
        assert!(textures[0] < 6 && textures[1] < 6);
    }

    #[test]
    #[ignore]
    fn render_guess() {
        use aurorix_render::skia_safe::{Data, Image};
        use std::time::Instant;

        let root = crate::assets_root();
        crate::cards::register_fonts(&root);
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context = aurorix_render::RenderContext::new(
            aurorix_render::SkiaBackend::handle(),
            64,
            256 * 1024 * 1024,
        )
        .with_asset_loader(assets.loader());
        let cases = [
            (false, "/tmp/phigros-verify/17-guess.png"),
            (true, "/tmp/phigros-verify/17-guess-reveal.png"),
        ];
        for (is_final, out) in cases {
            let mut req = request();
            req.r#final = is_final;
            let (width, height) = canvas_size(&req);
            assert_eq!((width, height), (920, 598));
            let start = Instant::now();
            let bytes = context
                .draw(
                    aurorix_render::RenderRequest::new(width, height),
                    |canvas, ctx| {
                    let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
                    draw(&kit, &root, &req);
                })
                .expect("渲染失败")
                .to_vec();
            let elapsed = start.elapsed();
            if let Some(parent) = Path::new(out).parent() {
                std::fs::create_dir_all(parent).expect("建出图目录");
            }
            std::fs::write(out, &bytes).expect("写出 PNG");
            let image = Image::from_encoded(Data::new_copy(&bytes)).expect("解码");
            let variance = luma_variance(&image);
            println!(
                "猜曲绘 {out} 耗时 {} ms ({} 字节, {}x{})",
                elapsed.as_millis(),
                bytes.len(),
                image.width(),
                image.height()
            );
            assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
            assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
            assert!(bytes.len() > 20000, "字节数必须大于 20000");
            assert!(variance > 100.0, "亮度方差必须大于 100");
        }
    }

    fn luma_variance(image: &Image) -> f64 {
        let (width, height) = (image.width(), image.height());
        let info = aurorix_render::skia_safe::ImageInfo::new(
            (width, height),
            aurorix_render::skia_safe::ColorType::RGBA8888,
            aurorix_render::skia_safe::AlphaType::Unpremul,
            None,
        );
        let row = width as usize * 4;
        let mut data = vec![0_u8; row * height as usize];
        if !image.read_pixels(
            &info,
            &mut data,
            row,
            (0, 0),
            aurorix_render::skia_safe::image::CachingHint::Allow,
        ) {
            return 0.0;
        }
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut count = 0.0;
        let mut y = 0usize;
        while y < height as usize {
            let mut x = 0usize;
            while x < width as usize {
                let at = y * row + x * 4;
                let luma = 0.299 * f64::from(data[at])
                    + 0.587 * f64::from(data[at + 1])
                    + 0.114 * f64::from(data[at + 2]);
                sum += luma;
                sum_sq += luma * luma;
                count += 1.0;
                x += 7;
            }
            y += 7;
        }
        if count <= 0.0 {
            return 0.0;
        }
        let mean = sum / count;
        sum_sq / count - mean * mean
    }
}
