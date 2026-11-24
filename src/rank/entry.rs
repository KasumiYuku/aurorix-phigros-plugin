use std::cmp::Ordering;
use std::collections::HashSet;

use crate::domain::catalog::Catalog;
use crate::domain::rks;
use crate::domain::span::Span;
use crate::model::{Difficulty, Scored, Song, BEST_SIZE, COUNTED_BEST};
use crate::rank::level::{level_of, Level};
use crate::rank::phi;
use crate::rank::prior::{chart_key, Prior};

pub const ENTRY_FLOOR_ACC: f64 = 70.0;

pub const ENTRY_PHI_SLACK: f64 = 0.5;

pub const RETIRED_SONGS: [&str; 1] = ["テリトリーバトル.0"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntryCandidate {
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub notes: i32,
    pub target: f64,
    pub required_acc: f64,
    pub reach_acc: f64,
    pub reach_ok: bool,
    pub gain: f64,
    pub short: f64,
    pub via_phi: bool,
    pub peer_median: f64,
    pub peer_users: i32,
    pub rate: f64,
}

impl EntryCandidate {
    pub fn reachable(&self) -> bool {
        self.reach_ok && self.short <= 0.0
    }

    fn short_rank(&self) -> f64 {
        if self.reach_ok {
            self.short.max(0.0)
        } else {
            f64::INFINITY
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntryScope {
    pub span: Span,
    pub levels: Vec<Difficulty>,
}

impl EntryScope {
    pub fn contains(&self, constant: f64, difficulty: Difficulty) -> bool {
        if !self.span.contains(constant) {
            return false;
        }
        self.levels.is_empty() || self.levels.contains(&difficulty)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntryBoard {
    pub candidates: Vec<EntryCandidate>,
    pub total: i32,
    pub unreached: i32,
    pub too_hard: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EntryFit {
    #[default]
    Skipped,
    Reached,
    Unreached,
}

pub fn board(
    scored: &[Scored],
    catalog: &Catalog,
    scope: &EntryScope,
    limit: usize,
    prior: &Prior,
) -> EntryBoard {
    let cutoff = counted_cutoff(scored);
    let step = rks::display_step(rks::rks(scored)) * BEST_SIZE as f64;
    let rule = EntryRule {
        target: cutoff + step,
        cutoff,
        phi_floor: phi::floor(scored),
        phi_top: phi::top(scored),
        level: level_of(scored).with_curvature(prior.curvature),
        prior,
    };
    let played = played_charts(scored, catalog);
    let mut out: Vec<EntryCandidate> = Vec::new();
    let mut board = EntryBoard::default();
    for song in catalog.songs() {
        if song.variant() || RETIRED_SONGS.contains(&song.id.as_str()) {
            continue;
        }
        let (candidate, fit) = entry_of(song, &played, scope, &rule);
        match fit {
            EntryFit::Skipped => continue,
            EntryFit::Unreached => {
                board.total += 1;
                board.unreached += 1;
            }
            EntryFit::Reached => {
                board.total += 1;
                out.push(candidate);
            }
        }
    }
    out.sort_by(|left, right| {
        if entry_less(left, right) {
            Ordering::Less
        } else if entry_less(right, left) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
    let (reachable, unreachable): (Vec<EntryCandidate>, Vec<EntryCandidate>) =
        out.into_iter().partition(EntryCandidate::reachable);
    board.too_hard = unreachable.len() as i32;
    board.candidates = reachable.into_iter().take(limit).collect();
    board
}

pub fn entry_less(left: &EntryCandidate, right: &EntryCandidate) -> bool {
    if left.reachable() != right.reachable() {
        return left.reachable();
    }
    if left.reachable() {
        if left.gain != right.gain {
            return left.gain > right.gain;
        }
        if left.required_acc != right.required_acc {
            return left.required_acc < right.required_acc;
        }
        if left.constant != right.constant {
            return left.constant > right.constant;
        }
    } else {
        if left.short_rank() != right.short_rank() {
            return left.short_rank() < right.short_rank();
        }
        if left.constant != right.constant {
            return left.constant < right.constant;
        }
    }
    if left.difficulty != right.difficulty {
        return left.difficulty < right.difficulty;
    }
    left.song_id < right.song_id
}

struct EntryRule<'a> {
    target: f64,
    cutoff: f64,
    phi_floor: f64,
    phi_top: f64,
    level: Level,
    prior: &'a Prior,
}

impl EntryRule<'_> {
    fn fill(&self, candidate: &mut EntryCandidate) {
        if let Some(peer) = self.prior.peer(&candidate.song_id, candidate.difficulty) {
            candidate.reach_acc = peer.median.min(100.0);
            candidate.reach_ok = true;
            candidate.peer_median = peer.median;
            candidate.peer_users = peer.users;
            candidate.rate = peer.rate(candidate.required_acc);
        } else if let Some(reach) = self.level.acc_at(candidate.constant) {
            candidate.reach_acc = reach;
            candidate.reach_ok = true;
        } else {
            return;
        }
        candidate.short = candidate.required_acc - candidate.reach_acc;
        let mut gain = (rks::single_rks(candidate.required_acc, candidate.constant) - self.cutoff)
            .max(0.0)
            / BEST_SIZE as f64;
        if candidate.short <= 0.0 {
            gain = (rks::single_rks(candidate.reach_acc, candidate.constant) - self.cutoff).max(0.0)
                / BEST_SIZE as f64;
        }
        if candidate.peer_users > 0
            && candidate.rate > 0.0
            && self.phi_floor > 0.0
            && candidate.constant > self.phi_floor
            && self.phi_top > 0.0
            && candidate.constant <= self.phi_top + ENTRY_PHI_SLACK
        {
            let mut phi = (rks::single_rks(100.0, candidate.constant) - self.cutoff).max(0.0)
                / BEST_SIZE as f64;
            phi += (candidate.constant - self.phi_floor) / BEST_SIZE as f64;
            if phi > gain {
                gain = phi;
                candidate.via_phi = true;
                candidate.required_acc = 100.0;
                candidate.reach_acc = 100.0;
                candidate.short = 0.0;
            }
        }
        candidate.gain = gain;
    }
}

fn entry_of(
    song: &Song,
    played: &HashSet<String>,
    scope: &EntryScope,
    rule: &EntryRule<'_>,
) -> (EntryCandidate, EntryFit) {
    let mut fit = EntryFit::Skipped;
    let mut best = EntryCandidate::default();
    for level in Difficulty::playable_all() {
        let Some(chart) = song.chart(*level) else {
            continue;
        };
        if played.contains(&chart_key(&song.id, *level)) {
            continue;
        }
        if !scope.contains(chart.constant, *level) {
            continue;
        }
        let Some(required_acc) = entry_acc(rule.target, chart.constant) else {
            if fit == EntryFit::Skipped {
                fit = EntryFit::Unreached;
            }
            continue;
        };
        let mut candidate = EntryCandidate {
            song_id: song.id.clone(),
            title: song.title.clone(),
            difficulty: *level,
            constant: chart.constant,
            notes: chart.notes.combo(),
            target: rule.target,
            required_acc,
            ..EntryCandidate::default()
        };
        rule.fill(&mut candidate);
        if fit != EntryFit::Reached || entry_less(&candidate, &best) {
            best = candidate;
        }
        fit = EntryFit::Reached;
    }
    (best, fit)
}

fn entry_acc(target: f64, constant: f64) -> Option<f64> {
    let floor = rks::single_rks(ENTRY_FLOOR_ACC, constant);
    let target = target.max(floor);
    rks::required_acc(target, constant).map(ceil_acc)
}

fn ceil_acc(acc: f64) -> f64 {
    (acc * 100.0).ceil() / 100.0
}

pub(crate) fn counted_cutoff(scored: &[Scored]) -> f64 {
    let mut counted: Vec<f64> = scored
        .iter()
        .filter(|item| item.counted() && !item.unrecorded())
        .map(|item| item.rks)
        .collect();
    counted.sort_by(|left, right| right.partial_cmp(left).unwrap_or(Ordering::Equal));
    counted.get(COUNTED_BEST - 1).copied().unwrap_or_default()
}

fn played_charts(scored: &[Scored], catalog: &Catalog) -> HashSet<String> {
    let mut out: HashSet<String> = HashSet::with_capacity(scored.len());
    for item in scored {
        if item.unrecorded() {
            continue;
        }
        let id = match catalog.song(item.song_id()) {
            Some(song) if song.variant() => song.variant_of.as_str(),
            _ => item.song_id(),
        };
        out.insert(chart_key(id, item.difficulty()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BandStat, ChartStat, PlayRecord, ServerStats};
    use crate::rank::prior::prior_from;

    fn stats(median: f64) -> ServerStats {
        ServerStats {
            bands: vec![BandStat { band: 15, users: 40 }],
            charts: vec![ChartStat {
                song_id: String::from("song"),
                difficulty: Difficulty::At,
                band: 15,
                users: 40,
                median,
                p75: 99.0,
                hits: [40, 40, 40, 40, 40, 40, 40],
            }],
            ..ServerStats::default()
        }
    }

    fn rule<'a>(prior: &'a Prior, phi_top: f64) -> EntryRule<'a> {
        EntryRule {
            target: 14.03,
            cutoff: 14.0,
            phi_floor: 14.0,
            phi_top,
            level: Level::default(),
            prior,
        }
    }

    fn scored(acc: f64, constant: f64) -> Scored {
        Scored {
            record: PlayRecord {
                song_id: String::from("played"),
                difficulty: Difficulty::In,
                score: 900_000,
                acc,
                fc: false,
            },
            constant,
            known: true,
            ..Scored::default()
        }
    }

    fn candidate(constant: f64) -> EntryCandidate {
        EntryCandidate {
            song_id: String::from("song"),
            title: String::from("Song"),
            difficulty: Difficulty::At,
            constant,
            target: 14.03,
            required_acc: 99.11,
            ..EntryCandidate::default()
        }
    }

    #[test]
    fn entry_less_prefers_reachable_then_gain() {
        let low = EntryCandidate {
            gain: 0.01,
            reach_ok: true,
            short: -0.5,
            ..EntryCandidate::default()
        };
        let high = EntryCandidate {
            gain: 0.02,
            reach_ok: true,
            short: -0.1,
            ..EntryCandidate::default()
        };
        let far = EntryCandidate {
            gain: 0.5,
            reach_ok: false,
            short: 2.0,
            ..EntryCandidate::default()
        };
        assert!(entry_less(&high, &low));
        assert!(entry_less(&low, &far));
        assert!(!entry_less(&far, &low));
        assert!(!far.reachable());
    }

    #[test]
    fn via_phi_switches_the_target_to_a_full_score() {
        let prior = prior_from(&stats(98.0), 15);
        let mut entry = candidate(14.6);
        rule(&prior, 14.6).fill(&mut entry);
        assert!(entry.via_phi);
        assert_eq!(entry.required_acc, 100.0);
        assert_eq!(entry.short, 0.0);
        assert!(entry.reachable());
        assert!(entry.gain > 0.03);
    }

    #[test]
    fn via_phi_needs_a_full_score_within_reach() {
        let prior = prior_from(&stats(98.0), 15);
        let mut entry = candidate(15.4);
        rule(&prior, 14.6).fill(&mut entry);
        assert!(!entry.via_phi);
        assert!((entry.required_acc - 99.11).abs() < 1e-9);
        assert!(entry.short > 0.0);
        assert!(!entry.reachable());
    }

    #[test]
    fn entry_reach_falls_back_to_the_players_own_curve() {
        let prior = Prior::default();
        let level = level_of(&[
            scored(90.0, 13.0),
            scored(88.0, 13.0),
            scored(86.0, 13.0),
            scored(80.0, 14.0),
            scored(78.0, 14.0),
            scored(76.0, 14.0),
        ]);
        let mut entry = candidate(14.0);
        EntryRule {
            level,
            ..rule(&prior, 0.0)
        }
        .fill(&mut entry);
        assert!(entry.reach_ok);
        assert_eq!(entry.peer_users, 0);
        assert!((entry.reach_acc - 78.0).abs() < 1e-9);
        assert!(entry.short > 0.0);
    }
}
