use std::cmp::Ordering;
use std::collections::HashMap;

use crate::model::Scored;

const LEVEL_BUCKET_WIDTH: f64 = 1.0;

const LEVEL_MIN_SAMPLES_FLOOR: i32 = 3;

const LEVEL_MIN_SAMPLES_CAP: i32 = 10;

const LEVEL_BUCKET_EDGE: f64 = 2.0;

const LEVEL_REACH_MAX: f64 = 2.5;

const LEVEL_ACC_FLOOR: f64 = 60.0;

const LEVEL_SLOPE_FLOOR: f64 = 1.5;

const LEVEL_BEST_SLACK: f64 = 0.3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Level {
    buckets: Vec<LevelBucket>,
    curv: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct LevelBucket {
    constant: f64,
    acc: f64,
    best: f64,
    best_at: f64,
    n: i32,
}

#[derive(Clone, Copy, Debug, Default)]
struct LevelAcc {
    sum: f64,
    best: f64,
    best_at: f64,
    count: f64,
}

pub fn level_of(scored: &[Scored]) -> Level {
    let mut accs: HashMap<i32, LevelAcc> = HashMap::with_capacity(20);
    let mut total = 0;
    for item in scored {
        if item.unrecorded() || !item.counted() || item.acc() <= 0.0 {
            continue;
        }
        let slot = accs.entry(bucket_key(item.constant)).or_default();
        slot.sum += item.acc();
        if item.acc() > slot.best {
            slot.best = item.acc();
            slot.best_at = item.constant;
        }
        slot.count += 1.0;
        total += 1;
    }
    let need = level_min_samples(total);
    let top = accs
        .keys()
        .map(|key| f64::from(*key) * LEVEL_BUCKET_WIDTH)
        .fold(0.0, f64::max);
    let multi = accs.len() >= 2;
    let mut buckets: Vec<LevelBucket> = Vec::with_capacity(accs.len());
    for (key, slot) in &accs {
        let constant = f64::from(*key) * LEVEL_BUCKET_WIDTH;
        if slot.count < f64::from(level_bucket_need(constant, top, need, multi)) {
            continue;
        }
        buckets.push(LevelBucket {
            constant,
            acc: slot.sum / slot.count,
            best: slot.best,
            best_at: slot.best_at,
            n: slot.count as i32,
        });
    }
    buckets.sort_by(|left, right| {
        left.constant
            .partial_cmp(&right.constant)
            .unwrap_or(Ordering::Equal)
    });
    Level {
        buckets: monotone(buckets),
        curv: 0.0,
    }
}

fn bucket_key(constant: f64) -> i32 {
    (constant / LEVEL_BUCKET_WIDTH).round() as i32
}

fn monotone(buckets: Vec<LevelBucket>) -> Vec<LevelBucket> {
    let mut blocks: Vec<LevelBucket> = Vec::with_capacity(buckets.len());
    for bucket in buckets {
        blocks.push(bucket);
        while blocks.len() >= 2 {
            let last = blocks.len() - 1;
            let (a, c) = (blocks[last - 1], blocks[last]);
            if a.acc >= c.acc {
                break;
            }
            let n = a.n + c.n;
            let mut merged = LevelBucket {
                n,
                ..LevelBucket::default()
            };
            if n > 0 {
                let wa = f64::from(a.n) / f64::from(n);
                let wc = f64::from(c.n) / f64::from(n);
                merged.constant = a.constant * wa + c.constant * wc;
                merged.acc = a.acc * wa + c.acc * wc;
                if c.best > a.best {
                    merged.best = c.best;
                    merged.best_at = c.best_at;
                } else {
                    merged.best = a.best;
                    merged.best_at = a.best_at;
                }
            }
            blocks.truncate(last - 1);
            blocks.push(merged);
        }
    }
    blocks
}

fn level_min_samples(total: i32) -> i32 {
    (total / 20).clamp(LEVEL_MIN_SAMPLES_FLOOR, LEVEL_MIN_SAMPLES_CAP)
}

fn level_bucket_need(constant: f64, top: f64, base: i32, multi: bool) -> i32 {
    if multi && constant >= top - LEVEL_BUCKET_EDGE {
        1
    } else {
        base
    }
}

impl Level {
    pub fn with_curvature(mut self, curv: f64) -> Level {
        if curv > 0.0 {
            self.curv = curv;
        }
        self
    }

    pub fn acc_at(&self, constant: f64) -> Option<f64> {
        let lo = *self.buckets.first()?;
        let hi = *self.buckets.last()?;
        if constant <= 0.0 {
            return None;
        }
        if constant < lo.constant {
            if lo.constant - constant > LEVEL_REACH_MAX {
                return None;
            }
            return Some((lo.acc + self.slope_up() * (lo.constant - constant)).min(100.0));
        }
        if constant > hi.constant {
            if constant - hi.constant > LEVEL_REACH_MAX {
                return None;
            }
            let d = constant - hi.constant;
            let acc = hi.acc - self.slope_down() * d - self.curv * d * d / 2.0;
            return (acc >= LEVEL_ACC_FLOOR).then_some(acc);
        }
        for pair in self.buckets.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if constant > b.constant {
                continue;
            }
            if b.constant == a.constant {
                return Some(b.acc);
            }
            let t = (constant - a.constant) / (b.constant - a.constant);
            return Some(a.acc + (b.acc - a.acc) * t);
        }
        Some(hi.acc)
    }

    pub fn best_at(&self, constant: f64) -> Option<f64> {
        let lo = *self.buckets.first()?;
        let hi = *self.buckets.last()?;
        if constant <= 0.0 {
            return None;
        }
        let half = LEVEL_BUCKET_WIDTH / 2.0;
        if constant >= hi.constant {
            if constant > hi.constant + half {
                return None;
            }
            if hi.best_at > 0.0 && constant - hi.best_at > LEVEL_BEST_SLACK {
                return None;
            }
            return Some(hi.best);
        }
        if constant <= lo.constant {
            return Some(lo.best);
        }
        for pair in self.buckets.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if constant > b.constant {
                continue;
            }
            if b.constant == a.constant {
                return Some(b.best);
            }
            let t = (constant - a.constant) / (b.constant - a.constant);
            return Some(a.best + (b.best - a.best) * t);
        }
        Some(hi.best)
    }

    fn slope_down(&self) -> f64 {
        let n = self.buckets.len();
        if n < 2 {
            return LEVEL_SLOPE_FLOOR;
        }
        let (a, b) = (self.buckets[n - 2], self.buckets[n - 1]);
        if b.constant <= a.constant {
            return LEVEL_SLOPE_FLOOR;
        }
        ((a.acc - b.acc) / (b.constant - a.constant)).max(LEVEL_SLOPE_FLOOR)
    }

    fn slope_up(&self) -> f64 {
        if self.buckets.len() < 2 {
            return LEVEL_SLOPE_FLOOR;
        }
        let (a, b) = (self.buckets[0], self.buckets[1]);
        if b.constant <= a.constant {
            return LEVEL_SLOPE_FLOOR;
        }
        ((a.acc - b.acc) / (b.constant - a.constant)).max(LEVEL_SLOPE_FLOOR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Difficulty, PlayRecord};

    fn scored(acc: f64, constant: f64) -> Scored {
        Scored {
            record: PlayRecord {
                song_id: String::from("s"),
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

    fn bucket(constant: f64, acc: f64, n: i32) -> LevelBucket {
        LevelBucket {
            constant,
            acc,
            best: acc,
            best_at: constant,
            n,
        }
    }

    #[test]
    fn monotone_merges_the_inversion_weighted_by_samples() {
        let merged = monotone(vec![
            bucket(6.0, 88.0, 40),
            bucket(7.0, 82.0, 10),
            bucket(8.0, 84.0, 10),
        ]);
        assert_eq!(merged.len(), 2);
        assert!((merged[0].acc - 88.0).abs() < 1e-9);
        assert!((merged[1].constant - 7.5).abs() < 1e-9);
        assert!((merged[1].acc - 83.0).abs() < 1e-9);
        assert_eq!(merged[1].n, 20);
        assert!((merged[1].best - 84.0).abs() < 1e-9);
        assert!((merged[1].best_at - 8.0).abs() < 1e-9);
        for pair in merged.windows(2) {
            assert!(pair[0].acc >= pair[1].acc);
        }
    }

    #[test]
    fn level_of_keeps_counted_records_only() {
        let mut suspect = scored(99.0, 9.0);
        suspect.suspect = true;
        let level = level_of(&[
            scored(88.0, 6.0),
            scored(84.0, 6.0),
            scored(80.0, 6.0),
            scored(70.0, 7.0),
            suspect,
        ]);
        let high = level.best_at(6.0).expect("最低那一桶有成绩");
        assert!((high - 88.0).abs() < 1e-9);
        assert!(level.best_at(9.0).is_none());
    }

    #[test]
    fn acc_at_interpolates_inside_and_extrapolates_outside() {
        let level = Level {
            buckets: vec![bucket(6.0, 88.0, 10), bucket(7.0, 80.0, 10)],
            curv: 0.0,
        };
        let inside = level.acc_at(6.5).expect("桶之间");
        assert!((inside - 84.0).abs() < 1e-9);
        let easier = level.acc_at(5.0).expect("更容易的一侧");
        assert!((easier - 96.0).abs() < 1e-9);
        let harder = level.acc_at(8.0).expect("更难的一侧");
        assert!((harder - 72.0).abs() < 1e-9);
        let curved = Level {
            curv: 2.0,
            buckets: level.buckets.clone(),
        };
        let bent = curved.acc_at(8.0).expect("挂上曲率");
        assert!((bent - 71.0).abs() < 1e-9);
        assert!(level.acc_at(9.6).is_none());
        assert!(level.acc_at(3.4).is_none());
        assert!(Level::default().acc_at(6.0).is_none());
        let steep = Level {
            buckets: vec![bucket(6.0, 90.0, 10), bucket(7.0, 70.0, 10)],
            curv: 0.0,
        };
        assert!(steep.acc_at(8.0).is_none());
    }

    #[test]
    fn with_curvature_only_takes_a_positive_value() {
        let level = Level {
            buckets: vec![bucket(6.0, 88.0, 10)],
            curv: 0.0,
        };
        assert_eq!(level.clone().with_curvature(0.0).curv, 0.0);
        assert_eq!(level.with_curvature(1.5).curv, 1.5);
    }
}
