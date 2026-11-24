use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Read};

use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};

use crate::model::{
    Archive, Difficulty, GameUser, PlayRecord, Progress, Song, Summary, BEST_SIZE,
    CHALLENGE_SONG_COUNT, COUNTED_BEST, COUNTED_PHI, MAX_CHALLENGE_COLOR, MAX_CONSTANT, MAX_SCORE,
    NUM_DIFFICULTIES, NUM_PLAYABLE, RKS_TOLERANCE,
};

const KEY: [u8; 32] = [0xE8, 0x96, 0x9A, 0xD2, 0xA5, 0x40, 0x25, 0x9B, 0x97, 0x91, 0x90, 0x8B, 0x88, 0xE6, 0xBF, 0x03, 0x1E, 0x6D, 0x21, 0x95, 0x6E, 0xFA, 0xD6, 0x8A, 0x50, 0xDD, 0x55, 0xD6, 0x7A, 0xB0, 0x92, 0x4B];

const IV: [u8; 16] = [0x2A, 0x4F, 0xF0, 0x8A, 0xC8, 0x0D, 0x63, 0x07, 0x00, 0x57, 0xC5, 0x95, 0x18, 0xC8, 0x32, 0x53];

const ENTRY_RECORD: &str = "gameRecord";
const ENTRY_PROGRESS: &str = "gameProgress";
const ENTRY_USER: &str = "user";
const ENTRY_SETTINGS: &str = "settings";

const RECORD_VERSION: u8 = 1;
const PROGRESS_VERSION_MIN: u8 = 1;
const PROGRESS_VERSION_MAX: u8 = 6;
const USER_VERSION_MIN: u8 = 1;
const USER_VERSION_MAX: u8 = 1;
const MAX_ENTRY_SIZE: u64 = 8 << 20;

#[derive(Clone, Debug, PartialEq)]
pub enum SaveError {
    Truncated,
    BadBlock,
    Padding,
    Zip,
    Missing(&'static str),
    Version,
    Summary,
    Tampered(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Truncated => formatter.write_str("存档字节流不完整"),
            SaveError::BadBlock => formatter.write_str("存档密文长度不是块大小的整数倍"),
            SaveError::Padding => formatter.write_str("存档填充字节非法"),
            SaveError::Zip => formatter.write_str("存档压缩包无法解开"),
            SaveError::Missing(name) => write!(formatter, "存档缺少 {name} 文件"),
            SaveError::Version => formatter.write_str("存档版本不受支持"),
            SaveError::Summary => formatter.write_str("存档摘要无法解析"),
            SaveError::Tampered(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for SaveError {}

pub fn decode(entry: &[u8]) -> Result<Vec<u8>, SaveError> {
    if entry.len() < 2 {
        return Err(SaveError::Truncated);
    }
    decrypt(&entry[1..])
}

fn decrypt(cipher_text: &[u8]) -> Result<Vec<u8>, SaveError> {
    if cipher_text.is_empty() || !cipher_text.len().is_multiple_of(16) {
        return Err(SaveError::BadBlock);
    }
    let plain = cbc::Decryptor::<aes::Aes256>::new(&KEY.into(), &IV.into())
        .decrypt_padded_vec_mut::<Pkcs7>(cipher_text)
        .map_err(|_| SaveError::Padding)?;
    Ok(plain)
}

pub fn parse_zip(data: &[u8]) -> Result<Archive, SaveError> {
    let files = unzip(data)?;
    let present = [ENTRY_RECORD, ENTRY_PROGRESS, ENTRY_USER, ENTRY_SETTINGS]
        .iter()
        .filter(|name| files.iter().any(|(key, _)| key == *name))
        .count();
    if present == 0 {
        return Err(SaveError::Version);
    }
    if present < 4 {
        for name in [ENTRY_RECORD, ENTRY_PROGRESS, ENTRY_USER, ENTRY_SETTINGS] {
            if !files.iter().any(|(key, _)| key == name) {
                return Err(SaveError::Missing(match name {
                    ENTRY_RECORD => "gameRecord",
                    ENTRY_PROGRESS => "gameProgress",
                    _ => "user",
                }));
            }
        }
    }
    let read = |name: &str| -> Option<&[u8]> {
        files
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_slice())
    };
    let records = parse_records(read(ENTRY_RECORD).unwrap_or(&[]))?;
    let progress = parse_progress(read(ENTRY_PROGRESS).unwrap_or(&[]))?;
    let user = parse_user(read(ENTRY_USER).unwrap_or(&[]))?;
    Ok(Archive {
        progress,
        user,
        records,
    })
}

fn unzip(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>, SaveError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data)).map_err(|_| SaveError::Zip)?;
    let mut files = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|_| SaveError::Zip)?;
        if file.size() > MAX_ENTRY_SIZE {
            return Err(SaveError::Zip);
        }
        let mut content = Vec::new();
        file.by_ref()
            .take(MAX_ENTRY_SIZE)
            .read_to_end(&mut content)
            .map_err(|_| SaveError::Zip)?;
        files.push((file.name().to_string(), content));
    }
    Ok(files)
}

