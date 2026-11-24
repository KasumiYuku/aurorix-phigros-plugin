use std::path::Path;
use std::sync::Arc;

use aurorix_render::skia_safe::{color_filters, Image, Paint, PathBuilder, Rect};
use aurorix_render::{CanvasKit, Color, GradientStop, LinearGradient, Texture};

use super::settlement::{self, TextAt};
use super::{memo, player, sketch};

pub const LEAN: f32 = settlement::LEAN;

pub const MID_RATIO: f32 = settlement::MID_RATIO;

pub const SEL_OUT: f32 = 1.0;

pub const SEL_CANVAS_W: i32 = 2376;

pub const SEL_CANVAS_H: i32 = 1080;

pub const SEL_W: f32 = SEL_CANVAS_W as f32;

pub const SEL_H: f32 = SEL_CANVAS_H as f32;

pub const SEL_COVER_X0: f32 = 1081.0;

pub const SEL_COVER_Y0: f32 = 193.0;

pub const SEL_COVER_X1: f32 = 2135.0;

pub const SEL_COVER_Y1: f32 = 749.0;

pub const SEL_COVER_SHADE_H: f32 = 220.0;

pub const SEL_DIFF_BAR_Y0: f32 = 805.0;

pub const SEL_DIFF_BAR_Y1: f32 = 892.0;

pub const SEL_DIFF_BAR_W: f32 = 384.0;

pub const SEL_DIFF_BAR_X0: f32 = SEL_COVER_X0 + LEAN * (SEL_COVER_Y1 - SEL_DIFF_BAR_Y1);

pub const SEL_DIFF_BAR_X1: f32 = SEL_DIFF_BAR_X0 + SEL_DIFF_BAR_W;

pub const SEL_DIFF_SEL_W: f32 = 113.0;

pub const SEL_DIFF_SEL_Y0: f32 = 800.0;

pub const SEL_DIFF_SEL_Y1: f32 = 896.0;

pub const SEL_DIFF_NUM_BASE: f32 = 849.0;

pub const SEL_DIFF_LB_BASE: f32 = 875.0;

pub const SEL_DIFF_TAB_MAX: usize = 4;

pub const SEL_MIRROR_X0: f32 = 990.0;

pub const SEL_MIRROR_Y0: f32 = 934.0;

pub const SEL_MIRROR_X1: f32 = 1152.0;

pub const SEL_MIRROR_Y1: f32 = 993.0;

pub const SEL_MIRROR_ICON_X: f32 = 1029.0;

pub const SEL_MIRROR_ICON_Y: f32 = 948.0;

pub const SEL_MIRROR_ICON_SZ: f32 = 26.0;

pub const SEL_MIRROR_TEXT_X: f32 = 1034.0;

pub const SEL_MIRROR_BASE: f32 = 975.0;

pub const SEL_SONG_X0: f32 = 264.0;

pub const SEL_SONG_Y0: f32 = 347.0;

pub const SEL_SONG_X1: f32 = 1084.0;

pub const SEL_SONG_Y1: f32 = 502.0;

pub const SEL_TITLE_X: f32 = 336.0;

pub const SEL_TITLE_Y: f32 = 373.0;

pub const SEL_TITLE_W: f32 = 473.0;

pub const SEL_TITLE_H: f32 = 60.0;

pub const SEL_ARTIST_X: f32 = 334.0;

pub const SEL_ARTIST_Y: f32 = 447.0;

pub const SEL_ARTIST_W: f32 = 451.0;

pub const SEL_ARTIST_H: f32 = 32.0;

pub const SEL_LEVEL_X: f32 = 849.0;

pub const SEL_LEVEL_Y: f32 = 333.0;

pub const SEL_LEVEL_W: f32 = 213.0;

pub const SEL_LEVEL_H: f32 = 176.0;

pub const SEL_LEVEL_NUM_BASE: f32 = 430.0;

pub const SEL_LEVEL_LB_BASE: f32 = 468.0;

pub const SEL_ROWS: usize = 8;

pub const SEL_ROW_SEL: usize = 2;

pub const SEL_ROW_W: f32 = 800.0;

pub const SEL_ROW_CENTER: f32 = 421.0;

pub const SEL_ROW_NAME_W: f32 = 375.0;

pub const SEL_ROW_ICON_W: f32 = 82.0;

pub const SEL_ROW_ICON_CX: f32 = 484.0;

pub const SEL_ROW_CONST_R: f32 = 621.0;

pub const SEL_ROW_ICON_INK_W: f32 = 48.0;

pub const SEL_ROW_CONST_GAP: f32 = 42.0;

pub const SEL_ROW_BASE_OFF: f32 = 18.0;

pub const SEL_ROW_OFFSET: [f32; SEL_ROWS] =
    [-244.5, -136.0, 0.0, 135.5, 249.5, 361.0, 469.0, 581.0];

pub const SEL_ROW_NAME_LEFT: [f32; SEL_ROWS] = [422.0, 391.0, 334.0, 318.0, 288.0, 258.0, 229.0, 200.0];

pub const SEL_PLAY_X0: f32 = 2121.0;

pub const SEL_PLAY_Y0: f32 = 842.0;

pub const SEL_PLAY_X1: f32 = 2376.0;

pub const SEL_PLAY_Y1: f32 = 1004.0;

pub const SEL_PLAY_TRI_X0: f32 = 153.0;

pub const SEL_PLAY_TRI_X1: f32 = 206.0;

pub const SEL_PLAY_TRI_Y0: f32 = 70.0;

pub const SEL_PLAY_TRI_Y1: f32 = 146.0;

pub const SEL_PLAY_TRI_X0S: f32 = 2230.0;

pub const SEL_PLAY_TRI_X1S: f32 = 2294.0;

pub const SEL_PLAY_TRI_Y0S: f32 = 887.0;

pub const SEL_PLAY_TRI_Y1S: f32 = 958.0;

pub const SEL_PLAY_CROP_X0: f32 = 138.0;

pub const SEL_PLAY_CROP_Y0: f32 = 58.0;

pub const SEL_PLAY_CROP_X1: f32 = 222.0;

pub const SEL_PLAY_CROP_Y1: f32 = 162.0;

pub const SEL_NAV_SPRITE_X: f32 = 6.8;

pub const SEL_NAV_SPRITE_Y: f32 = 1.8;

pub const SEL_NAV_SPRITE_W: f32 = 190.0;

pub const SEL_NAV_SPRITE_H: f32 = 145.0;

pub const SEL_NAV_SCALE: f32 = 1.15;

pub const SEL_NAV_FILL_W: f32 = SEL_NAV_SPRITE_X;

pub const SEL_NAV_FILL_Y0: f32 = SEL_NAV_SPRITE_Y + 9.0 * SEL_NAV_SCALE;

pub const SEL_NAV_FILL_Y1: f32 = SEL_NAV_SPRITE_Y + 112.0 * SEL_NAV_SCALE;

pub const SEL_ARROW_X: f32 = 55.0;

pub const SEL_ARROW_Y: f32 = 51.0;

pub const SEL_ARROW_W: f32 = 44.0;

pub const SEL_ARROW_H: f32 = 42.0;

pub const SEL_TOP_X0: f32 = 386.0;

pub const SEL_TOP_Y0: f32 = 0.0;

pub const SEL_TOP_X1: f32 = 1157.3;

pub const SEL_TOP_Y1: f32 = 128.0;

pub const SEL_TOP_FADE_H: f32 = 44.0;

pub const SEL_TOP_RAND_ICON: [f32; 4] = [881.0, 42.0, 931.0, 76.0];

pub const SEL_TOP_GEAR_ICON: [f32; 4] = [1042.0, 42.0, 1082.0, 78.0];

pub const SEL_TOP_RAND_LABEL: [f32; 3] = [906.0, 108.0, 22.0];

pub const SEL_TOP_GEAR_LABEL: [f32; 3] = [1062.0, 107.0, 22.0];

