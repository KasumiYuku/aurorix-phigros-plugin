use crate::model::{Scored, BEST_SIZE, COUNTED_BEST, COUNTED_PHI, MAX_SCORE};

pub fn single_rks(acc: f64, constant: f64) -> f64 {
    if (acc - 100.0).abs() <= f64::EPSILON {
        return constant;
    }
    if acc < 70.0 {
        return 0.0;
    }
    let k = (acc - 55.0) / 45.0;
    constant * k * k
}

pub fn rks(records: &[Scored]) -> f64 {
    let mut ranked: Vec<&Scored> = records.iter().filter(|item| !item.suspect).collect();
    ranked.sort_by(|a, b| b.rks.partial_cmp(&a.rks).unwrap_or(std::cmp::Ordering::Equal));
    let mut sum = 0.0;
    for item in ranked.iter().take(COUNTED_BEST) {
        sum += item.rks;
    }
    let mut bonus = 0;
    for item in &ranked {
        if item.record.score < MAX_SCORE {
            continue;
        }
        sum += item.rks;
        bonus += 1;
        if bonus >= COUNTED_PHI {
            break;
        }
    }
    sum / BEST_SIZE as f64
}

pub fn display_step(value: f64) -> f64 {
    let mut step = (value * 100.0).floor() / 100.0 + 0.005 - value;
    if step < 0.0 {
        step += 0.01;
    }
    step
}

pub fn required_acc(target: f64, constant: f64) -> Option<f64> {
    if constant <= 0.0 || target <= 0.0 {
        return None;
    }
    let acc = 45.0 * (target / constant).sqrt() + 55.0;
    if acc >= 100.0 {
        return None;
    }
    Some(acc)
}

pub fn board_count(size: usize) -> usize {
    let count = size.saturating_sub(COUNTED_PHI);
    count.max(1)
}

pub fn overflow_shown(size: usize) -> usize {
    if size > BEST_SIZE {
        0
    } else {
        6
    }
}

pub fn one_good_score(notes: i32) -> i32 {
    if notes < 2 {
        return 0;
    }
    let notes = i64::from(notes);
    let den = 20 * notes;
    let weighted = 20 * (notes - 1) + 13;
    let num = 900_000 * weighted + 2_000_000 * notes;
    ((num + den / 2) / den) as i32
}

pub fn just_one_good(score: i32, notes: i32) -> bool {
    let target = one_good_score(notes);
    if target <= 0 || score <= 0 {
        return false;
    }
    (score - target).abs() <= 2
}

pub fn round_up(official: f64) -> f64 {
    let mut delta = ((official * 100.0) as i64) as f64 / 100.0 + 0.005 - official;
    if delta < 0.0 {
        delta += 0.01;
    }
    delta
}

pub fn suggest(top: &mut [Scored], round: f64) {
    for index in 0..top.len() {
        let pivot = top[index.min(COUNTED_BEST - 1)].rks;
        let item = &mut top[index];
        if !item.known || item.record.score >= MAX_SCORE {
            continue;
        }
        let target = pivot + round * BEST_SIZE as f64;
        match required_acc(target, item.constant) {
            Some(acc) => {
                item.suggest = acc;
                item.suggest_ok = true;
            }
            None => {
                item.suggest = 0.0;
                item.suggest_ok = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Difficulty, PlayRecord};

    fn scored(acc: f64, constant: f64, score: i32) -> Scored {
        Scored {
            record: PlayRecord {
                song_id: String::from("s"),
                difficulty: Difficulty::In,
                score,
                acc,
                fc: false,
            },
            constant,
            known: true,
            rks: single_rks(acc, constant),
            ..Scored::default()
        }
    }

    #[test]
    fn single_rks_matches_go() {
        assert_eq!(single_rks(100.0, 16.5), 16.5);
        assert_eq!(single_rks(69.99, 16.5), 0.0);
        let value = single_rks(98.0, 16.0);
        let expect = 16.0 * ((98.0 - 55.0) / 45.0_f64).powi(2);
        assert!((value - expect).abs() < 1e-12);
        assert!((single_rks(96.0, 14.0) - 14.0 * (41.0_f64 / 45.0).powi(2)).abs() < 1e-12);
    }

    #[test]
    fn rks_sums_top_27_plus_3_phi_over_30() {
        let mut records: Vec<Scored> = (0..40)
            .map(|index| scored(98.0, 1.0 + index as f64 * 0.1, 990_000))
            .collect();
        records.push(scored(100.0, 16.0, MAX_SCORE));
        let value = rks(&records);
        let mut rks_all: Vec<f64> = records.iter().map(|item| item.rks).collect();
        rks_all.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let expect: f64 = rks_all.iter().take(27).sum::<f64>() + 16.0;
        assert!((value - expect / 30.0).abs() < 1e-12);
    }

    #[test]
    fn board_counts_match_go() {
        assert_eq!(board_count(30), 27);
        assert_eq!(board_count(50), 47);
        assert_eq!(overflow_shown(30), 6);
        assert_eq!(overflow_shown(50), 0);
    }

    #[test]
    fn one_good_threshold_is_round_half_up() {
        assert_eq!(one_good_score(1), 0);
        assert_eq!(one_good_score(1000), 999_685);
        assert!(just_one_good(999_685, 1000));
        assert!(just_one_good(999_687, 1000));
        assert!(!just_one_good(999_690, 1000));
    }

    #[test]
    fn display_step_rounds_up_to_next_hundredth() {
        let step = display_step(12.3456);
        assert!((step - 0.0094).abs() < 1e-9);
        assert!((display_step(12.34) - 0.005).abs() < 1e-9);
    }

    #[test]
    fn required_acc_inverts_single_rks() {
        let acc = required_acc(single_rks(97.0, 15.0), 15.0).unwrap();
        assert!((acc - 97.0).abs() < 1e-9);
        assert!(required_acc(99.0, 1.0).is_none());
    }
}
