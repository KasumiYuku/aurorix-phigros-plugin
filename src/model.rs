pub const CLEAR_SCORE: i32 = 700_000;
pub const MAX_SCORE: i32 = 1_000_000;
pub const MAX_CONSTANT: f64 = 18.0;
pub const COUNTED_BEST: usize = 27;
pub const COUNTED_PHI: usize = 3;
pub const BEST_SIZE: usize = 30;
pub const BEST50_SIZE: usize = 50;
pub const GRADE_COUNT: usize = 9;
pub const NUM_DIFFICULTIES: usize = 6;
pub const NUM_PLAYABLE: usize = 4;
pub const NUM_MONEY: usize = 5;
pub const SUSPECT_ACC_EPSILON: f64 = 1e-6;
pub const RKS_TOLERANCE: f64 = 1e-4;
pub const CHALLENGE_SONG_COUNT: usize = 3;
pub const MAX_CHALLENGE_COLOR: u8 = 5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Difficulty {
    #[default]
    Ez = 0,
    Hd = 1,
    In = 2,
    At = 3,
    Legacy = 4,
    Sp = 5,
}

impl Difficulty {
    pub const ALL: [Difficulty; NUM_DIFFICULTIES] = [
        Difficulty::Ez,
        Difficulty::Hd,
        Difficulty::In,
        Difficulty::At,
        Difficulty::Legacy,
        Difficulty::Sp,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(value: usize) -> Option<Self> {
        Self::ALL.get(value).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Ez => "EZ",
            Difficulty::Hd => "HD",
            Difficulty::In => "IN",
            Difficulty::At => "AT",
            Difficulty::Legacy => "LEGACY",
            Difficulty::Sp => "SP",
        }
    }

    pub fn playable(self) -> bool {
        (self as usize) < NUM_PLAYABLE
    }

    pub fn valid(self) -> bool {
        (self as usize) < NUM_DIFFICULTIES
    }

    pub fn videoable(self) -> bool {
        self.valid()
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_lowercase().as_str() {
            "ez" | "easy" | "简单" | "绿" | "绿谱" => Some(Difficulty::Ez),
            "hd" | "hard" | "困难" | "蓝" | "蓝谱" => Some(Difficulty::Hd),
            "in" | "insane" | "黄" | "黄谱" => Some(Difficulty::In),
            "at" | "another" | "红" | "红谱" | "额外" => Some(Difficulty::At),
            "sp" | "特殊" | "愚人节" => Some(Difficulty::Sp),
            "lg" | "legacy" | "ly" | "旧" => Some(Difficulty::Legacy),
            _ => None,
        }
    }

    pub fn playable_all() -> &'static [Difficulty] {
        &Self::ALL[..NUM_PLAYABLE]
    }

    pub fn video_all() -> &'static [Difficulty] {
        &Self::ALL[..5]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Grade {
    #[default]
    New = 0,
    F = 1,
    C = 2,
    B = 3,
    A = 4,
    S = 5,
    V = 6,
    Fc = 7,
    Phi = 8,
}