pub const SEL_TOP_SORT_LABEL: [f32; 3] = [463.0, 59.0, 22.0];

pub const SEL_TOP_SORT_ICON: [f32; 4] = [420.0, 82.0, 450.0, 105.0];

pub const SEL_TOP_SORT_VALUE: [f32; 3] = [494.5, 105.0, 30.0];

pub const SEL_TOP_SEP_CX: f32 = 983.5;

pub const SEL_TOP_SEP_Y0: f32 = 44.0;

pub const SEL_TOP_SEP_Y1: f32 = 100.0;

pub const SEL_TOP_SEP_W: f32 = 1.0;

pub const SEL_TOP_RULE_X0: f32 = 1027.0;

pub const SEL_TOP_RULE_X1: f32 = 1032.0;

pub const SEL_TOP_RULE_Y0: f32 = 44.0;

pub const SEL_TOP_RULE_Y1: f32 = 86.0;

pub const SEL_TOP_SEP_INK: Color = Color::new(0x8D, 0x8D, 0x8D, 0xFF);

pub const SEL_TOP_RULE_INK: Color = Color::new(0x62, 0x62, 0x62, 0xFF);

pub const SEL_SCORE_X: f32 = 1498.0;

pub const SEL_SCORE_BAR_X0: f32 = SEL_DIFF_BAR_X1 - LEAN * (SEL_DIFF_BAR_Y1 - SEL_DIFF_BAR_Y0) / 2.0;

pub const SEL_SCORE_BAR_Y0: f32 = 805.0;

pub const SEL_SCORE_BAR_X1: f32 = 2110.0;

pub const SEL_SCORE_BAR_Y1: f32 = 891.0;

pub const SEL_SCORE_BAR_FADE: f32 = 1930.0;

pub const SEL_SCORE_NUM_BASE: f32 = 866.0;

pub const SEL_SCORE_ACC_X: f32 = 1786.0;

pub const SEL_SCORE_ACC_BASE: f32 = 866.0;

pub const SEL_SCORE_NEW_X: f32 = 1864.0;

pub const SEL_SCORE_NEW_BASE: f32 = 848.0;

pub const SEL_CONTENT_DX: f32 = 32.0;

pub const SEL_PLAYER_X0: f32 = 2013.0 - SEL_CONTENT_DX;

pub const SEL_PLAYER_Y0: f32 = 38.0;

pub const SEL_PLAYER_X1: f32 = 2376.0;

pub const SEL_PLAYER_Y1: f32 = 156.0;

pub const SEL_AVATAR_X0: f32 = 2036.0 - SEL_CONTENT_DX;

pub const SEL_AVATAR_X1: f32 = 2250.0 - SEL_CONTENT_DX;

pub const SEL_RANK_X0: f32 = 2276.0;

pub const SEL_RANK_Y0: f32 = 64.0;

pub const SEL_RANK_X1: f32 = 2349.0;

pub const SEL_RANK_Y1: f32 = 88.0;

pub const SEL_RKS_X0: f32 = 2240.0 - SEL_CONTENT_DX - 8.0;

pub const SEL_RKS_Y0: f32 = 105.0;

pub const SEL_RKS_X1: f32 = 2376.0;

pub const SEL_RKS_Y1: f32 = 141.0;

pub const SEL_PLAYER_NAME_GAP: f32 = 18.0;

pub const SEL_PLAYER_NAME_PAD: f32 = 26.0;

pub const SEL_PLAYER_NAME_MAX_WIDTH: f32 = 520.0;

pub const SEL_PLAYER_NAME_BASE: f32 = 107.8;

pub const SEL_RKS_TEXT_PAD: f32 = 3.0;

pub const SEL_SIZE_TITLE: f32 = 61.0;

pub const SEL_SIZE_ARTIST: f32 = 31.0;

pub const SEL_SIZE_LEVEL: f32 = 68.0;

pub const SEL_SIZE_LEVEL_LB: f32 = 30.0;

pub const SEL_SIZE_ROW_NAME: f32 = 38.5;

pub const SEL_SIZE_ROW_CONST: f32 = 58.0;

pub const SEL_SIZE_TAB_NUM: f32 = 43.0;

pub const SEL_SIZE_TAB_LB: f32 = 30.0;

pub const SEL_SIZE_MIRROR: f32 = 26.0;

pub const SEL_SIZE_NAV: f32 = 28.0;

pub const SEL_SIZE_NAME: f32 = 30.0;

pub const SEL_SIZE_RANK: f32 = 38.0;

pub const SEL_SIZE_RKS: f32 = 40.0;

pub const SEL_SIZE_TOP: f32 = 26.0;

pub const SEL_SIZE_SCORE: f32 = 55.0;

pub const SEL_SIZE_SCORE_ACC: f32 = 17.0;

pub const SEL_SIZE_SCORE_NEW: f32 = 38.0;

pub const SEL_RKS_SIZE: f32 = 34.0;

pub const SEL_RANK_BASE: f32 = 80.0;

pub const SEL_SCRIM: Color = Color::new(0x05, 0x07, 0x0C, 0x5C);

pub const SEL_SIDE_SCRIM: Color = Color::new(0x03, 0x05, 0x0A, 0x2E);

pub const SEL_BOTTOM_SCRIM: Color = Color::new(0x03, 0x05, 0x0A, 0x30);

pub const SEL_BOTTOM_SCRIM_Y: f32 = 858.0;

pub const SEL_BOTTOM_SCRIM_RAMP_H: f32 = 44.0;

pub const SEL_LIST_SCRIM: Color = Color::new(0x02, 0x03, 0x06, 0x30);

pub const SEL_PLATE_INK: Color = Color::new(0x06, 0x08, 0x0E, 0xB4);

pub const SEL_NAME_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const SEL_DIM_INK: Color = Color::new(0xD6, 0xDC, 0xE6, 0xE0);

pub const SEL_TAB_ON_INK: Color = Color::new(0xCE, 0x13, 0x12, 0xFF);

pub const SEL_TAB_OFF_INK: Color = Color::new(0xFC, 0xFC, 0xFC, 0xFF);

pub const SEL_TAB_ON_TEXT: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const SEL_TAB_OFF_TEXT: Color = Color::new(0x33, 0x35, 0x38, 0xFF);

pub const SEL_LEVEL_INK: Color = Color::new(0xFC, 0xFC, 0xFC, 0xFF);

pub const SEL_LEVEL_TEXT: Color = Color::new(0x2A, 0x2C, 0x30, 0xFF);

pub const SEL_PLAY_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const SEL_RANK_TEXT_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const SEL_TOP_INK: Color = Color::new(0x02, 0x02, 0x04, 0xA7);

pub const SEL_TOP_TEXT: Color = Color::new(0xF2, 0xF2, 0xF2, 0xFF);

pub const SEL_MIRROR_INK: Color = Color::new(0x0C, 0x0C, 0x10, 0xE6);

pub const SEL_SCORE_INK: Color = Color::new(0x0A, 0x0A, 0x0C, 0x5C);

pub const SEL_SCORE_TEXT: Color = Color::new(0xF4, 0xF4, 0xF4, 0xFF);

pub const SEL_SCORE_NEW_INK: Color = Color::new(0x8F, 0x8F, 0x8F, 0xFF);

pub const SEL_SHADE_INK: Color = Color::new(0x04, 0x06, 0x0A, 0x24);

pub const SEL_RKS_PILL_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xFF);

pub const SEL_RKS_TEXT_INK: Color = Color::new(0x00, 0x00, 0x00, 0xFF);

pub const SEL_STRIP_Y0: f32 = 557.7;

pub const SEL_STRIP_Y1: f32 = 698.1;

pub const SEL_STRIP_W: f32 = 942.3;

pub const SEL_STRIP_UNDO_DX: f32 = 10.9;