pub fn parse_records(entry: &[u8]) -> Result<Vec<PlayRecord>, SaveError> {
    if entry.first().copied() != Some(RECORD_VERSION) {
        return Err(SaveError::Version);
    }
    let plain = decode(entry)?;
    let mut reader = Reader::new(&plain);
    let declared = reader.varint() as usize;
    let mut records = Vec::with_capacity(declared.min(8192));
    let mut songs = 0usize;
    while reader.remaining() > 0 {
        let song_id = reader.text();
        reader.skip_varint();
        let mask = reader.byte();
        let fc_mask = reader.byte();
        if reader.failed() {
            return Err(SaveError::Truncated);
        }
        songs += 1;
        for level in 0..NUM_DIFFICULTIES {
            if mask & (1 << level) == 0 {
                continue;
            }
            let score = reader.int32();
            let acc = reader.float();
            if reader.failed() {
                return Err(SaveError::Truncated);
            }
            let fc = (score == MAX_SCORE && acc == 100.0) || (fc_mask & (1 << level) != 0);
            if let Some(difficulty) = crate::model::Difficulty::from_index(level) {
                records.push(PlayRecord {
                    song_id: song_id.clone(),
                    difficulty,
                    score,
                    acc,
                    fc,
                });
            }
        }
    }
    if songs != declared {
        return Err(SaveError::Version);
    }
    Ok(records)
}

pub fn parse_progress(entry: &[u8]) -> Result<Progress, SaveError> {
    let version = entry.first().copied().unwrap_or(0);
    if !(PROGRESS_VERSION_MIN..=PROGRESS_VERSION_MAX).contains(&version) {
        return Err(SaveError::Version);
    }
    let plain = decode(entry)?;
    let mut reader = Reader::new(&plain);
    let flags = reader.byte();
    let mut progress = Progress {
        is_first_run: flags & 1 != 0,
        chapter_finished: flags & 2 != 0,
        flags,
        ..Progress::default()
    };
    progress.completed = reader.text();
    progress.song_update_info = reader.varint();
    progress.challenge_mode_rank = reader.short();
    for index in 0..progress.money.len() {
        progress.money[index] = reader.varint();
    }
    for _ in 0..5 {
        reader.byte();
    }
    if reader.failed() {
        return Err(SaveError::Truncated);
    }
    Ok(progress)
}

pub fn parse_user(entry: &[u8]) -> Result<GameUser, SaveError> {
    let version = entry.first().copied().unwrap_or(0);
    if !(USER_VERSION_MIN..=USER_VERSION_MAX).contains(&version) {
        return Err(SaveError::Version);
    }
    let plain = decode(entry)?;
    let mut reader = Reader::new(&plain);
    let flags = reader.byte();
    let mut user = GameUser {
        show_player_id: flags & 1 != 0,
        ..GameUser::default()
    };
    user.self_intro = reader.text();
    user.avatar = reader.text();
    user.background = reader.text();
    if reader.failed() {
        return Err(SaveError::Truncated);
    }
    Ok(user)
}

