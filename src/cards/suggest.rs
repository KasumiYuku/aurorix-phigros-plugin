use std::path::Path;

use aurorix_render::skia_safe::Rect;
use aurorix_render::{CanvasKit, Color};

use super::player::{self, Player};
use super::settlement::{self, Align, TextAt};
use super::{memo, sketch};

pub const SUG_W: f32 = 1920.0;

pub const SUG_SHRINK: f32 = 1.0;

pub const SUG_COLS: i32 = 3;

pub const SUG_KPI_N: i32 = 4;

pub const SUG_MARGIN_X: f32 = 56.0;

pub const SUG_CONTENT_X0: f32 = SUG_MARGIN_X;

pub const SUG_CONTENT_X1: f32 = SUG_W - SUG_MARGIN_X;

pub const SUG_TITLE_BASE: f32 = 108.0;

pub const SUG_NOTE_BASE: f32 = 156.0;

pub const SUG_TITLE_SIZE: f32 = 50.0;

pub const SUG_NOTE_SIZE: f32 = 24.0;

pub const SUG_CHIP_H: f32 = 44.0;

pub const SUG_CHIP_PAD: f32 = 20.0;

pub const SUG_CHIP_RADIUS: f32 = 12.0;

pub const SUG_CHIP_GAP: f32 = 28.0;

pub const SUG_CHIP_TEXT_SIZE: f32 = 24.0;

pub const SUG_SUMMARY_GAP: f32 = 24.0;

pub const SUG_SUMMARY_H: f32 = 150.0;

pub const SUG_SUMMARY_VALUE_SIZE: f32 = 44.0;

pub const SUG_SUMMARY_LABEL_SIZE: f32 = 22.0;

pub const SUG_SUMMARY_VALUE_BASE: f32 = 50.0;

pub const SUG_SUMMARY_LABEL_BASE: f32 = 92.0;

pub const SUG_SUMMARY_UNDER_W: f32 = 72.0;

pub const SUG_SUMMARY_UNDER_H: f32 = 3.0;

pub const SUG_BLOCK_GAP: f32 = 16.0;

pub const SUG_CARD_H: f32 = 220.0;

pub const SUG_COL_GAP: f32 = 32.0;

pub const SUG_ROW_GAP: f32 = 16.0;

pub const SUG_CARD_PAD: f32 = 24.0;

pub const SUG_CARD_RADIUS: f32 = 16.0;

pub const SUG_TIER_HEAD_H: f32 = 76.0;

pub const SUG_TIER_HEAD_BASE: f32 = 50.0;

pub const SUG_TIER_DOT_SIDE: f32 = 16.0;

pub const SUG_TIER_DOT_GAP: f32 = 20.0;

pub const SUG_TIER_HEAD_SIZE: f32 = 32.0;

pub const SUG_ENTRY_HEAD_H: f32 = 70.0;

pub const SUG_ENTRY_HEAD_BASE: f32 = 36.0;

pub const SUG_ENTRY_DOT_SIDE: f32 = 13.0;

pub const SUG_ENTRY_DOT_GAP: f32 = 18.0;

pub const SUG_ENTRY_NOTE_GAP: f32 = 14.0;

pub const SUG_ENTRY_HEAD_SIZE: f32 = 24.0;

pub const SUG_DOT_RATIO: f32 = 0.35;

pub const SUG_THUMB: f32 = 112.0;

pub const SUG_THUMB_RADIUS: f32 = 14.0;

pub const SUG_THUMB_GAP: f32 = 20.0;

pub const SUG_BAR_H: f32 = 8.0;

pub const SUG_TICK_W: f32 = 3.0;

pub const SUG_TICK_H: f32 = 20.0;

pub const SUG_TRACK_LIFT: f32 = 8.0;

pub const SUG_DIFF_CHIP_H: f32 = 36.0;

pub const SUG_DIFF_CHIP_PAD: f32 = 14.0;

pub const SUG_DIFF_CHIP_RADIUS: f32 = 8.0;

pub const SUG_NAME_SIZE: f32 = 30.0;

pub const SUG_SMALL_SIZE: f32 = 22.0;

pub const SUG_LEAD_SIZE: f32 = 36.0;

pub const SUG_DIFF_CHIP_SIZE: f32 = 20.0;

pub const SUG_TEXT_RATIO: f32 = 0.36;

pub const SUG_NAME_BASE: f32 = 34.0;

pub const SUG_NOW_BASE: f32 = 78.0;

pub const SUG_LEAD_BASE: f32 = 136.0;

pub const SUG_NAME_GAP: f32 = 14.0;

pub const SUG_FOOTER_H: f32 = 92.0;

pub const SUG_FOOTER_BASE: f32 = 46.0;

pub const SUG_SIGN_SIZE: f32 = 24.0;
pub const SUG_TITLE: &str = "推分建议";

pub const SUG_NOTE: &str = "大字是这条打完能涨多少 RKS；按定数分档、档内按收益降序；末段是还没打过的谱面";

pub const SUG_ALL_CONST: &str = "全部定数";

pub const SUG_LABEL_CAND: &str = "候选谱面";

pub const SUG_LABEL_RKS: &str = "当前 RKS";

pub const SUG_LABEL_CUTOFF: &str = "计入线";

pub const SUG_LABEL_STEP: &str = "单曲增幅";

pub const SUG_LABEL_NOW_ACC: &str = "现在 ACC";

pub const SUG_LABEL_TARGET_ACC: &str = "目标 ACC";

pub const SUG_LABEL_TO_PHI: &str = "距满分";

pub const SUG_CUTOFF_OPEN: &str = "未满";

pub const SUG_GAIN_LEAD: &str = "RKS ";

pub const SUG_NEED_LEAD: &str = "需 ";

pub const SUG_LABEL_REACH: &str = "目标 ACC";

pub const SUG_LABEL_SHORT: &str = "还差";