pub const SEL_STRIP_UNDO_W: f32 = 140.4;

pub const SEL_STRIP_CARD_DX: f32 = 123.7;

pub const SEL_STRIP_CARD_W: f32 = 302.4;

pub const SEL_STRIP_CARD_PITCH: f32 = 259.5;

pub const SEL_STRIP_CARD_Y0: f32 = 548.0;

pub const SEL_STRIP_CARD_Y1: f32 = 707.8;

pub const SEL_STRIP_BADGE_W: f32 = 81.0;

pub const SEL_STRIP_BADGE_H: f32 = 43.2;

pub const SEL_STRIP_BADGE_RIGHT: f32 = 39.5;

pub const SEL_STRIP_BADGE_SIZE: f32 = 34.0;

pub const SEL_STRIP_BADGE_INSET: f32 = 0.35;

pub const SEL_STRIP_UNDO_SZ: f32 = 41.0;

pub const SEL_STRIP_UNDO_ICON_CY: f32 = 50.0;

pub const SEL_STRIP_UNDO_BASE: f32 = 107.0;

pub const SEL_STRIP_UNDO_SIZE: f32 = 26.0;

pub const SEL_CHECK_X0: f32 = 1710.7;

pub const SEL_CHECK_X1: f32 = 1941.1;

pub const SEL_CHECK_H: f32 = 92.9;

pub const SEL_CHECK_ICON_W: f32 = 53.7;

pub const SEL_CHECK_ICON_H: f32 = 36.2;

pub const SEL_CHECK_Y0: f32 = (SEL_SCORE_BAR_Y0 + SEL_SCORE_BAR_Y1) / 2.0 - SEL_CHECK_H / 2.0;

pub const SEL_CHECK_Y1: f32 = SEL_CHECK_Y0 + SEL_CHECK_H;

pub const SEL_CHECK_INK: Color = Color::new(0xBE, 0xBE, 0xBE, 0xFF);

pub const SEL_CHECK_MARK_INK: Color = Color::new(0x40, 0x40, 0x40, 0xFF);

pub const SEL_CHECK_SPRITE: &str = "sel_check";

pub const SEL_PLAY_SPRITE: &str = "sel_play";

pub const SEL_BACK_SPRITE: &str = "sel_back";

pub const SEL_ARROW_SPRITE: &str = "sel_arrow";

pub const SEL_UNDO_SPRITE: &str = "sel_undo";

pub const SEL_GRADE_SPRITE: &str = "grade_phi";

pub const SEL_RANDOM_SPRITE: &str = "sel_random";

pub const SEL_GEAR_SPRITE: &str = "sel_gear";

pub const SEL_SORT_SPRITE: &str = "sel_sort";

pub const SEL_UNDO_TEXT: &str = "UNDO";

pub const SEL_MIRROR_TEXT: &str = "Mirror";

pub const SEL_SCORE_TEXT_VALUE: &str = "1000000";

pub const SEL_SCORE_ACC_VALUE: &str = "100.00%";

pub const SEL_SCORE_NEW_VALUE: &str = "NEW";

pub const SEL_RANDOM_LABEL: &str = "随机";

pub const SEL_GEAR_LABEL: &str = "设置";

pub const SEL_SORT_LABEL: &str = "排序方式";

pub const SEL_SORT_VALUE: &str = "难度";

pub const SEL_UNKNOWN_CONST: &str = "?";

pub const SEL_ARTIST_BLANK: &str = "-";

