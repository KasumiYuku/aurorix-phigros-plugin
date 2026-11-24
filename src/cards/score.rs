use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{Color, CanvasKit};

use super::settlement::{self, CardData, SettlementRequest, TextAt};
use super::{memo, player, sketch};

pub const SC_U: f32 = 0.5;

pub const SC_SHIFT: f32 = 645.0 * SC_U;

pub const SC_TOP: f32 = 305.92 * SC_U;

pub const SC_BOTTOM: f32 = 1462.37 * SC_U;

pub const SC_COVER_EDGE_LEFT: f32 = 430.0 * SC_U;

pub const SC_COVER_WIDTH: f32 = 1190.0 * SC_U;

pub const SC_BOARD_EDGE_LEFT: f32 = 3024.6 * SC_U;

pub const SC_BOARD_WIDTH: f32 = 889.4 * SC_U;

pub const SC_PLAYER_BAR_DY: f32 = -21.7 * SC_U;

pub const SC_TITLE_SIZE: f32 = 44.0;

pub const SC_LEVEL_SIZE: f32 = 29.0;

pub const SC_BOARD_ROWS: usize = 7;

pub const SC_BOARD_PAD_X: f32 = 40.0 * SC_U;

pub const SC_BOARD_PAD_T: f32 = 40.0 * SC_U;

pub const SC_BOARD_PAD_B: f32 = 48.0 * SC_U;

pub const SC_BOARD_HEAD_H: f32 = 60.0 * SC_U;

pub const SC_BOARD_HEAD_GAP: f32 = 24.0 * SC_U;

pub const SC_ROW_UP_LIFT: f32 = 22.0 * SC_U;

pub const SC_ROW_LINE_GAP: f32 = 48.0 * SC_U;

pub const SC_ROW_RANK_W: f32 = 72.0 * SC_U;

pub const SC_ROW_AVATAR_X: f32 = 84.0 * SC_U;

pub const SC_ROW_AVATAR_S: f32 = 84.0 * SC_U;

pub const SC_ROW_TEXT_X: f32 = 186.0 * SC_U;

pub const SC_ROW_BADGE_W: f32 = 100.0 * SC_U;

pub const SC_ROW_RKS_GAP: f32 = 12.0 * SC_U;

pub const SC_ROW_NAME_GAP: f32 = 20.0 * SC_U;

pub const SC_ROW_SCORE_W: f32 = 170.0;

pub const SC_RANK_SIZE: f32 = 36.0 * SC_U;

pub const SC_NAME_SIZE: f32 = 40.0 * SC_U;

pub const SC_SCORE_SIZE: f32 = 46.0 * SC_U;

pub const SC_BADGE_SIZE: f32 = 24.0 * SC_U;

pub const SC_RKS_SIZE: f32 = 24.0 * SC_U;

pub const SC_ACC_SIZE: f32 = 22.0 * SC_U;

pub const SC_BOARD_HEAD_SIZE: f32 = 30.0;

pub const SC_BOARD_COUNT_SIZE: f32 = 18.0;

pub const SC_HEAD_BASE_DROP: f32 = 20.0 * SC_U;

pub const SC_COVER_TEXT_PAD: f32 = 64.0 * SC_U;

pub const SC_COVER_LEVEL_PAD: f32 = 40.0 * SC_U;

pub const SC_COVER_LINE_DROP: f32 = 76.0 * SC_U;

pub const SC_COVER_LEVEL_RISE: f32 = 76.0 * SC_U;

pub const SC_COVER_SHADE_H: f32 = 220.0 * SC_U;

pub const SC_SELF_INSET: f32 = 7.0 * SC_U;

pub const SC_SELF_MARK_W: f32 = 4.0 * SC_U;

pub const SC_AVATAR_RADIUS: f32 = 6.0;

pub const LEAN: f32 = settlement::LEAN;

pub const MID_RATIO: f32 = settlement::MID_RATIO;

pub const SCORE_INK: Color = settlement::SCORE_INK;

pub const LABEL_INK: Color = settlement::LABEL_INK;

