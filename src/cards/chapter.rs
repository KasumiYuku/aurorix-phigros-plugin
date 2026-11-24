use std::path::Path;

use aurorix_render::skia_safe::{ClipOp, RRect, Rect, Vector};
use aurorix_render::{CanvasKit, Color};

use super::player::{self, Player};
use super::settlement::{self, TextAt};
use super::{b30, memo, sketch};

pub const CHAP_SIZE: i32 = 30;

pub const CHAP_COLS: i32 = 4;

pub const CHAP_TITLE: &str = "CHAPTER PROGRESS";

pub const CHAP_TITLE_SIZE: f32 = 46.0;

pub const CHAP_NOTE_SIZE: f32 = 16.0;

pub const CHAP_NAME_SIZE: f32 = 14.0;

pub const CHAP_META_SIZE: f32 = 13.0;

pub const CHAP_STATS_VALUE_SIZE: f32 = 20.0;

pub const CHAP_STATS_LABEL_SIZE: f32 = CHAP_META_SIZE;

pub const CHAP_STATS_H: f32 = 80.0;

pub const CHAP_STATS_TOP: f32 = 152.0;

pub const CHAP_STATS_PAD: f32 = 18.0;

pub const CHAP_TITLE_BASELINE: f32 = 76.0;

pub const CHAP_NOTE_BASELINE: f32 = 118.0;

pub const CHAP_GRID_TOP: f32 = 258.0;

pub const CHAP_BANNER_RATIO: f32 = 0.50;

pub const CHAP_INFO_H: f32 = 140.0;

pub const CHAP_ROW_GAP: f32 = 26.0;

pub const CHAP_FOOTER_H: f32 = 40.0;

pub const CHAP_CARD_PAD: f32 = 14.0;

pub const CHAP_NAME_GAP: f32 = 12.0;

pub const CHAP_COLUMN_GAP: f32 = 10.0;

pub const CHAP_TOP_LIFT: f32 = 36.0;

pub const CHAP_BAR_TOP: f32 = 54.0;

pub const CHAP_BAR_H: f32 = 10.0;

pub const CHAP_META1_LIFT: f32 = 94.0;

pub const CHAP_META2_LIFT: f32 = 126.0;

pub const CHAP_CARD_RADIUS: f32 = 18.0;

pub const CHAP_CARD_INK: Color = Color::new(0x08, 0x0A, 0x10, 0x8C);

pub const CHAP_BAR_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x26);

pub const CHAP_SEAM_INK: Color = Color::new(0x00, 0x00, 0x00, 0x3C);

pub const CHAP_BACKDROP_BLUR: f32 = sketch::BACKDROP_BLUR;

pub const CHAP_STATS_VALUE_LIFT: f32 = 1.25;

pub const CHAP_LABEL_PLAYED: &str = "已打谱面";

pub const CHAP_LABEL_CLEARED: &str = "通关";

pub const CHAP_LABEL_FC: &str = "FC";

pub const CHAP_LABEL_PHI: &str = "Phi";

pub const CHAP_KEY_PLAYED: &str = "已玩";

pub const CHAP_MISSING: &str = "—";

pub const CHAP_SEPARATOR: &str = " · ";

pub const CHAP_CHAPTER_UNIT_PREFIX: &str = "共 ";

pub const CHAP_CHAPTER_UNIT_SUFFIX: &str = " 章";