#[derive(Clone, Debug, PartialEq)]
pub struct SongSelectLevel {
    pub diff: String,
    pub const_text: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SongSelectRow {
    pub title: String,
    pub constant: f64,
    pub known: bool,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SongSelectThumb {
    pub song_id: String,
    pub badge: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SongSelectRequest {
    pub backdrop: String,
    pub song_id: String,
    pub title: String,
    pub composer: String,
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub levels: Vec<SongSelectLevel>,
    pub rows: Vec<SongSelectRow>,
    pub strip: Vec<SongSelectThumb>,
    pub player: player::Player,
}

pub fn sel_px(value: f32) -> f32 {
    value * SEL_OUT
}

pub fn sel_pxi(value: f32) -> f32 {
    (value * SEL_OUT).round()
}

pub fn sel_rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_xywh(sel_px(x0), sel_px(y0), sel_px(x1 - x0), sel_px(y1 - y0))
}

pub fn sel_boxed(x: f32, y: f32, w: f32, h: f32) -> Rect {
    let pad = LEAN * sel_px(h) / 2.0;
    Rect::from_xywh(sel_px(x) + pad, sel_px(y), sel_px(w) - 2.0 * pad, sel_px(h))
}

pub fn sel_boxed_rect(area: Rect) -> Rect {
    let pad = LEAN * area.height() / 2.0;
    Rect::from_xywh(
        area.left() + pad,
        area.top(),
        area.width() - 2.0 * pad,
        area.height(),
    )
}

pub fn skew_at(area: Rect, y: f32) -> f32 {
    LEAN * (area.top() + area.height() / 2.0 - y)
}

pub fn flush_right(h: f32) -> f32 {
    LEAN * h / 2.0
}

pub fn cover_visible() -> Rect {
    sel_rect(SEL_COVER_X0, SEL_COVER_Y0, SEL_COVER_X1, SEL_COVER_Y1)
}

pub fn cover_rect() -> Rect {
    sel_boxed(
        SEL_COVER_X0,
        SEL_COVER_Y0,
        SEL_COVER_X1 - SEL_COVER_X0,
        SEL_COVER_Y1 - SEL_COVER_Y0,
    )
}

pub fn diff_tab_center(index: usize, count: usize) -> f32 {
    if count == 0 {
        return SEL_DIFF_BAR_X0;
    }
    SEL_DIFF_BAR_X0
        + (index as f32 + 0.5) * (SEL_DIFF_BAR_X1 - SEL_DIFF_BAR_X0) / count as f32
}

pub fn diff_bar_rect() -> Rect {
    sel_boxed(
        SEL_DIFF_BAR_X0,
        SEL_DIFF_BAR_Y0,
        SEL_DIFF_BAR_W,
        SEL_DIFF_BAR_Y1 - SEL_DIFF_BAR_Y0,
    )
}

pub fn diff_sel_rect(index: usize, count: usize) -> Rect {
    let center = diff_tab_center(index, count);
    sel_rect(
        center - SEL_DIFF_SEL_W / 2.0,
        SEL_DIFF_SEL_Y0,
        center + SEL_DIFF_SEL_W / 2.0,
        SEL_DIFF_SEL_Y1,
    )
}

pub fn level_text_w() -> f32 {
    sel_boxed(SEL_LEVEL_X, SEL_LEVEL_Y, SEL_LEVEL_W, SEL_LEVEL_H).width() / SEL_OUT - 24.0
}

pub fn row_const_w() -> f32 {
    SEL_ROW_CONST_R - (SEL_ROW_ICON_CX + SEL_ROW_ICON_INK_W / 2.0) - SEL_ROW_CONST_GAP
}

pub fn row_center_at(index: usize) -> f32 {
    let slot = if index >= SEL_ROWS { SEL_ROW_SEL } else { index };
    SEL_ROW_CENTER + SEL_ROW_OFFSET[slot]
}

pub fn row_name_left_at(index: usize) -> f32 {
    let slot = if index >= SEL_ROWS { SEL_ROW_SEL } else { index };
    SEL_ROW_NAME_LEFT[slot]
}

pub fn play_plate_rect() -> Rect {
    let h = 162.0;
    let pad = LEAN * sel_px(h) / 2.0;
    let w = (2376.0 - 2162.0) + 2.0 * pad / SEL_OUT;
    Rect::from_xywh(
        sel_px(2162.0) - pad,
        sel_px(SEL_PLAY_Y0),
        sel_px(w),
        sel_px(h),
    )
}

pub fn play_tri_draw() -> Rect {
    let sx = sel_px(SEL_PLAY_TRI_X1S - SEL_PLAY_TRI_X0S) / (SEL_PLAY_TRI_X1 - SEL_PLAY_TRI_X0);
    let sy = sel_px(SEL_PLAY_TRI_Y1S - SEL_PLAY_TRI_Y0S) / (SEL_PLAY_TRI_Y1 - SEL_PLAY_TRI_Y0);
    Rect::from_xywh(
        sel_px(SEL_PLAY_TRI_X0S) - (SEL_PLAY_TRI_X0 - SEL_PLAY_CROP_X0) * sx,
        sel_px(SEL_PLAY_TRI_Y0S) - (SEL_PLAY_TRI_Y0 - SEL_PLAY_CROP_Y0) * sy,
        (SEL_PLAY_CROP_X1 - SEL_PLAY_CROP_X0) * sx,
        (SEL_PLAY_CROP_Y1 - SEL_PLAY_CROP_Y0) * sy,
    )
}

pub fn nav_corner_fill() -> Rect {
    sel_rect(0.0, SEL_NAV_FILL_Y0, SEL_NAV_FILL_W, SEL_NAV_FILL_Y1)
}

pub fn nav_sprite_rect() -> Rect {
    sel_rect(
        SEL_NAV_SPRITE_X,
        SEL_NAV_SPRITE_Y,
        SEL_NAV_SPRITE_X + SEL_NAV_SPRITE_W * SEL_NAV_SCALE,
        SEL_NAV_SPRITE_Y + SEL_NAV_SPRITE_H * SEL_NAV_SCALE,
    )
}

pub fn top_ref_rect() -> Rect {
    let h = sel_px(SEL_TOP_Y1 + SEL_TOP_FADE_H);
    let area = sel_rect(
        SEL_TOP_X0,
        SEL_TOP_Y0,
        SEL_TOP_X1,
        SEL_TOP_Y1 + SEL_TOP_FADE_H,
    );
    Rect::from_ltrb(
        area.left() - LEAN * (h - 1.0) / 2.0,
        area.top(),
        area.right(),
        area.bottom(),
    )
}

pub fn list_scrim_rect() -> Rect {
    let top = top_ref_rect();
    Rect::from_xywh(top.left(), 0.0, top.width(), sel_px(SEL_H))
}

pub fn player_rect() -> Rect {
    let area = sel_rect(SEL_PLAYER_X0, SEL_PLAYER_Y0, SEL_PLAYER_X1, SEL_PLAYER_Y1);
    let pad = 2.0 * flush_right(sel_px(SEL_PLAYER_Y1 - SEL_PLAYER_Y0));
    Rect::from_xywh(area.left(), area.top(), area.width() + pad, area.height())
}

pub fn avatar_rect() -> Rect {
    sel_rect(SEL_AVATAR_X0, SEL_PLAYER_Y0, SEL_AVATAR_X1, SEL_PLAYER_Y1)
}

pub fn rks_mid_y() -> f32 {
    (sel_px(SEL_RKS_Y0) + sel_px(SEL_RKS_Y1)) / 2.0
}

pub fn rks_right() -> f32 {
    (SEL_W + 2.0 * flush_right(sel_px(SEL_RKS_Y1 - SEL_RKS_Y0))) / SEL_OUT
}

pub fn rks_rect() -> Rect {
    sel_rect(SEL_RKS_X0, SEL_RKS_Y0, rks_right(), SEL_RKS_Y1)
}

pub fn rks_visible_left() -> f32 {
    let win = sel_boxed_rect(avatar_rect());
    win.right() + skew_at(win, rks_mid_y())
}

pub fn rks_visible_right() -> f32 {
    let plate = sel_boxed_rect(rks_rect());
    (plate.right() + skew_at(plate, rks_mid_y())).min(SEL_W)
}

pub fn rks_visible_center_x() -> f32 {
    (rks_visible_left() + rks_visible_right()) / 2.0
}

pub fn rank_rect() -> Rect {
    let w = SEL_RANK_X1 - SEL_RANK_X0;
    let mid = (rks_visible_center_x() + sel_px(2.0)) / SEL_OUT;
    sel_rect(mid - w / 2.0, SEL_RANK_Y0, mid + w / 2.0, SEL_RANK_Y1)
}

pub fn avatar_visible_left() -> f32 {
    sel_boxed_rect(avatar_rect()).left()
}

pub fn player_name_at() -> TextAt {
    TextAt::right(
        avatar_visible_left() - sel_px(SEL_PLAYER_NAME_GAP),
        sel_px(SEL_PLAYER_NAME_BASE),
        sel_px(SEL_SIZE_NAME),
    )
}

pub fn player_panel(name_width: f32) -> Rect {
    let panel = player_rect();
    if name_width <= 0.0 {
        return panel;
    }
    let left =
        avatar_visible_left() - sel_px(SEL_PLAYER_NAME_GAP) - name_width - sel_px(SEL_PLAYER_NAME_PAD);
    if left >= panel.left() {
        return panel;
    }
    Rect::from_xywh(
        left,
        panel.top(),
        panel.width() + (panel.left() - left),
        panel.height(),
    )
}

pub fn rank_text_at() -> TextAt {
    let badge = rank_rect();
    let baseline = sel_px(SEL_RANK_BASE);
    let size = sel_px(SEL_SIZE_RANK);
    let mid = baseline - size * MID_RATIO;
    let x = badge.left() + badge.width() / 2.0 + LEAN * (badge.top() + badge.height() / 2.0 - mid);
    TextAt::center(x, baseline, size)
}

pub fn rks_text_at() -> TextAt {
    TextAt::center(
        sel_px((SEL_RKS_X0 + SEL_RKS_X1) / 2.0),
        sel_px(SEL_RKS_Y1 - 7.0),
        sel_px(SEL_RKS_SIZE),
    )
}

pub fn strip_x0() -> f32 {
    SEL_COVER_X0 + LEAN * (SEL_COVER_Y1 - SEL_STRIP_Y1)
}

pub fn strip_rect() -> Rect {
    let x0 = strip_x0();
    sel_rect(x0, SEL_STRIP_Y0, x0 + SEL_STRIP_W, SEL_STRIP_Y1)
}

pub fn strip_undo_rect() -> Rect {
    let x0 = strip_x0() + SEL_STRIP_UNDO_DX;
    sel_rect(x0, SEL_STRIP_Y0, x0 + SEL_STRIP_UNDO_W, SEL_STRIP_Y1)
}

pub fn strip_card_rect(index: usize) -> Rect {
    let x = strip_x0() + SEL_STRIP_CARD_DX + index as f32 * SEL_STRIP_CARD_PITCH;
    sel_rect(x, SEL_STRIP_CARD_Y0, x + SEL_STRIP_CARD_W, SEL_STRIP_CARD_Y1)
}

pub fn strip_badge_rect(index: usize) -> Rect {
    let card = strip_card_rect(index);
    let right = card.right() - sel_px(SEL_STRIP_BADGE_RIGHT);
    let bottom = card.bottom();
    Rect::from_xywh(
        right - sel_px(SEL_STRIP_BADGE_W),
        bottom - sel_px(SEL_STRIP_BADGE_H),
        sel_px(SEL_STRIP_BADGE_W),
        sel_px(SEL_STRIP_BADGE_H),
    )
}

pub fn check_rect() -> Rect {
    sel_boxed(
        SEL_CHECK_X0,
        SEL_CHECK_Y0,
        SEL_CHECK_X1 - SEL_CHECK_X0,
        SEL_CHECK_Y1 - SEL_CHECK_Y0,
    )
}

pub fn canvas_size(_req: &SongSelectRequest) -> (i32, i32) {
    (SEL_CANVAS_W, SEL_CANVAS_H)
}

fn tint_paint(ink: Color) -> Paint {
    let matrix: [f32; 20] = [
        0.0,
        0.0,
        0.0,
        0.0,
        f32::from(ink.r),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::from(ink.g),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::from(ink.b),
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
    ];
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color_filter(color_filters::matrix_row_major(&matrix, None));
    paint
}

fn skew_ref_path(area: Rect, reference: Rect) -> aurorix_render::skia_safe::Path {
    let top = skew_at(reference, area.top());
    let bottom = skew_at(reference, area.bottom());
    let mut builder = PathBuilder::new();
    builder.move_to((area.left() + top, area.top()));
    builder.line_to((area.right() + top, area.top()));
    builder.line_to((area.right() + bottom, area.bottom()));
    builder.line_to((area.left() + bottom, area.bottom()));
    builder.close();
    builder.detach()
}

fn fill_skew_ref(kit: &CanvasKit<'_>, area: Rect, reference: Rect, ink: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(ink.to_skia());
    kit.canvas()
        .draw_path(&skew_ref_path(area, reference), &paint);
}

fn fill_skew_ref_gradient(
    kit: &CanvasKit<'_>,
    area: Rect,
    reference: Rect,
    top: Color,
    bottom: Color,
) {
    let gradient = LinearGradient::vertical([GradientStop::new(0.0, top), GradientStop::new(1.0, bottom)]);
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas()
        .draw_path(&skew_ref_path(area, reference), &paint);
}

fn fill_vgradient(kit: &CanvasKit<'_>, area: Rect, top: Color, bottom: Color) {
    let gradient = LinearGradient::vertical([GradientStop::new(0.0, top), GradientStop::new(1.0, bottom)]);
    let Ok(shader) = gradient.shader(area) else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_shader(shader);
    kit.canvas().draw_rect(area, &paint);
}

fn cover_texture(kit: &CanvasKit<'_>, root: &Path, song_id: &str) -> Option<Arc<Texture>> {
    if song_id.is_empty() {
        return None;
    }
    memo::texture(kit, &super::thumb_key(root, song_id))
        .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
}

fn backdrop_texture(kit: &CanvasKit<'_>, root: &Path, song_id: &str) -> Option<Arc<Texture>> {
    if song_id.is_empty() {
        return None;
    }
    memo::texture(kit, &super::blur_key(root, song_id))
        .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
}

fn mirror_cover(kit: &CanvasKit<'_>, image: &Image, area: Rect) {
    let canvas = kit.canvas();
    canvas.save();
    canvas.translate((area.left() + area.right(), 0.0));
    canvas.scale((-1.0, 1.0));
    sketch::image_cover(kit, image, area);
    canvas.restore();
}

fn cover_skew(kit: &CanvasKit<'_>, root: &Path, song_id: &str, visible: Rect) {
    let win = sel_boxed_rect(visible);
    let Some(texture) = cover_texture(kit, root, song_id) else {
        sketch::fill_skew(kit, win, LEAN, settlement::COVER_FALLBACK);
        return;
    };
    let Some(image) = texture.image() else {
        sketch::fill_skew(kit, win, LEAN, settlement::COVER_FALLBACK);
        return;
    };
    let dest = sketch::skew_bounds(win, LEAN);
    sketch::clip_skew(kit, win, LEAN, || mirror_cover(kit, image, dest));
}

fn sprite(kit: &CanvasKit<'_>, root: &Path, name: &str, area: Rect) {
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    sketch::image_fill(kit, image, sketch::ph_rect(area.left(), area.top(), area.width(), area.height()));
}

fn sprite_nearest(kit: &CanvasKit<'_>, root: &Path, name: &str, area: Rect) {
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    sketch::image_nearest(kit, image, sketch::ph_rect(area.left(), area.top(), area.width(), area.height()));
}

fn sprite_tinted(kit: &CanvasKit<'_>, root: &Path, name: &str, area: Rect, ink: Color) {
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, name)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let source = Rect::from_xywh(0.0, 0.0, image.width() as f32, image.height() as f32);
    let paint = tint_paint(ink);
    sketch::image_part_paint(
        kit,
        image,
        source,
        sketch::ph_rect(area.left(), area.top(), area.width(), area.height()),
        &paint,
    );
}

fn line(kit: &CanvasKit<'_>, at: TextAt, value: &str, ink: Color) {
    settlement::line(kit, at, value, ink);
}

fn fit_size(
    kit: &CanvasKit<'_>,
    font: &str,
    value: &str,
    max_width: f32,
    max_size: f32,
    min_size: f32,
) -> f32 {
    let limit = max_width.round();
    let mut size = max_size;
    while size > min_size {
        let style = sketch::text_style(font, size, settlement::SCORE_INK);
        if sketch::text_width(kit, &style, value).round() <= limit {
            break;
        }
        size -= 1.0;
    }
    size
}

fn const_text(value: f64, known: bool) -> String {
    if !known {
        return String::from(SEL_UNKNOWN_CONST);
    }
    sketch::one_decimal(value)
}

fn backdrop(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    let canvas = sketch::ph_rect(0.0, 0.0, SEL_W, SEL_H);
    sketch::ph_fill(kit, canvas, settlement::COVER_FALLBACK);
    if let Some(texture) = backdrop_texture(kit, root, song_id) {
        if let Some(image) = texture.image() {
            sketch::image_cover(kit, image, canvas);
        }
    }
    sketch::ph_fill(kit, canvas, SEL_SCRIM);
    side_scrim(kit);
    fill_skew_ref(kit, list_scrim_rect(), top_ref_rect(), SEL_LIST_SCRIM);
    bottom_scrim(kit);
}

fn side_scrim(kit: &CanvasKit<'_>) {
    let edge = (SEL_W * 0.72).round();
    if edge <= 0.0 {
        return;
    }
    let base = f32::from(SEL_SIDE_SCRIM.a);
    let mut x = 0.0;
    while x < edge {
        let t = 1.0 - x / edge;
        let alpha = (base * t * t) as u8;
        if alpha > 0 {
            sketch::ph_fill(
                kit,
                Rect::from_ltrb(x, 0.0, x + 1.0, SEL_H),
                SEL_SIDE_SCRIM.with_alpha(alpha),
            );
        }
        x += 1.0;
    }
}

fn bottom_scrim(kit: &CanvasKit<'_>) {
    let top = sel_px(SEL_BOTTOM_SCRIM_Y);
    let end = sel_px(SEL_BOTTOM_SCRIM_Y + SEL_BOTTOM_SCRIM_RAMP_H);
    let ramp = Rect::from_ltrb(0.0, top, SEL_W, end);
    fill_vgradient(kit, ramp, SEL_BOTTOM_SCRIM.with_alpha(0), SEL_BOTTOM_SCRIM);
    sketch::ph_fill(
        kit,
        Rect::from_ltrb(0.0, end, SEL_W, SEL_H),
        SEL_BOTTOM_SCRIM,
    );
}

fn cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    let view = cover_visible();
    cover_skew(kit, root, song_id, view);
    let shade_h = sel_px(SEL_COVER_SHADE_H);
    let shade = Rect::from_ltrb(
        view.left(),
        view.bottom() - shade_h,
        view.right(),
        view.bottom(),
    );
    fill_skew_ref_gradient(
        kit,
        shade,
        view,
        SEL_SHADE_INK.with_alpha(0),
        SEL_SHADE_INK,
    );
}

