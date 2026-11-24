use std::collections::HashMap;

use crate::model::{
    BandStat, ChartStat, Difficulty, LevelStat, ServerStats, BAND_MAX, PEER_THRESHOLD_COUNT,
    STAT_MIN_USERS,
};

pub const PEER_MIN_USERS: i32 = 5;

pub const PEER_MAX_SPAN: i32 = 2;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Peer {
    pub users: i32,
    pub median: f64,
    pub p75: f64,
    pub hits: [i32; PEER_THRESHOLD_COUNT],
}

impl Peer {
    pub fn available(&self) -> bool {
        self.users >= PEER_MIN_USERS && self.p75 > 0.0
    }

    pub fn rate(&self, acc: f64) -> f64 {
        ChartStat {
            users: self.users,
            hits: self.hits,
            ..ChartStat::default()
        }
        .rate(acc)
    }

    fn add(&mut self, item: &ChartStat) {
        let weight = f64::from(item.users) / f64::from(self.users + item.users);
        self.median += (item.median - self.median) * weight;
        self.p75 += (item.p75 - self.p75) * weight;
        for index in 0..PEER_THRESHOLD_COUNT {
            self.hits[index] += item.hits[index];
        }
        self.users += item.users;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Prior {
    pub curvature: f64,
    peers: HashMap<String, Peer>,
}

impl Prior {
    pub fn peer(&self, song_id: &str, difficulty: Difficulty) -> Option<&Peer> {
        self.peers
            .get(&chart_key(song_id, difficulty))
            .filter(|peer| peer.available())
    }
}

pub fn chart_key(song_id: &str, difficulty: Difficulty) -> String {
    format!("{song_id}#{}", difficulty.index())
}

pub fn widen_bands(bands: &[BandStat], band: i32, max_span: i32) -> (i32, i32, i32) {
    let mut users: HashMap<i32, i32> = HashMap::with_capacity(bands.len());
    for item in bands {
        users.insert(item.band, item.users);
    }
    let at = |band: i32| users.get(&band).copied().unwrap_or(0);
    let mut lo = band;
    let mut hi = band;
    let mut total = at(band);
    while total < STAT_MIN_USERS && (lo > 0 || hi < BAND_MAX) {
        let left = (lo > 0 && lo > band - max_span).then(|| at(lo - 1));
        let right = (hi < BAND_MAX && hi < band + max_span).then(|| at(hi + 1));
        match (left, right) {
            (None, None) => return (lo, hi, total),
            (Some(left), None) => {
                lo -= 1;
                total += left;
            }
            (None, Some(right)) => {
                hi += 1;
                total += right;
            }
            (Some(left), Some(right)) => {
                if left <= right {
                    lo -= 1;
                    total += left;
                } else {
                    hi += 1;
                    total += right;
                }
            }
        }
    }
    (lo, hi, total)
}

pub fn prior_from(stats: &ServerStats, band: i32) -> Prior {
    let mut out = Prior {
        curvature: curvature_of(&stats.levels),
        peers: HashMap::new(),
    };
    if band < 0 || stats.charts.is_empty() {
        return out;
    }
    let (lo, hi, total) = widen_bands(&stats.bands, band, PEER_MAX_SPAN);
    if total < STAT_MIN_USERS {
        return out;
    }
    out.peers = HashMap::with_capacity(stats.charts.len());
    for item in &stats.charts {
        if item.band < lo || item.band > hi {
            continue;
        }
        out.peers
            .entry(chart_key(&item.song_id, item.difficulty))
            .or_default()
            .add(item);
    }
    out
}

pub fn curvature_of(levels: &[LevelStat]) -> f64 {
    if levels.len() < 4 {
        return 0.0;
    }
    let tail = &levels[levels.len() - 4..];
    for pair in tail.windows(2) {
        if pair[1].acc > pair[0].acc {
            return 0.0;
        }
    }
    let first = slope_between(&tail[1], &tail[2]);
    let second = slope_between(&tail[2], &tail[3]);
    let span = (tail[3].constant - tail[1].constant) / 2.0;
    if span <= 0.0 {
        return 0.0;
    }
    let curv = (second - first) / span;
    if curv > 0.0 {
        curv
    } else {
        0.0
    }
}

fn slope_between(low: &LevelStat, high: &LevelStat) -> f64 {
    if high.constant <= low.constant {
        return 0.0;
    }
    (low.acc - high.acc) / (high.constant - low.constant)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(band: i32, users: i32) -> BandStat {
        BandStat { band, users }
    }

    fn chart(band: i32, users: i32, median: f64) -> ChartStat {
        ChartStat {
            song_id: String::from("song"),
            difficulty: Difficulty::In,
            band,
            users,
            median,
            p75: median + 1.0,
            hits: [users, users, users, users, users, users, users],
        }
    }

    #[test]
    fn widen_bands_expands_toward_the_thinner_side() {
        let bands = [stat(10, 10), stat(11, 12), stat(12, 9)];
        assert_eq!(widen_bands(&bands, 11, PEER_MAX_SPAN), (10, 13, 31));
        assert_eq!(widen_bands(&bands, 11, 0), (11, 11, 12));
    }

    #[test]
    fn widen_bands_stops_at_the_span_limit() {
        let bands = [stat(0, 1), stat(17, 1)];
        assert_eq!(widen_bands(&bands, 17, PEER_MAX_SPAN), (15, 17, 1));
    }

    #[test]
    fn prior_merges_the_same_chart_across_bands() {
        let stats = ServerStats {
            bands: vec![stat(15, 20), stat(16, 20)],
            charts: vec![chart(15, 20, 98.0), chart(16, 20, 96.0)],
            ..ServerStats::default()
        };
        let prior = prior_from(&stats, 15);
        let peer = prior.peer("song", Difficulty::In).unwrap();
        assert_eq!(peer.users, 40);
        assert!((peer.median - 97.0).abs() < 1e-9);
        assert!(peer.available());
        assert!((peer.rate(96.0) - 1.0).abs() < 1e-12);
        assert!(prior.peer("song", Difficulty::At).is_none());
    }

    #[test]
    fn prior_is_empty_without_a_band_or_enough_people() {
        let stats = ServerStats {
            bands: vec![stat(15, 4)],
            charts: vec![chart(15, 4, 98.0)],
            ..ServerStats::default()
        };
        assert!(prior_from(&stats, 15).peer("song", Difficulty::In).is_none());
        assert!(prior_from(&stats, -1).peer("song", Difficulty::In).is_none());
    }

    #[test]
    fn peer_needs_five_users_and_a_filled_p75() {
        let mut peer = Peer {
            users: 4,
            median: 99.0,
            p75: 99.5,
            hits: [1, 1, 1, 1, 1, 1, 1],
        };
        assert!(!peer.available());
        peer.users = 5;
        assert!(peer.available());
        peer.p75 = 0.0;
        assert!(!peer.available());
    }

    fn level(constant: f64, acc: f64) -> LevelStat {
        LevelStat {
            constant,
            records: 10,
            acc,
        }
    }

    #[test]
    fn curvature_of_reads_the_bend_of_the_top_buckets() {
        let levels = [
            level(6.0, 88.0),
            level(7.0, 84.0),
            level(8.0, 78.0),
            level(9.0, 70.0),
        ];
        assert!((curvature_of(&levels) - 2.0).abs() < 1e-9);
        let stats = ServerStats {
            levels: levels.to_vec(),
            ..ServerStats::default()
        };
        assert!((prior_from(&stats, -1).curvature - 2.0).abs() < 1e-9);
    }

    #[test]
    fn curvature_of_gives_up_without_a_clean_tail() {
        let short = [level(7.0, 84.0), level(8.0, 78.0), level(9.0, 70.0)];
        assert_eq!(curvature_of(&short), 0.0);
        let rebound = [
            level(6.0, 88.0),
            level(7.0, 84.0),
            level(8.0, 86.0),
            level(9.0, 70.0),
        ];
        assert_eq!(curvature_of(&rebound), 0.0);
        let flat = [
            level(6.0, 88.0),
            level(7.0, 80.0),
            level(8.0, 74.0),
            level(9.0, 69.0),
        ];
        assert_eq!(curvature_of(&flat), 0.0);
        assert_eq!(curvature_of(&[]), 0.0);
    }
}
