use crate::domain::rks;
use crate::model::{Difficulty, Scored, BEST_SIZE, MAX_SCORE};
use crate::rank::entry::counted_cutoff;
use crate::rank::level::{level_of, Level};
use crate::rank::phi;
use crate::rank::prior::Prior;

pub const PUSH_MAX_GAP: f64 = 5.0;

pub const PHI_REACHABLE_ACC: f64 = 99.5;

const PUSH_TIER_LIMITS: [f64; 4] = [0.2, 0.5, 1.0, 3.0];

pub fn tier_gap_limit(tier: i32) -> Option<f64> {
    usize::try_from(tier)
        .ok()
        .and_then(|index| PUSH_TIER_LIMITS.get(index).copied())
}

pub fn tier_of_gap(gap: f64) -> i32 {
    let mut tier = 0;
    for limit in PUSH_TIER_LIMITS {
        if gap <= limit {
            break;
        }
        tier += 1;
    }
    tier
}

#[derive(Clone, Debug, PartialEq)]
pub struct PushCandidate {
    pub index: usize,
    pub song_id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub constant: f64,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
    pub target_acc: f64,
    pub gap: f64,
    pub gain: f64,
    pub via_phi: bool,
    pub band: i32,
    pub peer_median: f64,
    pub peer_users: i32,
    pub rate: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PushBoard {
    pub candidates: Vec<PushCandidate>,
    pub cutoff: f64,
    pub rks: f64,
    pub step: f64,
    pub out_of_reach: i32,
    pub out_of_reach_phi: i32,
}

pub fn push(scored: &[Scored], limit: usize, prior: &Prior) -> PushBoard {
    let base = rks::rks(scored);
    let step = rks::display_step(base) * BEST_SIZE as f64;
    let cutoff = counted_cutoff(scored);
    let level = level_of(scored).with_curvature(prior.curvature);
    let phi_floor = phi::floor(scored);
    let enter = cutoff + step;
    let mut work: Vec<Scored> = scored.to_vec();
    let mut candidates: Vec<PushCandidate> = Vec::new();
    let mut out_of_reach = 0;
    let mut out_of_reach_phi = 0;
    for (index, item) in scored.iter().enumerate() {
        if item.unrecorded() || !item.counted() {
            continue;
        }
        if item.record.score >= MAX_SCORE {
            continue;
        }
        if item.constant < cutoff {
            out_of_reach += 1;
            if item.constant > phi_floor {
                out_of_reach_phi += 1;
            }
            continue;
        }
        let Some((reach, via_phi)) = reach_of(item, &level, prior, enter) else {
            continue;
        };
        if reach <= item.acc() || reach - item.acc() > PUSH_MAX_GAP {
            continue;
        }
        let gain = rks_with(&mut work, index, reach) - base;
        if gain <= 0.0 {
            continue;
        }
        let mut candidate = PushCandidate {
            index,
            song_id: item.song_id().to_string(),
            title: item.title.clone(),
            difficulty: item.difficulty(),
            constant: item.constant,
            score: item.record.score,
            acc: item.acc(),
            rks: item.rks,
            target_acc: reach,
            gap: reach - item.acc(),
            gain,
            via_phi,
            band: item.constant.trunc() as i32,
            peer_median: 0.0,
            peer_users: 0,
            rate: 0.0,
        };
        if let Some(peer) = prior.peer(item.song_id(), item.difficulty()) {
            candidate.peer_median = peer.median;
            candidate.peer_users = peer.users;
            candidate.rate = peer.rate(reach);
        }
        candidates.push(candidate);
    }
    candidates.sort_by(|left, right| {
        right
            .gain
            .partial_cmp(&left.gain)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.gap
                    .partial_cmp(&right.gap)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| {
                right
                    .constant
                    .partial_cmp(&left.constant)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.song_id.cmp(&right.song_id))
    });
    if limit > 0 && candidates.len() > limit {
        candidates.truncate(limit);
    }
    PushBoard {
        candidates,
        cutoff,
        rks: base,
        step,
        out_of_reach,
        out_of_reach_phi,
    }
}

fn reach_of(item: &Scored, level: &Level, prior: &Prior, enter: f64) -> Option<(f64, bool)> {
    if item.acc() >= PHI_REACHABLE_ACC {
        return Some((100.0, true));
    }
    let mut reach = match prior.peer(item.song_id(), item.difficulty()) {
        Some(peer) => peer.median.min(100.0),
        None => level.best_at(item.constant)?,
    };
    if let Some(need) = rks::required_acc(enter, item.constant) {
        if need > reach {
            reach = need;
        }
    }
    Some((reach, false))
}

fn rks_with(scored: &mut [Scored], index: usize, acc: f64) -> f64 {
    let saved = scored[index].clone();
    scored[index].record.acc = acc;
    scored[index].rks = rks::single_rks(acc, scored[index].constant);
    if acc >= 100.0 {
        scored[index].record.score = MAX_SCORE;
    } else if scored[index].record.score >= MAX_SCORE {
        scored[index].record.score = MAX_SCORE - 1;
    }
    let got = rks::rks(scored);
    scored[index] = saved;
    got
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PlayRecord;

    fn scored(constant: f64, acc: f64, score: i32) -> Scored {
        Scored {
            record: PlayRecord {
                song_id: format!("s{constant}"),
                difficulty: Difficulty::In,
                score,
                acc,
                fc: false,
            },
            title: format!("s{constant}"),
            constant,
            known: true,
            rks: rks::single_rks(acc, constant),
            ..Scored::default()
        }
    }

    #[test]
    fn gain_is_total_rks_delta_not_single_chart_delta() {
        let mut records: Vec<Scored> = (0..30)
            .map(|index| scored(15.0 + index as f64 * 0.1, 99.0, 990_000))
            .collect();
        records.push(scored(16.0, 96.0, 960_000));
        let board = push(&records, 0, &Prior::default());
        let item = board
            .candidates
            .iter()
            .find(|candidate| candidate.song_id == "s16")
            .expect("16.0 那条应该能推");
        let mut hypothetical = records.clone();
        hypothetical[30].record.acc = item.target_acc;
        hypothetical[30].rks = rks::single_rks(item.target_acc, 16.0);
        let expect = rks::rks(&hypothetical) - rks::rks(&records);
        assert!(
            (item.gain - expect).abs() < 1e-12,
            "增益必须是整份 RKS 的差：{} 对 {}",
            item.gain,
            expect
        );
        let single = item.rks - hypothetical[30].rks;
        assert!(
            (item.gain - single).abs() > 1e-6,
            "不该把单曲 RKS 的差当总增量：{} 对 {}",
            item.gain,
            single
        );
        assert!(item.gain < 0.1, "总 RKS 增量不可能这么大：{}", item.gain);
    }

    #[test]
    fn cutoff_uses_whole_score_set() {
        let records: Vec<Scored> = (0..40)
            .map(|index| scored(10.0 + index as f64 * 0.1, 97.0, 970_000))
            .collect();
        let board = push(&records, 0, &Prior::default());
        let mut ranked: Vec<f64> = records.iter().map(|item| item.rks).collect();
        ranked.sort_by(|left, right| right.partial_cmp(left).unwrap());
        assert!((board.cutoff - ranked[26]).abs() < 1e-9);
    }

    #[test]
    fn tier_limits_follow_go() {
        assert_eq!(tier_of_gap(0.2), 0);
        assert_eq!(tier_of_gap(0.21), 1);
        assert_eq!(tier_of_gap(1.0), 2);
        assert_eq!(tier_of_gap(2.5), 3);
        assert_eq!(tier_of_gap(3.5), 4);
        assert_eq!(tier_gap_limit(3), Some(3.0));
        assert_eq!(tier_gap_limit(4), None);
    }

    #[test]
    fn records_below_cutoff_are_counted_not_listed() {
        let mut records: Vec<Scored> = (0..30)
            .map(|index| scored(16.0 + index as f64 * 0.05, 99.0, 990_000))
            .collect();
        records.push(scored(9.0, 99.0, 990_000));
        let board = push(&records, 0, &Prior::default());
        assert_eq!(board.out_of_reach, 1, "定数低于计入线的那条只记账");
        assert!(
            board
                .candidates
                .iter()
                .all(|candidate| candidate.constant >= board.cutoff)
        );
    }
}