fn list(kit: &CanvasKit<'_>, root: &Path, req: &SongSelectRequest) {
    let icon = sel_pxi(SEL_ROW_ICON_W);
    for (index, item) in req.rows.iter().enumerate() {
        if item.title.is_empty() || item.selected {
            continue;
        }
        let center_y = sel_px(row_center_at(index));
        let name_x = sel_px(row_name_left_at(index));
        let base = center_y + sel_px(SEL_ROW_BASE_OFF);
        row_name(kit, name_x, base, &item.title, SEL_DIM_INK);
        let center_x = name_x.floor() + sel_pxi(SEL_ROW_ICON_CX);
        sprite(
            kit,
            root,
            SEL_GRADE_SPRITE,
            Rect::from_xywh(
                (center_x - icon / 2.0).round(),
                (center_y.floor() - icon / 2.0).round(),
                icon,
                icon,
            ),
        );
        let value = const_text(item.constant, item.known);
        let size = fit_size(
            kit,
            sketch::FONT_BODY,
            &value,
            sel_pxi(row_const_w()),
            SEL_SIZE_ROW_CONST,
            SEL_SIZE_ROW_CONST * 0.6,
        );
        line(
            kit,
            TextAt::right(
                name_x + sel_px(SEL_ROW_CONST_R),
                base,
                sel_px(size),
            ),
            &value,
            SEL_NAME_INK,
        );
    }
}

