use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::domain::rks;
use crate::model::Difficulty;
use crate::repo::{LeaderBoard, LeaderRow, RksPoint, ScoreEvent};

pub const DEFAULT_BASE: &str = "https://phib19.top:8080";

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

pub const SONG_BOARD_BUDGET: Duration = Duration::from_millis(1500);

pub const HISTORY_VERSION: i64 = 4;

const RKS_RANK_PATH: &str = "/get/ranklist/rksRank";

const RANK_PATH: &str = "/get/ranklist/rank";

const SCORE_LIST_PATH: &str = "/get/scoreList/user";

const HISTORY_PATH: &str = "/get/history/historyV4";

const INVALID_TOKEN_CODE: &str = "sstk_invalid";

const GUARD_RATE: f64 = 1.0;

const GUARD_BURST: f64 = 3.0;

const GUARD_TRIP: u32 = 3;

const GUARD_SOFT_TRIP: u32 = 8;

const GUARD_COOLOFF: Duration = Duration::from_secs(60);

const GUARD_PATIENCE: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhiError {
    Offline,
    NoHistory,
    InvalidToken,
    Unavailable,
}

pub type Constants = Box<dyn Fn(&str, Difficulty) -> Option<f64> + Send + Sync>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HistoryArchive {
    pub points: Vec<RksPoint>,
    pub events: Vec<ScoreEvent>,
}

impl HistoryArchive {
    pub fn is_empty(&self) -> bool {
        self.points.is_empty() && self.events.is_empty()
    }
}

pub struct BoardAsk {
    pub song_id: String,
    pub difficulty: Difficulty,
    pub token: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongRankRow {
    pub rank: i32,
    pub name: String,
    pub avatar: String,
    pub score: i32,
    pub acc: f64,
    pub rks: f64,
    pub challenge: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SongRank {
    pub total: i32,
    pub me_rank: i32,
    pub rows: Vec<SongRankRow>,
}

struct GuardState {
    tokens: f64,
    refilled: Instant,
    failures: u32,
    slows: u32,
    open_until: Option<Instant>,
}

struct Guard {
    state: Mutex<GuardState>,
}

impl Guard {
    fn new() -> Self {
        Self {
            state: Mutex::new(GuardState {
                tokens: GUARD_BURST,
                refilled: Instant::now(),
                failures: 0,
                slows: 0,
                open_until: None,
            }),
        }
    }

    async fn acquire(&self) -> Result<(), PhiError> {
        let deadline = Instant::now() + GUARD_PATIENCE;
        loop {
            let wait = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let now = Instant::now();
                if state.open_until.is_some_and(|until| now < until) {
                    return Err(PhiError::Unavailable);
                }
                let gained = now.duration_since(state.refilled).as_secs_f64() * GUARD_RATE;
                state.tokens = (state.tokens + gained).min(GUARD_BURST);
                state.refilled = now;
                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    return Ok(());
                }
                Duration::from_secs_f64((1.0 - state.tokens) / GUARD_RATE)
            };
            if Instant::now() + wait > deadline {
                return Err(PhiError::Unavailable);
            }
            tokio::time::sleep(wait).await;
        }
    }

    fn success(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.failures = 0;
        state.slows = 0;
    }

    fn failure(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.failures += 1;
        state.slows = 0;
        if state.failures >= GUARD_TRIP {
            state.open_until = Some(Instant::now() + GUARD_COOLOFF);
            state.failures = 0;
        }
    }

    fn slow(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.slows += 1;
        state.failures = 0;
        if state.slows >= GUARD_SOFT_TRIP {
            state.open_until = Some(Instant::now() + GUARD_COOLOFF);
            state.slows = 0;
        }
    }
}

pub struct Client {
    base: String,
    http: reqwest::Client,
    guard: Guard,
}

