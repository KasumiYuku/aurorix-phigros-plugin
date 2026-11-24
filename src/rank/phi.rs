use std::cmp::Ordering;

use crate::domain::rks;
use crate::model::{Scored, COUNTED_PHI, MAX_SCORE};

pub const PHI_REACHABLE_ACC: f64 = 99.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhiCandidate {
    pub index: usize,
    pub gain: f64,
}

pub fn floor(scored: &[Scored]) -> f64 {
    let mut constants: Vec<f64> = scored
        .iter()
        .filter(|item| item.record.score >= MAX_SCORE)
        .map(|item| item.constant)
        .collect();
    if constants.len() < COUNTED_PHI {
        return 0.0;
    }
    constants.sort_by(|left, right| {
        right
            .partial_cmp(left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    constants[COUNTED_PHI - 1]
}

pub fn top(scored: &[Scored]) -> f64 {
    scored
        .iter()
        .filter(|item| item.record.score >= MAX_SCORE)
        .fold(0.0_f64, |best, item| best.max(item.constant))
}

pub fn candidates(scored: &[Scored], limit: usize) -> Vec<PhiCandidate> {
    let floor = floor(scored);
    let base = rks::rks(scored);
    let mut out: Vec<PhiCandidate> = Vec::new();
    for (index, item) in scored.iter().enumerate() {
        if !item.counted() || item.record.score >= MAX_SCORE || item.constant <= floor {
            continue;
        }
        if item.acc() < PHI_REACHABLE_ACC {
            continue;
        }
        let mut hypothetical: Vec<Scored> = scored.to_vec();
        hypothetical[index].record.score = MAX_SCORE;
        hypothetical[index].record.acc = 100.0;
        hypothetical[index].rks = item.constant;
        let gain = rks::rks(&hypothetical) - base;
        if gain <= 0.0 {
            continue;
        }
        out.push(PhiCandidate { index, gain });
    }
    out.sort_by(|left, right| {
        right
            .gain
            .partial_cmp(&left.gain)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                scored[right.index]
                    .constant
                    .partial_cmp(&scored[left.index].constant)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                let a = scored[left.index].song_id();
                let b = scored[right.index].song_id();
                a.cmp(b)
            })
    });
    out.truncate(limit);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Difficulty, PlayRecord};

    fn scored(constant: f64, acc: f64, score: i32) -> Scored {
        Scored {
            record: PlayRecord {
                song_id: format!("s{constant}"),
                difficulty: Difficulty::In,
                score,
                acc,
                fc: false,
            },
            constant,
            known: true,
            rks: rks::single_rks(acc, constant),
            ..Scored::default()
        }
    }

    #[test]
    fn case_555() {
        let mut records = vec![
            scored(15.0, 99.0, 990_000),
            scored(14.0, 100.0, MAX_SCORE),
            scored(14.4, 100.0, MAX_SCORE),
            scored(14.8, 100.0, MAX_SCORE),
        ];
        assert!((floor(&records) - 14.0).abs() < 1e-9);
        assert!((top(&records) - 14.8).abs() < 1e-9);
        records.truncate(2);
        assert_eq!(floor(&records), 0.0);
        assert!((top(&records) - 14.0).abs() < 1e-9);
    }

    #[test]
    fn case_556() {
        let records = vec![
            scored(14.0, 100.0, MAX_SCORE),
            scored(14.4, 100.0, MAX_SCORE),
            scored(14.8, 100.0, MAX_SCORE),
            scored(15.0, 99.0, 990_000),
            scored(15.2, 99.7, 997_000),
        ];
        let out = candidates(&records, 8);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].index, 4);
        assert!(out[0].gain > 0.0);
    }
}