pub const SUG_LABEL_PHI: &str = "这条路";

pub const SUG_PHI_GOAL: &str = "打满 φ";

pub const SUG_ENTRY_NOW: &str = "未打过";

pub const SUG_ENTRY_HEAD: &str = "未打过的谱面 · 进榜线";

pub const SUG_ENTRY_NOTE_OPEN: &str = "(";

pub const SUG_ENTRY_NOTE_CLOSE: &str = ")";

pub const SUG_ENTRY_TOTAL_PREFIX: &str = "共 ";

pub const SUG_ENTRY_TOTAL_SUFFIX: &str = " 首";

pub const SUG_ENTRY_NOTE_PREFIX: &str = " · 其中 ";

pub const SUG_ENTRY_NOTE_SUFFIX: &str = " 首打满也进不了 B27";

pub const SUG_ENTRY_HARD_PREFIX: &str = " · 另有 ";

pub const SUG_ENTRY_HARD_SUFFIX: &str = " 首按你现在的水平还够不着";

pub const SUG_DASH: &str = "-";

pub const SUG_SKIP_PREFIX: &str = "定数低于计入线，已略过 ";

pub const SUG_SKIP_SUFFIX: &str = " 首";

pub const SUG_SKIP_PHI_PREFIX: &str = "（其中 ";

pub const SUG_SKIP_PHI_SUFFIX: &str = " 首打满 φ 可顶 P3）";

pub const SUG_SHORT_THRESHOLD: f64 = 0.01;

pub const PUSH_CLOSE: i32 = 0;

pub const PUSH_NEAR: i32 = 1;

pub const PUSH_MID: i32 = 2;

pub const PUSH_FAR: i32 = 3;

pub const PUSH_DISTANT: i32 = 4;

pub const SUG_TRACK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x18);

pub const SUG_TICK_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0xC0);

pub const SUG_TIER_CLOSE_INK: Color = settlement::RANK_ACCENT;

pub const SUG_TIER_NEAR_INK: Color = Color::new(0x9F, 0xD8, 0xE8, 0xFF);

pub const SUG_TIER_MID_INK: Color = settlement::SCORE_INK;

pub const SUG_TIER_FAR_INK: Color = Color::new(0xE8, 0x9A, 0x5A, 0xFF);

pub const SUG_TIER_DISTANT_INK: Color = Color::new(0xE2, 0x6D, 0x5A, 0xFF);

pub const SUG_PHI_INK: Color = Color::new(0xF0, 0xC7, 0x5E, 0xFF);

pub const SUG_ENTRY_INK: Color = Color::new(0xB2, 0x8C, 0xF0, 0xFF);

pub const SUG_DIFF_CHIP_INK: Color = Color::new(0xFF, 0xFF, 0xFF, 0x10);

pub const SUG_DIFF_CHIP_TEXT: Color = settlement::SCORE_INK;

pub const SUG_CARD_INK: Color = settlement::PANEL_INK;

pub const SUG_ACCENT_INK: Color = settlement::RANK_ACCENT;

pub const SUG_SIGN_SHADOW_INK: Color = Color::new(0x04, 0x05, 0x09, 0xB4);

pub const SUG_SIGN_SHADOW_OFFSET: f32 = 1.0;

pub const SUG_SIGN_TRACKING: f32 = 1.4;

pub const SUG_SIGNATURE_PREFIX: &str = "Generated By ";