impl Client {
    pub fn new(base: &str) -> Self {
        let text = if base.trim().is_empty() {
            DEFAULT_BASE
        } else {
            base.trim()
        };
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            base: text.trim_end_matches('/').to_string(),
            http,
            guard: Guard::new(),
        }
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value, PhiError> {
        self.guard.acquire().await?;
        let url = format!("{}{}", self.base, path);
        let response = match self.http.post(&url).json(&body).send().await {
            Ok(response) => response,
            Err(error) => {
                self.note_failure(&error);
                return Err(PhiError::Unavailable);
            }
        };
        let status = response.status();
        let text = match response.text().await {
            Ok(text) => text,
            Err(error) => {
                self.note_failure(&error);
                return Err(PhiError::Unavailable);
            }
        };
        if !status.is_success() {
            self.guard.failure();
            return Err(PhiError::Unavailable);
        }
        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(_) => {
                self.guard.failure();
                return Err(PhiError::Unavailable);
            }
        };
        if let Some(code) = api_error_code(&value) {
            if code == INVALID_TOKEN_CODE {
                self.guard.success();
                return Err(PhiError::InvalidToken);
            }
            self.guard.failure();
            return Err(PhiError::Unavailable);
        }
        self.guard.success();
        Ok(value)
    }

    fn note_failure(&self, error: &reqwest::Error) {
        if error.is_timeout() {
            self.guard.slow();
        } else {
            self.guard.failure();
        }
    }
}

pub struct Source {
    client: Client,
    constants: Constants,
    online: AtomicBool,
}

impl Source {
    pub fn new(base: &str, constants: Constants) -> Self {
        Self {
            client: Client::new(base),
            constants,
            online: AtomicBool::new(false),
        }
    }

    pub fn set_online_rank(&self, on: bool) {
        self.online.store(on, Ordering::Relaxed);
    }

    fn online_rank(&self) -> bool {
        self.online.load(Ordering::Relaxed)
    }

    pub async fn board(&self, me: &LeaderRow, limit: usize) -> Result<LeaderBoard, PhiError> {
        if !self.online_rank() {
            return Err(PhiError::Offline);
        }
        if me.rks <= 0.0 {
            return Err(PhiError::Unavailable);
        }
        let ranked = self
            .client
            .post(RKS_RANK_PATH, json!({ "request_rks": me.rks }))
            .await?;
        let (total, rank) = decode_rks_rank(ranked)?;
        let me_place = rank + 1;
        if total <= 0 || me_place <= 0 {
            return Err(PhiError::Unavailable);
        }
        let page = self
            .client
            .post(RANK_PATH, json!({ "request_rank": 1 }))
            .await?;
        let mut board = LeaderBoard {
            total,
            me_place,
            me: Some(me.clone()),
            top: Vec::new(),
        };
        for row in decode_rank_page(page)? {
            if board.top.len() >= limit {
                break;
            }
            board.top.push(row);
        }
        Ok(board)
    }

    pub async fn song_board(&self, ask: &BoardAsk) -> Result<SongRank, PhiError> {
        if !self.online_rank() || ask.song_id.is_empty() || ask.token.is_empty() {
            return Err(PhiError::Offline);
        }
        let body = json!({
            "token": ask.token,
            "songId": ask.song_id,
            "rank": ask.difficulty.name(),
            "orderBy": "score",
        });
        let sent = tokio::time::timeout(SONG_BOARD_BUDGET, self.client.post(SCORE_LIST_PATH, body)).await;
        match sent {
            Ok(page) => decode_song_board(page?),
            Err(_) => Err(PhiError::Unavailable),
        }
    }

    pub async fn history(&self, token: &str) -> Result<HistoryArchive, PhiError> {
        if token.is_empty() {
            return Err(PhiError::NoHistory);
        }
        let doc = match self
            .client
            .post(HISTORY_PATH, json!({ "token": token }))
            .await
        {
            Ok(doc) => doc,
            Err(error) => return Err(mirror_error(error)),
        };
        let Some(data) = doc.get("data") else {
            return Err(PhiError::NoHistory);
        };
        decode_history(data, self.constants.as_ref())
    }
}

fn mirror_error(error: PhiError) -> PhiError {
    match error {
        PhiError::InvalidToken => PhiError::NoHistory,
        other => other,
    }
}

