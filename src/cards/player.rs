use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::memo;
use super::sketch;

pub const SKEW: f32 = 0.2679492;

pub const CALIB_WIDTH: f32 = 1920.0;

pub const CALIB_HEIGHT: f32 = 886.0;

pub const SCENE_WIDTH: f32 = 2072.0;

pub const SCENE_HEIGHT: f32 = 955.4;

pub const SCENE_SCALE: f32 = CALIB_WIDTH / SCENE_WIDTH;

pub const PLAYER_BAR_SCALE: f32 = 0.82;

pub const NICKNAME_FONT_SIZE: f32 = 30.0;

pub const NICKNAME_BASELINE: f32 = 87.2;

pub const NICKNAME_RIGHT: f32 = 1636.0 - 18.0 - CONTENT_OFFSET;

pub const NICKNAME_PAD: f32 = 26.0;

pub const NICKNAME_MAX_WIDTH: f32 = 420.0;

pub const CONTENT_OFFSET: f32 = 20.0;

pub const PANEL_RIGHT: f32 = CALIB_WIDTH;

pub const PANEL_TOP: f32 = 27.8;

pub const PANEL_HEIGHT: f32 = 97.3;

pub const PANEL_BASE_WIDTH: f32 = 284.0 + CONTENT_OFFSET;

pub const RANK_BADGE_HEIGHT: f32 = 20.0;

pub const RANK_BADGE_WIDTH: f32 = 61.0;

pub const RANK_BADGE_TOP: f32 = 51.0;

pub const RANK_FONT_SIZE: f32 = 30.0;

pub const RANK_BASELINE: f32 = 64.8;

pub const PILL_TOP: f32 = 83.7;

pub const PILL_HEIGHT: f32 = 29.7;

pub const PILL_LEFT: f32 = 1799.0;

pub const PILL_FONT_SIZE: f32 = 26.0;

pub const PILL_BASELINE: f32 = 108.0;

pub const PILL_PAD: f32 = 3.0;

pub const MID_RATIO: f32 = 0.36;

pub const PANEL_INK: Color = Color::new(0x08, 0x08, 0x0C, 0x89);

pub const SCORE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const PILL_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const PILL_TEXT_INK: Color = Color::new(0x00, 0x00, 0x00, 0xFF);

pub const RANK_ACCENT_INK: Color = Color::new(0x57, 0xCB, 0xD0, 0xFF);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Player {
    pub has_player: bool,
    pub name: String,
    pub avatar: String,
    pub rank: i32,
    pub rank_level: i32,
    pub rks: f64,
}

pub fn player_of(binding: &crate::model::Binding, profile: Option<&crate::model::Profile>) -> Player {
    let dan = crate::domain::overview::dan_of(
        profile
            .map(|item| item.challenge_mode_rank)
            .unwrap_or_default() as i64,
    );
    Player {
        has_player: true,
        name: binding.nickname.clone(),
        avatar: binding.avatar_key(),
        rank: dan.progress,
        rank_level: dan.level,
        rks: profile.map(|item| item.rks).unwrap_or_default(),
    }
}

#[derive(Clone, Copy)]
pub struct Anchor {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub pivot_x: f32,
    pub pivot_y: f32,
}

pub const TOP_RIGHT: Anchor = Anchor {
    min_x: 1.0,
    min_y: 1.0,
    max_x: 1.0,
    max_y: 1.0,
    pivot_x: 1.0,
    pivot_y: 1.0,
};