pub fn parse_summary(encoded: &str) -> Result<Summary, SaveError> {
    let text = encoded.trim();
    if text.is_empty() {
        return Ok(Summary::default());
    }
    let raw = base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|_| SaveError::Summary)?;
    let mut reader = Reader::new(&raw);
    let mut summary = Summary {
        save_version: i32::from(reader.byte()),
        challenge_mode_rank: reader.short(),
        ranking_score: reader.float(),
        game_version: reader.varint(),
        avatar: reader.text(),
        ..Summary::default()
    };
    for level in 0..NUM_PLAYABLE {
        summary.cleared[level] = reader.short();
        summary.full_combo[level] = reader.short();
        summary.phi[level] = reader.short();
    }
    if reader.failed() {
        return Err(SaveError::Summary);
    }
    Ok(summary)
}

const WHITELIST_SONG: &str = "Starduster.Quree.0";
const WHITELIST_LEVEL: Difficulty = Difficulty::Ez;
const WHITELIST_MAX_ACC: f64 = 102.57;
const MAX_MONEY_SEGMENT: i32 = 1023;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SongLimits {
    pub game_version: i32,
    pub max_rks: f64,
    pub max_challenge: i32,
    pub fallback: bool,
}

impl SongLimits {
    pub fn fallback() -> Self {
        Self {
            game_version: 0,
            max_rks: MAX_CONSTANT,
            max_challenge: MAX_CONSTANT.ceil() as i32 * CHALLENGE_SONG_COUNT as i32,
            fallback: true,
        }
    }

    pub fn for_game_version(self, game_version: i32) -> Self {
        if self.fallback || self.game_version == 0 || self.game_version != game_version {
            return Self::fallback();
        }
        self
    }