fn row_name(kit: &CanvasKit<'_>, x: f32, base: f32, value: &str, ink: Color) {
    let avail = sel_pxi(SEL_ROW_NAME_W);
    let mut size = SEL_SIZE_ROW_NAME;
    while size > SEL_SIZE_ROW_NAME * 0.6 {
        let style = sketch::text_style(sketch::FONT_BODY, sel_px(size), settlement::SCORE_INK);
        if sketch::text_width(kit, &style, value).round() <= avail {
            break;
        }
        size -= 1.0;
    }
    let style = sketch::text_style(sketch::FONT_BODY, sel_px(size), settlement::SCORE_INK);
    let text = sketch::ellipsize(kit, &style, value, avail);
    line(kit, TextAt::left(x, base, sel_px(size)), &text, ink);
}

fn strip(kit: &CanvasKit<'_>, root: &Path, req: &SongSelectRequest) {
    if req.strip.is_empty() {
        return;
    }
    sketch::fill_skew(
        kit,
        sel_boxed_rect(strip_rect()),
        LEAN,
        SEL_PLATE_INK,
    );
    let undo = strip_undo_rect();
    let center_x = (undo.left() + undo.right()) / 2.0;
    let undo_side = sel_pxi(SEL_STRIP_UNDO_SZ);
    sprite(
        kit,
        root,
        SEL_UNDO_SPRITE,
        Rect::from_xywh(
            (center_x - undo_side / 2.0).round(),
            (undo.top() + sel_px(SEL_STRIP_UNDO_ICON_CY) - undo_side / 2.0).round(),
            undo_side,
            undo_side,
        ),
    );
    line(
        kit,
        TextAt::center(
            center_x,
            undo.top() + sel_px(SEL_STRIP_UNDO_BASE),
            sel_px(SEL_STRIP_UNDO_SIZE),
        )
        .with_font(sketch::FONT_DISPLAY),
        SEL_UNDO_TEXT,
        SEL_NAME_INK,
    );
    let badge_size = sel_px(SEL_STRIP_BADGE_SIZE);
    for (index, thumb) in req.strip.iter().enumerate() {
        cover_skew(kit, root, &thumb.song_id, strip_card_rect(index));
        let badge = sel_boxed_rect(strip_badge_rect(index));
        sketch::fill_skew(kit, badge, LEAN, SEL_TAB_ON_INK);
        line(
            kit,
            TextAt::center(
                (badge.left() + badge.right()) / 2.0,
                (badge.top() + badge.bottom()) / 2.0 + badge_size * SEL_STRIP_BADGE_INSET,
                badge_size,
            )
            .with_font(sketch::FONT_DISPLAY),
            &thumb.badge,
            SEL_TAB_ON_TEXT,
        );
    }
}

fn check(kit: &CanvasKit<'_>, root: &Path) {
    let box_rect = check_rect();
    sketch::fill_skew(kit, box_rect, LEAN, SEL_CHECK_INK);
    let w = sel_pxi(SEL_CHECK_ICON_W);
    let h = sel_pxi(SEL_CHECK_ICON_H);
    sprite_tinted(
        kit,
        root,
        SEL_CHECK_SPRITE,
        Rect::from_xywh(
            ((box_rect.left().round() + box_rect.right().round()) / 2.0 - w / 2.0).round(),
            ((box_rect.top().round() + box_rect.bottom().round()) / 2.0 - h / 2.0).round(),
            w,
            h,
        ),
        SEL_CHECK_MARK_INK,
    );
}

fn song_info(kit: &CanvasKit<'_>, req: &SongSelectRequest) {
    sketch::fill_skew(
        kit,
        sel_boxed(
            SEL_SONG_X0,
            SEL_SONG_Y0,
            SEL_SONG_X1 - SEL_SONG_X0,
            SEL_SONG_Y1 - SEL_SONG_Y0,
        ),
        LEAN,
        SEL_PLATE_INK,
    );
    let title_at = TextAt::left(
        sel_px(SEL_TITLE_X),
        sel_px(SEL_TITLE_Y + SEL_TITLE_H),
        sel_px(SEL_SIZE_TITLE),
    );
    let title_style = sketch::text_style(title_at.font_key(), title_at.size, settlement::SCORE_INK);
    let title = sketch::ellipsize(kit, &title_style, &req.title, sel_pxi(SEL_TITLE_W));
    line(kit, title_at, &title, SEL_NAME_INK);
    if !req.composer.is_empty() && req.composer != SEL_ARTIST_BLANK {
        let artist_at = TextAt::left(
            sel_px(SEL_ARTIST_X),
            sel_px(SEL_ARTIST_Y + SEL_ARTIST_H),
            sel_px(SEL_SIZE_ARTIST),
        );
        let style = sketch::text_style(artist_at.font_key(), artist_at.size, settlement::SCORE_INK);
        let artist = sketch::ellipsize(kit, &style, &req.composer, sel_pxi(SEL_ARTIST_W));
        line(kit, artist_at, &artist, SEL_DIM_INK);
    }
    let badge = sel_boxed(SEL_LEVEL_X, SEL_LEVEL_Y, SEL_LEVEL_W, SEL_LEVEL_H);
    sketch::fill_skew(kit, badge, LEAN, SEL_LEVEL_INK);
    let center_x = (badge.left() + badge.right()) / 2.0;
    let number = const_text(req.constant, req.known);
    let size = fit_size(
        kit,
        sketch::FONT_BODY,
        &number,
        sel_pxi(level_text_w()),
        SEL_SIZE_LEVEL,
        SEL_SIZE_LEVEL * 0.6,
    );
    line(
        kit,
        TextAt::center(center_x, sel_px(SEL_LEVEL_NUM_BASE), sel_px(size)),
        &number,
        SEL_LEVEL_TEXT,
    );
    line(
        kit,
        TextAt::center(
            center_x,
            sel_px(SEL_LEVEL_LB_BASE),
            sel_px(SEL_SIZE_LEVEL_LB),
        ),
        &req.diff,
        SEL_LEVEL_TEXT,
    );
}

fn tabs(kit: &CanvasKit<'_>, req: &SongSelectRequest) {
    let levels = if req.levels.len() > SEL_DIFF_TAB_MAX {
        &req.levels[..SEL_DIFF_TAB_MAX]
    } else {
        &req.levels[..]
    };
    if levels.is_empty() {
        return;
    }
    let count = levels.len();
    sketch::fill_skew(kit, diff_bar_rect(), LEAN, SEL_TAB_OFF_INK);
    for (index, level) in levels.iter().enumerate() {
        if !level.active {
            continue;
        }
        sketch::fill_skew(
            kit,
            sel_boxed_rect(diff_sel_rect(index, count)),
            LEAN,
            SEL_TAB_ON_INK,
        );
    }
    for (index, level) in levels.iter().enumerate() {
        let center_x = sel_px(diff_tab_center(index, count));
        let ink = if level.active {
            SEL_TAB_ON_TEXT
        } else {
            SEL_TAB_OFF_TEXT
        };
        line(
            kit,
            TextAt::center(center_x, sel_px(SEL_DIFF_NUM_BASE), sel_px(SEL_SIZE_TAB_NUM))
                .with_font(sketch::FONT_DISPLAY),
            &level.const_text,
            ink,
        );
        line(
            kit,
            TextAt::center(center_x, sel_px(SEL_DIFF_LB_BASE), sel_px(SEL_SIZE_TAB_LB))
                .with_font(sketch::FONT_DISPLAY),
            &level.diff,
            ink,
        );
    }
}