pub const SUG_SIGNATURE_MARK: &str = "Okamikko";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuggestCandidate {
    pub song_id: String,
    pub title: String,
    pub diff: String,
    pub constant: f64,
    pub known: bool,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
    pub target: f64,
    pub target_acc: f64,
    pub gap: f64,
    pub via_phi: bool,
    pub gain: f64,
    pub peer_median: f64,
    pub peer_users: i32,
    pub rate: f64,
    pub counted: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuggestEntry {
    pub song_id: String,
    pub title: String,
    pub diff: String,
    pub constant: f64,
    pub notes: i32,
    pub target: f64,
    pub required_acc: f64,
    pub via_phi: bool,
    pub reach_acc: f64,
    pub reach_ok: bool,
    pub gain: f64,
    pub short: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuggestGroup {
    pub tier: i32,
    pub phi: bool,
    pub note: String,
    pub cand: Vec<SuggestCandidate>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SuggestRequest {
    pub player: Player,
    pub backdrop: String,
    pub filter: String,
    pub groups: Vec<SuggestGroup>,
    pub total: i32,
    pub cutoff: f64,
    pub rks: f64,
    pub step: f64,
    pub out_of_reach: i32,
    pub empty: String,
    pub out_of_reach_phi: i32,
    pub entry: Vec<SuggestEntry>,
    pub entry_total: i32,
    pub entry_unreached: i32,
    pub entry_too_hard: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SectionKind {
    #[default]
    Cards,
    Entry,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Section {
    pub kind: SectionKind,
    pub cards: i32,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlotBody {
    Group(SuggestGroup),
    Entry(Vec<SuggestEntry>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub section: Section,
    pub body: SlotBody,
}

impl Slot {
    pub fn group(&self) -> Option<&SuggestGroup> {
        match &self.body {
            SlotBody::Group(group) => Some(group),
            SlotBody::Entry(_) => None,
        }
    }

    pub fn entries(&self) -> &[SuggestEntry] {
        match &self.body {
            SlotBody::Group(_) => &[],
            SlotBody::Entry(entry) => entry,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub slots: Vec<Slot>,
    pub heads: Vec<f32>,
    pub cards: Vec<f32>,
    pub height: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Push,
    Phi,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Face<'a> {
    pub song_id: &'a str,
    pub title: &'a str,
    pub diff: &'a str,
    pub constant: f64,
}

pub fn rks_text(value: f64) -> String {
    let clamped = if value < 0.0 { 0.0 } else { value };
    format!("{clamped:.4}")
}

pub fn delta_text(delta: f64) -> String {
    let text = format!("{delta:+.4}");
    if text == "-0.0000" {
        return String::from("+0.0000");
    }
    text
}

pub fn rks_gain_text(gain: f64) -> String {
    format!("{gain:+.4}")
}

pub fn cutoff_text(cutoff: f64) -> String {
    if cutoff <= 0.0 {
        return String::from(SUG_CUTOFF_OPEN);
    }
    rks_text(cutoff)
}

pub fn skip_note(out_of_reach: i32, out_of_reach_phi: i32) -> String {
    if out_of_reach <= 0 {
        return String::new();
    }
    let mut note = format!("{SUG_SKIP_PREFIX}{out_of_reach}{SUG_SKIP_SUFFIX}");
    if out_of_reach_phi > 0 && out_of_reach_phi <= out_of_reach {
        note.push_str(SUG_SKIP_PHI_PREFIX);
        note.push_str(&out_of_reach_phi.to_string());
        note.push_str(SUG_SKIP_PHI_SUFFIX);
    }
    note
}

pub fn entry_note(req: &SuggestRequest) -> String {
    if req.entry_total <= 0 {
        return String::new();
    }
    let mut note = format!(
        "{SUG_ENTRY_TOTAL_PREFIX}{}{SUG_ENTRY_TOTAL_SUFFIX}",
        req.entry_total
    );
    if req.entry_unreached > 0 {
        note.push_str(SUG_ENTRY_NOTE_PREFIX);
        note.push_str(&req.entry_unreached.to_string());
        note.push_str(SUG_ENTRY_NOTE_SUFFIX);
    }
    if req.entry_too_hard > 0 {
        note.push_str(SUG_ENTRY_HARD_PREFIX);
        note.push_str(&req.entry_too_hard.to_string());
        note.push_str(SUG_ENTRY_HARD_SUFFIX);
    }
    format!("{SUG_ENTRY_NOTE_OPEN}{note}{SUG_ENTRY_NOTE_CLOSE}")
}

pub fn entry_now_text(entry: &SuggestEntry) -> String {
    if entry.required_acc <= 0.0 {
        return String::from(SUG_ENTRY_NOW);
    }
    format!(
        "{SUG_ENTRY_NOW} · {SUG_NEED_LEAD}{}",
        settlement::acc_text(entry.required_acc)
    )
}

pub fn entry_reference(entry: &SuggestEntry) -> Cell {
    if entry.via_phi {
        return Cell {
            label: String::from(SUG_LABEL_PHI),
            value: String::from(SUG_PHI_GOAL),
        };
    }
    if entry.short >= SUG_SHORT_THRESHOLD {
        return Cell {
            label: String::from(SUG_LABEL_SHORT),
            value: settlement::acc_text(entry.short),
        };
    }
    let mut reference = Cell {
        label: String::from(SUG_LABEL_REACH),
        value: String::from(SUG_DASH),
    };
    if entry.reach_ok {
        reference.value = settlement::acc_text(entry.reach_acc);
    }
    reference
}

pub fn now_text(candidate: &SuggestCandidate) -> String {
    format!(
        "{SUG_LABEL_NOW_ACC} {}",
        settlement::acc_text(candidate.acc)
    )
}

fn lead_text(candidate: &SuggestCandidate, mode: Mode) -> String {
    mode.lead(candidate)
}

impl Mode {
    pub fn lead(self, candidate: &SuggestCandidate) -> String {
        format!("{SUG_GAIN_LEAD}{}", rks_gain_text(candidate.gain))
    }

    pub fn reference(self, candidate: &SuggestCandidate) -> Cell {
        if matches!(self, Mode::Phi) {
            return Cell {
                label: String::from(SUG_LABEL_TO_PHI),
                value: settlement::acc_text(100.0 - candidate.acc),
            };
        }
        Cell {
            label: String::from(SUG_LABEL_TARGET_ACC),
            value: settlement::acc_text(candidate.target_acc),
        }
    }

    pub fn now(self, candidate: &SuggestCandidate) -> f32 {
        if candidate.constant <= 1e-9 {
            return 0.0;
        }
        (candidate.rks / candidate.constant) as f32
    }

    pub fn aim(self, candidate: &SuggestCandidate) -> f32 {
        if candidate.constant <= 1e-9 {
            return 1.0;
        }
        (candidate.target / candidate.constant) as f32
    }
}

pub fn group_ink(group: &SuggestGroup) -> Color {
    if group.phi {
        return SUG_PHI_INK;
    }
    match group.tier {
        PUSH_NEAR => SUG_TIER_NEAR_INK,
        PUSH_MID => SUG_TIER_MID_INK,
        PUSH_FAR => SUG_TIER_FAR_INK,
        PUSH_DISTANT => SUG_TIER_DISTANT_INK,
        _ => SUG_TIER_CLOSE_INK,
    }
}

pub fn group_mode(group: &SuggestGroup) -> Mode {
    if group.phi {
        return Mode::Phi;
    }
    Mode::Push
}

pub fn player_bar_bottom() -> f32 {
    player::panel_rect(0.0)
        .bottom()
        .max(player::avatar_window().bottom())
}

pub fn summary_y0() -> f32 {
    player_bar_bottom() + SUG_SUMMARY_GAP
}

pub fn grid_y0() -> f32 {
    summary_y0() + SUG_SUMMARY_H + SUG_SUMMARY_GAP
}

pub fn summary_rect() -> Rect {
    Rect::from_xywh(
        SUG_CONTENT_X0,
        summary_y0(),
        SUG_CONTENT_X1 - SUG_CONTENT_X0,
        SUG_SUMMARY_H,
    )
}

pub fn summary_boxed() -> Rect {
    let area = summary_rect();
    let pad = settlement::LEAN * area.height() / 2.0;
    Rect::from_xywh(
        area.left() + pad,
        area.top(),
        area.width() - 2.0 * pad,
        area.height(),
    )
}

pub fn summary_content() -> Rect {
    let area = summary_rect();
    Rect::from_ltrb(
        area.left() + SUG_CARD_PAD,
        area.top() + SUG_CARD_PAD,
        area.right() - SUG_CARD_PAD,
        area.bottom() - SUG_CARD_PAD,
    )
}

pub fn summary_kpi_at(content: Rect, index: i32) -> (TextAt, TextAt) {
    if !(0..SUG_KPI_N).contains(&index) {
        return (TextAt::left(0.0, 0.0, 0.0), TextAt::left(0.0, 0.0, 0.0));
    }
    let cell = content.width() / SUG_KPI_N as f32;
    let cx = content.left() + (index as f32 + 0.5) * cell;
    (
        TextAt::center(cx, content.top() + SUG_SUMMARY_VALUE_BASE, SUG_SUMMARY_VALUE_SIZE)
            .with_font(sketch::FONT_DISPLAY),
        TextAt::center(
            cx,
            content.top() + SUG_SUMMARY_LABEL_BASE,
            SUG_SUMMARY_LABEL_SIZE,
        ),
    )
}

pub fn rows_in(n: i32) -> i32 {
    if n <= 0 {
        return 0;
    }
    (n + SUG_COLS - 1) / SUG_COLS
}

pub fn block_h(n: i32) -> f32 {
    let rows = rows_in(n);
    if rows <= 0 {
        return 0.0;
    }
    rows as f32 * SUG_CARD_H + (rows - 1) as f32 * SUG_ROW_GAP
}

pub fn section_head_h(section: &Section) -> f32 {
    if section.label.is_empty() {
        return 0.0;
    }
    match section.kind {
        SectionKind::Entry => SUG_ENTRY_HEAD_H,
        SectionKind::Cards => SUG_TIER_HEAD_H,
    }
}

pub fn section_h(section: &Section) -> f32 {
    section_head_h(section) + block_h(section.cards)
}

pub fn canvas_h(sections: &[Section]) -> i32 {
    let mut height = 0.0;
    if sections.is_empty() {
        height = block_h(0);
    }
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            height += SUG_BLOCK_GAP;
        }
        height += section_h(section);
    }
    (grid_y0() + height + SUG_FOOTER_H) as i32
}

pub fn card_rect(cards_y: f32, k: i32) -> Rect {
    if k < 0 {
        return Rect::from_xywh(0.0, 0.0, 0.0, 0.0);
    }
    let row = k / SUG_COLS;
    let col = k % SUG_COLS;
    let width = (SUG_CONTENT_X1 - SUG_CONTENT_X0 - (SUG_COLS - 1) as f32 * SUG_COL_GAP)
        / SUG_COLS as f32;
    Rect::from_xywh(
        SUG_CONTENT_X0 + col as f32 * (width + SUG_COL_GAP),
        cards_y + row as f32 * (SUG_CARD_H + SUG_ROW_GAP),
        width,
        SUG_CARD_H,
    )
}

pub fn title_at() -> TextAt {
    TextAt::left(SUG_CONTENT_X0, SUG_TITLE_BASE, SUG_TITLE_SIZE).with_font(sketch::FONT_DISPLAY)
}

pub fn note_at() -> TextAt {
    TextAt::left(SUG_CONTENT_X0, SUG_NOTE_BASE, SUG_NOTE_SIZE)
}

pub fn chip_rect(title_w: f32, chip_w: f32) -> Rect {
    let mid = SUG_TITLE_BASE - SUG_TITLE_SIZE * settlement::MID_RATIO;
    Rect::from_xywh(
        SUG_CONTENT_X0 + title_w + SUG_CHIP_GAP,
        mid - SUG_CHIP_H / 2.0,
        chip_w,
        SUG_CHIP_H,
    )
}

pub fn chip_text_at(chip: Rect) -> TextAt {
    TextAt::center(
        chip.left() + chip.width() / 2.0,
        chip.top() + chip.height() / 2.0 + SUG_CHIP_TEXT_SIZE * SUG_TEXT_RATIO,
        SUG_CHIP_TEXT_SIZE,
    )
}

pub fn thumb_rect(content: Rect) -> Rect {
    Rect::from_xywh(content.left(), content.top(), SUG_THUMB, SUG_THUMB)
}

pub fn text_x(content: Rect) -> f32 {
    content.left() + SUG_THUMB + SUG_THUMB_GAP
}

pub fn name_at(content: Rect) -> TextAt {
    TextAt::left(text_x(content), content.top() + SUG_NAME_BASE, SUG_NAME_SIZE)
}

pub fn now_at(content: Rect) -> TextAt {
    TextAt::left(
        text_x(content),
        content.top() + SUG_NOW_BASE,
        SUG_SMALL_SIZE,
    )
}

pub fn reference_at(content: Rect) -> TextAt {
    TextAt::right(
        content.right(),
        content.top() + SUG_NOW_BASE,
        SUG_SMALL_SIZE,
    )
}

pub fn lead_at(content: Rect) -> TextAt {
    TextAt::left(
        text_x(content),
        content.top() + SUG_LEAD_BASE,
        SUG_LEAD_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn diff_chip_rect(content: Rect, width: f32) -> Rect {
    Rect::from_xywh(
        content.right() - width,
        content.top(),
        width,
        SUG_DIFF_CHIP_H,
    )
}

pub fn track_rect(content: Rect) -> Rect {
    Rect::from_xywh(
        content.left(),
        content.bottom() - SUG_TRACK_LIFT - SUG_BAR_H,
        content.width(),
        SUG_BAR_H,
    )
}

pub fn tick_x(track: Rect, frac: f32) -> f32 {
    track.left() + track.width() * frac.clamp(0.0, 1.0)
}

pub fn tier_head_at(head_y: f32) -> TextAt {
    TextAt::left(
        SUG_CONTENT_X0 + SUG_TIER_DOT_SIDE + SUG_TIER_DOT_GAP,
        head_y + SUG_TIER_HEAD_BASE,
        SUG_TIER_HEAD_SIZE,
    )
}

pub fn tier_dot_at(head_y: f32) -> (f32, f32) {
    (
        SUG_CONTENT_X0 + SUG_TIER_DOT_SIDE / 2.0,
        head_y + SUG_TIER_HEAD_BASE - SUG_TIER_HEAD_SIZE * SUG_DOT_RATIO,
    )
}

pub fn entry_head_at(head_y: f32) -> TextAt {
    TextAt::left(
        SUG_CONTENT_X0 + SUG_ENTRY_DOT_SIDE + SUG_ENTRY_DOT_GAP,
        head_y + SUG_ENTRY_HEAD_BASE,
        SUG_ENTRY_HEAD_SIZE,
    )
}

pub fn entry_note_at(head_y: f32, label_w: f32) -> TextAt {
    TextAt::left(
        SUG_CONTENT_X0 + SUG_ENTRY_DOT_SIDE + SUG_ENTRY_DOT_GAP + label_w + SUG_ENTRY_NOTE_GAP,
        head_y + SUG_ENTRY_HEAD_BASE,
        SUG_ENTRY_HEAD_SIZE,
    )
}

pub fn entry_dot_at(head_y: f32) -> (f32, f32) {
    (
        SUG_CONTENT_X0 + SUG_ENTRY_DOT_SIDE / 2.0,
        head_y + SUG_ENTRY_HEAD_BASE - SUG_ENTRY_HEAD_SIZE * SUG_DOT_RATIO,
    )
}

pub fn sign_at(height: i32) -> TextAt {
    TextAt::left(
        SUG_CONTENT_X0,
        height as f32 - SUG_FOOTER_BASE,
        SUG_SIGN_SIZE,
    )
    .with_font(sketch::FONT_DISPLAY)
}

pub fn slots(req: &SuggestRequest) -> Vec<Slot> {
    let mut out: Vec<Slot> = Vec::with_capacity(req.groups.len() + 1);
    for group in req.groups.iter() {
        out.push(Slot {
            section: Section {
                kind: SectionKind::Cards,
                cards: group.cand.len() as i32,
                label: group.note.clone(),
            },
            body: SlotBody::Group(group.clone()),
        });
    }
    if out.is_empty() && req.entry.is_empty() && !req.empty.is_empty() {
        return vec![Slot {
            section: Section {
                kind: SectionKind::Entry,
                cards: 0,
                label: req.empty.clone(),
            },
            body: SlotBody::Entry(Vec::new()),
        }];
    }
    if !req.entry.is_empty() {
        out.push(Slot {
            section: Section {
                kind: SectionKind::Entry,
                cards: req.entry.len() as i32,
                label: String::from(SUG_ENTRY_HEAD),
            },
            body: SlotBody::Entry(req.entry.clone()),
        });
    }
    out
}

pub fn plan_for(slots: &[Slot]) -> Plan {
    let mut heads = Vec::with_capacity(slots.len());
    let mut cards = Vec::with_capacity(slots.len());
    let mut y = grid_y0();
    for slot in slots.iter() {
        heads.push(y);
        cards.push(y + section_head_h(&slot.section));
        y += section_h(&slot.section) + SUG_BLOCK_GAP;
    }
    let sections: Vec<Section> = slots.iter().map(|slot| slot.section.clone()).collect();
    Plan {
        slots: slots.to_vec(),
        heads,
        cards,
        height: canvas_h(&sections),
    }
}

pub fn plan(req: &SuggestRequest) -> Plan {
    plan_for(&slots(req))
}

pub fn canvas_size(req: &SuggestRequest) -> (i32, i32) {
    (SUG_W as i32, plan(req).height)
}

fn fill_rect(kit: &CanvasKit<'_>, area: Rect, ink: Color) -> Rect {
    let frame = sketch::ph_rect(
        area.left(),
        area.top(),
        area.width().max(0.0),
        area.height().max(0.0),
    );
    if frame.width() <= 0.0 || frame.height() <= 0.0 {
        return frame;
    }
    sketch::ph_fill(kit, frame, ink);
    frame
}

fn signature_line(kit: &CanvasKit<'_>, at: TextAt, tracking: f32, segments: &[&str]) {
    let base = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK)
        .with_letter_spacing(tracking);
    let shadow = base.clone().with_color(SUG_SIGN_SHADOW_INK);
    let mut widths = Vec::with_capacity(segments.len());
    let mut total = 0.0;
    for segment in segments {
        let width = sketch::text_width(kit, &base, segment);
        widths.push(width);
        total += width + tracking;
    }
    let start = match at.align {
        Align::Right => at.x - total,
        Align::Center => at.x - total / 2.0,
        Align::Left => at.x,
    };
    for (index, segment) in segments.iter().enumerate() {
        let x = start + widths.iter().take(index).sum::<f32>() + index as f32 * tracking;
        sketch::draw_text(
            kit,
            &shadow,
            segment,
            x + SUG_SIGN_SHADOW_OFFSET,
            at.baseline + SUG_SIGN_SHADOW_OFFSET,
            SUG_SIGN_SHADOW_INK,
        );
    }
    for (index, segment) in segments.iter().enumerate() {
        let x = start + widths.iter().take(index).sum::<f32>() + index as f32 * tracking;
        sketch::draw_text(kit, &base, segment, x, at.baseline, settlement::SCORE_INK);
    }
}

fn backdrop_cover(kit: &CanvasKit<'_>, root: &Path, song_id: &str, area: Rect, tint: Color) {
    sketch::backdrop(
        kit,
        root,
        song_id,
        area,
        settlement::COVER_FALLBACK,
        tint,
        sketch::BACKDROP_BLUR,
    );
}

fn panel(kit: &CanvasKit<'_>, area: Rect, ink: Color) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    sketch::fill_skew(kit, area, settlement::LEAN, ink);
}

fn card(kit: &CanvasKit<'_>, area: Rect) -> Rect {
    let content = Rect::from_ltrb(
        area.left() + SUG_CARD_PAD,
        area.top() + SUG_CARD_PAD,
        area.right() - SUG_CARD_PAD,
        area.bottom() - SUG_CARD_PAD,
    );
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return content;
    }
    sketch::fill_round(kit, area, SUG_CARD_RADIUS, SUG_CARD_INK);
    content
}

fn dot(kit: &CanvasKit<'_>, x: f32, y: f32, side: f32, ink: Color) {
    if side <= 0.0 {
        return;
    }
    let radius = side / 2.0;
    let area = Rect::from_ltrb(
        (x - radius).round(),
        (y - radius).round(),
        (x + radius).round(),
        (y + radius).round(),
    );
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    sketch::fill_round(kit, area, radius, ink);
}

fn thumb(kit: &CanvasKit<'_>, root: &Path, area: Rect, song_id: &str) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let texture = if song_id.is_empty() {
        None
    } else {
        memo::texture(kit, &super::thumb_key(root, song_id))
            .or_else(|| memo::texture(kit, &super::illust_key(root, song_id)))
    };
    match texture.as_ref().and_then(|item| item.image()) {
        Some(image) => sketch::clip_round(kit, area, SUG_THUMB_RADIUS, || {
            sketch::image_cover(kit, image, area);
        }),
        None => sketch::fill_round(kit, area, SUG_THUMB_RADIUS, settlement::COVER_FALLBACK),
    }
}

fn progress_width(track: f32, ratio: f32) -> f32 {
    if track <= 0.0 || ratio <= 0.0 || ratio.is_nan() {
        return 0.0;
    }
    let ratio = if ratio > 1.0 { 1.0 } else { ratio };
    (track * ratio).round().max(1.0).min(track)
}

fn track(kit: &CanvasKit<'_>, area: Rect, now: f32, aim: f32, ink: Color) {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return;
    }
    let radius = SUG_BAR_H / 2.0;
    sketch::fill_round(kit, area, radius, SUG_TRACK_INK);
    let width = progress_width(area.width(), now.clamp(0.0, 1.0));
    if width > 0.0 {
        sketch::fill_round(
            kit,
            Rect::from_ltrb(area.left(), area.top(), area.left() + width, area.bottom()),
            radius,
            ink,
        );
    }
    let top = area.top() - (SUG_TICK_H - SUG_BAR_H) / 2.0;
    let x = tick_x(area, aim.clamp(0.0, 1.0)).round() - SUG_TICK_W / 2.0;
    fill_rect(
        kit,
        Rect::from_xywh(x, top, SUG_TICK_W, SUG_TICK_H),
        SUG_TICK_INK,
    );
}

fn diff_chip(kit: &CanvasKit<'_>, content: Rect, diff: &str, constant: f64) -> Rect {
    let text = format!("{diff} {}", settlement::const_text1(constant));
    let style = sketch::text_style(sketch::FONT_BODY, SUG_DIFF_CHIP_SIZE, SUG_DIFF_CHIP_TEXT);
    let width = sketch::text_width(kit, &style, &text).round() + 2.0 * SUG_DIFF_CHIP_PAD;
    let area = diff_chip_rect(content, width);
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return area;
    }
    sketch::fill_round(kit, area, SUG_DIFF_CHIP_RADIUS, SUG_DIFF_CHIP_INK);
    let at = TextAt::center(
        area.left() + area.width() / 2.0,
        area.top() + area.height() / 2.0 + SUG_DIFF_CHIP_SIZE * SUG_TEXT_RATIO,
        SUG_DIFF_CHIP_SIZE,
    );
    settlement::line(kit, at, &text, SUG_DIFF_CHIP_TEXT);
    area
}

fn frame(kit: &CanvasKit<'_>, root: &Path, area: Rect, face: &Face<'_>) -> Rect {
    let content = card(kit, area);
    thumb(kit, root, thumb_rect(content), face.song_id);
    let chip = diff_chip(kit, content, face.diff, face.constant);
    if !face.title.is_empty() {
        let at = name_at(content);
        let style = sketch::text_style(at.font_key(), at.size, settlement::SCORE_INK);
        let width = chip.left() - at.x - SUG_NAME_GAP;
        if width > 0.0 {
            let cut = sketch::ellipsize(kit, &style, face.title, width);
            settlement::line(kit, at, &cut, settlement::SCORE_INK);
        }
    }
    content
}

fn candidate_card(
    kit: &CanvasKit<'_>,
    root: &Path,
    area: Rect,
    candidate: &SuggestCandidate,
    mode: Mode,
    ink: Color,
) {
    let content = frame(
        kit,
        root,
        area,
        &Face {
            song_id: &candidate.song_id,
            title: &candidate.title,
            diff: &candidate.diff,
            constant: candidate.constant,
        },
    );
    settlement::line(kit, now_at(content), &now_text(candidate), settlement::LABEL_INK);
    let reference = mode.reference(candidate);
    if !reference.label.is_empty() {
        settlement::line(
            kit,
            reference_at(content),
            &format!("{} {}", reference.label, reference.value),
            settlement::LABEL_INK,
        );
    }
    settlement::line(kit, lead_at(content), &lead_text(candidate, mode), ink);
    track(
        kit,
        track_rect(content),
        mode.now(candidate),
        mode.aim(candidate),
        ink,
    );
}

fn entry_card(kit: &CanvasKit<'_>, root: &Path, area: Rect, entry: &SuggestEntry) {
    let content = frame(
        kit,
        root,
        area,
        &Face {
            song_id: &entry.song_id,
            title: &entry.title,
            diff: &entry.diff,
            constant: entry.constant,
        },
    );
    settlement::line(
        kit,
        now_at(content),
        &entry_now_text(entry),
        settlement::LABEL_INK,
    );
    let reference = entry_reference(entry);
    settlement::line(
        kit,
        reference_at(content),
        &format!("{} {}", reference.label, reference.value),
        settlement::LABEL_INK,
    );
    settlement::line(
        kit,
        lead_at(content),
        &format!("{SUG_GAIN_LEAD}{}", rks_gain_text(entry.gain)),
        SUG_ENTRY_INK,
    );
    let aim = if entry.constant > 1e-9 {
        (entry.target / entry.constant) as f32
    } else {
        1.0
    };
    track(kit, track_rect(content), 0.0, aim, SUG_ENTRY_INK);
}

fn tier_head(kit: &CanvasKit<'_>, head_y: f32, label: &str, ink: Color) {
    if label.is_empty() {
        return;
    }
    let (x, y) = tier_dot_at(head_y);
    dot(kit, x, y, SUG_TIER_DOT_SIDE, ink);
    settlement::line(kit, tier_head_at(head_y), label, ink);
}

fn entry_head(kit: &CanvasKit<'_>, head_y: f32, label: &str, req: &SuggestRequest) {
    let (x, y) = entry_dot_at(head_y);
    dot(kit, x, y, SUG_ENTRY_DOT_SIDE, SUG_ENTRY_INK);
    let at = entry_head_at(head_y);
    settlement::line(kit, at, label, SUG_ENTRY_INK);
    let note = entry_note(req);
    if note.is_empty() {
        return;
    }
    let width = settlement::measure(kit, at, label);
    settlement::line(kit, entry_note_at(head_y, width), &note, settlement::LABEL_INK);
}

fn kpi(
    kit: &CanvasKit<'_>,
    at: TextAt,
    number: &str,
    ink: Color,
    under_w: f32,
    under_ink: Color,
) {
    if at.size <= 0.0 || number.is_empty() {
        return;
    }
    settlement::line(kit, at, number, ink);
    if under_w <= 0.0 {
        return;
    }
    let y = at.baseline + at.size * 0.22;
    let x0 = at.x - under_w / 2.0;
    fill_rect(
        kit,
        Rect::from_ltrb(x0, y, x0 + under_w, y + SUG_SUMMARY_UNDER_H),
        under_ink,
    );
}

fn head(kit: &CanvasKit<'_>, req: &SuggestRequest) {
    settlement::line(kit, title_at(), SUG_TITLE, settlement::SCORE_INK);
    settlement::line(kit, note_at(), SUG_NOTE, settlement::PLACEHOLDER_INK);
    let filter = if req.filter.is_empty() {
        SUG_ALL_CONST
    } else {
        req.filter.as_str()
    };
    let chip_style = sketch::text_style(sketch::FONT_BODY, SUG_CHIP_TEXT_SIZE, settlement::LABEL_INK);
    let width = sketch::text_width(kit, &chip_style, filter).round() + 2.0 * SUG_CHIP_PAD;
    let title_style = sketch::text_style(sketch::FONT_DISPLAY, SUG_TITLE_SIZE, settlement::SCORE_INK);
    let title_w = sketch::text_width(kit, &title_style, SUG_TITLE).round();
    let chip = chip_rect(title_w, width);
    sketch::fill_round(kit, chip, SUG_CHIP_RADIUS, SUG_DIFF_CHIP_INK);
    settlement::line(kit, chip_text_at(chip), filter, settlement::LABEL_INK);
}

fn summary(kit: &CanvasKit<'_>, req: &SuggestRequest) {
    panel(kit, summary_boxed(), settlement::PANEL_INK);
    let content = summary_content();
    let total = req.total.max(req.groups.len() as i32);
    let values = [
        total.to_string(),
        rks_text(req.rks),
        cutoff_text(req.cutoff),
        delta_text(req.step),
    ];
    let labels = [
        SUG_LABEL_CAND,
        SUG_LABEL_RKS,
        SUG_LABEL_CUTOFF,
        SUG_LABEL_STEP,
    ];
    for (index, number) in values.iter().enumerate() {
        let (value, label) = summary_kpi_at(content, index as i32);
        let (ink, under) = if index == 3 {
            (SUG_ACCENT_INK, SUG_SUMMARY_UNDER_W)
        } else {
            (settlement::SCORE_INK, 0.0)
        };
        kpi(kit, value, number, ink, under, SUG_ACCENT_INK);
        settlement::line(kit, label, labels[index], settlement::LABEL_INK);
    }
}

fn cards(kit: &CanvasKit<'_>, root: &Path, req: &SuggestRequest, plan: &Plan) {
    for (index, slot) in plan.slots.iter().enumerate() {
        let head_y = plan.heads[index];
        let cards_y = plan.cards[index];
        if slot.section.kind == SectionKind::Entry {
            entry_head(kit, head_y, &slot.section.label, req);
            for (k, entry) in slot.entries().iter().enumerate() {
                if k as i32 >= slot.section.cards {
                    break;
                }
                entry_card(kit, root, card_rect(cards_y, k as i32), entry);
            }
            continue;
        }
        let Some(group) = slot.group() else {
            continue;
        };
        let ink = group_ink(group);
        let mode = group_mode(group);
        tier_head(kit, head_y, &slot.section.label, ink);
        for (k, candidate) in group.cand.iter().enumerate() {
            if k as i32 >= slot.section.cards {
                break;
            }
            candidate_card(kit, root, card_rect(cards_y, k as i32), candidate, mode, ink);
        }
    }
}

fn foot(kit: &CanvasKit<'_>, req: &SuggestRequest, height: i32) {
    signature_line(
        kit,
        sign_at(height),
        SUG_SIGN_TRACKING,
        &[SUG_SIGNATURE_PREFIX, SUG_SIGNATURE_MARK],
    );
    let note = skip_note(req.out_of_reach, req.out_of_reach_phi);
    if note.is_empty() {
        return;
    }
    let at = TextAt::right(
        SUG_CONTENT_X1,
        height as f32 - SUG_FOOTER_BASE,
        SUG_SIGN_SIZE,
    );
    settlement::line(kit, at, &note, settlement::PLACEHOLDER_INK);
}

pub fn draw(kit: &CanvasKit<'_>, root: &Path, req: &SuggestRequest) {
    let plan = plan(req);
    let canvas = Rect::from_xywh(0.0, 0.0, SUG_W, plan.height as f32);
    backdrop_cover(kit, root, &req.backdrop, canvas, sketch::BACKDROP_TINT);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SCRIM);
    sketch::ph_fill(kit, canvas, sketch::BACKDROP_SHEET_SCRIM);
    head(kit, req);
    player::draw(kit, root, SUG_W, 1.0, 0.0, &req.player);
    summary(kit, req);
    cards(kit, root, req, &plan);
    foot(kit, req, plan.height);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Instant;

    use aurorix_render::skia_safe::{Data, Image};
    use aurorix_render::{RenderContext, SkiaBackend};

    use super::*;

    const OUT_DIR: &str = "/tmp/phigros-verify";

    const OUT_SUGGEST: &str = "/tmp/phigros-verify/13-suggest.png";

    fn candidate(index: i32, phi: bool) -> SuggestCandidate {
        SuggestCandidate {
            song_id: String::from("Spasmodic.姜米條颶風元力上人.0"),
            title: String::from("Spasmodic"),
            diff: String::from("IN"),
            constant: 15.7,
            known: true,
            score: 985_000 + index * 900,
            acc: 98.12 + f64::from(index) * 0.08,
            rks: 14.92 + f64::from(index) * 0.03,
            target: 15.22,
            target_acc: 99.24,
            gap: 1.12,
            via_phi: phi,
            gain: 0.0212 - f64::from(index) * 0.0011,
            peer_median: 99.02,
            peer_users: 124,
            rate: 0.74,
            counted: true,
        }
    }

    fn group(tier: i32, phi: bool, note: &str) -> SuggestGroup {
        SuggestGroup {
            tier,
            phi,
            note: String::from(note),
            cand: (0..5).map(|index| candidate(index, phi)).collect(),
        }
    }

    fn request() -> SuggestRequest {
        let entry: Vec<SuggestEntry> = (0..4)
            .map(|index| SuggestEntry {
                song_id: String::from("Igorrr.Exitium.0"),
                title: String::from("Exitium"),
                diff: String::from("AT"),
                constant: 16.4,
                notes: 1420,
                target: 15.86,
                required_acc: 98.62,
                via_phi: index == 1,
                reach_acc: 98.44,
                reach_ok: index != 2,
                gain: 0.0131 - f64::from(index) * 0.0009,
                short: if index == 3 { 0.42 } else { 0.0 },
            })
            .collect();
        SuggestRequest {
            player: Player {
                has_player: true,
                name: String::from("Okamikko"),
                avatar: String::new(),
                rank: 213,
                rank_level: 4,
                rks: 15.6234,
            },
            backdrop: String::from("Spasmodic.姜米條颶風元力上人.0"),
            filter: String::from("15.4-16.0 · IN"),
            groups: vec![
                group(PUSH_CLOSE, true, "打满 φ"),
                group(PUSH_NEAR, false, "ACC 差 ≤ 0.20%"),
                group(PUSH_MID, false, "ACC 差 ≤ 0.50%"),
                group(PUSH_FAR, false, "ACC 差 ≤ 1.00%"),
            ],
            total: 128,
            cutoff: 15.1042,
            rks: 15.6234,
            step: 0.0134,
            out_of_reach: 36,
            empty: String::from("这一段没有候选"),
            out_of_reach_phi: 7,
            entry,
            entry_total: 42,
            entry_unreached: 9,
            entry_too_hard: 5,
        }
    }

    #[test]
    #[ignore]
    fn render_suggest() {
        let root = crate::assets_root();
        let _ = crate::cards::register_fonts(&root);
        let assets = Arc::new(crate::assets_in(&root));
        let context = RenderContext::new(SkiaBackend::handle(), 64, 256 * 1024 * 1024)
            .with_asset_loader(assets.loader());
        let req = request();
        let (width, height) = canvas_size(&req);
        let request =
            crate::cards::card_request(width, height).with_format(aurorix_render::RenderFormat::Png);
        let start = Instant::now();
        let bytes = context
            .draw(request, |canvas, ctx| {
                let kit = CanvasKit::with_chain(canvas, ctx, sketch::font_chain());
                crate::cards::with_device_scale(&kit, width, height, || draw(&kit, &root, &req));
            })
            .expect("渲染失败")
            .to_vec();
        let elapsed = start.elapsed();
        std::fs::create_dir_all(OUT_DIR).expect("建出图目录");
        std::fs::write(OUT_SUGGEST, &bytes).expect("写出 PNG");
        std::fs::write("/tmp/phigros-suggest-preview.png", &bytes).expect("写出预览 PNG");
        let image = Image::from_encoded(Data::new_copy(&bytes)).expect("解码 PNG");
        println!(
            "suggest 渲染耗时 {} ms ({} 字节, {}x{})",
            elapsed.as_millis(),
            bytes.len(),
            image.width(),
            image.height()
        );
        assert_eq!(
            (image.width(), image.height()),
            crate::cards::device_size(width, height),
            "出图尺寸应与设备倍率换算后的一致"
        );
        assert!(bytes.len() > 1000, "出图字节数应大于 1000");
    }
}