    pub fn compute(game_version: i32, songs: &[Song]) -> Self {
        let mut constants: Vec<f64> = Vec::with_capacity(songs.len() * NUM_PLAYABLE);
        let mut by_floor: BTreeMap<i32, i32> = BTreeMap::new();
        for song in songs {
            for difficulty in Difficulty::playable_all() {
                let Some(chart) = song.chart(*difficulty) else {
                    continue;
                };
                constants.push(chart.constant);
                *by_floor.entry(chart.constant.trunc() as i32).or_insert(0) += 1;
            }
        }
        if constants.is_empty() {
            return Self::fallback();
        }
        constants.sort_by(|left, right| {
            right
                .partial_cmp(left)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let max_rks = (sum_top(&constants, COUNTED_BEST) + sum_top(&constants, COUNTED_PHI))
            / BEST_SIZE as f64;
        let max_challenge = match max_challenge_total(&by_floor) {
            0 => Self::fallback().max_challenge,
            total => total,
        };
        Self {
            game_version,
            max_rks,
            max_challenge,
            fallback: false,
        }
    }
}

fn sum_top(sorted: &[f64], count: usize) -> f64 {
    sorted.iter().take(count).sum()
}

fn max_challenge_total(by_floor: &BTreeMap<i32, i32>) -> i32 {
    let floors: Vec<i32> = by_floor.keys().copied().collect();
    let mut best = 0;
    for i in 0..floors.len() {
        for j in i..floors.len() {
            for k in j..floors.len() {
                let picked = [floors[i], floors[j], floors[k]];
                let mut need: BTreeMap<i32, i32> = BTreeMap::new();
                for floor in picked {
                    *need.entry(floor).or_insert(0) += 1;
                }
                let enough = need
                    .iter()
                    .all(|(floor, count)| by_floor.get(floor).copied().unwrap_or(0) >= *count);
                if !enough {
                    continue;
                }
                let total: i32 = picked.iter().sum();
                if total > best {
                    best = total;
                }
            }
        }
    }
    best
}

pub fn validate(archive: &Archive, summary: &Summary, limits: SongLimits) -> Result<(), SaveError> {
    let limits = limits.for_game_version(summary.game_version);
    for record in &archive.records {
        let at = format!("{} {}", record.song_id, record.difficulty.name());
        if !(0..=MAX_SCORE).contains(&record.score) {
            return Err(SaveError::Tampered(format!(
                "{at} 成绩 {} 超出 0~{MAX_SCORE}",
                record.score
            )));
        }
        if record.acc < 0.0 {
            return Err(SaveError::Tampered(format!(
                "{at} 准确率 {} 为负",
                record.acc
            )));
        }
        if record.acc > 100.0 && !whitelisted(record) {
            return Err(SaveError::Tampered(format!(
                "{at} 准确率 {} 超出 100%",
                record.acc
            )));
        }
    }
    for (index, segment) in archive.progress.money.iter().enumerate() {
        if !(0..=MAX_MONEY_SEGMENT).contains(segment) {
            return Err(SaveError::Tampered(format!(
                "货币第 {} 段 {segment} 超出 0~{MAX_MONEY_SEGMENT}",
                index + 1
            )));
        }
    }
    if summary.ranking_score != 0.0 {
        if summary.ranking_score.is_nan() || summary.ranking_score < 0.0 {
            return Err(SaveError::Tampered(format!(
                "RKS {} 不是有效数值",
                summary.ranking_score
            )));
        }
        if summary.ranking_score > limits.max_rks + RKS_TOLERANCE {
            return Err(SaveError::Tampered(format!(
                "RKS {:.4} 超过上限 {:.4}",
                summary.ranking_score, limits.max_rks
            )));
        }
    }
    let rank = summary.challenge_mode_rank;
    if rank == 0 {
        return Ok(());
    }
    if rank < 0 {
        return Err(SaveError::Tampered(format!(
            "课题模式分数 {rank} 为负"
        )));
    }
    let color = rank / 100;
    let total = rank % 100;
    if !(1..=i32::from(MAX_CHALLENGE_COLOR)).contains(&color) {
        return Err(SaveError::Tampered(format!(
            "课题模式段位 {color} 超出 1~{}",
            MAX_CHALLENGE_COLOR
        )));
    }
    if !(1..=limits.max_challenge).contains(&total) {
        return Err(SaveError::Tampered(format!(
            "课题模式总值 {total} 超出 1~{}",
            limits.max_challenge
        )));
    }
    Ok(())
}

fn whitelisted(record: &PlayRecord) -> bool {
    record.song_id == WHITELIST_SONG
        && record.difficulty == WHITELIST_LEVEL
        && record.acc <= WHITELIST_MAX_ACC
}

pub fn sanitize(archive: &Archive) -> Archive {
    let mut kept: Vec<PlayRecord> = Vec::with_capacity(archive.records.len());
    let mut seen: HashMap<(String, Difficulty), usize> =
        HashMap::with_capacity(archive.records.len());
    for record in &archive.records {
        if record.acc > 100.0 {
            continue;
        }
        let key = (record.song_id.clone(), record.difficulty);
        if let Some(index) = seen.get(&key).copied() {
            if better_score(record, &kept[index]) {
                kept[index] = record.clone();
            }
        } else {
            seen.insert(key, kept.len());
            kept.push(record.clone());
        }
    }
    let mut sanitized = archive.clone();
    sanitized.records = kept;
    sanitized
}

fn better_score(candidate: &PlayRecord, current: &PlayRecord) -> bool {
    if candidate.score != current.score {
        return candidate.score > current.score;
    }
    candidate.acc > current.acc
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
    err: bool,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            pos: 0,
            err: false,
        }
    }

    fn failed(&self) -> bool {
        self.err
    }

    fn remaining(&self) -> usize {
        if self.err {
            0
        } else {
            self.buf.len().saturating_sub(self.pos)
        }
    }