fn mirror(kit: &CanvasKit<'_>) {
    sketch::fill_skew(
        kit,
        sel_boxed(
            SEL_MIRROR_X0,
            SEL_MIRROR_Y0,
            SEL_MIRROR_X1 - SEL_MIRROR_X0,
            SEL_MIRROR_Y1 - SEL_MIRROR_Y0,
        ),
        LEAN,
        SEL_MIRROR_INK,
    );
    line(
        kit,
        TextAt::left(
            sel_px(SEL_MIRROR_TEXT_X),
            sel_px(SEL_MIRROR_BASE),
            sel_px(SEL_SIZE_MIRROR),
        ),
        SEL_MIRROR_TEXT,
        SEL_DIM_INK,
    );
}

fn score_track(kit: &CanvasKit<'_>, end: f32) {
    let y0 = sel_pxi(SEL_SCORE_BAR_Y0);
    let y1 = sel_pxi(SEL_SCORE_BAR_Y1);
    let x0 = sel_pxi(SEL_SCORE_BAR_X0);
    let x1 = sel_pxi(end);
    let fade = sel_pxi(SEL_SCORE_BAR_FADE);
    let step = sel_pxi(2.0);
    let mut x = x0;
    while x < x1 {
        let mut t = 1.0;
        if x > fade && x1 > fade {
            t = 1.0 - 0.8 * (x - fade) / (x1 - fade);
        }
        let mut ink = SEL_SCORE_INK;
        ink.a = (f32::from(ink.a) * t) as u8;
        sketch::fill_skew(kit, Rect::from_ltrb(x, y0, x + step, y1), LEAN, ink);
        x += step;
    }
}

fn score_text(kit: &CanvasKit<'_>) {
    line(
        kit,
        TextAt::left(
            sel_px(SEL_SCORE_X),
            sel_px(SEL_SCORE_NUM_BASE),
            sel_px(SEL_SIZE_SCORE),
        ),
        SEL_SCORE_TEXT_VALUE,
        SEL_SCORE_TEXT,
    );
    line(
        kit,
        TextAt::left(
            sel_px(SEL_SCORE_ACC_X),
            sel_px(SEL_SCORE_ACC_BASE),
            sel_px(SEL_SIZE_SCORE_ACC),
        ),
        SEL_SCORE_ACC_VALUE,
        SEL_SCORE_TEXT,
    );
    line(
        kit,
        TextAt::left(
            sel_px(SEL_SCORE_NEW_X),
            sel_px(SEL_SCORE_NEW_BASE),
            sel_px(SEL_SIZE_SCORE_NEW),
        ),
        SEL_SCORE_NEW_VALUE,
        SEL_SCORE_NEW_INK,
    );
}

fn play(kit: &CanvasKit<'_>, root: &Path) {
    sketch::fill_skew(kit, play_plate_rect(), LEAN, SEL_PLAY_INK);
    let Some(texture) = memo::texture(kit, &super::ui_asset(root, SEL_PLAY_SPRITE)) else {
        return;
    };
    let Some(image) = texture.image() else {
        return;
    };
    let crop = Rect::from_ltrb(
        SEL_PLAY_CROP_X0.round(),
        SEL_PLAY_CROP_Y0.round(),
        SEL_PLAY_CROP_X1.round(),
        SEL_PLAY_CROP_Y1.round(),
    );
    let draw = play_tri_draw();
    sketch::image_part(
        kit,
        image,
        crop,
        sketch::ph_rect(draw.left(), draw.top(), draw.width(), draw.height()),
    );
}

fn nav(kit: &CanvasKit<'_>, root: &Path) {
    sketch::ph_fill(kit, nav_corner_fill(), SEL_TOP_INK);
    let sprite_rect = nav_sprite_rect();
    if memo::texture(kit, &super::ui_asset(root, SEL_BACK_SPRITE)).is_some() {
        sprite_nearest(kit, root, SEL_BACK_SPRITE, sprite_rect);
    } else {
        sketch::fill_skew(kit, sprite_rect, LEAN, SEL_PLATE_INK);
    }
    sprite_nearest(
        kit,
        root,
        SEL_ARROW_SPRITE,
        Rect::from_xywh(
            sel_pxi(SEL_ARROW_X),
            sel_pxi(SEL_ARROW_Y),
            sel_pxi(SEL_ARROW_W),
            sel_pxi(SEL_ARROW_H),
        ),
    );
}

fn top_bar(kit: &CanvasKit<'_>, root: &Path) {
    let reference = top_ref_rect();
    let body = Rect::from_ltrb(
        reference.left(),
        reference.top(),
        reference.right(),
        sel_pxi(SEL_TOP_Y1),
    );
    fill_skew_ref(kit, body, reference, SEL_TOP_INK);
    let fade = Rect::from_ltrb(
        reference.left(),
        body.bottom(),
        reference.right(),
        reference.bottom(),
    );
    fill_skew_ref_gradient(kit, fade, reference, SEL_TOP_INK, SEL_TOP_INK.with_alpha(0));
    let sep_half = sel_px(SEL_TOP_SEP_W) / 2.0;
    let sep_x = sel_pxi(SEL_TOP_SEP_CX);
    sketch::fill_skew(
        kit,
        Rect::from_ltrb(
            sep_x - sep_half,
            sel_pxi(SEL_TOP_SEP_Y0),
            sep_x + sep_half.max(1.0),
            sel_pxi(SEL_TOP_SEP_Y1),
        ),
        LEAN,
        SEL_TOP_SEP_INK,
    );
    sketch::ph_fill(
        kit,
        sel_rect(SEL_TOP_RULE_X0, SEL_TOP_RULE_Y0, SEL_TOP_RULE_X1, SEL_TOP_RULE_Y1),
        SEL_TOP_RULE_INK,
    );
    top_icon(kit, root, SEL_RANDOM_SPRITE, &SEL_TOP_RAND_ICON);
    top_icon(kit, root, SEL_GEAR_SPRITE, &SEL_TOP_GEAR_ICON);
    top_label(kit, SEL_RANDOM_LABEL, &SEL_TOP_RAND_LABEL);
    top_label(kit, SEL_GEAR_LABEL, &SEL_TOP_GEAR_LABEL);
    top_label(kit, SEL_SORT_LABEL, &SEL_TOP_SORT_LABEL);
    top_icon(kit, root, SEL_SORT_SPRITE, &SEL_TOP_SORT_ICON);
    top_label(kit, SEL_SORT_VALUE, &SEL_TOP_SORT_VALUE);
}

fn top_icon(kit: &CanvasKit<'_>, root: &Path, name: &str, area: &[f32; 4]) {
    sprite(
        kit,
        root,
        name,
        sel_rect(area[0], area[1], area[2], area[3]),
    );
}

fn top_label(kit: &CanvasKit<'_>, value: &str, area: &[f32; 3]) {
    line(
        kit,
        TextAt::center(sel_px(area[0]), sel_px(area[1]), sel_px(area[2])),
        value,
        SEL_TOP_TEXT,
    );
}

fn player_name(kit: &CanvasKit<'_>, raw: &str) -> (String, f32) {
    let name = raw.trim();
    if name.is_empty() {
        return (String::new(), 0.0);
    }
    let at = player_name_at();
    let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
    let trimmed = sketch::ellipsize(kit, &style, name, sel_px(SEL_PLAYER_NAME_MAX_WIDTH));
    if trimmed.is_empty() {
        return (String::new(), 0.0);
    }
    let width = sketch::text_width(kit, &style, &trimmed).round();
    (trimmed, width)
}