impl Grade {
    pub const ALL: [Grade; GRADE_COUNT] = [
        Grade::New,
        Grade::F,
        Grade::C,
        Grade::B,
        Grade::A,
        Grade::S,
        Grade::V,
        Grade::Fc,
        Grade::Phi,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Grade::New => "NEW",
            Grade::F => "F",
            Grade::C => "C",
            Grade::B => "B",
            Grade::A => "A",
            Grade::S => "S",
            Grade::V => "V",
            Grade::Fc => "FC",
            Grade::Phi => "PHI",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

pub fn grade_of(score: i32, fc: bool) -> Grade {
    if score >= MAX_SCORE {
        Grade::Phi
    } else if fc {
        Grade::Fc
    } else if score <= 0 {
        Grade::New
    } else if score < CLEAR_SCORE {
        Grade::F
    } else if score < 820_000 {
        Grade::C
    } else if score < 880_000 {
        Grade::B
    } else if score < 920_000 {
        Grade::A
    } else if score < 960_000 {
        Grade::S
    } else {
        Grade::V
    }
}

pub fn next_grade(score: i32) -> (Grade, i32) {
    const CUTS: [(i32, Grade); 6] = [
        (CLEAR_SCORE, Grade::C),
        (820_000, Grade::B),
        (880_000, Grade::A),
        (920_000, Grade::S),
        (960_000, Grade::V),
        (MAX_SCORE, Grade::Phi),
    ];
    for (cut, grade) in CUTS {
        if score < cut {
            return (grade, cut - score);
        }
    }
    (Grade::Phi, 0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Notes {
    pub tap: i32,
    pub drag: i32,
    pub hold: i32,
    pub flick: i32,
}

impl Notes {
    pub fn combo(self) -> i32 {
        self.tap + self.drag + self.hold + self.flick
    }

    pub fn empty(self) -> bool {
        self.combo() == 0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConstChange {
    pub version: i32,
    pub version_label: String,
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chart {
    pub difficulty: Difficulty,
    pub constant: f64,
    pub charter: String,
    pub notes: Notes,
    pub duration: f64,
    pub history: Vec<ConstChange>,
}

impl Chart {
    pub fn known(&self) -> bool {
        self.constant > 0.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtraChart {
    pub kind: String,
    pub label: String,
    pub charter: String,
    pub notes: Notes,
}

impl ExtraChart {
    pub fn display(&self) -> String {
        match self.kind.to_lowercase().as_str() {
            "ez" | "hd" | "in" | "ln" | "at" => String::from("SP"),
            _ => self.kind.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub composer: String,
    pub illustrator: String,
    pub cover_luma: i32,
    pub cover_colors: Vec<String>,
    pub bpm: i32,
    pub bpm_text: String,
    pub length: String,
    pub chapter: String,
    pub version: i32,
    pub version_label: String,
    pub original: bool,
    pub sp: String,
    pub aliases: Vec<String>,
    pub charts: [Chart; NUM_DIFFICULTIES],
    pub extra: Vec<ExtraChart>,
    pub variant_of: String,
    pub audio_url: String,
    pub ill_url: String,
    pub wiki_url: String,
}

impl Song {
    pub fn variant(&self) -> bool {
        !self.variant_of.is_empty()
    }

    pub fn chart(&self, difficulty: Difficulty) -> Option<&Chart> {
        if !difficulty.valid() {
            return None;
        }
        let chart = &self.charts[difficulty.index()];
        chart.known().then_some(chart)
    }

    pub fn constant(&self, difficulty: Difficulty) -> Option<f64> {
        self.chart(difficulty).map(|chart| chart.constant)
    }

    pub fn difficulties(&self) -> Vec<Difficulty> {
        Difficulty::playable_all()
            .iter()
            .copied()
            .filter(|difficulty| self.chart(*difficulty).is_some())
            .collect()
    }

    pub fn all_difficulties(&self) -> Vec<Difficulty> {
        let mut result = self.difficulties();
        if self.chart(Difficulty::Legacy).is_some() {
            result.push(Difficulty::Legacy);
        }
        result
    }

    pub fn max_constant(&self) -> f64 {
        Difficulty::playable_all()
            .iter()
            .filter_map(|difficulty| self.constant(*difficulty))
            .fold(0.0_f64, f64::max)
    }

    pub fn min_constant(&self) -> f64 {
        Difficulty::playable_all()
            .iter()
            .filter_map(|difficulty| self.constant(*difficulty))
            .fold(0.0_f64, |best, value| {
                if best == 0.0 || value < best {
                    value
                } else {
                    best
                }
            })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chapter {
    pub name: String,
    pub display: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayRecord {
    pub song_id: String,
    pub difficulty: Difficulty,
    pub score: i32,
    pub acc: f64,
    pub fc: bool,
}

impl PlayRecord {
    pub fn is_suspect(&self) -> bool {
        self.score >= MAX_SCORE && self.acc < 100.0 - SUSPECT_ACC_EPSILON
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scored {
    pub record: PlayRecord,
    pub title: String,
    pub composer: String,
    pub constant: f64,
    pub known: bool,
    pub notes: i32,
    pub rks: f64,
    pub grade: Grade,
    pub rank: usize,
    pub suggest: f64,
    pub suggest_ok: bool,
    pub suspect: bool,
    pub variant: bool,
    pub cover_luma: i32,
}

impl Scored {
    pub fn song_id(&self) -> &str {
        &self.record.song_id
    }

    pub fn difficulty(&self) -> Difficulty {
        self.record.difficulty
    }

    pub fn score(&self) -> i32 {
        self.record.score
    }

    pub fn acc(&self) -> f64 {
        self.record.acc
    }

    pub fn fc(&self) -> bool {
        self.record.fc
    }

    pub fn unrecorded(&self) -> bool {
        self.record.score <= 0
    }

    pub fn counted(&self) -> bool {
        self.record.difficulty.playable() && self.known && !self.variant && !self.suspect
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Best {
    pub official: f64,
    pub computed: f64,
    pub podium: Vec<Scored>,
    pub top: Vec<Scored>,
    pub phi: Vec<Scored>,
    pub fc: Vec<Scored>,
    pub one_good: Vec<Scored>,
    pub records: usize,
    pub unknown: usize,
    pub suspect: usize,
    pub avg_all: f64,
    pub round_up: f64,
}

impl Best {
    pub fn overflow(&self) -> usize {
        COUNTED_BEST
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DifficultyStat {
    pub played: usize,
    pub total: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Progress {
    pub is_first_run: bool,
    pub chapter_finished: bool,
    pub flags: u8,
    pub completed: String,
    pub song_update_info: i32,
    pub challenge_mode_rank: i32,
    pub money: [i32; 5],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameUser {
    pub show_player_id: bool,
    pub self_intro: String,
    pub avatar: String,
    pub background: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Summary {
    pub save_version: i32,
    pub challenge_mode_rank: i32,
    pub ranking_score: f64,
    pub game_version: i32,
    pub avatar: String,
    pub cleared: [i32; NUM_PLAYABLE],
    pub full_combo: [i32; NUM_PLAYABLE],
    pub phi: [i32; NUM_PLAYABLE],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Archive {
    pub progress: Progress,
    pub user: GameUser,
    pub records: Vec<PlayRecord>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Server {
    #[default]
    Cn,
    Global,
}

impl Server {
    pub fn code(self) -> i64 {
        match self {
            Server::Cn => 0,
            Server::Global => 1,
        }
    }

    pub fn from_code(code: i64) -> Self {
        if code == 1 {
            Server::Global
        } else {
            Server::Cn
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Binding {
    pub user_id: String,
    pub token: String,
    pub server: Server,
    pub object_id: String,
    pub nickname: String,
    pub avatar: String,
    pub game_avatar: String,
    pub save_modified_at: i64,
    pub bound_at: i64,
    pub last_sync_at: i64,
    pub status: i64,
    pub error: String,
    pub joined_at: i64,
}

impl Binding {
    pub fn avatar_key(&self) -> String {
        self.game_avatar.clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Profile {
    pub user_id: String,
    pub rks: f64,
    pub challenge_mode_rank: i32,
    pub self_intro: String,
    pub background: String,
    pub game_version: i32,
    pub save_version: i32,
    pub updated_at: i64,
    pub counters: Counters,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Counters {
    pub cleared: [i32; NUM_PLAYABLE],
    pub full_combo: [i32; NUM_PLAYABLE],
    pub phi: [i32; NUM_PLAYABLE],
    pub money: [i32; NUM_MONEY],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerCard {
    pub has_player: bool,
    pub name: String,
    pub avatar: String,
    pub rank: i32,
    pub rank_level: i32,
    pub rks_milli: i64,
}

impl PlayerCard {
    pub fn rks(&self) -> f64 {
        self.rks_milli as f64 / 1000.0
    }

    pub fn with_rks(mut self, value: f64) -> Self {
        self.rks_milli = (value * 1000.0).round() as i64;
        self
    }

    pub fn demo() -> Self {
        Self {
            has_player: true,
            name: String::new(),
            avatar: String::new(),
            rank: 99,
            rank_level: 5,
            rks_milli: 17_420,
        }
    }
}

impl Profile {
    pub fn dan(&self) -> (i32, i32) {
        if self.challenge_mode_rank <= 0 {
            return (0, 0);
        }
        let level = (self.challenge_mode_rank / 100).clamp(0, 5);
        let rank = (self.challenge_mode_rank % 100).clamp(0, 99);
        (level, rank)
    }

    pub fn money_text(&self) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(NUM_MONEY);
        for index in (0..NUM_MONEY).rev() {
            let value = self.counters.money[index];
            if value != 0 {
                parts.push(format!("{value} {}", MONEY_UNITS[index]));
            }
        }
        if parts.is_empty() {
            return String::from("0 KiB");
        }
        parts.join(" ")
    }
}

pub const MONEY_UNITS: [&str; NUM_MONEY] = ["KiB", "MiB", "GiB", "TiB", "PiB"];

pub fn player_card_of(binding: &Binding, profile: &Profile, has_profile: bool) -> PlayerCard {
    if !has_profile {
        return PlayerCard::default();
    }
    let (rank_level, rank) = profile.dan();
    PlayerCard {
        has_player: true,
        name: binding.nickname.trim().to_string(),
        avatar: binding.avatar_key(),
        rank,
        rank_level,
        rks_milli: (profile.rks * 1000.0).round() as i64,
    }
}

pub const BAND_WIDTH: f64 = 1.0;
pub const BAND_MAX: i32 = 17;
pub const STAT_MIN_USERS: i32 = 30;
pub const PEER_THRESHOLD_COUNT: usize = 7;
pub const PEER_THRESHOLDS: [f64; PEER_THRESHOLD_COUNT] =
    [100.0, 99.5, 99.0, 98.5, 98.0, 97.0, 96.0];

pub fn band_of(rks: f64) -> i32 {
    if rks <= 0.0 {
        return -1;
    }
    ((rks / BAND_WIDTH) as i32).min(BAND_MAX)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LevelStat {
    pub constant: f64,
    pub records: i64,
    pub acc: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BandStat {
    pub band: i32,
    pub users: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartStat {
    pub song_id: String,
    pub difficulty: Difficulty,
    pub band: i32,
    pub users: i32,
    pub median: f64,
    pub p75: f64,
    pub hits: [i32; PEER_THRESHOLD_COUNT],
}

impl ChartStat {
    pub fn rate(&self, acc: f64) -> f64 {
        if self.users <= 0 {
            return 0.0;
        }
        let users = f64::from(self.users);
        let last = PEER_THRESHOLD_COUNT - 1;
        match PEER_THRESHOLDS.iter().position(|threshold| acc >= *threshold) {
            Some(0) => f64::from(self.hits[0]) / users,
            Some(index) => {
                let high = PEER_THRESHOLDS[index - 1];
                let low = PEER_THRESHOLDS[index];
                let frac = (high - acc) / (high - low);
                let hits = f64::from(self.hits[index - 1])
                    - frac * f64::from(self.hits[index - 1] - self.hits[index]);
                hits / users
            }
            None => f64::from(self.hits[last]) / users,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServerStats {
    pub levels: Vec<LevelStat>,
    pub bands: Vec<BandStat>,
    pub charts: Vec<ChartStat>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_order_is_archive_contract() {
        assert_eq!(Difficulty::Ez.index(), 0);
        assert_eq!(Difficulty::At.index(), 3);
        assert_eq!(Difficulty::Legacy.index(), 4);
        assert_eq!(Difficulty::Sp.index(), 5);
        assert!(Difficulty::Ez.playable());
        assert!(!Difficulty::Legacy.playable());
        assert!(!Difficulty::Sp.playable());
    }

    #[test]
    fn difficulty_aliases_match_go_table() {
        assert_eq!(Difficulty::parse(" 绿谱 "), Some(Difficulty::Ez));
        assert_eq!(Difficulty::parse("红谱"), Some(Difficulty::At));
        assert_eq!(Difficulty::parse("ly"), Some(Difficulty::Legacy));
        assert_eq!(Difficulty::parse("愚人节"), Some(Difficulty::Sp));
        assert_eq!(Difficulty::parse("zz"), None);
    }

    #[test]
    fn grade_thresholds_match_go() {
        assert_eq!(grade_of(1_000_000, false), Grade::Phi);
        assert_eq!(grade_of(999_999, true), Grade::Fc);
        assert_eq!(grade_of(0, false), Grade::New);
        assert_eq!(grade_of(699_999, false), Grade::F);
        assert_eq!(grade_of(700_000, false), Grade::C);
        assert_eq!(grade_of(819_999, false), Grade::C);
        assert_eq!(grade_of(820_000, false), Grade::B);
        assert_eq!(grade_of(880_000, false), Grade::A);
        assert_eq!(grade_of(920_000, false), Grade::S);
        assert_eq!(grade_of(960_000, false), Grade::V);
        assert_eq!(next_grade(950_000), (Grade::V, 10_000));
        assert_eq!(next_grade(1_000_000), (Grade::Phi, 0));
    }

    #[test]
    fn suspect_only_when_score_full_and_acc_short() {
        let mut record = PlayRecord {
            score: MAX_SCORE,
            acc: 99.5,
            ..PlayRecord::default()
        };
        assert!(record.is_suspect());
        record.acc = 100.0;
        assert!(!record.is_suspect());
        record.score = 999_999;
        record.acc = 99.0;
        assert!(!record.is_suspect());
    }

    #[test]
    fn player_card_keeps_milli_precision() {
        let card = PlayerCard::demo();
        assert_eq!(card.rks(), 17.42);
        assert_eq!(card.with_rks(13.4567).rks(), 13.457);
    }

    #[test]
    fn band_of_clamps_both_ends() {
        assert_eq!(band_of(0.0), -1);
        assert_eq!(band_of(-1.5), -1);
        assert_eq!(band_of(0.4), 0);
        assert_eq!(band_of(15.99), 15);
        assert_eq!(band_of(16.0), 16);
        assert_eq!(band_of(17.9), 17);
        assert_eq!(band_of(18.4), 17);
    }

    #[test]
    fn chart_stat_rate_interpolates_between_thresholds() {
        let stat = ChartStat {
            users: 10,
            hits: [2, 3, 4, 5, 6, 7, 8],
            ..ChartStat::default()
        };
        assert!((stat.rate(100.0) - 0.2).abs() < 1e-12);
        assert!((stat.rate(99.75) - 0.25).abs() < 1e-12);
        assert!((stat.rate(96.0) - 0.8).abs() < 1e-12);
        assert!((stat.rate(95.0) - 0.8).abs() < 1e-12);
        assert_eq!(ChartStat::default().rate(99.0), 0.0);
    }

}