pub const CHAP_BANNER_DIR: &str = "chapter";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChapterProgress {
    pub name: String,
    pub banner: String,
    pub played: i32,
    pub total: i32,
    pub cleared: i32,
    pub full_combo: i32,
    pub phi: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ChapterRequest {
    pub chapters: Vec<ChapterProgress>,
    pub backdrop: String,
    pub player: Player,
}

pub fn chap_width() -> f32 {
    b30::width(CHAP_SIZE)
}

pub fn chap_margin() -> f32 {
    b30::margin(CHAP_SIZE)
}

pub fn chap_col_gap() -> f32 {
    b30::col_gap(CHAP_SIZE)
}

pub fn chap_card_w() -> f32 {
    let cols = CHAP_COLS as f32;
    (chap_width() - 2.0 * chap_margin() - (cols - 1.0) * chap_col_gap()) / cols
}

pub fn chap_card_h() -> f32 {
    chap_card_w() * CHAP_BANNER_RATIO + CHAP_INFO_H
}

pub fn chap_rows(count: usize) -> i32 {
    if count == 0 {
        return 1;
    }
    (count as i32 + CHAP_COLS - 1) / CHAP_COLS
}

pub fn chap_height(rows: i32) -> f32 {
    let rows = rows.max(1) as f32;
    CHAP_GRID_TOP + rows * chap_card_h() + (rows - 1.0) * CHAP_ROW_GAP + CHAP_FOOTER_H
}

pub fn chap_grid_area(rows: i32) -> Rect {
    let rows = rows.max(1) as f32;
    Rect::from_xywh(
        chap_margin(),
        CHAP_GRID_TOP,
        chap_width() - 2.0 * chap_margin(),
        rows * chap_card_h() + (rows - 1.0) * CHAP_ROW_GAP,
    )
}

fn grid_cell(area: Rect, rows: i32, cols: i32, col: i32, row: i32) -> Rect {
    if cols <= 0 || rows <= 0 || col < 0 || col >= cols || row < 0 || row >= rows {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let w = (area.width() - (cols - 1) as f32 * chap_col_gap()) / cols as f32;
    let h = (area.height() - (rows - 1) as f32 * CHAP_ROW_GAP) / rows as f32;
    if w <= 0.0 || h <= 0.0 {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    Rect::from_xywh(
        area.left() + col as f32 * (w + chap_col_gap()),
        area.top() + row as f32 * (h + CHAP_ROW_GAP),
        w,
        h,
    )
}

pub fn chap_card(index: i32, rows: i32, total: i32) -> Rect {
    let rows = rows.max(1);
    if index < 0 || index >= CHAP_COLS * rows {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let col = index % CHAP_COLS;
    let row = index / CHAP_COLS;
    let mut cell = grid_cell(chap_grid_area(rows), rows, CHAP_COLS, col, row);
    if row == rows - 1 && cell.width() > 0.0 {
        let n = total - (rows - 1) * CHAP_COLS;
        if n > 0 && n < CHAP_COLS {
            let shift = (CHAP_COLS - n) as f32 * (cell.width() + chap_col_gap()) / 2.0;
            cell = Rect::from_xywh(
                cell.left() + shift,
                cell.top(),
                cell.width(),
                cell.height(),
            );
        }
    }
    cell
}

pub fn chap_out_size(rows: i32) -> (i32, i32) {
    let factor = b30::shrink(CHAP_SIZE);
    let w = (chap_width().round() * factor).round() as i32;
    let h = (chap_height(rows).round() * factor).round() as i32;
    (w, h)
}

pub fn canvas_size(req: &ChapterRequest) -> (i32, i32) {
    chap_out_size(chap_rows(req.chapters.len()))
}

pub fn stats_panel() -> Rect {
    Rect::from_xywh(
        chap_margin(),
        CHAP_STATS_TOP,
        chap_width() - 2.0 * chap_margin(),
        CHAP_STATS_H,
    )
}

pub fn stats_rect() -> Rect {
    let panel = stats_panel();
    Rect::from_xywh(
        panel.left() + CHAP_STATS_PAD,
        panel.top() + CHAP_STATS_PAD,
        panel.width() - 2.0 * CHAP_STATS_PAD,
        panel.height() - 2.0 * CHAP_STATS_PAD,
    )
}

pub fn count_text(played: i32, total: i32) -> String {
    if total <= 0 {
        return String::new();
    }
    format!("{played} / {total}")
}

pub fn percent_text(played: i32, total: i32) -> String {
    if total <= 0 {
        return String::new();
    }
    let pct = (f64::from(played) / f64::from(total) * 100.0).round() as i32;
    format!("{pct}%")
}

pub fn played_text(played: i32, total: i32) -> String {
    let out = count_text(played, total);
    if out.is_empty() {
        return String::from(CHAP_MISSING);
    }
    let pct = percent_text(played, total);
    if pct.is_empty() {
        return out;
    }
    format!("{out}{CHAP_SEPARATOR}{pct}")
}

pub fn ratio(played: i32, total: i32) -> f64 {
    if total <= 0 {
        return 0.0;
    }
    f64::from(played) / f64::from(total)
}

pub fn count_ink(value: i32) -> Color {
    if value <= 0 {
        return settlement::LABEL_INK;
    }
    settlement::SCORE_INK
}

pub fn totals(req: &ChapterRequest) -> (i32, i32, i32, i32, i32) {
    let mut played = 0;
    let mut total = 0;
    let mut cleared = 0;
    let mut full_combo = 0;
    let mut phi = 0;
    for chapter in &req.chapters {
        if chapter.total <= 0 {
            continue;
        }
        played += chapter.played;
        total += chapter.total;
        cleared += chapter.cleared;
        full_combo += chapter.full_combo;
        phi += chapter.phi;
    }
    (played, total, cleared, full_combo, phi)
}

pub fn stat_cells(req: &ChapterRequest) -> [(&'static str, String); 4] {
    let (played, total, cleared, full_combo, phi) = totals(req);
    [
        (CHAP_LABEL_PLAYED, played_text(played, total)),
        (CHAP_LABEL_CLEARED, cleared.to_string()),
        (CHAP_LABEL_FC, full_combo.to_string()),
        (CHAP_LABEL_PHI, phi.to_string()),
    ]
}

pub fn note_text(req: &ChapterRequest) -> String {
    if req.chapters.is_empty() {
        return String::new();
    }
    format!(
        "{CHAP_CHAPTER_UNIT_PREFIX}{}{CHAP_CHAPTER_UNIT_SUFFIX}",
        req.chapters.len()
    )
}

fn head_limit(kit: &CanvasKit<'_>, req: &ChapterRequest) -> f32 {
    head_right(kit, req) - chap_margin()
}

fn head_right(kit: &CanvasKit<'_>, req: &ChapterRequest) -> f32 {
    let canvas_w = chap_width();
    if !req.player.has_player {
        return canvas_w;
    }
    let name_width = player::nickname(kit, &req.player.name)
        .map(|(_, width)| width)
        .unwrap_or(0.0);
    let panel = player::panel_rect(name_width);
    canvas_w - player::CALIB_WIDTH * b30::B30_SCALE + panel.left() * b30::B30_SCALE
}

fn ellipsized(kit: &CanvasKit<'_>, at: TextAt, value: &str, limit: f32) -> String {
    if value.is_empty() || limit <= 0.0 {
        return String::new();
    }
    let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
    sketch::ellipsize(kit, &style, value, limit)
}

fn head(kit: &CanvasKit<'_>, req: &ChapterRequest) {
    let limit = head_limit(kit, req);
    let title_at = TextAt::left(chap_margin(), CHAP_TITLE_BASELINE, CHAP_TITLE_SIZE)
        .with_font(sketch::FONT_DISPLAY);
    let title = ellipsized(kit, title_at, CHAP_TITLE, limit);
    settlement::line(kit, title_at, &title, settlement::SCORE_INK);
    let note = note_text(req);
    if note.is_empty() {
        return;
    }
    let note_at = TextAt::left(chap_margin(), CHAP_NOTE_BASELINE, CHAP_NOTE_SIZE);
    let note = ellipsized(kit, note_at, &note, limit);
    settlement::line(kit, note_at, &note, b30::OV_NOTE_INK);
}

fn stats(kit: &CanvasKit<'_>, req: &ChapterRequest) {
    sketch::fill_round(kit, stats_panel(), CHAP_CARD_RADIUS, CHAP_CARD_INK);
    let area = stats_rect();
    let cells = stat_cells(req);
    let width = area.width() / cells.len() as f32;
    for (index, (label, value)) in cells.iter().enumerate() {
        let center = area.left() + index as f32 * width + width / 2.0;
        let label_at = TextAt::center(center, area.top() + CHAP_STATS_LABEL_SIZE, CHAP_STATS_LABEL_SIZE);
        settlement::line(kit, label_at, label, settlement::LABEL_INK);
        let value_at = TextAt::center(
            center,
            area.top() + CHAP_STATS_LABEL_SIZE + CHAP_STATS_VALUE_SIZE * CHAP_STATS_VALUE_LIFT,
            CHAP_STATS_VALUE_SIZE,
        )
        .with_font(sketch::FONT_DISPLAY);
        settlement::line(kit, value_at, value, settlement::SCORE_INK);
    }
}

fn banner_key(root: &Path, banner: &str) -> Option<String> {
    if banner.is_empty() {
        return None;
    }
    for ext in ["jpg", "png", "jpeg", "webp"] {
        let key = format!("{CHAP_BANNER_DIR}/{banner}.{ext}");
        if root.join(&key).is_file() {
            return Some(key);
        }
    }
    None
}

fn banner_box(card: Rect) -> Rect {
    Rect::from_xywh(
        card.left(),
        card.top(),
        card.width(),
        card.width() * CHAP_BANNER_RATIO,
    )
}

fn banner(kit: &CanvasKit<'_>, root: &Path, card: Rect, item: &ChapterProgress) {
    let area = banner_box(card);
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let radius = CHAP_CARD_RADIUS;
    let shape = RRect::new_rect_radii(
        area,
        &[
            Vector::new(radius, radius),
            Vector::new(radius, radius),
            Vector::new(0.0, 0.0),
            Vector::new(0.0, 0.0),
        ],
    );
    let canvas = kit.canvas();
    canvas.save();
    canvas.clip_rrect(shape, ClipOp::Intersect, true);
    let texture = banner_key(root, &item.banner).and_then(|key| memo::texture(kit, &key));
    match texture.as_ref().and_then(|item| item.image()) {
        Some(image) => sketch::image_cover(kit, image, area),
        None => sketch::ph_fill(kit, area, settlement::COVER_FALLBACK),
    }
    canvas.restore();
    sketch::ph_fill(
        kit,
        Rect::from_ltrb(area.left(), area.bottom() - 1.0, area.right(), area.bottom()),
        CHAP_SEAM_INK,
    );
}

fn progress_fill(track: f32, value: f64) -> f32 {
    if track <= 0.0 || value <= 0.0 || value.is_nan() {
        return 0.0;
    }
    let width = (f64::from(track) * value.min(1.0)).round() as f32;
    width.max(1.0).min(track)
}

fn progress(kit: &CanvasKit<'_>, card: Rect, item: &ChapterProgress) {
    let info_top = card.top() + card.width() * CHAP_BANNER_RATIO;
    let bar = Rect::from_xywh(
        card.left() + CHAP_CARD_PAD,
        info_top + CHAP_BAR_TOP,
        card.width() - 2.0 * CHAP_CARD_PAD,
        CHAP_BAR_H,
    );
    if bar.width() <= 0.0 || bar.height() <= 0.0 {
        return;
    }
    sketch::ph_fill(kit, bar, CHAP_BAR_TRACK_INK);
    let filled = progress_fill(bar.width(), ratio(item.played, item.total));
    if filled <= 0.0 {
        return;
    }
    sketch::ph_fill(
        kit,
        Rect::from_ltrb(bar.left(), bar.top(), bar.left() + filled, bar.bottom()),
        settlement::RANK_ACCENT,
    );
}

fn key_value(
    kit: &CanvasKit<'_>,
    label_at: TextAt,
    label: &str,
    label_ink: Color,
    value: &str,
    value_ink: Color,
    right: f32,
) {
    settlement::line(kit, label_at, label, label_ink);
    if value.is_empty() {
        return;
    }
    let value_at = TextAt::right(right, label_at.baseline, label_at.size);
    settlement::line(kit, value_at, value, value_ink);
}

fn card(kit: &CanvasKit<'_>, root: &Path, area: Rect, item: &ChapterProgress) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    sketch::fill_round(kit, area, CHAP_CARD_RADIUS, CHAP_CARD_INK);
    banner(kit, root, area, item);
    let info_top = area.top() + area.width() * CHAP_BANNER_RATIO;
    let name_at = TextAt::left(
        area.left() + CHAP_CARD_PAD,
        info_top + CHAP_TOP_LIFT,
        CHAP_NAME_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY);
    let budget = area.right() - CHAP_CARD_PAD - name_at.x;
    let style = sketch::text_style(sketch::FONT_DISPLAY, CHAP_NAME_SIZE, settlement::TITLE_INK);
    let name = sketch::ellipsize(kit, &style, &item.name, budget);
    settlement::line(kit, name_at, &name, settlement::TITLE_INK);
    progress(kit, area, item);
    let mid = area.left() + area.width() / 2.0;
    let value_left = mid - CHAP_COLUMN_GAP;
    let value_right = area.right() - CHAP_CARD_PAD;
    key_value(
        kit,
        TextAt::left(
            area.left() + CHAP_CARD_PAD,
            info_top + CHAP_META1_LIFT,
            CHAP_META_SIZE,
        ),
        CHAP_KEY_PLAYED,
        b30::OV_NOTE_INK,
        &count_text(item.played, item.total),
        settlement::SCORE_INK,
        value_left,
    );
    key_value(
        kit,
        TextAt::left(mid + CHAP_COLUMN_GAP, info_top + CHAP_META1_LIFT, CHAP_META_SIZE),
        CHAP_LABEL_CLEARED,
        b30::OV_NOTE_INK,
        &item.cleared.to_string(),
        settlement::SCORE_INK,
        value_right,
    );
    key_value(
        kit,
        TextAt::left(
            area.left() + CHAP_CARD_PAD,
            info_top + CHAP_META2_LIFT,
            CHAP_META_SIZE,
        ),
        CHAP_LABEL_FC,
        b30::OV_NOTE_INK,
        &item.full_combo.to_string(),
        count_ink(item.full_combo),
        value_left,
    );
    key_value(
        kit,
        TextAt::left(mid + CHAP_COLUMN_GAP, info_top + CHAP_META2_LIFT, CHAP_META_SIZE),
        CHAP_LABEL_PHI,
        b30::OV_NOTE_INK,
        &item.phi.to_string(),
        count_ink(item.phi),
        value_right,
    );
}

fn grid(kit: &CanvasKit<'_>, root: &Path, req: &ChapterRequest, rows: i32) {
    let total = req.chapters.len() as i32;
    for (index, item) in req.chapters.iter().enumerate() {
        card(kit, root, chap_card(index as i32, rows, total), item);
    }
}

fn backdrop(kit: &CanvasKit<'_>, root: &Path, req: &ChapterRequest) {
    let rows = chap_rows(req.chapters.len());
    let full = Rect::from_xywh(0.0, 0.0, chap_width(), chap_height(rows));
    sketch::backdrop(
        kit,
        root,
        &req.backdrop,
        full,
        settlement::COVER_FALLBACK,
        sketch::BACKDROP_TINT,
        CHAP_BACKDROP_BLUR,
    );
    sketch::ph_fill(kit, full, sketch::BACKDROP_SCRIM);
    sketch::ph_fill(kit, full, sketch::BACKDROP_SHEET_SCRIM);
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &ChapterRequest) {
    let rows = chap_rows(req.chapters.len());
    let factor = b30::shrink(CHAP_SIZE);
    let canvas = kit.canvas();
    canvas.save();
    if (factor - 1.0).abs() > 1e-6 {
        canvas.scale((factor, factor));
    }
    backdrop(kit, root, req);
    head(kit, req);
    player::draw(
        kit,
        root,
        chap_width(),
        b30::B30_SCALE,
        b30::player_bar_dy(),
        &req.player,
    );
    stats(kit, req);
    grid(kit, root, req, rows);
    canvas.restore();
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_render::skia_safe::Image;
    use aurorix_render::{skia_safe, CanvasKit};
    use std::time::Instant;

    const OUT: &str = "/tmp/phigros-verify/16-chapter.png";

    const BANNERS: [&str; 14] = [
        "Chapter 5 霓虹灯牌",
        "Chapter 6 方舟蜃景",
        "Chapter 7 时钟链接",
        "Chapter 8 凌日潮汐",
        "Chapter 9 穹顶孤舟",
        "Chapter EX-茶鸣拾贰律 精选集",
        "Chapter EX-姜米條 精选集",
        "Chapter EX-节奏大师 精选集",
        "Chapter EX-CHUNITHM 精选集",
        "Chapter EX-CYTUS II 精选集",
        "Chapter 5 霓虹灯牌",
        "Chapter 6 方舟蜃景",
        "Chapter 7 时钟链接",
        "Chapter 8 凌日潮汐",
    ];

    fn progress(index: usize) -> ChapterProgress {
        let total = 24 + index as i32 * 3;
        ChapterProgress {
            name: format!("凌日潮汐 {}", index + 1),
            banner: String::from(BANNERS[index % BANNERS.len()]),
            played: total - (index as i32 % 7),
            total,
            cleared: total - 2 - (index as i32 % 5),
            full_combo: (index as i32).max(0) % 9,
            phi: (index as i32) % 4,
        }
    }

    fn request(count: usize, backdrop: &str) -> ChapterRequest {
        ChapterRequest {
            chapters: (0..count).map(progress).collect(),
            backdrop: String::from(backdrop),
            player: player::Player {
                has_player: true,
                name: String::from("Okamikko"),
                avatar: String::from("avatar_1"),
                rank: 3,
                rank_level: 4,
                rks: 16.1234,
            },
        }
    }

    fn song_ids(root: &Path) -> Vec<String> {
        let catalog = crate::domain::catalog::Catalog::open(root.join("../data/phi.db")).ok();
        let mut out: Vec<String> = Vec::new();
        if let Some(catalog) = catalog {
            for song in catalog.songs().iter().take(4) {
                out.push(song.id.clone());
            }
        }
        if out.is_empty() {
            out.push(String::from("000AinSophAur.Yumeji.0"));
        }
        out
    }

    #[test]
    fn rows_and_out_size_follow_the_grid() {
        assert_eq!(chap_rows(0), 1);
        assert_eq!(chap_rows(1), 1);
        assert_eq!(chap_rows(4), 1);
        assert_eq!(chap_rows(5), 2);
        assert_eq!(chap_rows(9), 3);
        assert_eq!(chap_rows(14), 4);
        assert_eq!(chap_rows(35), 9);
        assert!((chap_card_w() - 243.6).abs() < 1e-3);
        assert!((chap_card_h() - 261.8).abs() < 1e-3);
        let (width, height) = chap_out_size(9);
        assert_eq!(width, 1500);
        assert!(height > 3000 && height < 3600);
    }

    #[test]
    fn out_size_matches_canvas_size() {
        let req = request(35, "000AinSophAur.Yumeji.0");
        assert_eq!(canvas_size(&req), chap_out_size(9));
        let req = request(3, "000AinSophAur.Yumeji.0");
        assert_eq!(canvas_size(&req), chap_out_size(1));
    }

    #[test]
    fn last_row_is_centered() {
        let rows = 4;
        let first = chap_card(12, rows, 14);
        let second = chap_card(13, rows, 14);
        let base = chap_card(12, rows, 16);
        let stride = base.width() + chap_col_gap();
        assert!((first.left() - (base.left() + stride)).abs() < 1e-3);
        assert!((second.left() - (first.left() + stride)).abs() < 1e-3);
        assert!((second.right() - (base.right() + 2.0 * stride)).abs() < 1e-3);
        assert!((base.left() - chap_margin()).abs() < 1e-3);
        assert!(chap_card(16, rows, 16).width() <= 0.0);
        let grid = chap_grid_area(rows);
        assert!((grid.width() - (chap_width() - 2.0 * chap_margin())).abs() < 1e-3);
    }

    #[test]
    fn stat_texts_match_the_go_wording() {
        assert_eq!(count_text(3, 0), "");
        assert_eq!(count_text(3, 12), "3 / 12");
        assert_eq!(percent_text(3, 12), "25%");
        assert_eq!(played_text(0, 0), CHAP_MISSING);
        assert_eq!(played_text(6, 12), "6 / 12 · 50%");
        assert_eq!(note_text(&request(14, "")), "共 14 章");
        assert_eq!(note_text(&request(0, "")), "");
        assert_eq!(count_ink(0), settlement::LABEL_INK);
        assert_eq!(count_ink(2), settlement::SCORE_INK);
    }

    fn luma_variance(image: &Image) -> f64 {
        let (width, height) = (image.width(), image.height());
        let info = skia_safe::ImageInfo::new(
            (width, height),
            skia_safe::ColorType::RGBA8888,
            skia_safe::AlphaType::Unpremul,
            None,
        );
        let row = (width as usize) * 4;
        let mut data = vec![0u8; row * height as usize];
        if !image.read_pixels(
            &info,
            &mut data,
            row,
            (0, 0),
            skia_safe::image::CachingHint::Allow,
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

    #[test]
    #[ignore]
    fn render_chapter() {
        let root = crate::assets_root();
        crate::cards::register_fonts(&root);
        let assets = std::sync::Arc::new(crate::assets_in(&root));
        let context = aurorix_render::RenderContext::new(
            aurorix_render::SkiaBackend::handle(),
            64,
            512 * 1024 * 1024,
        )
        .with_asset_loader(assets.loader());
        let ids = song_ids(&root);
        for count in [14usize, 35usize] {
            let req = request(count, &ids[count % ids.len()]);
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
            let out = if count == 14 {
                String::from(OUT)
            } else {
                String::from("/tmp/phigros-verify/16-chapter-35.png")
            };
            if let Some(parent) = Path::new(&out).parent() {
                std::fs::create_dir_all(parent).expect("建出图目录");
            }
            std::fs::write(&out, &bytes).expect("写出 PNG");
            let image = Image::from_encoded(skia_safe::Data::new_copy(&bytes)).expect("解码头");
            assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
            assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
            assert!(bytes.len() > 20000, "字节数必须大于 20000");
            let variance = luma_variance(&image);
            println!(
                "章节 {} 章渲染耗时 {} ms ({} 字节, {width}x{height}, 亮度方差 {variance:.1})",
                count,
                elapsed.as_millis(),
                bytes.len()
            );
            assert!(variance > 100.0, "亮度方差必须大于 100");
        }
    }
}