    fn take(&mut self, count: usize) -> &'a [u8] {
        if self.pos + count > self.buf.len() {
            self.err = true;
            return &[];
        }
        let slice = &self.buf[self.pos..self.pos + count];
        self.pos += count;
        slice
    }

    fn byte(&mut self) -> u8 {
        self.take(1).first().copied().unwrap_or(0)
    }

    fn short(&mut self) -> i32 {
        let bytes = self.take(2);
        if bytes.len() < 2 {
            return 0;
        }
        i32::from(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn int32(&mut self) -> i32 {
        let bytes = self.take(4);
        if bytes.len() < 4 {
            return 0;
        }
        i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    fn float(&mut self) -> f64 {
        let bytes = self.take(4);
        if bytes.len() < 4 {
            return 0.0;
        }
        f64::from(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn varint(&mut self) -> i32 {
        let first = self.byte();
        if self.err {
            return 0;
        }
        if first <= 127 {
            return i32::from(first);
        }
        let second = self.byte();
        i32::from(first & 0x7f) | (i32::from(second) << 7)
    }

    fn skip_varint(&mut self) {
        let first = self.byte();
        if self.err {
            return;
        }
        if first > 127 {
            self.byte();
        }
    }

    fn text(&mut self) -> String {
        let length = self.varint();
        if self.err || length < 0 {
            return String::new();
        }
        let bytes = self.take(length as usize);
        String::from_utf8_lossy(bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cbc::cipher::BlockEncryptMut;

    use crate::model::Chart;

    fn encrypt(plain: &[u8]) -> Vec<u8> {
        let cipher = cbc::Encryptor::<aes::Aes256>::new(&KEY.into(), &IV.into());
        cipher.encrypt_padded_vec_mut::<Pkcs7>(plain)
    }

    #[test]
    fn decrypt_round_trips_pkcs7() {
        let plain = b"phigros-save-entry";
        let cipher_text = encrypt(plain);
        assert_eq!(decrypt(&cipher_text).unwrap(), plain);
        let mut entry = vec![1u8];
        entry.extend_from_slice(&cipher_text);
        assert_eq!(decode(&entry).unwrap(), plain);
        assert_eq!(decrypt(&[]), Err(SaveError::BadBlock));
        assert_eq!(decode(&[1]), Err(SaveError::Truncated));
    }

    #[test]
    fn reader_matches_go_varint_text() {
        let mut buf = vec![0x03u8];
        buf.push(0x82);
        buf.push(0x01);
        let mut reader = Reader::new(&buf);
        assert_eq!(reader.varint(), 3);
        assert_eq!(reader.varint(), 130);
    }

    #[test]
    fn summary_reads_go_layout() {
        let mut raw = vec![1u8];
        raw.extend_from_slice(&7u16.to_le_bytes());
        raw.extend_from_slice(&13.25f32.to_le_bytes());
        raw.push(0x2a);
        raw.push(0x02);
        raw.extend_from_slice(b"ab");
        for _ in 0..NUM_PLAYABLE {
            for value in [1u16, 2, 3] {
                raw.extend_from_slice(&value.to_le_bytes());
            }
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(&raw);
        let summary = parse_summary(&encoded).unwrap();
        assert_eq!(summary.save_version, 1);
        assert_eq!(summary.challenge_mode_rank, 7);
        assert!((summary.ranking_score - 13.25).abs() < 1e-6);
        assert_eq!(summary.game_version, 42);
        assert_eq!(summary.avatar, "ab");
        assert_eq!(summary.cleared[0], 1);
        assert_eq!(summary.phi[3], 3);
        assert_eq!(parse_summary("  ").unwrap(), Summary::default());
    }

    #[test]
    fn records_read_song_mask_and_scores() {
        let mut plain = vec![1u8];
        plain.push(4);
        plain.extend_from_slice(b"Song");
        plain.push(0);
        plain.push(0x01);
        plain.push(0x01);
        plain.extend_from_slice(&999_000i32.to_le_bytes());
        plain.extend_from_slice(&98.5f32.to_le_bytes());
        let mut entry = vec![RECORD_VERSION];
        entry.extend_from_slice(&encrypt(&plain));
        let records = parse_records(&entry).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].song_id, "Song");
        assert_eq!(records[0].score, 999_000);
        assert!(records[0].fc);
        assert_eq!(records[0].difficulty, crate::model::Difficulty::Ez);
    }

    #[test]
    fn records_reject_mismatched_song_count() {
        let mut plain = vec![2u8];
        plain.push(1);
        plain.extend_from_slice(b"A");
        plain.push(0);
        plain.push(0x01);
        plain.push(0x00);
        plain.extend_from_slice(&100i32.to_le_bytes());
        plain.extend_from_slice(&0f32.to_le_bytes());
        let mut entry = vec![RECORD_VERSION];
        entry.extend_from_slice(&encrypt(&plain));
        assert_eq!(parse_records(&entry), Err(SaveError::Version));
    }

    fn limits() -> SongLimits {
        SongLimits {
            game_version: 157,
            max_rks: 17.3533,
            max_challenge: 52,
            fallback: false,
        }
    }

    fn summary_at(version: i32, rank: i32, rks: f64) -> Summary {
        Summary {
            game_version: version,
            challenge_mode_rank: rank,
            ranking_score: rks,
            ..Summary::default()
        }
    }

    fn record(song: &str, difficulty: Difficulty, score: i32, acc: f64) -> PlayRecord {
        PlayRecord {
            song_id: song.to_string(),
            difficulty,
            score,
            acc,
            fc: false,
        }
    }

    fn archive_with_money(money: [i32; 5]) -> Archive {
        Archive {
            progress: Progress {
                money,
                ..Progress::default()
            },
            ..Archive::default()
        }
    }

    #[test]
    fn validate_accepts_normal_archive() {
        let archive = Archive {
            progress: Progress {
                money: [1023; 5],
                ..Progress::default()
            },
            records: vec![
                record(
                    "Spasmodic.姜米條颶風元力上人.0",
                    Difficulty::In,
                    MAX_SCORE,
                    100.0,
                ),
                record("Starduster.Quree.0", Difficulty::Ez, 999_999, 102.57),
            ],
            ..Archive::default()
        };
        let summary = summary_at(157, 346, 17.3533);
        assert_eq!(validate(&archive, &summary, limits()), Ok(()));
        assert_eq!(
            validate(&Archive::default(), &Summary::default(), limits()),
            Ok(())
        );
    }

    #[test]
    fn validate_rejects_rks_above_limit() {
        let bounds = limits();
        for rks in [
            0.0,
            12.5,
            bounds.max_rks,
            bounds.max_rks + RKS_TOLERANCE / 2.0,
        ] {
            let summary = summary_at(157, 0, rks);
            assert_eq!(validate(&Archive::default(), &summary, bounds), Ok(()));
        }
        let summary = summary_at(157, 0, bounds.max_rks + 0.01);
        let error = validate(&Archive::default(), &summary, bounds).unwrap_err();
        assert!(error.to_string().contains("17.3533"), "{error}");
        for rks in [f64::NAN, -1.0] {
            let summary = summary_at(157, 0, rks);
            let error = validate(&Archive::default(), &summary, bounds).unwrap_err();
            assert!(error.to_string().contains("不是有效数值"), "{error}");
        }
    }

    #[test]
    fn validate_version_gate_falls_back() {
        let bounds = limits();
        let loose = summary_at(158, 0, 17.5);
        assert_eq!(validate(&Archive::default(), &loose, bounds), Ok(()));
        let dan = summary_at(158, 554, 0.0);
        assert_eq!(validate(&Archive::default(), &dan, bounds), Ok(()));
        let over = summary_at(158, 0, 18.5);
        assert!(validate(&Archive::default(), &over, bounds).is_err());
        let over_dan = summary_at(158, 555, 0.0);
        assert!(validate(&Archive::default(), &over_dan, bounds).is_err());
    }

    #[test]
    fn validate_rejects_impossible_scores() {
        for score in [-1, MAX_SCORE + 1] {
            let archive = Archive {
                records: vec![record(
                    "Spasmodic.姜米條颶風元力上人.0",
                    Difficulty::In,
                    score,
                    99.0,
                )],
                ..Archive::default()
            };
            let error = validate(&archive, &summary_at(157, 0, 0.0), limits()).unwrap_err();
            let text = error.to_string();
            assert!(text.contains("Spasmodic.姜米條颶風元力上人.0"), "{text}");
            assert!(text.contains("IN"), "{text}");
            assert!(text.contains(&score.to_string()), "{text}");
        }
    }

    #[test]
    fn validate_rejects_out_of_range_acc() {
        let narrow = |song: &str, difficulty: Difficulty, acc: f64| Archive {
            records: vec![record(song, difficulty, 900_000, acc)],
            ..Archive::default()
        };
        let error = validate(
            &narrow("A.0", Difficulty::In, -0.5),
            &summary_at(157, 0, 0.0),
            limits(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("为负"), "{error}");
        let error = validate(
            &narrow("A.0", Difficulty::In, 100.5),
            &summary_at(157, 0, 0.0),
            limits(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("超出 100%"), "{error}");
        let allowed = narrow("Starduster.Quree.0", Difficulty::Ez, 102.57);
        assert_eq!(
            validate(&allowed, &summary_at(157, 0, 0.0), limits()),
            Ok(())
        );
        let too_far = narrow("Starduster.Quree.0", Difficulty::Ez, 102.58);
        assert!(validate(&too_far, &summary_at(157, 0, 0.0), limits()).is_err());
        let other_level = narrow("Starduster.Quree.0", Difficulty::Hd, 102.57);
        assert!(validate(&other_level, &summary_at(157, 0, 0.0), limits()).is_err());
    }

    #[test]
    fn validate_challenge_rank_bounds() {
        let bounds = limits();
        for rank in [0, 346, 152, 552] {
            let summary = summary_at(157, rank, 0.0);
            assert_eq!(validate(&Archive::default(), &summary, bounds), Ok(()));
        }
        let cases = [
            (553, "总值 53"),
            (652, "段位 6"),
            (100, "总值 0"),
            (53, "段位 0"),
            (-1, "为负"),
        ];
        for (rank, want) in cases {
            let summary = summary_at(157, rank, 0.0);
            let error = validate(&Archive::default(), &summary, bounds).unwrap_err();
            assert!(error.to_string().contains(want), "{rank}: {error}");
        }
    }

    #[test]
    fn validate_money_segments() {
        let full = archive_with_money([1023; 5]);
        let summary = summary_at(157, 0, 0.0);
        assert_eq!(validate(&full, &summary, limits()), Ok(()));
        let third = archive_with_money([0, 0, 1024, 0, 0]);
        let error = validate(&third, &summary, limits()).unwrap_err();
        assert!(error.to_string().contains("第 3 段"), "{error}");
        let second = archive_with_money([0, -1, 0, 0, 0]);
        let error = validate(&second, &summary, limits()).unwrap_err();
        assert!(error.to_string().contains("第 2 段"), "{error}");
    }

    #[test]
    fn sanitize_drops_out_of_range_acc() {
        let archive = Archive {
            records: vec![
                record("Starduster.Quree.0", Difficulty::Ez, 999_999, 102.57),
                record("Starduster.Quree.0", Difficulty::Hd, 990_000, 99.5),
                record(
                    "Spasmodic.姜米條颶風元力上人.0",
                    Difficulty::In,
                    MAX_SCORE,
                    100.0,
                ),
            ],
            ..Archive::default()
        };
        let sanitized = sanitize(&archive);
        assert_eq!(sanitized.records.len(), 2);
        assert_eq!(sanitized.records[0].difficulty, Difficulty::Hd);
        assert_eq!(sanitized.records[1].difficulty, Difficulty::In);
        assert_eq!(archive.records.len(), 3);
    }

    #[test]
    fn sanitize_merges_duplicate_records() {
        let archive = Archive {
            records: vec![
                record("song.a", Difficulty::In, 980_000, 98.0),
                record("song.b", Difficulty::In, 990_000, 99.0),
                record("song.a", Difficulty::In, 995_000, 99.3),
                record("song.b", Difficulty::In, 990_000, 99.6),
                record("song.a", Difficulty::At, 900_000, 95.0),
                record("song.a", Difficulty::In, 900_000, 96.0),
            ],
            ..Archive::default()
        };
        let sanitized = sanitize(&archive);
        assert_eq!(sanitized.records.len(), 3);
        assert_eq!(sanitized.records[0].song_id, "song.a");
        assert_eq!(sanitized.records[0].difficulty, Difficulty::In);
        assert_eq!(sanitized.records[0].score, 995_000);
        assert_eq!(sanitized.records[1].song_id, "song.b");
        assert_eq!(sanitized.records[1].score, 990_000);
        assert_eq!(sanitized.records[1].acc, 99.6);
        assert_eq!(sanitized.records[2].song_id, "song.a");
        assert_eq!(sanitized.records[2].difficulty, Difficulty::At);
    }

    #[test]
    fn suspect_full_score_survives_sanitize() {
        let archive = Archive {
            records: vec![
                record("X.0", Difficulty::In, MAX_SCORE, 100.0),
                record("Y.0", Difficulty::In, MAX_SCORE, 99.8959),
                record("Z.0", Difficulty::In, 999_063, 99.8959),
            ],
            ..Archive::default()
        };
        assert_eq!(
            validate(&archive, &summary_at(157, 0, 0.0), limits()),
            Ok(())
        );
        let sanitized = sanitize(&archive);
        assert_eq!(sanitized.records.len(), 3);
        let suspects: Vec<&PlayRecord> = sanitized
            .records
            .iter()
            .filter(|item| item.is_suspect())
            .collect();
        assert_eq!(suspects.len(), 1);
        assert_eq!(suspects[0].song_id, "Y.0");
    }

    fn song_with(id: &str, entries: &[(Difficulty, f64)]) -> Song {
        let mut song = Song {
            id: id.to_string(),
            ..Song::default()
        };
        for (difficulty, constant) in entries {
            song.charts[difficulty.index()] = Chart {
                difficulty: *difficulty,
                constant: *constant,
                ..Chart::default()
            };
        }
        song
    }

    #[test]
    fn compute_limits_follows_catalog_constants() {
        let songs = vec![
            song_with("A", &[(Difficulty::In, 18.0), (Difficulty::At, 17.0)]),
            song_with("B", &[(Difficulty::In, 17.0), (Difficulty::At, 17.0)]),
        ];
        let limits = SongLimits::compute(157, &songs);
        assert!(!limits.fallback, "曲库有定数时不该退回保守上界");
        assert_eq!(limits.game_version, 157, "推算上限要记下它对应的版本");
        assert!(
            (limits.max_rks - (18.0 + 17.0 + 17.0 + 17.0 + 18.0 + 17.0 + 17.0) / 30.0).abs()
                < 1e-12,
            "RKS 上限 = (定数前 27 之和 + 前 3 之和) / 30，实际 {}",
            limits.max_rks
        );
        assert_eq!(limits.max_challenge, 52, "一张 18 + 三张 17：18+17+17");
        assert_eq!(limits.for_game_version(157), limits);
        assert!(
            limits.for_game_version(158).fallback,
            "存档版本与定数表对不上时退回保守上界"
        );
    }

    #[test]
    fn compute_limits_falls_back_without_constants() {
        let limits = SongLimits::compute(157, &[]);
        assert!(limits.fallback);
        assert_eq!(limits.max_rks, MAX_CONSTANT);
        assert_eq!(limits.max_challenge, 54);
    }

    #[test]
    fn compute_limits_keeps_fallback_challenge_when_charts_are_short() {
        let songs = vec![song_with("A", &[(Difficulty::In, 18.0)])];
        let limits = SongLimits::compute(157, &songs);
        assert!(!limits.fallback);
        assert!((limits.max_rks - (18.0 + 18.0) / 30.0).abs() < 1e-12);
        assert_eq!(limits.max_challenge, 54, "凑不出三张谱时退回保守上界");
    }

    #[test]
    fn compute_limits_on_installed_catalog() {
        let Ok(catalog) = crate::domain::catalog::Catalog::open(
            crate::assets_root().join("../data/phi.db"),
        ) else {
            return;
        };
        let limits = SongLimits::compute(catalog.game_version(), catalog.songs());
        assert!(!limits.fallback, "真实曲库必须推得出上限");
        assert!(limits.max_rks > 0.0 && limits.max_rks <= MAX_CONSTANT);
        assert!(limits.max_challenge > 0 && limits.max_challenge <= 54);
        println!(
            "曲库版本 {} → max_rks {:.4} max_challenge {}",
            limits.game_version, limits.max_rks, limits.max_challenge
        );
    }
}