pub const PLACEHOLDER_INK: Color = settlement::PLACEHOLDER_INK;

pub const TITLE_INK: Color = settlement::TITLE_INK;

pub const PANEL_INK: Color = settlement::PANEL_INK;

pub const RANK_ACCENT: Color = settlement::RANK_ACCENT;

pub const OV_NOTE_INK: Color = Color::new(0xC6, 0xCC, 0xD8, 0xFF);

pub const RANK_RULE_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x1A);

pub const RANK_ROW_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x14);

pub const OV_HIDDEN_RANK: &str = "？";

pub const BOARD_HEAD_TEXT: &str = "谱面排行";

pub const NEW_BEST_PREFIX: &str = "NEW BEST ";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScoreRankRow {
    pub rank: i32,
    pub name: String,
    pub avatar: String,
    pub rks: f64,
    pub dan_level: i32,
    pub dan_progress: i32,
    pub score: i32,
    pub acc: f64,
    pub self_row: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScoreRanking {
    pub diff: String,
    pub total: i32,
    pub global: bool,
    pub rows: Vec<ScoreRankRow>,
}

#[derive(Clone, Debug, Default)]
pub struct ScoreRequest {
    pub settlement: SettlementRequest,
    pub ranking: ScoreRanking,
}

pub fn canvas_size() -> (i32, i32) {
    settlement::canvas_size()
}

pub fn sc_band(edge_left: f32, width: f32, top: f32, height: f32) -> Rect {
    let cy = top + height / 2.0;
    Rect::from_xywh(edge_left - LEAN * cy, top, width, height)
}

pub fn cover() -> Rect {
    sc_band(
        SC_COVER_EDGE_LEFT,
        SC_COVER_WIDTH,
        SC_TOP,
        SC_BOTTOM - SC_TOP,
    )
}

pub fn board() -> Rect {
    sc_band(
        SC_BOARD_EDGE_LEFT,
        SC_BOARD_WIDTH,
        SC_TOP,
        SC_BOTTOM - SC_TOP,
    )
}

pub fn shifted(area: Rect, dx: f32) -> Rect {
    Rect::from_xywh(area.left() - dx, area.top(), area.width(), area.height())
}

pub fn shift_at(at: TextAt, dx: f32) -> TextAt {
    TextAt { x: at.x - dx, ..at }
}

pub fn score_cards() -> settlement::Cards {
    let g = settlement::settle_cards();
    settlement::Cards {
        score: shifted(g.score, SC_SHIFT),
        combo_and_acc: shifted(g.combo_and_acc, SC_SHIFT),
        more_info: shifted(g.more_info, SC_SHIFT),
        ratings: shifted(g.ratings, SC_SHIFT),
        score_text: shift_at(g.score_text, SC_SHIFT),
        history_text: shift_at(g.history_text, SC_SHIFT),
        backs: [
            shifted(g.score, SC_SHIFT),
            shifted(g.combo_and_acc, SC_SHIFT),
            shifted(g.more_info, SC_SHIFT),
        ],
    }
}

pub fn songs_name_at() -> TextAt {
    let win = cover();
    let y = win.top() + win.height() - SC_COVER_LINE_DROP;
    TextAt::left(
        win.left() + sketch::skew_offset(win, y, LEAN) + SC_COVER_TEXT_PAD,
        y,
        SC_TITLE_SIZE,
    )
}

pub fn level_at() -> TextAt {
    let win = cover();
    let y = win.top() + SC_COVER_LEVEL_RISE;
    TextAt::right(
        win.right() + sketch::skew_offset(win, y, LEAN) - SC_COVER_LEVEL_PAD,
        y,
        SC_LEVEL_SIZE,
    )
}

pub fn cover_shade() -> Rect {
    let win = cover();
    Rect::from_xywh(
        win.left(),
        win.top() + win.height() - SC_COVER_SHADE_H,
        win.width(),
        SC_COVER_SHADE_H,
    )
}

pub fn title_max_width() -> f32 {
    cover().width() - SC_COVER_TEXT_PAD - SC_COVER_LEVEL_PAD
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

pub fn row_badge_h() -> f32 {
    SC_ROW_BADGE_W * 20.0 / 61.0
}

pub fn board_row_h() -> f32 {
    let used = SC_BOARD_PAD_T + SC_BOARD_HEAD_H + SC_BOARD_HEAD_GAP + SC_BOARD_PAD_B;
    (board().height() - used) / SC_BOARD_ROWS as f32
}

pub fn board_row_at(index: usize) -> Rect {
    let area = board();
    let height = board_row_h();
    let y = area.top()
        + SC_BOARD_PAD_T
        + SC_BOARD_HEAD_H
        + SC_BOARD_HEAD_GAP
        + index as f32 * height;
    Rect::from_xywh(
        board_x(y + height / 2.0),
        y,
        area.width() - 2.0 * SC_BOARD_PAD_X,
        height,
    )
}

fn board_x(y: f32) -> f32 {
    let area = board();
    area.left() + sketch::skew_offset(area, y, LEAN) + SC_BOARD_PAD_X
}

fn board_x1(y: f32) -> f32 {
    let area = board();
    area.right() + sketch::skew_offset(area, y, LEAN) - SC_BOARD_PAD_X
}

pub fn row_mid(index: usize) -> f32 {
    board_row_at(index).top() + board_row_h() / 2.0
}

pub fn row_up_mid(index: usize) -> f32 {
    row_mid(index) - SC_ROW_UP_LIFT
}

pub fn row_down_mid(index: usize) -> f32 {
    row_up_mid(index) + SC_ROW_LINE_GAP
}

pub fn row_x(index: usize) -> f32 {
    board_x(row_mid(index))
}

pub fn row_x1(index: usize) -> f32 {
    board_x1(row_mid(index))
}

pub fn head_base() -> f32 {
    board().top() + SC_BOARD_PAD_T + SC_BOARD_HEAD_H - SC_HEAD_BASE_DROP
}

pub fn head_at() -> TextAt {
    TextAt::left(board_x(head_base()), head_base(), SC_BOARD_HEAD_SIZE)
        .with_font(sketch::FONT_DISPLAY)
}

pub fn count_at() -> TextAt {
    TextAt::right(board_x1(head_base()), head_base(), SC_BOARD_COUNT_SIZE)
}

pub fn head_rule_y() -> f32 {
    board().top() + SC_BOARD_PAD_T + SC_BOARD_HEAD_H
}

pub fn rank_at(index: usize, width: f32) -> TextAt {
    let mut size = SC_RANK_SIZE;
    if width > SC_ROW_RANK_W {
        size = SC_RANK_SIZE * SC_ROW_RANK_W / width;
    }
    TextAt::left(row_x(index), row_up_mid(index) + size * MID_RATIO, size)
        .with_font(sketch::FONT_DISPLAY)
}

pub fn avatar_at(index: usize) -> Rect {
    let mid = row_mid(index);
    Rect::from_xywh(
        row_x(index) + SC_ROW_AVATAR_X,
        mid - SC_ROW_AVATAR_S / 2.0,
        SC_ROW_AVATAR_S,
        SC_ROW_AVATAR_S,
    )
}

pub fn name_at(index: usize) -> TextAt {
    TextAt::left(
        row_x(index) + SC_ROW_TEXT_X,
        row_up_mid(index) + SC_NAME_SIZE * MID_RATIO,
        SC_NAME_SIZE,
    )
}

pub fn name_width() -> f32 {
    board().width() - 2.0 * SC_BOARD_PAD_X - SC_ROW_TEXT_X - SC_ROW_SCORE_W - SC_ROW_NAME_GAP
}

pub fn score_at(index: usize) -> TextAt {
    TextAt::right(
        row_x1(index),
        row_up_mid(index) + SC_SCORE_SIZE * MID_RATIO,
        SC_SCORE_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn badge_at(index: usize) -> Rect {
    let mid = row_down_mid(index);
    let height = row_badge_h();
    Rect::from_xywh(
        row_x(index) + SC_ROW_TEXT_X,
        mid - height / 2.0,
        SC_ROW_BADGE_W,
        height,
    )
}

pub fn rks_at(index: usize) -> TextAt {
    TextAt::left(
        row_x(index) + SC_ROW_TEXT_X + SC_ROW_BADGE_W + SC_ROW_RKS_GAP,
        row_down_mid(index) + SC_RKS_SIZE * MID_RATIO,
        SC_RKS_SIZE,
    )
}

pub fn acc_at(index: usize) -> TextAt {
    TextAt::right(
        row_x1(index),
        row_down_mid(index) + SC_ACC_SIZE * MID_RATIO,
        SC_ACC_SIZE,
    )
}

pub fn row_rule_y(index: usize) -> f32 {
    board_row_at(index).top() + board_row_h()
}

pub fn self_band(index: usize) -> Rect {
    let area = board();
    let y = board_row_at(index).top() + SC_SELF_INSET;
    let height = board_row_h() - 2.0 * SC_SELF_INSET;
    Rect::from_xywh(
        area.left() + sketch::skew_offset(area, y + height / 2.0, LEAN),
        y,
        area.width(),
        height,
    )
}

pub fn self_mark(index: usize) -> Rect {
    let band = self_band(index);
    Rect::from_xywh(band.left(), band.top(), SC_SELF_MARK_W, band.height())
}

pub fn score_board_count(ranking: &ScoreRanking) -> String {
    if ranking.global {
        return format!(
            "全服共 {} 人",
            sketch::group_digits(&ranking.total.max(0).to_string())
        );
    }
    format!("共 {} 人", ranking.total)
}

pub fn rank_label(row: &ScoreRankRow) -> String {
    if row.hidden {
        return String::from(OV_HIDDEN_RANK);
    }
    format!("#{}", row.rank)
}

fn fill_band(kit: &CanvasKit<'_>, win: Rect, top: f32, bottom: f32, color: Color) {
    let mut y = top.round();
    while y < bottom {
        let dx = sketch::skew_offset(win, y, LEAN);
        sketch::ph_fill(
            kit,
            Rect::from_ltrb(win.left() + dx, y, win.right() + dx, y + 1.0),
            color,
        );
        y += 1.0;
    }
}

pub fn cover_window(kit: &CanvasKit<'_>, root: &Path, song_id: &str) {
    let win = cover();
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::illust_key(root, song_id))
    };
    let image = texture.as_ref().and_then(|texture| texture.image());
    let Some(image) = image else {
        sketch::fill_skew(kit, win, LEAN, settlement::COVER_FALLBACK);
        return;
    };
    let dest = sketch::skew_bounds(win, LEAN);
    sketch::clip_skew(kit, win, LEAN, || {
        sketch::image_cover(kit, image, dest);
    });
    let shade_key = super::ui_asset(root, "bar_bottom_color");
    let Some(shade) = memo::texture(kit, &shade_key) else {
        return;
    };
    let Some(shade_image) = shade.image() else {
        return;
    };
    let band = cover_shade();
    let shade_dest = Rect::from_ltrb(dest.left(), band.top(), dest.right(), band.bottom());
    sketch::clip_skew(kit, win, LEAN, || {
        sketch::image_cover_paint(kit, shade_image, shade_dest, &settlement::shade_paint());
    });
}

pub fn cover_lines(kit: &CanvasKit<'_>, req: &SettlementRequest) {
    let style = sketch::text_style(sketch::FONT_BODY, SC_TITLE_SIZE, TITLE_INK);
    let title = sketch::ellipsize(kit, &style, &req.title, title_max_width());
    settlement::line(kit, songs_name_at(), &title, TITLE_INK);
    let ink = if req.played {
        SCORE_INK
    } else {
        PLACEHOLDER_INK
    };
    settlement::line(
        kit,
        level_at(),
        &settlement::level_text(&req.diff, req.constant, req.known),
        ink,
    );
}

pub fn draw_board(kit: &CanvasKit<'_>, root: &Path, ranking: &ScoreRanking) {
    if ranking.diff.is_empty() || ranking.rows.is_empty() {
        return;
    }
    let area = board();
    sketch::fill_skew(kit, area, LEAN, PANEL_INK);
    settlement::line(kit, head_at(), BOARD_HEAD_TEXT, TITLE_INK);
    if ranking.total > 0 {
        settlement::line(kit, count_at(), &score_board_count(ranking), OV_NOTE_INK);
    }
    let rule_y = head_rule_y();
    fill_band(kit, area, rule_y, rule_y + 1.0, OV_NOTE_INK);

    let name_w = name_width();
    for (index, row) in ranking.rows.iter().enumerate() {
        if index >= SC_BOARD_ROWS {
            break;
        }
        let mut rank_ink = OV_NOTE_INK;
        let mut name_ink = LABEL_INK;
        if row.self_row || row.rank <= 3 {
            rank_ink = RANK_ACCENT;
            name_ink = SCORE_INK;
        }
        if row.self_row {
            sketch::fill_skew(kit, self_band(index), LEAN, RANK_ROW_INK);
            sketch::fill_skew(kit, self_mark(index), LEAN, RANK_ACCENT);
        }
        if index + 1 < SC_BOARD_ROWS {
            let line_y = row_rule_y(index);
            fill_band(kit, area, line_y, line_y + 1.0, RANK_RULE_INK);
        }
        let rank = rank_label(row);
        let probe = TextAt::left(0.0, 0.0, SC_RANK_SIZE).with_font(sketch::FONT_DISPLAY);
        let rank_w = settlement::measure(kit, probe, &rank);
        settlement::line(kit, rank_at(index, rank_w), &rank, rank_ink);

        let avatar = avatar_at(index);
        if avatar.width() > 0.0 {
            if let Some(image) = settlement::avatar_image(kit, root, &row.avatar) {
                let frame = sketch::ph_rect(
                    avatar.left(),
                    avatar.top(),
                    avatar.width(),
                    avatar.height(),
                );
                sketch::clip_round(kit, frame, SC_AVATAR_RADIUS, || {
                    sketch::image_fill(kit, image.image(), frame);
                });
            }
        }

        let name_style = sketch::text_style(sketch::FONT_BODY, SC_NAME_SIZE, name_ink);
        let name = sketch::ellipsize(kit, &name_style, &row.name, name_w);
        settlement::line(kit, name_at(index), &name, name_ink);
        settlement::line(
            kit,
            score_at(index),
            &sketch::group_digits(&row.score.to_string()),
            SCORE_INK,
        );
        settlement::line(
            kit,
            acc_at(index),
            &settlement::acc_text(row.acc),
            OV_NOTE_INK,
        );

        let badge = badge_at(index);
        let mid = badge.top() + badge.height() / 2.0;
        let at = TextAt::center(
            badge.left() + badge.width() / 2.0,
            mid + SC_BADGE_SIZE * MID_RATIO,
            SC_BADGE_SIZE,
        );
        player::rank_frame(kit, root, badge, row.dan_level);
        settlement::line(kit, at, &row.dan_progress.to_string(), SCORE_INK);
        settlement::line(
            kit,
            rks_at(index),
            &settlement::rks_text2(row.rks),
            LABEL_INK,
        );
    }
}

pub fn cards_data(req: &SettlementRequest) -> CardData {
    let mut data = req.cards();
    data.score_line = sketch::group_digits(&req.score.to_string());
    if !req.played {
        data.score_line = String::from(settlement::NOT_PLAYED_TEXT);
    }
    if req.history > 0 {
        data.history_line = format!(
            "{NEW_BEST_PREFIX}{} {}",
            sketch::group_digits(&req.history.to_string()),
            sketch::signed_score(req.score - req.history)
        );
    }
    data
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &ScoreRequest) {
    let settled = &req.settlement;
    settlement::backdrop(kit, root, &settled.song_id);
    settlement::draw_cards(kit, root, &score_cards(), &cards_data(settled));
    cover_window(kit, root, &settled.song_id);
    cover_lines(kit, settled);
    draw_board(kit, root, &req.ranking);
    settlement::player_bar(kit, root, &settled.player, SC_PLAYER_BAR_DY);
    settlement::gained_bar(kit, &settled.chart_history);
    settlement::buttons(kit, root);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row(rank: i32) -> ScoreRankRow {
        ScoreRankRow {
            rank,
            name: String::from("Aurorix"),
            avatar: String::from("Introduction"),
            rks: 16.1234,
            dan_level: 5,
            dan_progress: 12,
            score: 996_233,
            acc: 99.45,
            self_row: false,
            hidden: false,
        }
    }

    #[test]
    fn canvas_stays_at_go_size() {
        assert_eq!(canvas_size(), (1920, 886));
        assert_eq!(SC_BOARD_ROWS, 7);
    }

    #[test]
    fn band_rects_share_the_same_vertical_span() {
        let top = cover();
        let side = board();
        assert!((top.top() - SC_TOP).abs() < 1e-4);
        assert!((top.bottom() - SC_BOTTOM).abs() < 1e-4);
        assert!((side.top() - top.top()).abs() < 1e-4);
        assert!((side.bottom() - top.bottom()).abs() < 1e-4);
        assert!(side.left() > top.right());
    }

    #[test]
    fn cards_are_the_settlement_set_shifted_left() {
        let settle = settlement::settle_cards();
        let page = score_cards();
        assert!((page.score.left() - (settle.score.left() - SC_SHIFT)).abs() < 1e-4);
        assert!((page.ratings.right() - (settle.ratings.right() - SC_SHIFT)).abs() < 1e-4);
        assert!((page.score_text.x - (settle.score_text.x - SC_SHIFT)).abs() < 1e-4);
        assert_eq!(page.backs.len(), 3);
    }

    #[test]
    fn board_rows_are_evenly_spaced_over_seven_lines() {
        let height = board_row_h();
        assert!(height > 0.0);
        let first = board_row_at(0);
        let last = board_row_at(SC_BOARD_ROWS - 1);
        assert!((last.top() + height - (board().bottom() - SC_BOARD_PAD_B)).abs() < 1e-3);
        assert!(last.top() > first.top());
        assert!((row_rule_y(0) - (first.top() + height)).abs() < 1e-4);
        assert!(row_up_mid(0) < row_down_mid(0));
    }

    #[test]
    fn row_texts_share_one_left_edge() {
        let name = name_at(2);
        let badge = badge_at(2);
        assert!((name.x - badge.left()).abs() < 1e-4);
        assert!(score_at(2).x > name.x);
        assert!(acc_at(2).x > badge.right());
        assert!(name_width() > 0.0);
    }

    #[test]
    fn board_labels_follow_go_wording() {
        let mut ranking = ScoreRanking {
            diff: String::from("IN"),
            total: 12345,
            global: false,
            rows: vec![sample_row(1)],
        };
        assert_eq!(score_board_count(&ranking), "共 12345 人");
        ranking.global = true;
        assert_eq!(score_board_count(&ranking), "全服共 12'345 人");
        assert_eq!(rank_label(&sample_row(7)), "#7");
        let mut hidden = sample_row(7);
        hidden.hidden = true;
        assert_eq!(rank_label(&hidden), "？");
    }

    #[test]
    fn score_line_uses_thousands_apostrophes() {
        let played = SettlementRequest {
            score: 996_233,
            played: true,
            ..SettlementRequest::default()
        };
        assert_eq!(cards_data(&played).score_line, "996'233");
        let unplayed = SettlementRequest {
            score: 996_233,
            played: false,
            ..SettlementRequest::default()
        };
        assert_eq!(cards_data(&unplayed).score_line, "尚未游玩");
        let with_history = SettlementRequest {
            score: 996_233,
            played: true,
            history: 990_614,
            ..SettlementRequest::default()
        };
        assert_eq!(
            cards_data(&with_history).history_line,
            "NEW BEST 990'614 +5'619"
        );
    }
}
