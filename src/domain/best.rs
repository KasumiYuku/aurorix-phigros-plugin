use crate::domain::rks::{board_count, just_one_good, overflow_shown, rks, round_up, suggest};
use crate::model::{Best, Scored, BEST_SIZE, MAX_SCORE};

pub fn board(scored: &[Scored], official: f64, size: usize) -> Best {
    let size = if size == 0 { BEST_SIZE } else { size };
    let mut played: Vec<Scored> = Vec::with_capacity(scored.len());
    let mut unknown = 0;
    let mut suspect = 0;
    let mut phi: Vec<Scored> = Vec::new();
    let mut fc: Vec<Scored> = Vec::new();
    let mut one_good: Vec<Scored> = Vec::new();
    for item in scored {
        if item.unrecorded() {
            continue;
        }
        if !item.counted() {
            if !item.known {
                unknown += 1;
            } else if item.suspect {
                suspect += 1;
            }
            continue;
        }
        if item.record.score >= MAX_SCORE {
            phi.push(item.clone());
        } else if item.record.fc {
            fc.push(item.clone());
        }
        if just_one_good(item.record.score, item.notes) {
            one_good.push(item.clone());
        }
        played.push(item.clone());
    }
    sort_by_rks(&mut played);
    sort_by_rks(&mut phi);
    sort_by_rks(&mut fc);
    sort_by_rks(&mut one_good);

    let podium: Vec<Scored> = phi.iter().take(crate::model::COUNTED_PHI).cloned().collect();
    let count = board_count(size) + overflow_shown(size);
    let mut top: Vec<Scored> = played.iter().take(count).cloned().collect();

    let mut best = Best {
        official,
        computed: rks(&played),
        podium,
        top: Vec::new(),
        phi,
        fc,
        one_good,
        records: played.len(),
        unknown,
        suspect,
        avg_all: 0.0,
        round_up: round_up(official),
    };
    if !played.is_empty() {
        let sum: f64 = played.iter().map(|item| item.rks).sum();
        best.avg_all = sum / played.len() as f64;
    }
    suggest(&mut top, best.round_up);
    best.top = top;
    best
}

fn sort_by_rks(items: &mut [Scored]) {
    items.sort_by(|a, b| {
        b.rks
            .partial_cmp(&a.rks)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.record.score.cmp(&a.record.score))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rks::single_rks;
    use crate::model::{Difficulty, PlayRecord};

    fn played(song: &str, constant: f64, score: i32, fc: bool) -> Scored {
        let acc = if score >= MAX_SCORE { 100.0 } else { 98.5 };
        Scored {
            record: PlayRecord {
                song_id: song.to_string(),
                difficulty: Difficulty::In,
                score,
                acc,
                fc,
            },
            constant,
            known: true,
            notes: 1000,
            rks: single_rks(acc, constant),
            ..Scored::default()
        }
    }

    #[test]
    fn podium_splits_phi_and_top_caps_at_board_size() {
        let mut records: Vec<Scored> = (0..40)
            .map(|index| played(&format!("s{index}"), 10.0 + index as f64 * 0.1, 990_000, false))
            .collect();
        for index in 0..5 {
            records.push(played(&format!("p{index}"), 14.0 + index as f64, MAX_SCORE, true));
        }
        let best = board(&records, 15.0, 30);
        assert_eq!(best.podium.len(), 3);
        assert_eq!(best.podium[0].record.song_id, "p4");
        assert_eq!(best.top.len(), 33);
        assert_eq!(best.phi.len(), 5);
        assert_eq!(best.records, 45);
        assert_eq!(best.unknown, 0);
        assert!(best.computed > 0.0);
        assert!(best.round_up >= 0.0);
    }

    #[test]
    fn unknown_and_suspect_are_counted_separately() {
        let mut unknown = played("u", 12.0, 950_000, false);
        unknown.known = false;
        let mut suspect = played("x", 12.0, MAX_SCORE, true);
        suspect.suspect = true;
        let best = board(&[unknown, suspect], 12.0, 30);
        assert_eq!(best.unknown, 1);
        assert_eq!(best.suspect, 1);
        assert_eq!(best.records, 0);
        assert!(best.top.is_empty());
    }

    #[test]
    fn board50_has_no_overflow_slack() {
        let records: Vec<Scored> = (0..60)
            .map(|index| played(&format!("s{index}"), 10.0 + index as f64 * 0.1, 990_000, false))
            .collect();
        let best = board(&records, 14.0, 50);
        assert_eq!(best.top.len(), 47);
    }
}