pub const TOP_RIGHT_CENTERED: Anchor = Anchor {
    min_x: 1.0,
    min_y: 1.0,
    max_x: 1.0,
    max_y: 1.0,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

pub const CENTER: Anchor = Anchor {
    min_x: 0.5,
    min_y: 0.5,
    max_x: 0.5,
    max_y: 0.5,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

pub const BOTTOM_MID: Anchor = Anchor {
    min_x: 0.5,
    min_y: 0.0,
    max_x: 0.5,
    max_y: 0.0,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

pub const TOP_LEFT_CENTERED: Anchor = Anchor {
    min_x: 0.0,
    min_y: 1.0,
    max_x: 0.0,
    max_y: 1.0,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

pub const BOTTOM_RIGHT_CENTERED: Anchor = Anchor {
    min_x: 1.0,
    min_y: 0.0,
    max_x: 1.0,
    max_y: 0.0,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

pub const LEFT_MIDDLE: Anchor = Anchor {
    min_x: 0.0,
    min_y: 0.5,
    max_x: 0.0,
    max_y: 0.5,
    pivot_x: 0.5,
    pivot_y: 0.5,
};

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn from_scene(parent: Rect, anchor: Anchor, x: f32, y: f32, w: f32, h: f32) -> Rect {
    let ax = parent.width() * lerp(anchor.min_x, anchor.max_x, anchor.pivot_x);
    let ay = parent.height() * lerp(anchor.min_y, anchor.max_y, anchor.pivot_y);
    let width = w * SCENE_SCALE + (anchor.max_x - anchor.min_x) * parent.width();
    let height = h * SCENE_SCALE + (anchor.max_y - anchor.min_y) * parent.height();
    let min_x = ax + x * SCENE_SCALE - width * anchor.pivot_x;
    let min_y = ay + y * SCENE_SCALE - height * anchor.pivot_y;
    Rect::from_xywh(
        parent.left() + min_x,
        parent.bottom() - (min_y + height),
        width,
        height,
    )
}

pub fn content_area() -> Rect {
    Rect::from_xywh(0.0, 0.0, CALIB_WIDTH, CALIB_HEIGHT)
}

pub fn player_bar() -> Rect {
    from_scene(content_area(), TOP_RIGHT, 0.0, -30.0, 326.3, 105.0)
}

pub fn avatar_window() -> Rect {
    let panel = player_bar();
    let inner = from_scene(panel, LEFT_MIDDLE, 125.0, 0.0, 210.0, 120.0);
    Rect::from_xywh(
        inner.left() - CONTENT_OFFSET,
        inner.top(),
        inner.width(),
        inner.height(),
    )
}

pub fn panel_rect(nickname_width: f32) -> Rect {
    let mut w = PANEL_BASE_WIDTH;
    if nickname_width > 0.0 {
        let need = CALIB_WIDTH - NICKNAME_RIGHT + nickname_width + NICKNAME_PAD;
        if need > w {
            w = need;
        }
    }
    let flush = SKEW * PANEL_HEIGHT / 2.0;
    Rect::from_xywh(PANEL_RIGHT - w, PANEL_TOP, w + flush, PANEL_HEIGHT)
}

fn pill_right() -> f32 {
    PANEL_RIGHT + SKEW * PILL_HEIGHT / 2.0
}

pub fn pill_rect() -> Rect {
    Rect::from_xywh(PILL_LEFT, PILL_TOP, pill_right() - PILL_LEFT, PILL_HEIGHT)
}

fn pill_center_y() -> f32 {
    PILL_TOP + PILL_HEIGHT / 2.0
}

fn skew_shift(y0: f32, y1: f32, y: f32) -> f32 {
    (SKEW * ((y0 + y1 - 1.0) / 2.0 - y)).round()
}

pub fn pill_visible_left() -> f32 {
    let avatar = avatar_window();
    avatar.right() + skew_shift(avatar.top(), avatar.bottom(), pill_center_y())
}

pub fn pill_visible_right() -> f32 {
    let pill = pill_rect();
    (pill.right() + skew_shift(pill.top(), pill.bottom(), pill_center_y())).min(PANEL_RIGHT)
}

pub fn pill_visible_center() -> f32 {
    (pill_visible_left() + pill_visible_right()) / 2.0
}

pub fn rank_badge() -> Rect {
    Rect::from_xywh(
        pill_visible_center() + 2.0 - RANK_BADGE_WIDTH / 2.0,
        RANK_BADGE_TOP,
        RANK_BADGE_WIDTH,
        RANK_BADGE_HEIGHT,
    )
}

pub fn shift_rect(area: Rect, k: f32, sx: f32, sy: f32) -> Rect {
    Rect::from_xywh(
        (area.left() * k).round() + sx,
        (area.top() * k).round() + sy,
        (area.right() * k).round() + sx - ((area.left() * k).round() + sx),
        (area.bottom() * k).round() + sy - ((area.top() * k).round() + sy),
    )
}

pub fn nickname(kit: &CanvasKit<'_>, raw: &str) -> Option<(String, f32)> {
    let name = raw.trim();
    if name.is_empty() {
        return None;
    }
    let style = sketch::text_style(sketch::FONT_BODY, NICKNAME_FONT_SIZE, SCORE_INK);
    let trimmed = sketch::ellipsize(kit, &style, name, NICKNAME_MAX_WIDTH);
    if trimmed.is_empty() {
        return None;
    }
    let width = sketch::text_width(kit, &style, &trimmed).round();
    Some((trimmed, width))
}

pub fn four_decimals(value: f64) -> String {
    format!("{:.4}", value.max(0.0))
}

fn rank_text_pos(badge: Rect, baseline: f32, size: f32) -> (f32, f32) {
    let mid = baseline - size * MID_RATIO;
    let x =
        badge.left() + badge.width() / 2.0 + SKEW * (badge.top() + badge.height() / 2.0 - mid);
    (x, baseline)
}

fn badge_two_slashes(kit: &CanvasKit<'_>, frame: Rect) {
    let w = 6.0;
    let left = Rect::from_xywh(frame.left(), frame.top(), w, frame.height());
    let right = Rect::from_xywh(frame.right() - w, frame.top(), w, frame.height());
    sketch::fill_skew(kit, left, SKEW, RANK_ACCENT_INK);
    sketch::fill_skew(kit, right, SKEW, RANK_ACCENT_INK);
}

pub fn rank_frame(kit: &CanvasKit<'_>, root: &Path, frame: Rect, level: i32) {
    let key = super::ui_asset(root, memo::rank_badge_name(level));
    let Some(texture) = memo::texture(kit, &key) else {
        badge_two_slashes(kit, frame);
        return;
    };
    let Some(image) = texture.image() else {
        badge_two_slashes(kit, frame);
        return;
    };
    let Some(solid) = sketch::content_bounds(image) else {
        badge_two_slashes(kit, frame);
        return;
    };
    let scale = frame.width() / solid.width().max(1.0);
    let width = image.width() as f32 * scale;
    let height = image.height() as f32 * scale;
    let at = Rect::from_xywh(
        (frame.left() - solid.left() * scale).round(),
        (frame.top() - solid.top() * scale).round(),
        width.round(),
        height.round(),
    );
    sketch::image_fill(kit, image, at);
}

fn pill_text_layout(
    kit: &CanvasKit<'_>,
    label: &str,
    plate: Rect,
    avatar: Rect,
    pad: f32,
) -> Option<(f32, f32, f32)> {
    let y = (plate.top() + plate.bottom()) / 2.0;
    let mut left = plate.left() + skew_shift(plate.top(), plate.bottom(), y);
    let right = plate.right() + skew_shift(plate.top(), plate.bottom(), y);
    let visible = avatar.right() + skew_shift(avatar.top(), avatar.bottom(), y);
    if visible > left {
        left = visible;
    }
    let avail = right - left - 2.0 * pad;
    if avail <= 0.0 {
        return None;
    }
    let mut size = PILL_FONT_SIZE;
    let style = sketch::text_style(sketch::FONT_BODY, size, PILL_TEXT_INK);
    let width = sketch::text_width(kit, &style, label);
    if width > avail {
        size *= avail / width.max(1.0);
    }
    let center = (left + right) / 2.0;
    Some((center, y + size * MID_RATIO, size))
}

pub fn avatar(kit: &CanvasKit<'_>, root: &Path, win: Rect, chosen: &str) {
    let mut keys: Vec<String> = Vec::with_capacity(2);
    if !chosen.is_empty() {
        keys.push(super::avatar_asset(root, chosen));
    }
    keys.push(super::ui_asset(root, memo::DEFAULT_AVATAR));
    let skew = sketch::ph_rect(win.left(), win.top(), win.width(), win.height());
    for key in keys {
        let Some(texture) = memo::texture(kit, &key) else {
            continue;
        };
        let Some(image) = texture.image() else {
            continue;
        };
        sketch::image_sprite_skew(kit, image, skew, SKEW);
        return;
    }
}

pub fn draw(
    kit: &CanvasKit<'_>,
    root: &Path,
    canvas_w: f32,
    k: f32,
    dy: f32,
    player: &Player,
) {
    if !player.has_player {
        return;
    }
    let sx = (canvas_w - CALIB_WIDTH * k).round();
    let sy = dy.round();
    let name = nickname(kit, &player.name);
    let name_width = name.as_ref().map(|(_, width)| *width).unwrap_or(0.0);
    let panel = shift_rect(panel_rect(name_width), k, sx, sy);
    sketch::fill_skew(kit, panel, SKEW, PANEL_INK);
    if let Some((value, _)) = name.as_ref() {
        let style = sketch::text_style(sketch::FONT_BODY, NICKNAME_FONT_SIZE * k, SCORE_INK);
        sketch::text_right(
            kit,
            &style,
            value,
            NICKNAME_RIGHT * k + sx,
            NICKNAME_BASELINE * k + sy,
            SCORE_INK,
        );
    }
    let badge = shift_rect(rank_badge(), k, sx, sy);
    rank_frame(kit, root, badge, player.rank_level);
    let (at_x, at_baseline) = rank_text_pos(badge, RANK_BASELINE * k + sy, RANK_FONT_SIZE * k);
    let style = sketch::text_style(sketch::FONT_BODY, RANK_FONT_SIZE * k, SCORE_INK);
    sketch::text_center(
        kit,
        &style,
        &player.rank.to_string(),
        at_x,
        at_baseline,
        SCORE_INK,
    );
    let avatar_win = shift_rect(avatar_window(), k, sx, sy);
    let pill = shift_rect(pill_rect(), k, sx, sy);
    sketch::fill_skew(kit, pill, SKEW, PILL_INK);
    let label = four_decimals(player.rks);
    let visible = Rect::from_xywh(
        pill.left(),
        pill.top(),
        (canvas_w - pill.left()).max(1.0),
        pill.height(),
    );
    if let Some((center, baseline, size)) =
        pill_text_layout(kit, &label, visible, avatar_win, PILL_PAD * k)
    {
        let style = sketch::text_style(sketch::FONT_BODY, size, PILL_TEXT_INK);
        sketch::text_center(kit, &style, &label, center, baseline, PILL_TEXT_INK);
    }
    avatar(kit, root, avatar_win, &player.avatar);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_of_puts_level_on_frame_and_progress_on_digit() {
        let binding = crate::model::Binding {
            nickname: String::from("新手"),
            ..crate::model::Binding::default()
        };
        let profile = crate::model::Profile {
            challenge_mode_rank: 227,
            rks: 8.59,
            ..crate::model::Profile::default()
        };
        let player = player_of(&binding, Some(&profile));
        assert_eq!(player.rank_level, 2, "段位等级决定框的颜色");
        assert_eq!(player.rank, 27, "段内进度是框里那个数字");
        assert!((player.rks - 8.59).abs() < 1e-9);
        let played = crate::model::Profile {
            challenge_mode_rank: 502,
            ..crate::model::Profile::default()
        };
        let color = player_of(&binding, Some(&played));
        assert_eq!(color.rank_level, 5);
        assert_eq!(color.rank, 2);
        let empty = player_of(&binding, None);
        assert_eq!(empty.rank_level, 0);
        assert_eq!(empty.rank, 0);
    }

    #[test]
    fn player_bar_sits_at_canvas_top_right() {
        let bar = player_bar();
        assert!((bar.right() - CALIB_WIDTH).abs() < 0.001);
        assert!((bar.top() - PANEL_TOP).abs() < 0.001);
    }

    #[test]
    fn pill_starts_under_avatar_and_reaches_canvas_edge() {
        assert!(pill_visible_left() > avatar_window().left());
        assert!(pill_visible_left() < avatar_window().right());
        assert!((pill_visible_right() - CALIB_WIDTH).abs() < 0.001);
        assert!(pill_visible_center() > PILL_LEFT);
    }

    #[test]
    fn shift_rect_moves_calibrated_coordinates() {
        let area = Rect::from_xywh(100.0, 50.0, 60.0, 20.0);
        let moved = shift_rect(area, 0.82, 10.0, 5.0);
        assert_eq!(moved.left(), (100.0f32 * 0.82).round() + 10.0);
        assert_eq!(moved.top(), (50.0f32 * 0.82).round() + 5.0);
        assert_eq!(moved.width(), (160.0f32 * 0.82).round() - (100.0f32 * 0.82).round());
    }

    #[test]
    fn badge_center_sits_right_of_pill_visible_center() {
        let badge = rank_badge();
        assert!((badge.left() + badge.width() / 2.0 - (pill_visible_center() + 2.0)).abs() < 0.001);
        assert!((badge.top() - RANK_BADGE_TOP).abs() < 0.001);
    }

    #[test]
    fn four_decimals_clamps_negative_to_zero() {
        assert_eq!(four_decimals(16.42312), "16.4231");
        assert_eq!(four_decimals(-1.0), "0.0000");
    }
}