fn avatar(kit: &CanvasKit<'_>, root: &Path, win: Rect, chosen: &str) {
    let mut keys: Vec<String> = Vec::with_capacity(2);
    if !chosen.is_empty() {
        keys.push(super::avatar_asset(root, chosen));
    }
    keys.push(super::ui_asset(root, memo::DEFAULT_AVATAR));
    for key in keys {
        let Some(texture) = memo::texture(kit, &key) else {
            continue;
        };
        let Some(image) = texture.image() else {
            continue;
        };
        sketch::image_sprite_skew(kit, image, win, LEAN);
        return;
    }
}

fn rks_layout(
    kit: &CanvasKit<'_>,
    label: &str,
    plate: Rect,
    avatar_win: Rect,
    pad: f32,
    at: TextAt,
) -> TextAt {
    if label.is_empty() || at.size <= 0.0 {
        return at;
    }
    let y = (plate.top() + plate.bottom()) / 2.0;
    let left = (plate.left() + skew_at(plate, y))
        .max(avatar_win.right() + skew_at(avatar_win, y));
    let right = plate.right() + skew_at(plate, y);
    let avail = right - left - 2.0 * pad;
    if avail <= 0.0 {
        return at;
    }
    let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
    let width = sketch::text_width(kit, &style, label);
    let size = if width > avail {
        at.size * avail / width
    } else {
        at.size
    };
    TextAt::center((left + right) / 2.0, y + size * MID_RATIO, size)
}

fn player_bar(kit: &CanvasKit<'_>, root: &Path, p: &player::Player) {
    if !p.has_player {
        return;
    }
    let (name, name_width) = player_name(kit, &p.name);
    let panel = sel_boxed_rect(player_panel(name_width));
    sketch::fill_skew(kit, panel, LEAN, SEL_PLATE_INK);
    if !name.is_empty() {
        line(kit, player_name_at(), &name, settlement::SCORE_INK);
    }
    let win = sel_boxed_rect(avatar_rect());
    avatar(kit, root, win, &p.avatar);
    let badge = sel_boxed_rect(rank_rect());
    player::rank_frame(kit, root, badge, p.rank_level);
    line(kit, rank_text_at(), &p.rank.to_string(), SEL_RANK_TEXT_INK);
    let plate = sel_boxed_rect(rks_rect());
    sketch::fill_skew(kit, plate, LEAN, SEL_RKS_PILL_INK);
    let label = format!("{:.4}", p.rks.max(0.0));
    let visible = Rect::from_xywh(
        plate.left(),
        plate.top(),
        (SEL_W - plate.left()).max(1.0),
        plate.height(),
    );
    let at = rks_layout(
        kit,
        &label,
        visible,
        win,
        sel_px(SEL_RKS_TEXT_PAD),
        rks_text_at(),
    );
    line(kit, at, &label, SEL_RKS_TEXT_INK);
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SongSelectRequest) {
    backdrop(kit, root, &req.backdrop);
    cover(kit, root, &req.song_id);
    list(kit, root, req);
    strip(kit, root, req);
    song_info(kit, req);
    tabs(kit, req);
    mirror(kit);
    if req.strip.is_empty() {
        score_track(kit, SEL_SCORE_BAR_X1);
        score_text(kit);
    } else {
        score_track(kit, SEL_CHECK_X1);
        check(kit, root);
    }
    play(kit, root);
    nav(kit, root);
    top_bar(kit, root);
    player_bar(kit, root, &req.player);
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurorix_render::skia_safe::image::CachingHint;
    use aurorix_render::skia_safe::{AlphaType, ColorType, Data, ImageInfo};
    use std::time::Instant;

    const OUT: &str = "/tmp/phigros-verify/14-songselect.png";

    const MAIN_ID: &str = "000AinSophAur.Yumeji.0";

    const SECOND_ID: &str = "Spasmodic.姜米條颶風元力上人.0";

    fn library() -> Vec<crate::model::Song> {
        let root = crate::assets_root();
        let Ok(catalog) = crate::domain::catalog::Catalog::open(root.join("../data/phi.db")) else {
            return Vec::new();
        };
        catalog.songs().to_vec()
    }

    fn pick(all: &[crate::model::Song], id: &str) -> crate::model::Song {
        all.iter()
            .find(|song| song.id == id)
            .or_else(|| all.first())
            .cloned()
            .expect("曲库必须有曲目")
    }

    fn request() -> SongSelectRequest {
        let all = library();
        let main = pick(&all, MAIN_ID);
        let second = pick(&all, SECOND_ID);
        let labels = ["EZ", "HD", "IN", "AT"];
        let mut levels: Vec<SongSelectLevel> = Vec::new();
        for (index, chart) in main.charts.iter().enumerate() {
            if !chart.known() {
                continue;
            }
            levels.push(SongSelectLevel {
                diff: String::from(labels[index]),
                const_text: format!("{:.1}", chart.constant),
                active: index == 2,
            });
        }
        let rows: Vec<SongSelectRow> = (0..SEL_ROWS)
            .map(|index| {
                let song = if index == 0 {
                    second.clone()
                } else {
                    pick(&all, SECOND_ID)
                };
                SongSelectRow {
                    title: format!("{} #{}", song.title, index),
                    constant: song.charts[2].constant,
                    known: song.charts[2].known(),
                    selected: index == SEL_ROW_SEL,
                }
            })
            .collect();
        let player = player::Player {
            has_player: true,
            name: String::from("Okamikko"),
            avatar: String::from("avatar_1"),
            rank: 3,
            rank_level: 4,
            rks: second.charts[2].constant + 1.2,
        };
        SongSelectRequest {
            backdrop: main.id.clone(),
            song_id: main.id.clone(),
            title: main.title.clone(),
            composer: main.composer.clone(),
            diff: String::from("IN"),
            constant: main.charts[2].constant,
            known: main.charts[2].known(),
            levels,
            rows,
            strip: Vec::new(),
            player,
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
    fn canvas_is_the_screenshot_size() {
        let (width, height) = canvas_size(&request());
        assert_eq!(width, SEL_CANVAS_W);
        assert_eq!(height, SEL_CANVAS_H);
        assert_eq!((width, height), (2376, 1080));
    }

    #[test]
    fn tabs_and_rows_keep_measured_pitch() {
        let step = diff_tab_center(1, 4) - diff_tab_center(0, 4);
        assert!((step - SEL_DIFF_BAR_W / 4.0).abs() < 0.001);
        assert!(diff_tab_center(0, 4) > SEL_DIFF_BAR_X0);
        assert!((row_center_at(SEL_ROW_SEL) - SEL_ROW_CENTER).abs() < 0.001);
        assert!(row_center_at(0) < row_center_at(SEL_ROW_SEL));
        assert!(row_center_at(SEL_ROW_SEL) < row_center_at(SEL_ROWS - 1));
        assert!(row_name_left_at(0) > row_name_left_at(SEL_ROWS - 1));
        assert!(strip_card_rect(1).left() > strip_card_rect(0).left());
        assert!(check_rect().width() > 0.0);
        assert!(cover_rect().width() < cover_visible().width());
    }

    #[test]
    #[ignore]
    fn render_songselect() {
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
            "选曲界面渲染耗时 {} ms ({} 字节, {}x{})",
            elapsed.as_millis(),
            bytes.len(),
            image.width(),
            image.height()
        );
        assert_eq!(image.width(), width, "画布宽必须等于 canvas_size");
        assert_eq!(image.height(), height, "画布高必须等于 canvas_size");
        assert!(bytes.len() > 20000, "字节数必须大于 20000");
        let variance = luma_variance(&image);
        println!("选曲界面 luma 方差 {variance:.2}");
        assert!(variance > 100.0, "luma 方差必须大于 100, 实际 {variance:.2}");
    }
}