fn api_error_code(value: &Value) -> Option<String> {
    let raw = value.get("error")?;
    if raw.is_null() {
        return None;
    }
    if let Some(text) = raw.as_str() {
        return if text.trim().is_empty() {
            None
        } else {
            Some(top_code(value))
        };
    }
    if let Some(object) = raw.as_object() {
        let code = object.get("code").and_then(Value::as_str).unwrap_or("");
        return Some(if code.is_empty() {
            top_code(value)
        } else {
            code.to_string()
        });
    }
    Some(top_code(value))
}

fn top_code(value: &Value) -> String {
    value
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RksRankDoc {
    data: RksRankData,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RksRankData {
    tot_num: i64,
    rks_rank: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RankPageDoc {
    data: RankPageData,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RankPageData {
    users: Vec<RankUser>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct RankUser {
    index: i64,
    #[serde(rename = "saveInfo")]
    save_info: RankSaveInfo,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RankSaveInfo {
    #[serde(rename = "PlayerId")]
    player_id: String,
    summary: RankSummary,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct RankSummary {
    ranking_score: f64,
    avatar: String,
    challenge_mode_rank: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct SongBoardDoc {
    data: SongBoardData,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct SongBoardData {
    tot_data_num: i64,
    user_rank: i64,
    users: Vec<SongBoardUser>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct SongBoardUser {
    index: i64,
    gameuser: SongBoardGameUser,
    record: SongBoardRecord,
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct SongBoardGameUser {
    #[serde(rename = "PlayerId")]
    player_id: String,
    avatar: String,
    ranking_score: f64,
    challenge_mode_rank: i64,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct SongBoardRecord {
    score: i64,
    acc: f64,
}

fn decode_rks_rank(value: Value) -> Result<(i64, i64), PhiError> {
    let doc: RksRankDoc =
        serde_json::from_value(value).map_err(|_| PhiError::Unavailable)?;
    Ok((doc.data.tot_num, doc.data.rks_rank))
}

fn decode_rank_page(value: Value) -> Result<Vec<LeaderRow>, PhiError> {
    let doc: RankPageDoc = serde_json::from_value(value).map_err(|_| PhiError::Unavailable)?;
    let mut users = doc.data.users;
    users.sort_by_key(|user| user.index);
    Ok(users
        .into_iter()
        .filter(|user| user.index > 0 && !user.save_info.player_id.is_empty())
        .map(|user| LeaderRow {
            user_id: String::new(),
            nickname: user.save_info.player_id,
            avatar: user.save_info.summary.avatar,
            rks: user.save_info.summary.ranking_score,
            challenge: user.save_info.summary.challenge_mode_rank,
        })
        .collect())
}

fn decode_song_board(value: Value) -> Result<SongRank, PhiError> {
    let doc: SongBoardDoc = serde_json::from_value(value).map_err(|_| PhiError::Unavailable)?;
    let mut users = doc.data.users;
    users.sort_by_key(|user| user.index);
    Ok(SongRank {
        total: doc.data.tot_data_num as i32,
        me_rank: doc.data.user_rank as i32,
        rows: users
            .into_iter()
            .map(|user| SongRankRow {
                rank: user.index as i32,
                name: user.gameuser.player_id,
                avatar: user.gameuser.avatar,
                score: user.record.score as i32,
                acc: user.record.acc,
                rks: user.gameuser.ranking_score,
                challenge: user.gameuser.challenge_mode_rank,
            })
            .collect(),
    })
}

fn decode_history(
    data: &Value,
    constants: &dyn Fn(&str, Difficulty) -> Option<f64>,
) -> Result<HistoryArchive, PhiError> {
    let version = data
        .get("version")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    if version != HISTORY_VERSION {
        return Err(PhiError::NoHistory);
    }
    let challenges = challenge_series(data.get("challengeModeRank"));
    let mut points: Vec<RksPoint> = Vec::new();
    if let Some(node) = data.get("rks") {
        walk_points(node, &mut |at, value| {
            let Some(value) = number(value) else {
                return;
            };
            points.push(RksPoint {
                observed_at: at,
                rks: value,
                computed: value,
                records: 0,
                phi: 0,
                challenge: challenge_at(&challenges, at),
            });
        });
    }
    points.sort_by_key(|point| point.observed_at);
    let mut events: Vec<ScoreEvent> = Vec::new();
    if let Some(songs) = data.get("scoreHistory").and_then(Value::as_object) {
        for (song_id, per_diff) in songs {
            let Some(per_diff) = per_diff.as_object() else {
                continue;
            };
            for (name, series) in per_diff {
                let Some(difficulty) = parse_difficulty(name) else {
                    continue;
                };
                events.extend(chart_events(song_id, difficulty, series, constants));
            }
        }
    }
    events.sort_by(|left, right| {
        left.observed_at
            .cmp(&right.observed_at)
            .then_with(|| left.song_id.cmp(&right.song_id))
            .then_with(|| left.difficulty.cmp(&right.difficulty))
    });
    Ok(HistoryArchive { points, events })
}

fn parse_difficulty(name: &str) -> Option<Difficulty> {
    match name.trim().to_uppercase().as_str() {
        "EZ" => Some(Difficulty::Ez),
        "HD" => Some(Difficulty::Hd),
        "IN" => Some(Difficulty::In),
        "AT" => Some(Difficulty::At),
        "SP" => Some(Difficulty::Sp),
        _ => None,
    }
}

fn chart_events(
    song_id: &str,
    difficulty: Difficulty,
    series: &Value,
    constants: &dyn Fn(&str, Difficulty) -> Option<f64>,
) -> Vec<ScoreEvent> {
    let mut points: Vec<(i64, f64, i32, bool)> = Vec::new();
    walk_points(series, &mut |at, value| {
        let Some(fields) = value.as_array() else {
            return;
        };
        if fields.len() < 3 {
            return;
        }
        let (Some(acc), Some(score), Some(fc)) = (
            number(&fields[0]),
            number(&fields[1]),
            number(&fields[2]),
        ) else {
            return;
        };
        points.push((at, acc, score as i32, fc > 0.0));
    });
    points.sort_by_key(|point| point.0);
    let constant = constants(song_id, difficulty);
    let mut out: Vec<ScoreEvent> = Vec::with_capacity(points.len());
    for (index, current) in points.iter().enumerate() {
        let mut event = ScoreEvent {
            observed_at: current.0,
            song_id: song_id.to_string(),
            difficulty,
            has_old: false,
            old_score: 0,
            old_acc: 0.0,
            old_fc: false,
            new_score: current.2,
            new_acc: current.1,
            new_fc: current.3,
            old_rks: 0.0,
            new_rks: single_rks(current.1, constant),
        };
        if index > 0 {
            let previous = points[index - 1];
            event.has_old = true;
            event.old_score = previous.2;
            event.old_acc = previous.1;
            event.old_fc = previous.3;
            event.old_rks = single_rks(previous.1, constant);
        }
        out.push(event);
    }
    out
}

fn single_rks(acc: f64, constant: Option<f64>) -> f64 {
    match constant {
        Some(constant) if constant > 0.0 => rks::single_rks(acc, constant),
        _ => 0.0,
    }
}

fn challenge_series(node: Option<&Value>) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::new();
    if let Some(node) = node {
        walk_points(node, &mut |at, value| {
            if let Some(value) = number(value) {
                out.push((at, value as i64));
            }
        });
    }
    out.sort_by_key(|point| point.0);
    out
}

fn challenge_at(series: &[(i64, i64)], at: i64) -> i64 {
    let Some(first) = series.first() else {
        return 0;
    };
    let mut best = first.1;
    for point in series {
        if point.0 > at {
            break;
        }
        best = point.1;
    }
    best
}

fn walk_points(node: &Value, visit: &mut dyn FnMut(i64, &Value)) {
    let Some(years) = node.as_array() else {
        return;
    };
    for year_node in years {
        let Some((year, months)) = pair(year_node) else {
            continue;
        };
        for month_node in months {
            let Some((month, days)) = pair(month_node) else {
                continue;
            };
            for day_node in days {
                let Some((day, points)) = pair(day_node) else {
                    continue;
                };
                let base = day_start(year, month, day);
                for point_node in points {
                    let Some(fields) = point_node.as_array() else {
                        continue;
                    };
                    if fields.len() != 2 {
                        continue;
                    }
                    let Some(offset) = number(&fields[0]) else {
                        continue;
                    };
                    visit(base + offset as i64 / 1000, &fields[1]);
                }
            }
        }
    }
}

fn pair(node: &Value) -> Option<(i64, &[Value])> {
    let fields = node.as_array()?;
    if fields.len() != 2 {
        return None;
    }
    let key = number(&fields[0])?;
    let children = fields[1].as_array()?;
    Some((key as i64, children.as_slice()))
}

fn number(value: &Value) -> Option<f64> {
    value.as_f64()
}

fn day_start(year: i64, month: i64, day: i64) -> i64 {
    chrono::NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|stamp| stamp.and_utc().timestamp())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Source {
        Source::new(
            "http://127.0.0.1:1",
            Box::new(|song_id: &str, difficulty: Difficulty| {
                if song_id == "song.0" && difficulty == Difficulty::In {
                    Some(15.7)
                } else {
                    None
                }
            }),
        )
    }

    #[tokio::test]
    async fn online_rank_off_returns_offline_without_asking() {
        let source = source();
        let me = LeaderRow {
            rks: 16.0,
            ..LeaderRow::default()
        };
        assert_eq!(source.board(&me, 10).await, Err(PhiError::Offline));
        let ask = BoardAsk {
            song_id: String::from("song.0"),
            difficulty: Difficulty::In,
            token: String::from("token"),
        };
        assert_eq!(source.song_board(&ask).await, Err(PhiError::Offline));
        assert_eq!(source.history("").await, Err(PhiError::NoHistory));
    }

    #[test]
    fn constants_match_the_go_client() {
        assert_eq!(DEFAULT_BASE, "https://phib19.top:8080");
        assert_eq!(REQUEST_TIMEOUT, Duration::from_secs(20));
        assert_eq!(HISTORY_VERSION, 4);
    }

    #[test]
    fn invalid_token_becomes_missing_history() {
        assert_eq!(mirror_error(PhiError::InvalidToken), PhiError::NoHistory);
        assert_eq!(mirror_error(PhiError::Unavailable), PhiError::Unavailable);
        assert_eq!(mirror_error(PhiError::Offline), PhiError::Offline);
    }

    #[test]
    fn api_error_reads_both_shapes() {
        let text: Value = serde_json::from_str(r#"{"error":"nope","code":"sstk_invalid"}"#).unwrap();
        assert_eq!(api_error_code(&text).as_deref(), Some("sstk_invalid"));
        let object: Value = serde_json::from_str(r#"{"error":{"message":"nope","code":"busy"}}"#).unwrap();
        assert_eq!(api_error_code(&object).as_deref(), Some("busy"));
        let empty: Value = serde_json::from_str(r#"{"error":"  ","code":"x"}"#).unwrap();
        assert_eq!(api_error_code(&empty), None);
        let none: Value = serde_json::from_str(r#"{"data":{}}"#).unwrap();
        assert_eq!(api_error_code(&none), None);
        let null: Value = serde_json::from_str(r#"{"error":null}"#).unwrap();
        assert_eq!(api_error_code(&null), None);
    }

    #[test]
    fn rank_page_takes_index_as_the_place() {
        let doc: Value = serde_json::from_str(
            r#"{"data":{"totDataNum":5,"users":[
                {"index":3,"saveInfo":{"PlayerId":"c","summary":{"rankingScore":15.0,"avatar":"a3","challengeModeRank":501}}},
                {"index":1,"saveInfo":{"PlayerId":"a","summary":{"rankingScore":16.4,"avatar":"a1","challengeModeRank":0}}},
                {"index":0,"saveInfo":{"PlayerId":"z","summary":{"rankingScore":17.0}}},
                {"index":2,"saveInfo":{"PlayerId":"","summary":{"rankingScore":15.9}}}
            ]}}"#,
        )
        .unwrap();
        let rows = decode_rank_page(doc).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].nickname, "a");
        assert_eq!(rows[0].rks, 16.4);
        assert_eq!(rows[1].nickname, "c");
        assert_eq!(rows[1].challenge, 501);
    }

    #[test]
    fn song_board_keeps_the_page_and_my_rank() {
        let doc: Value = serde_json::from_str(
            r#"{"data":{"totDataNum":109646,"userRank":23661,"users":[
                {"index":23662,"gameuser":{"PlayerId":"b","avatar":"ab","rankingScore":15.1,"challengeModeRank":302},"record":{"score":999000,"acc":99.5}},
                {"index":23661,"gameuser":{"PlayerId":"a","avatar":"aa","rankingScore":15.2,"challengeModeRank":401},"record":{"score":1000000,"acc":100.0}}
            ]}}"#,
        )
        .unwrap();
        let rank = decode_song_board(doc).unwrap();
        assert_eq!(rank.total, 109_646);
        assert_eq!(rank.me_rank, 23_661);
        assert_eq!(rank.rows.len(), 2);
        assert_eq!(rank.rows[0].rank, 23_661);
        assert_eq!(rank.rows[0].name, "a");
        assert_eq!(rank.rows[0].score, 1_000_000);
        assert_eq!(rank.rows[1].rank, 23_662);
        assert_eq!(rank.rows[1].acc, 99.5);
    }

    #[test]
    fn history_version_must_be_the_supported_one() {
        let older: Value = serde_json::from_str(r#"{"version":3,"rks":[]}"#).unwrap();
        assert_eq!(
            decode_history(&older, &|_, _| None),
            Err(PhiError::NoHistory)
        );
        let empty: Value = serde_json::from_str(r#"{"version":4}"#).unwrap();
        let archive = decode_history(&empty, &|_, _| None).unwrap();
        assert!(archive.is_empty());
    }

    #[test]
    fn history_decodes_points_challenges_and_events() {
        let doc: Value = serde_json::from_str(
            r#"{"version":4,
                "rks":[[2025,[[1,[[5,[[0,15.10],[3600000,15.20]]]]]]]],
                "challengeModeRank":[[2025,[[1,[[5,[[0,301]]],[7,[[0,402]]]]]]]],
                "scoreHistory":{"song.0":{"IN":[[2025,[[1,[[5,[[0,[99.0,950000,0]],[1000,[99.5,960000,1]]]]]]]]],"LEGACY":[[2025,[[1,[[5,[[0,[80.0,700000,0]]]]]]]]]}}}"#,
        )
        .unwrap();
        let archive = decode_history(&doc, &|song_id, difficulty| {
            if song_id == "song.0" && difficulty == Difficulty::In {
                Some(15.7)
            } else {
                None
            }
        })
        .unwrap();
        assert_eq!(archive.points.len(), 2);
        assert_eq!(archive.points[0].challenge, 301);
        assert_eq!(archive.points[1].challenge, 301);
        assert_eq!(archive.events.len(), 2);
        assert!(!archive.events[0].has_old);
        assert_eq!(archive.events[0].new_score, 950_000);
        assert_eq!(archive.events[0].new_rks, rks::single_rks(99.0, 15.7));
        assert!(archive.events[1].has_old);
        assert_eq!(archive.events[1].old_score, 950_000);
        assert!(archive.events[1].new_fc);
        assert!(archive.events.iter().all(|event| event.difficulty == Difficulty::In));
    }

    #[test]
    fn difficulty_names_follow_the_go_parser() {
        assert_eq!(parse_difficulty("in"), Some(Difficulty::In));
        assert_eq!(parse_difficulty(" AT "), Some(Difficulty::At));
        assert_eq!(parse_difficulty("SP"), Some(Difficulty::Sp));
        assert_eq!(parse_difficulty("LEGACY"), None);
        assert_eq!(parse_difficulty(""), None);
    }
}
