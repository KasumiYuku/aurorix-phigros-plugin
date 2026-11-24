use crate::model::MAX_SCORE;
use crate::repo::{LeaderBoard, LeaderRow, RksPoint, ScoreEvent};

pub const MILESTONE_BREAKS: usize = 2;

pub const MAX_CHALLENGE_COLOR: i64 = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MilestoneKind {
    #[default]
    Joined,
    FirstPhi,
    FirstFc,
    Rks,
    Peak,
    Recent,
}

impl MilestoneKind {
    pub fn label(self) -> &'static str {
        match self {
            MilestoneKind::Joined => "入坑",
            MilestoneKind::FirstPhi => "首个 φ",
            MilestoneKind::FirstFc => "首个 FC",
            MilestoneKind::Rks => "破 RKS",
            MilestoneKind::Peak => "最高 RKS",
            MilestoneKind::Recent => "最近提升",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Milestone {
    pub at: i64,
    pub day: String,
    pub kind: MilestoneKind,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RankRow {
    pub place: i32,
    pub nickname: String,
    pub avatar: String,
    pub badge: i32,
    pub badge_level: i32,
    pub rks: f64,
    pub me: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RankBoard {
    pub has_data: bool,
    pub scope: String,
    pub total: i32,
    pub me: RankRow,
    pub top: Vec<RankRow>,
    pub omitted: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dan {
    pub level: i32,
    pub progress: i32,
}

pub fn dan_of(challenge: i64) -> Dan {
    if challenge <= 0 {
        return Dan::default();
    }
    Dan {
        level: (challenge / 100).clamp(0, MAX_CHALLENGE_COLOR) as i32,
        progress: (challenge % 100).clamp(0, 99) as i32,
    }
}

fn rank_row(entry: &LeaderRow, place: i32) -> RankRow {
    let dan = dan_of(entry.challenge);
    RankRow {
        place,
        nickname: entry.nickname.trim().to_string(),
        avatar: entry.avatar.trim().to_string(),
        badge: dan.progress,
        badge_level: dan.level,
        rks: entry.rks,
        me: false,
        hidden: false,
    }
}

pub fn rank_rows(board: &LeaderBoard, hidden: bool, scope: &str) -> RankBoard {
    if board.total <= 0 {
        return RankBoard::default();
    }
    let mut out = RankBoard {
        has_data: true,
        scope: scope.trim().to_string(),
        total: board.total as i32,
        me: RankRow::default(),
        top: Vec::with_capacity(board.top.len()),
        omitted: 0,
    };
    for (index, entry) in board.top.iter().enumerate() {
        out.top.push(rank_row(entry, index as i32 + 1));
    }
    if board.me_place > 0 {
        if let Some(entry) = board.me.as_ref() {
            out.me = rank_row(entry, board.me_place as i32);
        }
    }
    if hidden {
        out.me.hidden = true;
        out.me.place = 0;
        out.omitted = 0;
        return out;
    }
    if out.me.place > 0 {
        let omitted = out.me.place - 1 - out.top.len() as i32;
        if omitted > 0 {
            out.omitted = omitted;
        }
        for row in out.top.iter_mut() {
            row.me = row.place == out.me.place;
        }
    }
    out
}

fn day_text(at: i64) -> String {
    match chrono::DateTime::from_timestamp(at, 0) {
        Some(stamp) => stamp.format("%Y-%m-%d").to_string(),
        None => String::new(),
    }
}

fn joined_span(from: i64, now: i64) -> String {
    if now <= from {
        return String::from("刚入坑");
    }
    let days = (now - from) / 86_400;
    if days >= 365 {
        return format!("{} 年 {} 天", days / 365, days % 365);
    }
    if days >= 30 {
        return format!("{} 个月 {} 天", days / 30, days % 30);
    }
    format!("{days} 天")
}

fn rks_breaks(points: &[RksPoint], count: usize) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::with_capacity(count);
    let mut prev = 0.0_f64;
    for (index, point) in points.iter().enumerate() {
        let value = point.rks;
        if value <= 0.0 {
            continue;
        }
        if prev > 0.0 && value as i32 > prev as i32 {
            out.push(index);
        }
        prev = value;
    }
    if out.len() > count {
        out = out[out.len() - count..].to_vec();
    }
    out
}

fn rks_peak(points: &[RksPoint]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (index, point) in points.iter().enumerate() {
        if point.rks <= 0.0 {
            continue;
        }
        match best {
            Some(at) if points[at].rks >= point.rks => {}
            _ => best = Some(index),
        }
    }
    best
}

fn rks_last_gain(points: &[RksPoint]) -> Option<(usize, f64)> {
    for index in (1..points.len()).rev() {
        let gain = points[index].rks - points[index - 1].rks;
        if gain > 0.0 {
            return Some((index, gain));
        }
    }
    None
}

pub fn milestones(
    events: &[ScoreEvent],
    points: &[RksPoint],
    joined_at: i64,
    now: i64,
) -> Vec<Milestone> {
    let mut out: Vec<Milestone> = Vec::with_capacity(6);
    if joined_at > 0 {
        out.push(Milestone {
            at: joined_at,
            day: day_text(joined_at),
            kind: MilestoneKind::Joined,
            text: format!("到现在 {}", joined_span(joined_at, now)),
        });
    }
    let first_phi = events.iter().find(|event| {
        event.has_old && event.new_score >= MAX_SCORE && event.old_score < MAX_SCORE
    });
    let first_fc = events
        .iter()
        .find(|event| event.has_old && event.new_fc && !event.old_fc);
    if let Some(event) = first_phi {
        out.push(Milestone {
            at: event.observed_at,
            day: day_text(event.observed_at),
            kind: MilestoneKind::FirstPhi,
            text: event.song_id.clone(),
        });
    }
    if let Some(event) = first_fc {
        let same = first_phi
            .map(|phi| {
                phi.song_id == event.song_id
                    && phi.difficulty == event.difficulty
                    && phi.observed_at == event.observed_at
            })
            .unwrap_or(false);
        if !same {
            out.push(Milestone {
                at: event.observed_at,
                day: day_text(event.observed_at),
                kind: MilestoneKind::FirstFc,
                text: event.song_id.clone(),
            });
        }
    }
    for index in rks_breaks(points, MILESTONE_BREAKS) {
        let point = &points[index];
        out.push(Milestone {
            at: point.observed_at,
            day: day_text(point.observed_at),
            kind: MilestoneKind::Rks,
            text: format!("RKS {}.00", point.rks as i32),
        });
    }
    if let Some(index) = rks_peak(points) {
        let point = &points[index];
        out.push(Milestone {
            at: point.observed_at,
            day: day_text(point.observed_at),
            kind: MilestoneKind::Peak,
            text: format!("RKS {:.4}", point.rks),
        });
    }
    if let Some((index, gain)) = rks_last_gain(points) {
        let point = &points[index];
        out.push(Milestone {
            at: point.observed_at,
            day: day_text(point.observed_at),
            kind: MilestoneKind::Recent,
            text: format!("+{gain:.4}"),
        });
    }
    out.sort_by_key(|item| item.at);
    out
}

pub fn pick_milestones(items: &[Milestone], max: usize) -> Vec<Milestone> {
    if items.len() <= max {
        return items.to_vec();
    }
    if max < 1 {
        return Vec::new();
    }
    let mut latest: Vec<(MilestoneKind, usize)> = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        match latest.iter_mut().find(|(kind, _)| *kind == item.kind) {
            Some(slot) => slot.1 = index,
            None => latest.push((item.kind, index)),
        }
    }
    latest.sort_by_key(|(_, index)| *index);
    let mut picked: Vec<usize> = Vec::with_capacity(max);
    for (_, index) in latest.iter().rev() {
        if picked.len() >= max {
            break;
        }
        picked.push(*index);
    }
    for index in (0..items.len()).rev() {
        if picked.len() >= max {
            break;
        }
        if !picked.contains(&index) {
            picked.push(index);
        }
    }
    picked.sort_by_key(|index| *index);
    picked
        .into_iter()
        .map(|index| items[index].clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Difficulty;

    fn entry(user: &str, rks: f64, challenge: i64) -> LeaderRow {
        LeaderRow {
            user_id: user.to_string(),
            nickname: format!("名 {user}"),
            avatar: String::from("Introduction"),
            rks,
            challenge,
        }
    }

    fn point(at: i64, rks: f64) -> RksPoint {
        RksPoint {
            observed_at: at,
            rks,
            computed: rks,
            records: 0,
            phi: 0,
            challenge: 0,
        }
    }

    fn event(at: i64, song: &str, has_old: bool, old_score: i32, new_score: i32, old_fc: bool, new_fc: bool) -> ScoreEvent {
        ScoreEvent {
            observed_at: at,
            song_id: song.to_string(),
            difficulty: Difficulty::In,
            has_old,
            old_score,
            old_acc: 0.0,
            old_fc,
            new_score,
            new_acc: 0.0,
            new_fc,
            old_rks: 0.0,
            new_rks: 0.0,
        }
    }

    #[test]
    fn rank_rows_place_me_and_count_the_omitted() {
        let board = LeaderBoard {
            total: 4_096,
            me_place: 12,
            me: Some(entry("me", 15.9, 502)),
            top: vec![
                entry("a", 16.4, 300),
                entry("b", 16.2, 0),
                entry("c", 16.0, 501),
            ],
        };
        let rows = rank_rows(&board, false, "全服排名");
        assert!(rows.has_data);
        assert_eq!(rows.total, 4_096);
        assert_eq!(rows.scope, "全服排名");
        assert_eq!(rows.top.len(), 3);
        assert_eq!(rows.top[0].place, 1);
        assert_eq!(rows.top[2].place, 3);
        assert!(rows.top.iter().all(|row| !row.me));
        assert_eq!(rows.me.place, 12);
        assert_eq!(rows.me.badge_level, 5);
        assert_eq!(rows.me.badge, 2);
        assert_eq!(rows.omitted, 8);
        assert_eq!(rows.top[0].badge_level, 3);
    }

    #[test]
    fn hidden_me_row_drops_place_and_omitted() {
        let board = LeaderBoard {
            total: 512,
            me_place: 2,
            me: Some(entry("me", 16.0, 0)),
            top: vec![entry("a", 16.4, 0), entry("me", 16.0, 0)],
        };
        let rows = rank_rows(&board, true, "");
        assert!(rows.me.hidden);
        assert_eq!(rows.me.place, 0);
        assert_eq!(rows.omitted, 0);
        assert!(rows.top.iter().all(|row| !row.me));
        assert_eq!(rank_rows(&LeaderBoard::default(), false, ""), RankBoard::default());
    }

    #[test]
    fn me_row_is_highlighted_inside_the_top() {
        let board = LeaderBoard {
            total: 90,
            me_place: 2,
            me: Some(entry("me", 16.0, 0)),
            top: vec![entry("a", 16.4, 0), entry("me", 16.0, 0), entry("b", 15.4, 0)],
        };
        let rows = rank_rows(&board, false, "");
        assert!(rows.top[1].me);
        assert_eq!(rows.omitted, 0);
    }

    #[test]
    fn milestones_cover_joined_phi_fc_break_peak_and_gain() {
        let events = vec![
            event(10, "A", true, 950_000, 1_000_000, true, true),
            event(20, "B", true, 900_000, 990_000, false, true),
        ];
        let points = vec![
            point(0, 13.90),
            point(100, 14.91),
            point(200, 15.02),
            point(300, 15.34),
            point(400, 15.30),
        ];
        let list = milestones(&events, &points, 5, 5 + 400 * 86_400);
        let kinds: Vec<MilestoneKind> = list.iter().map(|item| item.kind).collect();
        assert_eq!(kinds[0], MilestoneKind::Joined);
        assert_eq!(list[0].text, "到现在 1 年 35 天");
        assert!(kinds.contains(&MilestoneKind::FirstPhi));
        assert!(kinds.contains(&MilestoneKind::FirstFc));
        assert_eq!(
            list.iter().filter(|item| item.kind == MilestoneKind::Rks).count(),
            MILESTONE_BREAKS
        );
        let rks: Vec<&Milestone> = list.iter().filter(|item| item.kind == MilestoneKind::Rks).collect();
        assert_eq!(rks[0].text, "RKS 14.00");
        assert_eq!(rks[1].text, "RKS 15.00");
        let peak = list.iter().find(|item| item.kind == MilestoneKind::Peak).unwrap();
        assert_eq!(peak.text, "RKS 15.3400");
        assert_eq!(peak.day, day_text(300));
        let recent = list.iter().find(|item| item.kind == MilestoneKind::Recent).unwrap();
        assert_eq!(recent.text, "+0.3200");
        assert!(list.windows(2).all(|pair| pair[0].at <= pair[1].at));
        assert_eq!(list.iter().find(|item| item.kind == MilestoneKind::FirstPhi).unwrap().text, "A");
    }

    #[test]
    fn milestones_skip_the_fc_that_is_the_first_phi() {
        let events = vec![event(10, "A", true, 950_000, 1_000_000, true, true)];
        let list = milestones(&events, &[], 0, 0);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kind, MilestoneKind::FirstPhi);
    }

    #[test]
    fn pick_keeps_one_row_per_kind_newest_first() {
        let items: Vec<Milestone> = (0..7)
            .map(|index| Milestone {
                at: index,
                day: String::new(),
                kind: MilestoneKind::Rks,
                text: String::new(),
            })
            .collect();
        let picked = pick_milestones(&items, 5);
        assert_eq!(picked.len(), 5);
        assert_eq!(picked[picked.len() - 1].at, 6);
        let mixed = vec![
            Milestone { at: 1, day: String::new(), kind: MilestoneKind::Joined, text: String::new() },
            Milestone { at: 2, day: String::new(), kind: MilestoneKind::Peak, text: String::new() },
            Milestone { at: 3, day: String::new(), kind: MilestoneKind::Rks, text: String::new() },
            Milestone { at: 4, day: String::new(), kind: MilestoneKind::Rks, text: String::new() },
            Milestone { at: 5, day: String::new(), kind: MilestoneKind::Peak, text: String::new() },
            Milestone { at: 6, day: String::new(), kind: MilestoneKind::Recent, text: String::new() },
        ];
        let picked = pick_milestones(&mixed, 3);
        assert_eq!(picked.len(), 3);
        assert_eq!(picked[0].at, 4);
        assert_eq!(picked[1].at, 5);
        assert_eq!(picked[2].at, 6);
    }

    #[test]
    fn dan_of_splits_level_and_progress() {
        assert_eq!(dan_of(0), Dan::default());
        assert_eq!(dan_of(501), Dan { level: 5, progress: 1 });
        assert_eq!(dan_of(399), Dan { level: 3, progress: 99 });
        assert_eq!(dan_of(999), Dan { level: 5, progress: 99 });
        assert_eq!(dan_of(227), Dan { level: 2, progress: 27 });
    }
}
