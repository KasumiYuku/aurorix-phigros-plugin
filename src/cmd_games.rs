use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use aurorix_plugin::{CommandSpec, Invocation, Outgoing, PluginError, RenderCard};
use aurorix_template::Args;

use crate::cards::guess::{self, GuessRequest};
use crate::commands;
use crate::domain::catalog::{name_hits, normalize_title, Catalog, SCORE_NEAR};
use crate::domain::letters;
use crate::domain::span::{parse_constant, Span};
use crate::model::{Difficulty, Song};
use crate::service::{put, PhigrosService};

pub const GUESS: &str = "guess";

pub const LETTER: &str = "letter";

pub const CLIP: &str = "clip";

pub const VIDEO: &str = "video";

pub const ALIASES_GUESS: [&str; 3] = ["猜曲绘", "猜歌", "guesssong"];

pub const ALIASES_LETTER: [&str; 3] = ["猜字母", "猜歌名", "lettergame"];

pub const ALIASES_CLIP: [&str; 4] = ["猜曲目", "看谱猜歌", "guessclip", "vg"];

pub const ALIASES_VIDEO: [&str; 4] = ["视频", "看片段", "vid", "peek"];

pub const GUESS_TIMEOUT: i64 = 165;

pub const LETTER_TIMEOUT: i64 = 900;

pub const CLIP_TIMEOUT: i64 = 300;

pub const GUESS_STEP_INTERVAL: i64 = 10;

pub const GUESS_STEPS: usize = 15;

pub const HINT_COOLDOWN: i64 = 20;

pub const EXTEND_COOLDOWN: i64 = 30;

pub const LETTER_COOLDOWN: i64 = 3;

pub const LETTER_HINT_COOLDOWN: i64 = 20;

pub const LETTER_SONGS_MAX: usize = 20;

pub const LETTER_SHORT_INPUT: usize = 2;

pub const LETTER_DRAW_ROUNDS: usize = 12;

pub const GUESS_DRAW_ATTEMPTS: usize = 4;

pub const MATCH_CANDIDATES: usize = 5;

pub const VIDEO_RATE_LIMIT: usize = 2;

pub const VIDEO_RATE_WINDOW: i64 = 60;

pub const GUESS_OPEN_ZOOM: f64 = 0.26;

pub const GUESS_OPEN_LEVEL: f64 = 0.24;

pub const GUESS_OPEN_NOISE: f64 = 0.85;

pub const GUESS_ZOOM_STEP: f64 = 1.15;

pub const GUESS_LEVEL_STEP: f64 = 0.27;

pub const GUESS_LEVEL_MAX: f64 = 0.78;

pub const GUESS_NOISE_DECAY: f64 = 0.77;

pub const GUESS_NOISE_FLOOR: f64 = 0.12;

pub const GUESS_OP_JITTER: f64 = 0.04;

pub const GUESS_OP_SEED: u64 = 0x5EED_2A17;

pub const OP_ZOOM: &str = "zoom";

pub const OP_CLEAR: &str = "clear";

pub const OP_NOISE: &str = "noise";

pub const OP_ZOOM_LABEL: &str = "视野扩大";

pub const OP_CLEAR_LABEL: &str = "清晰度上升";

pub const OP_NOISE_LABEL: &str = "干扰减弱";

pub const CLIP_BASE_WINDOW: f64 = 4.0;

pub const CLIP_WINDOW_STEP: f64 = 3.0;

pub const CLIP_TIGHT_BASE: f64 = 2.0;

pub const CLIP_MAX_WINDOW: f64 = 30.0;

pub const CLIP_KEYFRAME: f64 = 0.5;

pub const VIDEO_TAIL_WINDOW: f64 = 30.0;

pub const STOP_TOKENS: [&str; 8] = ["stop", "end", "quit", "over", "停止", "结束", "收局", "不玩了"];

pub const HINT_TOKENS: [&str; 3] = ["提示", "hint", "线索"];

pub const EXTEND_TOKENS: [&str; 4] = ["延长", "extend", "放长", "more"];

pub const LETTER_TITLE_TOKENS: [&str; 2] = ["guess", "猜"];

pub const LETTER_OPEN_TOKENS: [&str; 2] = ["open", "开"];

pub const PLACEHOLDER: &str = "▢";

pub const VIDEO_USAGE: &str = "写个曲名，比如 /p video 风屿；位置可写 1:30、50%、#600、1:30-2:00，后面跟 +30 表示只放 30 秒；不写就发整条";

pub const CLIP_ARGS_USAGE: &str =
    "可以写 /p clip at 指定难度，或者 /p clip 14-16、15+ 指定定数";

pub const HEAD_HIT: &str = "🎉 恭喜回答正确";

pub const HEAD_STOP: &str = "这一局结束了";

pub const HEAD_TIMEOUT: &str = "时间到";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Game {
    #[default]
    Guess,
    Letter,
    Clip,
}

impl Game {
    pub const ALL: [Game; 3] = [Game::Guess, Game::Letter, Game::Clip];

    pub fn command(self) -> &'static str {
        match self {
            Game::Guess => GUESS,
            Game::Letter => LETTER,
            Game::Clip => CLIP,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Game::Guess => "猜曲绘",
            Game::Letter => "猜字母",
            Game::Clip => "看谱猜歌",
        }
    }

    pub fn timeout(self) -> i64 {
        match self {
            Game::Guess => GUESS_TIMEOUT,
            Game::Letter => LETTER_TIMEOUT,
            Game::Clip => CLIP_TIMEOUT,
        }
    }

    pub fn from_command(name: &str) -> Option<Self> {
        match name {
            GUESS => Some(Game::Guess),
            LETTER => Some(Game::Letter),
            CLIP => Some(Game::Clip),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stage {
    pub step: usize,
    pub zoom: f64,
    pub level: f64,
    pub noise: f64,
    pub label: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hint {
    pub label: String,
    pub text: String,
}

impl Hint {
    fn new(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Piece {
    pub text: char,
    pub keys: Vec<char>,
    pub given: bool,
    pub open: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Board {
    pub title: String,
    pub pieces: Vec<Piece>,
    pub solved: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Round {
    pub game: Game,
    pub song_id: String,
    pub title: String,
    pub code: String,
    pub variant: String,
    pub level: f64,
    pub start_at: i64,
    pub revealed: bool,
    pub opened: Vec<char>,
    pub tries: u32,
    pub seed: i64,
    pub difficulty: Difficulty,
    pub step: usize,
    pub hint_at: i64,
    pub guess_at: i64,
    pub extend_at: i64,
    pub hints: Vec<Hint>,
    pub tier: usize,
    pub boards: Vec<Board>,
    pub song_ids: Vec<String>,
    pub hit: Vec<char>,
    pub miss: Vec<char>,
}

enum Live {
    None,
    Alive,
    Expired(Box<Round>),
}

enum LetterEnd {
    Hit(String),
    Redraw { title: String, mention: bool },
    Notice(String),
}

struct Rng64 {
    state: u64,
}

impl Rng64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed | 1,
        }
    }

    fn advance(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn unit(&mut self) -> f64 {
        (self.advance() >> 11) as f64 / ((1_u64 << 53) as f64)
    }

    fn below(&mut self, limit: usize) -> usize {
        if limit == 0 {
            return 0;
        }
        (self.advance() % limit as u64) as usize
    }
}

fn rounds(game: Game) -> &'static Mutex<HashMap<String, Round>> {
    static GUESS_ROUNDS: OnceLock<Mutex<HashMap<String, Round>>> = OnceLock::new();
    static LETTER_ROUNDS: OnceLock<Mutex<HashMap<String, Round>>> = OnceLock::new();
    static CLIP_ROUNDS: OnceLock<Mutex<HashMap<String, Round>>> = OnceLock::new();
    match game {
        Game::Guess => GUESS_ROUNDS.get_or_init(|| Mutex::new(HashMap::new())),
        Game::Letter => LETTER_ROUNDS.get_or_init(|| Mutex::new(HashMap::new())),
        Game::Clip => CLIP_ROUNDS.get_or_init(|| Mutex::new(HashMap::new())),
    }
}

fn video_hits() -> &'static Mutex<HashMap<String, Vec<i64>>> {
    static VIDEO_HITS: OnceLock<Mutex<HashMap<String, Vec<i64>>>> = OnceLock::new();
    VIDEO_HITS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn table(game: Game) -> MutexGuard<'static, HashMap<String, Round>> {
    rounds(game)
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn key_of(scene: &str, target: &str, plugin: &str) -> String {
    format!("{scene}:{target}:{plugin}")
}

pub fn invocation_key(inv: &Invocation) -> String {
    let target = if inv.target.id.is_empty() {
        inv.target.user.as_str()
    } else {
        inv.target.id.as_str()
    };
    key_of(inv.scene.as_str(), target, crate::PLUGIN_ID)
}

fn peek(game: Game, key: &str) -> Option<Round> {
    table(game).get(key).cloned()
}

fn save(game: Game, key: &str, round: Round) {
    table(game).insert(key.to_string(), round);
}

fn claim(game: Game, key: &str, round: Round) -> bool {
    let mut guard = table(game);
    if guard.contains_key(key) {
        return false;
    }
    guard.insert(key.to_string(), round);
    true
}

fn take(game: Game, key: &str) -> Option<Round> {
    table(game).remove(key)
}

fn live(game: Game, key: &str, now: i64) -> Live {
    let mut guard = table(game);
    let Some(round) = guard.get(key).cloned() else {
        return Live::None;
    };
    if now - round.start_at >= game.timeout() {
        guard.remove(key);
        return Live::Expired(Box::new(round));
    }
    Live::Alive
}

pub fn subcommands(service: &Arc<PhigrosService>) -> Vec<CommandSpec> {
    vec![
        spec(
            GUESS,
            &ALIASES_GUESS,
            "猜曲绘：抢答这段受损的曲绘是哪首歌",
            commands::on(service, run_guess),
        ),
        spec(
            LETTER,
            &ALIASES_LETTER,
            "猜字母：猜出这首曲名，猜中一个字母就翻开所有该字母",
            commands::on(service, run_letter),
        ),
        spec(
            CLIP,
            &ALIASES_CLIP,
            "看谱猜歌：凭一段无声的谱面视频猜是哪首曲子（/p clip at 可指定难度）",
            commands::on(service, run_clip),
        ),
        spec(
            VIDEO,
            &ALIASES_VIDEO,
            "看谱面视频：/p video 曲名 [难度] [位置[+秒数]]，位置可写 1:30、50%、#600、1:30-2:00、结尾",
            commands::on(service, run_video),
        ),
    ]
}

fn spec(
    name: &str,
    aliases: &[&'static str],
    describe: &str,
    handler: Arc<dyn aurorix_plugin::CommandHandler>,
) -> CommandSpec {
    let mut spec = CommandSpec::new(name).describe(describe).handler(handler);
    for alias in aliases {
        spec = spec.alias(*alias);
    }
    spec
}

async fn run_guess(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let key = invocation_key(&inv);
    let before = peek(Game::Guess, &key);
    let out = guess_command(&service, &inv);
    if round_started(Game::Guess, &key, before) {
        spawn_ticker(Arc::clone(&service), inv.clone(), Game::Guess);
    }
    commands::ok(commands::with_at(out, &inv))
}

async fn run_letter(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let key = invocation_key(&inv);
    let before = peek(Game::Letter, &key);
    let out = letter_command(&service, &inv);
    if round_started(Game::Letter, &key, before) {
        spawn_ticker(Arc::clone(&service), inv.clone(), Game::Letter);
    }
    commands::ok(commands::with_at(out, &inv))
}

async fn run_clip(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let key = invocation_key(&inv);
    let before = peek(Game::Clip, &key);
    let out = clip_command(&service, &inv);
    let out = attach_clip_video(&key, before.clone(), out).await;
    if round_started(Game::Clip, &key, before) {
        spawn_ticker(Arc::clone(&service), inv.clone(), Game::Clip);
    }
    commands::ok(commands::with_at(out, &inv))
}

fn clip_window(round: &Round) -> Option<(crate::videoclip::Entry, u64, u64)> {
    let library = crate::videoclip::library()?;
    let entry = library.lookup(round.song_id.as_str(), round.difficulty.name())?;
    let (start, length) = crate::videoclip::plan(
        round.seed,
        round.step,
        &entry,
        narrow_window(round.difficulty),
    );
    Some((entry, start, length))
}

async fn attach_clip_video(key: &str, before: Option<Round>, out: Outgoing) -> Outgoing {
    let Some(round) = before.or_else(|| peek(Game::Clip, key)) else {
        return out;
    };
    let alive = peek(Game::Clip, key).is_some();
    let final_clip = round.revealed || !alive;
    let Some((entry, start, length)) = clip_window(&round) else {
        return out;
    };
    match crate::videoclip::cut(&entry, start, length, final_clip).await {
        Ok(data) => out.video(aurorix_plugin::Media::Bytes {
            data,
            filename: String::from("clip.mp4"),
        }),
        Err(_) => out,
    }
}

async fn run_video(
    service: Arc<PhigrosService>,
    inv: Invocation,
) -> Result<Option<Outgoing>, PluginError> {
    let key = invocation_key(&inv);
    let now = crate::service::now_seconds();
    let text = inv.text.trim().to_string();
    let out = match video_rate(&key, now) {
        Err(wait) => PhigrosService::notice(format!(
            "看谱视频每会话每分钟只能看 {VIDEO_RATE_LIMIT} 次，{wait} 秒后再试"
        )),
        Ok(()) if text.is_empty() => PhigrosService::notice(VIDEO_USAGE),
        Ok(()) => match video_show(&service, &text) {
            Ok((out, entry, start, length)) => {
                match crate::videoclip::cut(&entry, start, length, true).await {
                    Ok(data) => out.video(aurorix_plugin::Media::Bytes {
                        data,
                        filename: String::from("clip.mp4"),
                    }),
                    Err(reason) => out.text(format!("\n\n> 视频没切出来：{reason}")),
                }
            }
            Err(reason) => PhigrosService::notice(reason),
        },
    };
    commands::ok(commands::with_at(out, &inv))
}

fn guess_command(service: &Arc<PhigrosService>, inv: &Invocation) -> Outgoing {
    let key = invocation_key(inv);
    let now = crate::service::now_seconds();
    let text = inv.text.trim().to_string();
    let lower = text.to_lowercase();
    match live(Game::Guess, &key, now) {
        Live::Expired(round) => return over(service, &round, HEAD_TIMEOUT),
        Live::Alive => {
            if text.is_empty() {
                return busy(Game::Guess);
            }
            if STOP_TOKENS.contains(&lower.as_str()) {
                return reveal_round(service, Game::Guess, &key, HEAD_STOP, now);
            }
            if HINT_TOKENS.contains(&lower.as_str()) {
                return hint_round(service, Game::Guess, &key, now);
            }
            return judge_round(service, Game::Guess, &key, &text, now, &inv.user);
        }
        Live::None => {}
    }
    if !text.is_empty() {
        return no_round(Game::Guess);
    }
    start(service, Game::Guess, &key, None)
}

fn letter_command(service: &Arc<PhigrosService>, inv: &Invocation) -> Outgoing {
    let key = invocation_key(inv);
    let now = crate::service::now_seconds();
    let text = inv.text.trim().to_string();
    let lower = text.to_lowercase();
    match live(Game::Letter, &key, now) {
        Live::Expired(round) => return over(service, &round, HEAD_TIMEOUT),
        Live::Alive => {
            if text.is_empty() {
                return busy(Game::Letter);
            }
            if STOP_TOKENS.contains(&lower.as_str()) {
                return reveal_round(service, Game::Letter, &key, HEAD_STOP, now);
            }
            if HINT_TOKENS.contains(&lower.as_str()) {
                return hint_round(service, Game::Letter, &key, now);
            }
            return judge_round(service, Game::Letter, &key, &text, now, &inv.user);
        }
        Live::None => {}
    }
    if !text.is_empty() {
        return no_round(Game::Letter);
    }
    start(service, Game::Letter, &key, None)
}

fn clip_command(service: &Arc<PhigrosService>, inv: &Invocation) -> Outgoing {
    let key = invocation_key(inv);
    let now = crate::service::now_seconds();
    let text = inv.text.trim().to_string();
    let lower = text.to_lowercase();
    match live(Game::Clip, &key, now) {
        Live::Expired(round) => return over(service, &round, HEAD_TIMEOUT),
        Live::Alive => {
            if text.is_empty() {
                return busy(Game::Clip);
            }
            if STOP_TOKENS.contains(&lower.as_str()) {
                return reveal_round(service, Game::Clip, &key, HEAD_STOP, now);
            }
            if EXTEND_TOKENS.contains(&lower.as_str()) {
                return extend_round(service, &key, now);
            }
            if HINT_TOKENS.contains(&lower.as_str()) {
                return hint_round(service, Game::Clip, &key, now);
            }
            return judge_round(service, Game::Clip, &key, &text, now, &inv.user);
        }
        Live::None => {}
    }
    if text.is_empty() {
        return start(service, Game::Clip, &key, None);
    }
    if parse_clip_want(&text).is_none() {
        return no_round(Game::Clip);
    }
    start(service, Game::Clip, &key, Some(&text))
}

fn start(service: &Arc<PhigrosService>, game: Game, key: &str, args: Option<&str>) -> Outgoing {
    let now = crate::service::now_seconds();
    let out = match game {
        Game::Guess => start_guess(service, key, now),
        Game::Letter => start_letter(service, key, now),
        Game::Clip => start_clip(service, key, args, now),
    };
    out.supersede(card_slot(game))
}

fn no_round(game: Game) -> Outgoing {
    PhigrosService::notice(format!(
        "这里没有正在进行的{}，发 /p {} 开一局",
        game.label(),
        game.command()
    ))
    .supersede(said_slot(game))
}

fn busy(game: Game) -> Outgoing {
    PhigrosService::notice(format!(
        "这个会话已经有一局了，先猜出来，或者发 /p {} stop 结束当前对局",
        game.command()
    ))
    .supersede(said_slot(game))
}

fn notice(game: Game, reason: impl Into<String>) -> Outgoing {
    crate::games_buttons::attach(PhigrosService::notice(reason), game, true).supersede(said_slot(game))
}

fn over(service: &Arc<PhigrosService>, round: &Round, head: &str) -> Outgoing {
    match round.game {
        Game::Guess => over_guess(service, round, head),
        Game::Letter => over_letter(service, round, head),
        Game::Clip => over_clip(service, round, head),
    }
}

fn card_slot(game: Game) -> String {
    format!("phigros:{}:card", game.command())
}

fn said_slot(game: Game) -> String {
    format!("phigros:{}:said", game.command())
}

fn round_started(game: Game, key: &str, before: Option<Round>) -> bool {
    match peek(game, key) {
        Some(after) => before.map(|round| round.seed) != Some(after.seed),
        None => false,
    }
}

fn step_interval(game: Game) -> i64 {
    match game {
        Game::Guess => GUESS_STEP_INTERVAL,
        _ => 0,
    }
}

fn step_card(service: &Arc<PhigrosService>, round: &Round, now: i64) -> Outgoing {
    let out = match round.game {
        Game::Guess => {
            let stage = stage_at(round.seed, current_step(round, now));
            let label = if stage.label.is_empty() {
                "再看一眼这张曲绘"
            } else {
                stage.label
            };
            let mut args = Args::new();
            put(&mut args, "step", label);
            let out = service
                .text("guess.step.plain", &args)
                .card(guess_render(service, round, &stage, false));
            crate::games_buttons::attach(out, Game::Guess, true)
        }
        Game::Letter => {
            let mut args = Args::new();
            put(&mut args, "headline", "看看现在开到哪了");
            put(&mut args, "opened", opened_text(round));
            put(&mut args, "board", board_text(round));
            crate::games_buttons::attach(service.text("letter.step", &args), Game::Letter, true)
        }
        Game::Clip => {
            let mut args = Args::new();
            put(&mut args, "step", clip_step_label(round));
            crate::games_buttons::attach(service.text("clip.step.plain", &args), Game::Clip, true)
        }
    };
    out.supersede(card_slot(round.game))
}

pub fn spawn_ticker(service: Arc<PhigrosService>, inv: Invocation, game: Game) {
    let Some(sink) = inv.push_sink().cloned() else {
        return;
    };
    let target = inv.target.clone();
    let key = invocation_key(&inv);
    let Some(epoch) = peek(game, &key).map(|round| round.seed) else {
        return;
    };
    tokio::spawn(async move {
        let timeout = game.timeout();
        let interval = step_interval(game);
        loop {
            let Some(round) = peek(game, &key) else {
                return;
            };
            if round.seed != epoch {
                return;
            }
            let now = crate::service::now_seconds();
            let deadline = round.start_at + timeout;
            let next = match interval {
                0 => deadline,
                step => (round.start_at + (round.step as i64 + 1) * step).min(deadline),
            };
            let wait = (next - now).max(1) as u64;
            tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
            let now = crate::service::now_seconds();
            let Some(mut round) = peek(game, &key) else {
                return;
            };
            if round.seed != epoch {
                return;
            }
            if now >= deadline {
                let _ = take(game, &key);
                round.revealed = true;
                let out = over(&service, &round, HEAD_TIMEOUT);
                let _ = sink.push(target.clone(), out).await;
                return;
            }
            if interval == 0 {
                continue;
            }
            round.step = round.step.saturating_add(1);
            save(game, &key, round.clone());
            let out = step_card(&service, &round, now);
            if sink.push(target.clone(), out).await.is_err() {
                return;
            }
        }
    });
}

fn reveal_round(
    service: &Arc<PhigrosService>,
    game: Game,
    key: &str,
    head: &str,
    now: i64,
) -> Outgoing {
    match live(game, key, now) {
        Live::Expired(round) => over(service, &round, HEAD_TIMEOUT),
        Live::None => PhigrosService::notice("这个会话没有正在进行的对局"),
        Live::Alive => match take(game, key) {
            Some(mut round) => {
                round.revealed = true;
                over(service, &round, head)
            }
            None => PhigrosService::notice("这个会话没有正在进行的对局"),
        },
    }
}

fn hint_round(service: &Arc<PhigrosService>, game: Game, key: &str, now: i64) -> Outgoing {
    match live(game, key, now) {
        Live::Expired(round) => over(service, &round, HEAD_TIMEOUT),
        Live::None => PhigrosService::notice("这一局已经结束了"),
        Live::Alive => match game {
            Game::Guess => hint_guess(service, key, now),
            Game::Letter => hint_letter(service, key, now),
            Game::Clip => hint_clip(service, key, now),
        },
    }
}

fn judge_round(
    service: &Arc<PhigrosService>,
    game: Game,
    key: &str,
    text: &str,
    now: i64,
    user: &str,
) -> Outgoing {
    match game {
        Game::Guess => judge_guess(service, key, text),
        Game::Letter => judge_letter(service, key, text, now, user),
        Game::Clip => judge_clip(service, key, text),
    }
}

fn fresh_seed() -> i64 {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let step = SEQ.fetch_add(1, Ordering::Relaxed);
    let now = crate::service::now_seconds() as u64;
    now.wrapping_mul(1_000_000_007)
        .wrapping_add(step.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as i64
}

fn code_of(seed: i64) -> String {
    format!("SIG-{:04X}", seed as u16)
}

pub fn op_plan(seed: i64) -> Vec<&'static str> {
    let mut rng = Rng64::new((seed as u64) ^ GUESS_OP_SEED);
    let mut slots: Vec<(f64, &'static str)> = Vec::with_capacity(GUESS_STEPS);
    for (op, quota) in [(OP_ZOOM, 9_usize), (OP_CLEAR, 2_usize), (OP_NOISE, 4_usize)] {
        for index in 0..quota {
            let mut position = (index as f64 + 0.5) / quota as f64;
            position += (rng.unit() - 0.5) * 2.0 * GUESS_OP_JITTER;
            slots.push((position, op));
        }
    }
    slots.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    slots.into_iter().map(|(_, op)| op).collect()
}

pub fn stage_at(seed: i64, step: usize) -> Stage {
    let plan = op_plan(seed);
    let last = step.clamp(1, GUESS_STEPS + 1);
    let mut stage = Stage {
        step: 1,
        zoom: GUESS_OPEN_ZOOM,
        level: GUESS_OPEN_LEVEL,
        noise: GUESS_OPEN_NOISE,
        label: "",
    };
    for op in plan.iter().take(last - 1) {
        match *op {
            OP_ZOOM => {
                stage.zoom = (stage.zoom * GUESS_ZOOM_STEP).min(1.0);
                stage.label = OP_ZOOM_LABEL;
            }
            OP_CLEAR => {
                stage.level = (stage.level + GUESS_LEVEL_STEP).min(GUESS_LEVEL_MAX);
                stage.label = OP_CLEAR_LABEL;
            }
            OP_NOISE => {
                stage.noise *= GUESS_NOISE_DECAY;
                stage.label = OP_NOISE_LABEL;
            }
            _ => {}
        }
    }
    stage.step = last;
    stage
}

fn current_step(round: &Round, now: i64) -> usize {
    let elapsed = (now - round.start_at).max(0);
    ((elapsed / GUESS_STEP_INTERVAL) as usize + 1).min(GUESS_STEPS + 1)
}

fn has_cover(service: &PhigrosService, song_id: &str) -> bool {
    let root = service.root();
    let key = crate::cards::illust_key(root, song_id);
    !key.is_empty() && root.join(&key).is_file()
}

fn draw_song<'a>(service: &'a PhigrosService, rng: &mut Rng64) -> Option<&'a Song> {
    let songs = service.catalog().songs();
    if songs.is_empty() {
        return None;
    }
    for _ in 0..GUESS_DRAW_ATTEMPTS {
        let song = &songs[rng.below(songs.len())];
        if !has_cover(service, &song.id) {
            continue;
        }
        return Some(song);
    }
    None
}

fn start_guess(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let seed = fresh_seed();
    let mut rng = Rng64::new(seed as u64);
    let Some(song) = draw_song(service, &mut rng) else {
        return PhigrosService::fail("曲库里抽不出带曲绘的曲子");
    };
    let variants = guess::guess_variants();
    let variant = variants[rng.below(variants.len())].to_string();
    let round = Round {
        game: Game::Guess,
        song_id: song.id.clone(),
        title: song.title.clone(),
        code: code_of(seed),
        variant,
        level: GUESS_OPEN_LEVEL,
        start_at: now,
        seed,
        ..Round::default()
    };
    if !claim(Game::Guess, key, round.clone()) {
        return busy(Game::Guess);
    }
    let stage = stage_at(round.seed, 1);
    let out = service
        .text("guess.start", &Args::new())
        .card(guess_render(service, &round, &stage, false));
    crate::games_buttons::attach(out, Game::Guess, true)
}

fn guess_render(service: &PhigrosService, round: &Round, stage: &Stage, r#final: bool) -> RenderCard {
    let level = if r#final { 1.0 } else { stage.level };
    let zoom = if r#final { 1.0 } else { stage.zoom };
    let interference = if r#final { 0.0 } else { stage.noise };
    let request = GuessRequest {
        song_id: round.song_id.clone(),
        variant: round.variant.clone(),
        level,
        zoom,
        interference,
        seed: round.seed,
        code: round.code.clone(),
        r#final,
        frame_zoom: if r#final { stage.zoom } else { 0.0 },
    };
    let (width, height) = guess::canvas_size(&request);
    let root = service.root().to_path_buf();
    let drawn = request.clone();
    service.card(width, height, "猜曲绘", &request, move |kit| {
        guess::draw(kit, &root, &drawn)
    })
}

fn hint_guess(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let Some(mut round) = peek(Game::Guess, key) else {
        return PhigrosService::notice("这一局已经结束了");
    };
    if now - round.hint_at < HINT_COOLDOWN {
        let wait = HINT_COOLDOWN - (now - round.hint_at);
        return notice(Game::Guess, format!("刚给过提示，{wait} 秒之后再来"));
    }
    let tiers = guess_hint_tiers(service, &round);
    let Some(hint) = next_hint(&tiers, &mut round.tier, round.seed) else {
        return notice(Game::Guess, "这一局没有更多提示可给了");
    };
    round.hint_at = now;
    round.hints.push(hint);
    save(Game::Guess, key, round.clone());
    let mut args = Args::new();
    put(&mut args, "hints", hint_lines(&round.hints));
    put(&mut args, "left", "0");
    crate::games_buttons::attach(service.text("guess.hint", &args), Game::Guess, true)
        .supersede(said_slot(Game::Guess))
}

fn judge_guess(service: &Arc<PhigrosService>, key: &str, text: &str) -> Outgoing {
    let Some(mut round) = peek(Game::Guess, key) else {
        return no_round(Game::Guess);
    };
    round.tries = round.tries.saturating_add(1);
    if matches_song(service.catalog(), text, &round.song_id) {
        let _ = take(Game::Guess, key);
        round.revealed = true;
        return over_guess(service, &round, HEAD_HIT);
    }
    save(Game::Guess, key, round);
    notice(Game::Guess, judge_notice(service.catalog(), text))
}

fn top_matches<'a>(hits: &[(&'a Song, i32)]) -> Vec<&'a Song> {
    let Some(best) = hits.first().map(|hit| hit.1) else {
        return Vec::new();
    };
    if best < SCORE_NEAR {
        return Vec::new();
    }
    hits.iter()
        .filter(|hit| hit.1 == best)
        .map(|hit| hit.0)
        .collect()
}

fn best_matches<'a>(catalog: &'a Catalog, text: &str) -> Vec<&'a Song> {
    top_matches(&catalog.search(text, MATCH_CANDIDATES))
}

fn matches_song(catalog: &Catalog, text: &str, song_id: &str) -> bool {
    if best_matches(catalog, text)
        .iter()
        .any(|song| song.id == song_id)
    {
        return true;
    }
    match catalog.song(song_id) {
        Some(song) => name_hits(text, song),
        None => false,
    }
}

fn judge_notice(catalog: &Catalog, text: &str) -> String {
    if catalog.resolve(text).is_some() || !catalog.search(text, 1).is_empty() {
        return String::from("不是这首，再想想");
    }
    format!("曲库里没有「{text}」这个名字，可以发 /p search 找找看")
}

fn over_guess(service: &Arc<PhigrosService>, round: &Round, head: &str) -> Outgoing {
    let now = crate::service::now_seconds();
    let stage = stage_at(round.seed, current_step(round, now));
    let mut args = Args::new();
    put(&mut args, "title", round.title.clone());
    put(&mut args, "head", head);
    put(&mut args, "elapsed", elapsed_text(now - round.start_at));
    put(&mut args, "hints", hints_text(&round.hints));
    let out = service
        .text("guess.over", &args)
        .card(guess_render(service, round, &stage, true));
    let out = crate::games_buttons::attach_over(out, Game::Guess, &round.title)
        .supersede(card_slot(Game::Guess));
    match service.catalog().song(&round.song_id) {
        Some(song) if !song.audio_url.trim().is_empty() => {
            out.voice(aurorix_plugin::Media::source(song.audio_url.clone()))
        }
        _ => out,
    }
}

fn guess_hint_tiers(service: &PhigrosService, round: &Round) -> Vec<Vec<Hint>> {
    let Some(song) = service.catalog().song(&round.song_id) else {
        return Vec::new();
    };
    let mut anchor: Vec<Hint> = Vec::new();
    if !song.chapter.is_empty() {
        anchor.push(Hint::new("所属章节", song.chapter.clone()));
    }
    let (length, _) = title_shape(song);
    if !length.is_empty() {
        anchor.push(Hint::new("曲名长度", length));
    }
    if !song.cover_colors.is_empty() {
        let colors = song
            .cover_colors
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<String>>()
            .join(" / ");
        anchor.push(Hint::new("曲绘配色", colors));
    }
    let mut feature: Vec<Hint> = Vec::new();
    let (_, initial) = title_shape(song);
    if !initial.is_empty() {
        feature.push(Hint::new("曲名开头", initial));
    }
    if !song.bpm_text.is_empty() {
        feature.push(Hint::new("BPM", song.bpm_text.clone()));
    }
    let (top_chart, top_level) = top_chart(song);
    if let Some(chart) = top_chart {
        if chart.constant > 0.0 {
            feature.push(Hint::new(
                "最高难度定数",
                format!("{} {}", top_level.name(), decimal(chart.constant)),
            ));
        }
        if chart.notes.combo() > 0 {
            feature.push(Hint::new("最高难度物量", chart.notes.combo().to_string()));
        }
    }
    let levels = levels_text(song);
    if !levels.is_empty() {
        feature.push(Hint::new("收录难度", levels));
    }
    let mut face: Vec<Hint> = Vec::new();
    if !song.composer.is_empty() {
        face.push(Hint::new("曲师", song.composer.clone()));
    }
    if !song.illustrator.is_empty() {
        face.push(Hint::new("曲绘画师", song.illustrator.clone()));
    }
    vec![anchor, feature, face]
}

fn top_chart(song: &Song) -> (Option<&crate::model::Chart>, Difficulty) {
    let mut best: Option<&crate::model::Chart> = None;
    let mut level = Difficulty::Ez;
    for difficulty in song.difficulties() {
        let Some(chart) = song.chart(difficulty) else {
            continue;
        };
        if best.map(|current| chart.constant >= current.constant).unwrap_or(true) {
            best = Some(chart);
            level = difficulty;
        }
    }
    (best, level)
}

fn levels_text(song: &Song) -> String {
    let has = song.difficulties();
    if has.is_empty() {
        return String::new();
    }
    let mut text = has
        .iter()
        .map(|difficulty| difficulty.name())
        .collect::<Vec<&str>>()
        .join(" / ");
    let missing: Vec<&str> = Difficulty::playable_all()
        .iter()
        .filter(|difficulty| song.chart(**difficulty).is_none())
        .map(|difficulty| difficulty.name())
        .collect();
    if !missing.is_empty() {
        text.push_str("，无 ");
        text.push_str(&missing.join(" / "));
    }
    text
}

fn title_shape(song: &Song) -> (String, String) {
    let raw: Vec<char> = song.title.trim().chars().collect();
    if raw.is_empty() {
        return (String::new(), String::new());
    }
    let mut count = 0;
    let mut spaced = false;
    for ch in &raw {
        if ch.is_whitespace() {
            spaced = true;
            continue;
        }
        count += 1;
    }
    let mut length = format!("{count} 个字");
    if spaced {
        length.push_str("（不含空格）");
    }
    let first = raw[0];
    let initial = if first < '\u{2E80}' {
        format!("以 {first} 开头")
    } else {
        format!("以「{first}」开头")
    };
    (length, initial)
}

fn next_hint(tiers: &[Vec<Hint>], cursor: &mut usize, seed: i64) -> Option<Hint> {
    while *cursor < tiers.len() {
        let tier = &tiers[*cursor];
        *cursor += 1;
        if tier.is_empty() {
            continue;
        }
        let index = (seed.unsigned_abs() as usize) % tier.len();
        return Some(tier[index].clone());
    }
    None
}

fn judge_clip(service: &Arc<PhigrosService>, key: &str, text: &str) -> Outgoing {
    let Some(mut round) = peek(Game::Clip, key) else {
        return no_round(Game::Clip);
    };
    round.tries = round.tries.saturating_add(1);
    if matches_song(service.catalog(), text, &round.song_id) {
        let _ = take(Game::Clip, key);
        round.revealed = true;
        return over_clip(service, &round, HEAD_HIT);
    }
    save(Game::Clip, key, round);
    notice(Game::Clip, judge_notice(service.catalog(), text))
}

fn hint_clip(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let Some(mut round) = peek(Game::Clip, key) else {
        return PhigrosService::notice("这一局已经结束了");
    };
    if now - round.hint_at < HINT_COOLDOWN {
        let wait = HINT_COOLDOWN - (now - round.hint_at);
        return notice(Game::Clip, format!("刚给过提示，{wait} 秒之后再来"));
    }
    let tiers = clip_hint_tiers(service, &round);
    let Some(hint) = next_hint(&tiers, &mut round.tier, round.seed) else {
        return notice(Game::Clip, "这一局没有更多提示可给了");
    };
    round.hint_at = now;
    round.hints.push(hint);
    save(Game::Clip, key, round.clone());
    let mut args = Args::new();
    put(&mut args, "hints", hint_lines(&round.hints));
    put(&mut args, "left", "0");
    crate::games_buttons::attach(service.text("clip.hint", &args), Game::Clip, true)
        .supersede(said_slot(Game::Clip))
}

fn clip_hint_tiers(service: &PhigrosService, round: &Round) -> Vec<Vec<Hint>> {
    let Some(song) = service.catalog().song(&round.song_id) else {
        return Vec::new();
    };
    let mut anchor: Vec<Hint> = Vec::new();
    if !song.chapter.is_empty() {
        anchor.push(Hint::new("所属章节", song.chapter.clone()));
    }
    let (length, initial) = title_shape(song);
    if !length.is_empty() {
        anchor.push(Hint::new("曲名长度", length));
    }
    let mut chart_hints: Vec<Hint> = Vec::new();
    if !initial.is_empty() {
        chart_hints.push(Hint::new("曲名开头", initial));
    }
    if let Some(chart) = song.chart(round.difficulty) {
        if chart.constant > 0.0 {
            chart_hints.push(Hint::new("定数", decimal(chart.constant)));
        }
        if chart.notes.combo() > 0 {
            chart_hints.push(Hint::new("物量", chart.notes.combo().to_string()));
        }
    }
    if !song.bpm_text.is_empty() {
        chart_hints.push(Hint::new("BPM", song.bpm_text.clone()));
    }
    let mut face: Vec<Hint> = Vec::new();
    if !song.composer.is_empty() {
        face.push(Hint::new("曲师", song.composer.clone()));
    }
    if !song.illustrator.is_empty() {
        face.push(Hint::new("曲绘画师", song.illustrator.clone()));
    }
    vec![anchor, chart_hints, face]
}

fn over_clip(service: &Arc<PhigrosService>, round: &Round, head: &str) -> Outgoing {
    let now = crate::service::now_seconds();
    let mut args = Args::new();
    put(&mut args, "head", head);
    put(&mut args, "title", round.title.clone());
    put(&mut args, "hints", hints_text(&round.hints));
    put(&mut args, "window", seconds_text(window_at(round.step, round.difficulty)));
    put(&mut args, "elapsed", elapsed_text(now - round.start_at));
    put(&mut args, "chart", round.difficulty.name());
    let mut attach_title = round.title.clone();
    if let Some(song) = service.catalog().song(&round.song_id) {
        put(&mut args, "composer", dash(&song.composer));
        put(&mut args, "illustrator", dash(&song.illustrator));
        put(&mut args, "chapter", dash(&song.chapter));
        put(&mut args, "bpm", dash(&song.bpm_text));
        put(&mut args, "length", dash(&song.length));
    } else {
        for key in ["composer", "illustrator", "chapter", "bpm", "length"] {
            put(&mut args, key, "--");
        }
        attach_title.clear();
    }
    let out = service.text("clip.over.plain", &args);
    crate::games_buttons::attach_over(out, Game::Clip, &attach_title).supersede(card_slot(Game::Clip))
}

fn extend_round(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let Some(mut round) = peek(Game::Clip, key) else {
        return no_round(Game::Clip);
    };
    if now - round.extend_at < EXTEND_COOLDOWN {
        let wait = EXTEND_COOLDOWN - (now - round.extend_at);
        return notice(Game::Clip, format!("再等 {wait} 秒才能继续放长"));
    }
    let next = round.step + 1;
    if window_at(round.step, round.difficulty) >= CLIP_MAX_WINDOW {
        return notice(Game::Clip, "这一题已经没有更多可以放的了");
    }
    round.step = next;
    round.extend_at = now;
    save(Game::Clip, key, round.clone());
    let mut args = Args::new();
    put(&mut args, "step", clip_step_label(&round));
    let template = if round.hints.is_empty() {
        "clip.step.plain"
    } else {
        "clip.step"
    };
    if !round.hints.is_empty() {
        put(&mut args, "hints", hint_lines(&round.hints));
    }
    crate::games_buttons::attach(service.text(template, &args), Game::Clip, true)
        .supersede(card_slot(Game::Clip))
}

fn clip_step_label(round: &Round) -> String {
    format!(
        "放长到 {} 秒",
        seconds_text(window_at(round.step, round.difficulty))
    )
}

pub fn window_at(step: usize, level: Difficulty) -> f64 {
    let step = step.max(1);
    let base = if narrow_window(level) {
        CLIP_TIGHT_BASE
    } else {
        CLIP_BASE_WINDOW
    };
    let window = base + (step - 1) as f64 * CLIP_WINDOW_STEP;
    if window < CLIP_MAX_WINDOW {
        window
    } else {
        CLIP_MAX_WINDOW
    }
}

fn narrow_window(level: Difficulty) -> bool {
    matches!(level, Difficulty::At | Difficulty::Sp | Difficulty::Legacy)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ClipWant {
    pub level: Option<Difficulty>,
    pub span: Option<Span>,
}

pub fn parse_clip_want(text: &str) -> Option<ClipWant> {
    let mut want = ClipWant::default();
    let mut seen = false;
    for field in text.split_whitespace() {
        let lower = field.to_lowercase();
        if want.level.is_none() {
            if let Some(level) = Difficulty::parse(&lower) {
                if level.videoable() {
                    want.level = Some(level);
                    seen = true;
                    continue;
                }
            }
        }
        if want.span.is_none() {
            if let Some(span) = parse_constant(&lower) {
                if span.on() {
                    want.span = Some(span);
                    seen = true;
                    continue;
                }
            }
        }
        return None;
    }
    if seen {
        Some(want)
    } else {
        None
    }
}

pub const TIER_ORDER: [Difficulty; 6] = [
    Difficulty::Ez,
    Difficulty::Hd,
    Difficulty::In,
    Difficulty::At,
    Difficulty::Legacy,
    Difficulty::Sp,
];

pub fn tier_quota(level: Difficulty) -> f64 {
    match level {
        Difficulty::Ez | Difficulty::Hd => 0.1,
        Difficulty::In | Difficulty::At => 47.0,
        Difficulty::Legacy | Difficulty::Sp => 3.0,
    }
}

pub fn weighted_pick(weights: &[f64], unit: f64) -> Option<usize> {
    let total: f64 = weights.iter().filter(|weight| **weight > 0.0).sum();
    if total <= 0.0 {
        return None;
    }
    let mut target = unit.clamp(0.0, 0.999_999) * total;
    for (index, weight) in weights.iter().enumerate() {
        if *weight <= 0.0 {
            continue;
        }
        if target < *weight {
            return Some(index);
        }
        target -= *weight;
    }
    weights.iter().rposition(|weight| *weight > 0.0)
}

fn clip_pool<'a>(catalog: &'a Catalog, want: &ClipWant) -> Vec<(Difficulty, Vec<&'a Song>)> {
    let mut buckets: Vec<(Difficulty, Vec<&Song>)> = TIER_ORDER
        .iter()
        .map(|level| (*level, Vec::new()))
        .collect();
    for song in catalog.songs() {
        for (level, list) in buckets.iter_mut() {
            if !level.videoable() {
                continue;
            }
            if want.level.is_some_and(|wanted| wanted != *level) {
                continue;
            }
            let Some(constant) = song.constant(*level) else {
                continue;
            };
            if want.span.is_some_and(|span| !span.contains(constant)) {
                continue;
            }
            list.push(song);
        }
    }
    buckets.retain(|(_, list)| !list.is_empty());
    buckets
}

fn start_clip(
    service: &Arc<PhigrosService>,
    key: &str,
    args: Option<&str>,
    now: i64,
) -> Outgoing {
    let want = match args {
        Some(text) => match parse_clip_want(text) {
            Some(want) => want,
            None => {
                return PhigrosService::notice(format!(
                    "看不懂「{}」这个开局条件。{}",
                    text.trim(),
                    CLIP_ARGS_USAGE
                ))
            }
        },
        None => ClipWant::default(),
    };
    let pool = clip_pool(service.catalog(), &want);
    if pool.is_empty() {
        return match args {
            Some(text) => PhigrosService::notice(format!(
                "没有「{}」这个条件下的谱面素材，换个难度或者直接发 /p clip",
                text.trim()
            )),
            None => PhigrosService::fail("曲库里抽不出能出题的谱面"),
        };
    }
    let seed = fresh_seed();
    let mut rng = Rng64::new(seed as u64);
    let weights: Vec<f64> = pool.iter().map(|(level, _)| tier_quota(*level)).collect();
    let Some(index) = weighted_pick(&weights, rng.unit()) else {
        return PhigrosService::fail("曲库里抽不出能出题的谱面");
    };
    let (level, list) = &pool[index];
    let Some(song) = list.get(rng.below(list.len())).copied() else {
        return PhigrosService::fail("曲库里抽不出能出题的谱面");
    };
    let round = Round {
        game: Game::Clip,
        song_id: song.id.clone(),
        title: song.title.clone(),
        code: code_of(seed),
        variant: level.name().to_string(),
        difficulty: *level,
        start_at: now,
        seed,
        step: 1,
        ..Round::default()
    };
    if !claim(Game::Clip, key, round.clone()) {
        return busy(Game::Clip);
    }
    let mut args = Args::new();
    put(&mut args, "window", seconds_text(window_at(1, round.difficulty)));
    crate::games_buttons::attach(service.text("clip.start", &args), Game::Clip, true)
}

fn start_letter(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let catalog = service.catalog();
    let songs = catalog.songs();
    if songs.is_empty() {
        return PhigrosService::fail("曲库里还没有曲子");
    }
    let seed = fresh_seed();
    let mut rng = Rng64::new(seed as u64);
    let want = service.settings().letter_songs.min(LETTER_SONGS_MAX);
    let mut boards: Vec<Board> = Vec::new();
    let mut song_ids: Vec<String> = Vec::new();
    let mut titles: Vec<String> = Vec::new();
    for _ in 0..want * LETTER_DRAW_ROUNDS {
        if boards.len() >= want {
            break;
        }
        let song = &songs[rng.below(songs.len())];
        if song_ids.contains(&song.id) {
            continue;
        }
        let Some(board) = board_of(&song.title) else {
            continue;
        };
        song_ids.push(song.id.clone());
        titles.push(song.title.clone());
        boards.push(board);
    }
    if boards.is_empty() {
        return PhigrosService::fail("曲库里没有能出题的曲名");
    }
    let round = Round {
        game: Game::Letter,
        song_id: String::from("letter"),
        title: titles.first().cloned().unwrap_or_default(),
        code: code_of(seed),
        variant: String::new(),
        start_at: now,
        seed,
        boards,
        song_ids,
        ..Round::default()
    };
    if !claim(Game::Letter, key, round.clone()) {
        return busy(Game::Letter);
    }
    let mut args = Args::new();
    put(&mut args, "songs", round.boards.len().to_string());
    put(&mut args, "board", board_text(&round));
    crate::games_buttons::attach(service.text("letter.start", &args), Game::Letter, true)
}

fn throttle(guess_at: &mut i64, now: i64, cooldown: i64) -> Option<i64> {
    let wait = cooldown - (now - *guess_at);
    *guess_at = now;
    if wait > 0 {
        Some(wait)
    } else {
        None
    }
}

fn judge_letter(
    service: &Arc<PhigrosService>,
    key: &str,
    text: &str,
    now: i64,
    user: &str,
) -> Outgoing {
    let Some(mut round) = peek(Game::Letter, key) else {
        return no_round(Game::Letter);
    };
    if let Some(wait) = throttle(&mut round.guess_at, now, LETTER_COOLDOWN) {
        save(Game::Letter, key, round);
        return notice(Game::Letter, format!("慢一点，{wait} 秒之后再来"));
    }
    match letter_verdict(service, &mut round, text) {
        LetterEnd::Hit(head) => {
            let _ = take(Game::Letter, key);
            round.revealed = true;
            over_letter(service, &round, &head)
        }
        LetterEnd::Redraw { title, mention } => {
            save(Game::Letter, key, round.clone());
            let mut args = Args::new();
            put(&mut args, "headline", title);
            put(&mut args, "opened", opened_text(&round));
            put(&mut args, "board", board_text(&round));
            let out = crate::games_buttons::attach(
                service.text("letter.step", &args),
                Game::Letter,
                true,
            );
            letter_reply(out.supersede(card_slot(Game::Letter)), mention, user)
        }
        LetterEnd::Notice(reason) => {
            save(Game::Letter, key, round);
            notice(Game::Letter, reason)
        }
    }
}

fn letter_reply(out: Outgoing, mention: bool, user: &str) -> Outgoing {
    if mention {
        out.mention_user(user.to_string())
    } else {
        out
    }
}

fn hint_letter(service: &Arc<PhigrosService>, key: &str, now: i64) -> Outgoing {
    let Some(mut round) = peek(Game::Letter, key) else {
        return PhigrosService::notice("这一局已经结束了");
    };
    if now - round.hint_at < LETTER_HINT_COOLDOWN {
        let wait = LETTER_HINT_COOLDOWN - (now - round.hint_at);
        return notice(Game::Letter, format!("刚给过提示，{wait} 秒之后再来"));
    }
    let Some(key_char) = next_pending(&round) else {
        return notice(Game::Letter, "没有能再翻开的字符了，剩下的得自己想");
    };
    round.hint_at = now;

    match open_verdict(&mut round, &[key_char]) {
        LetterEnd::Hit(head) => {
            let _ = take(Game::Letter, key);
            round.revealed = true;
            over_letter(service, &round, &head)
        }
        LetterEnd::Redraw { .. } => {
            let headline = format!("提示：帮你翻开了 {key_char}");
            let mut args = Args::new();
            put(&mut args, "headline", headline);
            put(&mut args, "opened", opened_text(&round));
            put(&mut args, "board", board_text(&round));
            save(Game::Letter, key, round);
            crate::games_buttons::attach(service.text("letter.step", &args), Game::Letter, true)
                .supersede(card_slot(Game::Letter))
        }
        LetterEnd::Notice(reason) => {
            save(Game::Letter, key, round);
            notice(Game::Letter, reason)
        }
    }
}

fn over_letter(service: &Arc<PhigrosService>, round: &Round, head: &str) -> Outgoing {
    let mut args = Args::new();
    put(&mut args, "head", head);
    put(&mut args, "answers", answers_text(round));
    let out = service.text("letter.over", &args);
    crate::games_buttons::attach_over(out, Game::Letter, "").supersede(card_slot(Game::Letter))
}

fn letter_verdict(service: &PhigrosService, round: &mut Round, text: &str) -> LetterEnd {
    let text = text.trim();
    let lower = text.to_lowercase();
    if let Some(rest) = cut_word(&lower, text, &LETTER_TITLE_TOKENS) {
        let title = rest.trim().to_string();
        if title.is_empty() {
            return LetterEnd::Notice(String::from(
                "点这个按钮之后要把歌名写在后面，比如 /p letter guess Crave Wave",
            ));
        }
        if let Some(end) = title_verdict(service, round, &title) {
            return end;
        }
        return LetterEnd::Notice(format!("曲库里没有「{title}」这个名字，再想想"));
    }
    if let Some(rest) = cut_word(&lower, text, &LETTER_OPEN_TOKENS) {
        let Some(keys) = resolve_keys(round, rest.trim()) else {
            return LetterEnd::Notice(format!("「{}」不是能开的字符", rest.trim()));
        };
        return open_verdict(round, &keys);
    }
    if text.chars().count() > LETTER_SHORT_INPUT {
        if let Some(end) = title_verdict(service, round, text) {
            return end;
        }
    }
    if let Some(keys) = resolve_keys(round, text) {
        return open_verdict(round, &keys);
    }
    LetterEnd::Notice(format!("「{text}」既不是能开的字符，也不是曲库里的歌名"))
}

fn cut_word<'a>(lower: &str, raw: &'a str, words: &[&str]) -> Option<&'a str> {
    let head = lower.split_whitespace().next()?;
    if !words.contains(&head) {
        return None;
    }
    let offset = raw
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(index, _)| index)
        .unwrap_or(0);
    let rest = &raw[offset..];
    let cut = rest.find(char::is_whitespace).unwrap_or(rest.len());
    Some(rest[cut..].trim_start())
}

fn split_index(query: &str) -> (usize, String) {
    let text = query.trim();
    let (head, tail) = if let Some(rest) = text.strip_prefix('#') {
        (rest.to_string(), "")
    } else if let Some(rest) = text.strip_prefix('第') {
        (rest.to_string(), "首")
    } else {
        return (0, query.to_string());
    };
    let digits = head.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if digits == 0 {
        return (0, query.to_string());
    }
    let Ok(number) = head[..digits].parse::<usize>() else {
        return (0, query.to_string());
    };
    if number == 0 {
        return (0, query.to_string());
    }
    let mut rest = head[digits..].to_string();
    if tail == "首" {
        rest = rest.strip_prefix('首').unwrap_or(&rest).to_string();
    }
    rest = rest
        .trim_start_matches([' ', '.', '。', '、', '个'])
        .to_string();
    (number, rest.trim().to_string())
}

fn title_verdict(service: &PhigrosService, round: &mut Round, query: &str) -> Option<LetterEnd> {
    let (want, rest) = split_index(query);
    let matches = best_matches(service.catalog(), &rest);
    if matches.is_empty() {
        return None;
    }
    let index = round
        .song_ids
        .iter()
        .position(|id| matches.iter().any(|song| song.id == *id))
        .or_else(|| {
            round.boards.iter().position(|board| {
                matches
                    .iter()
                    .any(|song| normalize_title(&board.title) == normalize_title(&song.title))
            })
        })
        .map(|index| index as i64)
        .unwrap_or(-1);
    if want > 0 {
        if want > round.boards.len() {
            return Some(LetterEnd::Notice(format!(
                "一共就 {} 首，没有第 {want} 首",
                round.boards.len()
            )));
        }
        if index != want as i64 - 1 {
            return Some(LetterEnd::Notice(format!(
                "第 {want} 首不是「{rest}」，再想想"
            )));
        }
    }
    if index < 0 {
        return Some(LetterEnd::Notice(format!("「{rest}」不是这一局的曲子")));
    }
    let position = index as usize;
    if round.boards[position].solved {
        return Some(LetterEnd::Notice(format!("第 {} 首早就猜出来了", position + 1)));
    }
    round.boards[position].solved = true;
    if round_solved(round) {
        return Some(LetterEnd::Hit(String::from("全部猜出来了")));
    }
    Some(LetterEnd::Redraw {
        title: format!("第 {} 首猜中了", position + 1),
        mention: true,
    })
}

fn open_verdict(round: &mut Round, keys: &[char]) -> LetterEnd {
    let mut opened = 0_usize;
    let mut hit: Vec<char> = Vec::new();
    let mut miss: Vec<char> = Vec::new();
    let mut given: Vec<char> = Vec::new();
    let mut repeat: Vec<char> = Vec::new();
    for key in keys {
        if round.hit.contains(key) || round.miss.contains(key) {
            push_unique(&mut repeat, *key);
            continue;
        }
        if board_given_key(round, *key) {
            push_unique(&mut given, *key);
            continue;
        }
        let (count, accepted) = open_key(round, *key);
        if !accepted {
            push_unique(&mut miss, *key);
            round.miss.push(*key);
            continue;
        }
        push_unique(&mut hit, *key);
        round.hit.push(*key);
        opened += count;
    }
    if opened > 0 {
        for key in &hit {
            push_unique(&mut round.opened, *key);
        }
        if round_solved(round) {
            return LetterEnd::Hit(String::from("全部翻开了"));
        }
        return LetterEnd::Redraw {
            title: format!("翻开了 {opened} 格"),
            mention: false,
        };
    }
    if !hit.is_empty() {
        return LetterEnd::Notice(format!("{} 的位置早就开了", join_chars(&hit)));
    }
    if !given.is_empty() {
        return LetterEnd::Notice(format!(
            "{} 题面上本来就摆着，不用开",
            join_chars(&given)
        ));
    }
    if !miss.is_empty() {
        return LetterEnd::Notice(format!(
            "这几首里没有 {} 这个字符",
            join_chars(&miss)
        ));
    }
    if !repeat.is_empty() {
        return LetterEnd::Notice("这些都开过了，换一个".to_string());
    }
    LetterEnd::Notice(String::from("这些都开过了，换一个"))
}

fn join_chars(list: &[char]) -> String {
    list.iter().map(|ch| ch.to_string()).collect::<Vec<String>>().join(" ")
}

fn push_unique(list: &mut Vec<char>, value: char) {
    if !list.contains(&value) {
        list.push(value);
    }
}

fn round_solved(round: &Round) -> bool {
    !round.boards.is_empty() && round.boards.iter().all(|board| board.solved || board_solved(board))
}

fn board_solved(board: &Board) -> bool {
    board
        .pieces
        .iter()
        .all(|piece| piece.given || piece.open)
}

fn open_key(round: &mut Round, key: char) -> (usize, bool) {
    let mut opened = 0;
    let mut accepted = false;
    for board in round.boards.iter_mut() {
        if board.solved {
            continue;
        }
        for piece in board.pieces.iter_mut() {
            if piece.given || !piece.keys.contains(&key) {
                continue;
            }
            accepted = true;
            if piece.open {
                continue;
            }
            piece.open = true;
            opened += 1;
        }
    }
    (opened, accepted)
}

fn board_given_key(round: &Round, key: char) -> bool {
    let mut found = false;
    for board in &round.boards {
        for piece in &board.pieces {
            if !piece.keys.contains(&key) {
                continue;
            }
            if !piece.given {
                return false;
            }
            found = true;
        }
    }
    found
}

fn next_pending(round: &Round) -> Option<char> {
    let mut keys: Vec<char> = Vec::new();
    for board in &round.boards {
        if board.solved {
            continue;
        }
        for piece in &board.pieces {
            if piece.given || piece.open {
                continue;
            }
            for key in &piece.keys {
                push_unique(&mut keys, *key);
            }
        }
    }
    keys.sort_unstable();
    keys.into_iter().next()
}

fn resolve_keys(round: &Round, text: &str) -> Option<Vec<char>> {
    let mut out: Vec<char> = Vec::new();
    let mut tokens = 0;
    for token in text.split_whitespace() {
        tokens += 1;
        let lower = token.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        if chars.is_empty() {
            return None;
        }
        if chars.len() == 1 && chars[0].is_ascii_alphanumeric() {
            push_unique(&mut out, chars[0]);
            continue;
        }
        let head = lower.strip_suffix('行').unwrap_or(lower.as_str());
        if let Some(kana) = letters::kana_row(head) {
            if board_has_key(round, kana) || board_given_key(round, kana) {
                push_unique(&mut out, kana);
                continue;
            }
        }
        if chars.len() == 1 {
            let ch = chars[0];
            if board_has_key(round, ch) || board_given_key(round, ch) {
                push_unique(&mut out, ch);
                continue;
            }
            return None;
        }
        if chars.iter().all(|ch| ch.is_ascii_alphanumeric()) {
            for ch in chars {
                push_unique(&mut out, ch);
            }
            continue;
        }
        if chars
            .iter()
            .all(|ch| board_has_key(round, *ch) || board_given_key(round, *ch))
        {
            for ch in chars {
                push_unique(&mut out, ch);
            }
            continue;
        }
        return None;
    }
    if tokens == 0 || out.is_empty() {
        return None;
    }
    Some(out)
}

fn board_has_key(round: &Round, key: char) -> bool {
    round
        .boards
        .iter()
        .flat_map(|board| board.pieces.iter())
        .any(|piece| piece.keys.contains(&key))
}

pub fn board_of(title: &str) -> Option<Board> {
    let text = title.replace(['\u{a0}', '\u{3000}'], " ");
    let mut board = Board {
        title: text.clone(),
        ..Board::default()
    };
    let mut guessable = false;
    for ch in text.chars() {
        let keys = letters::keys_of(ch);
        let given = keys.is_empty() || !letters::is_letter_like(ch);
        if !given {
            guessable = true;
        }
        board.pieces.push(Piece {
            text: ch,
            keys,
            given,
            open: false,
        });
    }
    if !guessable {
        return None;
    }
    Some(board)
}

fn board_line(board: &Board) -> String {
    if board.solved {
        return board.title.clone();
    }
    let cells: Vec<String> = board
        .pieces
        .iter()
        .map(|piece| {
            if !piece.given && !piece.open {
                return String::from(PLACEHOLDER);
            }
            if piece.text == ' ' {
                return String::from(" ");
            }
            piece.text.to_string()
        })
        .collect();
    cells.join(" ")
}

fn board_text(round: &Round) -> String {
    if round.boards.is_empty() {
        return String::new();
    }
    let lines: Vec<String> = round
        .boards
        .iter()
        .enumerate()
        .map(|(index, board)| {
            let mut line = board_line(board);
            if board.solved || board_solved(board) {
                line.push_str(" ✅");
            }
            format!("{}. {line}", index + 1)
        })
        .collect();
    lines.join("\n")
}

fn opened_text(round: &Round) -> String {
    if round.opened.is_empty() {
        return String::from("已开字母 [ ]");
    }
    format!("已开字母 [ {} ]", join_chars(&round.opened))
}

fn answers_text(round: &Round) -> String {
    round
        .boards
        .iter()
        .enumerate()
        .map(|(index, board)| format!("{}. {}", index + 1, board.title))
        .collect::<Vec<String>>()
        .join("\n")
}

fn hint_lines(hints: &[Hint]) -> String {
    let width = hints
        .iter()
        .map(|hint| display_width(&hint.label))
        .max()
        .unwrap_or(0);
    hints
        .iter()
        .map(|hint| {
            format!(
                "{}{}{}",
                hint.label,
                " ".repeat(width - display_width(&hint.label) + 2),
                hint.text
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn hints_text(hints: &[Hint]) -> String {
    if hints.is_empty() {
        return String::from("没用提示");
    }
    hints
        .iter()
        .map(|hint| format!("{} {}", hint.label, hint.text))
        .collect::<Vec<String>>()
        .join(" · ")
}

fn display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| {
            if ('\u{2E80}'..='\u{FF60}').contains(&ch) {
                2
            } else {
                1
            }
        })
        .sum()
}

fn elapsed_text(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds < 60 {
        return format!("{seconds} 秒");
    }
    format!("{} 分 {} 秒", seconds / 60, seconds % 60)
}

fn decimal(value: f64) -> String {
    format!("{value:.1}")
}

fn dash(text: &str) -> String {
    if text.trim().is_empty() {
        String::from("--")
    } else {
        text.to_string()
    }
}

fn pad(value: i64) -> String {
    if value < 10 {
        format!("0{value}")
    } else {
        value.to_string()
    }
}

pub fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0) as i64;
    if total < 3600 {
        return format!("{}:{}", total / 60, pad(total % 60));
    }
    format!("{}:{}:{}", total / 3600, pad(total % 3600 / 60), pad(total % 60))
}

pub fn seconds_text(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PosKind {
    #[default]
    None,
    Time,
    Percent,
    Note,
    Head,
    Tail,
    Middle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub kind: PosKind,
    pub seconds: f64,
    pub percent: f64,
    pub note: i32,
}

impl Position {
    pub fn known(self) -> bool {
        self.kind != PosKind::None
    }

    fn of(kind: PosKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    fn time(seconds: f64) -> Self {
        Self {
            kind: PosKind::Time,
            seconds,
            ..Self::default()
        }
    }

    fn percent(value: f64) -> Self {
        Self {
            kind: PosKind::Percent,
            percent: value,
            ..Self::default()
        }
    }

    fn note(value: i32) -> Self {
        Self {
            kind: PosKind::Note,
            note: value,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct VideoQuery {
    pub song: String,
    pub level: Option<Difficulty>,
    pub start: Position,
    pub end: Position,
    pub length: f64,
    pub length_set: bool,
}

pub fn parse_video_query(text: &str) -> Result<VideoQuery, String> {
    let mut fields = split_glued_plus(text.split_whitespace().map(String::from).collect());
    if fields.is_empty() {
        return Err(String::from(VIDEO_USAGE));
    }
    let mut query = VideoQuery::default();
    while fields.len() > 1 {
        let last = fields.last().cloned().unwrap_or_default();
        let lower = last.to_lowercase();
        if query.level.is_none() {
            if let Some(level) = Difficulty::parse(&lower) {
                if level.videoable() {
                    query.level = Some(level);
                    fields.pop();
                    continue;
                }
            }
        }
        if let Some((start, end)) = parse_range(&last) {
            query.start = start;
            query.end = end;
            fields.pop();
            continue;
        }
        if !query.length_set {
            if let Some(length) = parse_length(&last) {
                query.length = length;
                query.length_set = true;
                fields.pop();
                continue;
            }
        }
        if let Some(position) = parse_position(&last) {
            query.start = position;
            fields.pop();
            continue;
        }
        break;
    }
    query.song = fields.join(" ").trim().to_string();
    if query.song.is_empty() {
        return Err(String::from(VIDEO_USAGE));
    }
    Ok(query)
}

fn split_glued_plus(fields: Vec<String>) -> Vec<String> {
    if fields.len() < 2 {
        return fields;
    }
    let Some(last) = fields.last() else {
        return fields;
    };
    let Some((head, plus)) = last.split_once('+') else {
        return fields;
    };
    if head.is_empty() || plus.is_empty() {
        return fields;
    }
    if parse_length(&format!("+{plus}")).is_none() {
        return fields;
    }
    if parse_position(head).is_none() && parse_range(head).is_none() {
        return fields;
    }
    let mut out: Vec<String> = fields[..fields.len() - 1].to_vec();
    out.push(head.to_string());
    out.push(format!("+{plus}"));
    out
}

pub fn parse_position(token: &str) -> Option<Position> {
    let lower = token.to_lowercase();
    match lower.as_str() {
        "start" | "head" | "开头" | "从头" | "前奏" => return Some(Position::of(PosKind::Head)),
        "end" | "tail" | "结尾" | "尾杀" | "最后" => return Some(Position::of(PosKind::Tail)),
        "mid" | "middle" | "中间" | "中段" | "一半" => return Some(Position::of(PosKind::Middle)),
        _ => {}
    }
    if let Some(value) = parse_percent(token) {
        return Some(Position::percent(value));
    }
    if let Some(value) = parse_note(token) {
        return Some(Position::note(value));
    }
    if let Some(value) = parse_clock(token) {
        return Some(Position::time(value));
    }
    if let Some(value) = parse_bare(token) {
        return Some(Position::time(value));
    }
    None
}

fn parse_range(token: &str) -> Option<(Position, Position)> {
    let cut = token
        .char_indices()
        .skip(1)
        .find(|(_, ch)| *ch == '-')
        .map(|(index, _)| index)?;
    let left = &token[..cut];
    let right = &token[cut + 1..];
    if left.is_empty() || right.is_empty() {
        return None;
    }
    if let (Some(start), Some(end)) = (parse_note(left), parse_note(right)) {
        return Some((Position::note(start), Position::note(end)));
    }
    if let (Some(start), Some(end)) = (parse_loose(left), parse_loose(right)) {
        return Some((Position::time(start), Position::time(end)));
    }
    None
}

fn parse_loose(token: &str) -> Option<f64> {
    parse_clock(token).or_else(|| parse_bare(token))
}

fn digits(text: &str) -> Option<(&str, &str)> {
    let count = text.chars().take_while(|ch| ch.is_ascii_digit()).count();
    if count == 0 {
        return None;
    }
    Some(text.split_at(count))
}

fn parse_percent(token: &str) -> Option<f64> {
    let body = token.strip_suffix('%')?;
    percent_of(body)
}

fn percent_of(body: &str) -> Option<f64> {
    let (head, tail) = match body.split_once('.') {
        Some((head, tail)) => (head, Some(tail)),
        None => (body, None),
    };
    if head.is_empty() || head.len() > 3 || !head.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if let Some(tail) = tail {
        if tail.is_empty() || !tail.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
    }
    let value: f64 = body.parse().ok()?;
    if (0.0..=100.0).contains(&value) {
        Some(value / 100.0)
    } else {
        None
    }
}

fn parse_note(token: &str) -> Option<i32> {
    let lower = token.to_lowercase();
    let body = if let Some(rest) = lower.strip_prefix('#') {
        rest
    } else if let Some(rest) = lower.strip_prefix("物量") {
        rest
    } else if let Some(rest) = lower.strip_prefix("note") {
        rest
    } else {
        lower.strip_suffix('n')?
    };
    let body = body.strip_suffix('n').unwrap_or(body);
    let (head, tail) = digits(body)?;
    if !tail.is_empty() || head.len() > 5 {
        return None;
    }
    let value: i32 = head.parse().ok()?;
    if value > 0 {
        Some(value)
    } else {
        None
    }
}

fn parse_bare(token: &str) -> Option<f64> {
    let (head, tail) = digits(token)?;
    if !tail.is_empty() || head.len() > 4 {
        return None;
    }
    head.parse::<f64>().ok()
}

fn parse_clock(token: &str) -> Option<f64> {
    let lower = token.to_lowercase();
    if let Some((left, right)) = lower.split_once(':') {
        if left.is_empty() || right.is_empty() || left.len() > 3 || right.len() > 2 {
            return None;
        }
        if !left.chars().all(|ch| ch.is_ascii_digit()) || !right.chars().all(|ch| ch.is_ascii_digit())
        {
            return None;
        }
        let minutes: f64 = left.parse().ok()?;
        let seconds: f64 = right.parse().ok()?;
        if seconds >= 60.0 {
            return None;
        }
        return Some(minutes * 60.0 + seconds);
    }
    if let Some((left, right)) = lower.split_once('m') {
        if left.is_empty() || left.len() > 3 || !left.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let right = right.strip_suffix('s').unwrap_or(right);
        if right.is_empty() || right.len() > 2 || !right.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let minutes: f64 = left.parse().ok()?;
        let seconds: f64 = right.parse().ok()?;
        return Some(minutes * 60.0 + seconds);
    }
    if let Some(body) = lower.strip_suffix('s') {
        let (head, tail) = digits(body)?;
        if !tail.is_empty() || head.len() > 4 {
            return None;
        }
        return head.parse().ok();
    }
    None
}

fn parse_length(token: &str) -> Option<f64> {
    let body = token.strip_prefix('+')?;
    let (head, tail) = digits(body)?;
    if !tail.is_empty() || head.len() > 3 {
        return None;
    }
    let value: f64 = head.parse().ok()?;
    if value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn video_anchor(position: Position, duration: f64, length: f64) -> Result<f64, String> {
    match position.kind {
        PosKind::Time => Ok(position.seconds),
        PosKind::Percent => Ok(duration * position.percent),
        PosKind::Head | PosKind::None => Ok(0.0),
        PosKind::Tail => Ok(duration - length),
        PosKind::Middle => Ok((duration - length) / 2.0),
        PosKind::Note => Err(String::from(
            "这一档暂时不能按物量定位，换 1:30 或 50% 这类说法",
        )),
    }
}

pub fn video_plan(query: &VideoQuery, duration: f64) -> Result<(f64, f64, bool), String> {
    if !query.start.known() && !query.end.known() {
        if !query.length_set || query.length >= duration {
            return Ok((0.0, duration, true));
        }
        if query.length < CLIP_KEYFRAME {
            return Err(String::from("这一段太短了，多给几秒"));
        }
        return Ok((0.0, query.length, false));
    }
    if query.start.known() && query.end.known() {
        let head = video_anchor(query.start, duration, 0.0)?;
        let tail = video_anchor(query.end, duration, 0.0)?;
        if tail <= head {
            return Err(String::from("起点要在终点前面"));
        }
        return Ok((head, tail - head, false));
    }
    let spot = if query.start.known() {
        query.start
    } else {
        query.end
    };
    let mut length = query.length;
    if spot.kind == PosKind::Tail && length == 0.0 {
        length = VIDEO_TAIL_WINDOW;
    }
    let mut start = video_anchor(spot, duration, length)?;
    if start < 0.0 {
        start = 0.0;
    }
    if start >= duration {
        start = (duration - VIDEO_TAIL_WINDOW).max(0.0);
    }
    if length == 0.0 {
        length = duration - start;
    }
    if start + length > duration {
        length = duration - start;
    }
    if length < CLIP_KEYFRAME {
        return Err(String::from("这一段太短了，换个位置试试"));
    }
    Ok((start, length, false))
}

fn video_song<'a>(service: &'a PhigrosService, text: &str) -> Result<&'a Song, String> {
    let catalog = service.catalog();
    if let Some(song) = catalog.resolve(text) {
        return Ok(song);
    }
    let hits = catalog.search(text, 3);
    if hits.len() > 1 {
        let names: Vec<String> = hits
            .iter()
            .take(3)
            .map(|(song, _)| format!("《{}》", song.title))
            .collect();
        return Err(format!("这几首都对得上：{}，写全一点", names.join(" ")));
    }
    Err(format!("曲库里没有「{text}」，发 /p search 找找看"))
}

fn video_level(song: &Song, level: Option<Difficulty>) -> Result<Difficulty, String> {
    if let Some(level) = level {
        if song.chart(level).is_some() {
            return Ok(level);
        }
        return Err(format!("《{}》的 {} 还没有谱面", song.title, level.name()));
    }
    for level in [
        Difficulty::In,
        Difficulty::At,
        Difficulty::Sp,
        Difficulty::Hd,
        Difficulty::Ez,
        Difficulty::Legacy,
    ] {
        if song.chart(level).is_some() {
            return Ok(level);
        }
    }
    Err(format!("《{}》还没有任何难度的谱面", song.title))
}

fn video_show(
    service: &PhigrosService,
    text: &str,
) -> Result<(Outgoing, crate::videoclip::Entry, u64, u64), String> {
    let query = parse_video_query(text)?;
    let song = video_song(service, &query.song)?;
    let level = video_level(song, query.level)?;
    let entry = match crate::videoclip::library()
        .and_then(|library| library.lookup(song.id.as_str(), level.name()))
    {
        Some(entry) => entry,
        None => return Err(format!("《{}》的 {} 还没有谱面视频", song.title, level.name())),
    };
    let duration = entry.duration_ms as f64 / 1000.0;
    let (start, length, whole) = video_plan(&query, duration)?;
    let notes = song
        .chart(level)
        .map(|chart| chart.notes.combo())
        .unwrap_or(0);
    let mut args = Args::new();
    put(&mut args, "title", song.title.clone());
    put(&mut args, "level", level.name());
    put(
        &mut args,
        "where",
        if whole {
            String::from("整条")
        } else {
            format!("{} 起 {}", clock(start), clock(length))
        },
    );
    put(&mut args, "notes", notes.to_string());
    put(&mut args, "length", clock(length));
    Ok((
        crate::games_buttons::attach_video(service.text("video.info", &args)),
        entry,
        (start * 1000.0).round() as u64,
        (length * 1000.0).round() as u64,
    ))
}

pub fn rate_tick(hits: &mut Vec<i64>, now: i64, limit: usize, window: i64) -> Result<(), i64> {
    hits.retain(|at| now - *at < window);
    if limit == 0 || window <= 0 {
        return Ok(());
    }
    if hits.len() >= limit {
        let first = hits.first().copied().unwrap_or(now);
        return Err((first + window - now).max(1));
    }
    hits.push(now);
    Ok(())
}

fn video_rate(key: &str, now: i64) -> Result<(), i64> {
    let mut guard = video_hits()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let hits = guard.entry(key.to_string()).or_default();
    rate_tick(hits, now, VIDEO_RATE_LIMIT, VIDEO_RATE_WINDOW)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces_text(board: &Board) -> Vec<(char, Vec<char>, bool)> {
        board
            .pieces
            .iter()
            .map(|piece| (piece.text, piece.keys.clone(), piece.given))
            .collect()
    }

    #[test]
    fn stage_plan_keeps_go_quotas_and_caps() {
        let seed = 0x1234_5678_i64;
        let plan = op_plan(seed);
        assert_eq!(plan.len(), GUESS_STEPS);
        assert_eq!(plan.iter().filter(|op| **op == OP_ZOOM).count(), 9);
        assert_eq!(plan.iter().filter(|op| **op == OP_CLEAR).count(), 2);
        assert_eq!(plan.iter().filter(|op| **op == OP_NOISE).count(), 4);
        let opening = stage_at(seed, 1);
        assert!((opening.zoom - GUESS_OPEN_ZOOM).abs() < f64::EPSILON);
        assert!((opening.level - GUESS_OPEN_LEVEL).abs() < f64::EPSILON);
        assert!((opening.noise - GUESS_OPEN_NOISE).abs() < f64::EPSILON);
        assert!(opening.label.is_empty());
        let last = stage_at(seed, GUESS_STEPS + 1);
        assert!(last.zoom > 0.9 && last.zoom <= 1.0);
        assert!(last.level <= GUESS_LEVEL_MAX);
        assert!(last.noise >= GUESS_NOISE_FLOOR);
        assert_eq!(stage_at(seed, 99).step, GUESS_STEPS + 1);
        assert_eq!(GUESS_STEPS, 9 + 4 + 2);
    }

    #[test]
    fn letter_board_masks_letters_and_opens_every_cell() {
        let board = board_of("A-1 光").expect("有可猜的格子");
        assert_eq!(
            pieces_text(&board),
            vec![
                ('A', vec!['a'], false),
                ('-', vec!['-'], true),
                ('1', vec!['1'], false),
                (' ', Vec::new(), true),
                ('光', vec!['a', 'g', 'n', 'u'], false),
            ]
        );
        assert!(board_line(&board).contains(PLACEHOLDER));
        assert!(board_of("-- ..").is_none());
        let mut round = Round {
            boards: vec![board, board_of("ab a").expect("第二首")],
            ..Round::default()
        };
        let count = open_key(&mut round, 'a');
        assert_eq!(count, (4, true));
        assert!(!round_solved(&round));
        assert_eq!(open_key(&mut round, 'a'), (0, true));
        assert!(board_given_key(&round, '-'));
        assert!(!board_given_key(&round, 'a'));
        round.boards[0].solved = true;
        let line = board_line(&round.boards[0]);
        assert_eq!(line, "A-1 光");
    }

    #[test]
    fn clip_window_matches_go_schedule() {
        assert!((window_at(1, Difficulty::In) - 4.0).abs() < f64::EPSILON);
        assert!((window_at(1, Difficulty::At) - 2.0).abs() < f64::EPSILON);
        assert!((window_at(3, Difficulty::At) - 8.0).abs() < f64::EPSILON);
        assert!((window_at(3, Difficulty::Ez) - 10.0).abs() < f64::EPSILON);
        assert!((window_at(40, Difficulty::In) - CLIP_MAX_WINDOW).abs() < f64::EPSILON);
        assert!((window_at(0, Difficulty::Legacy) - 2.0).abs() < f64::EPSILON);
        assert!((tier_quota(Difficulty::In) - 47.0).abs() < f64::EPSILON);
        assert_eq!(weighted_pick(&[1.0, 3.0], 0.0), Some(0));
        assert_eq!(weighted_pick(&[1.0, 3.0], 0.99), Some(1));
        assert_eq!(weighted_pick(&[0.0], 0.5), None);
        assert_eq!(parse_clip_want("at").map(|want| want.level), Some(Some(Difficulty::At)));
        assert!(parse_clip_want("15+").is_some());
        assert!(parse_clip_want("曲名").is_none());
    }

    #[test]
    fn video_rate_allows_two_per_minute() {
        let mut hits: Vec<i64> = Vec::new();
        assert_eq!(rate_tick(&mut hits, 1_000, 2, 60), Ok(()));
        assert_eq!(rate_tick(&mut hits, 1_010, 2, 60), Ok(()));
        assert_eq!(rate_tick(&mut hits, 1_020, 2, 60), Err(40));
        assert_eq!(rate_tick(&mut hits, 1_061, 2, 60), Ok(()));
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn video_positions_parse_like_go() {
        let query = parse_video_query("风屿 1:30+30").expect("粘连的长度要拆开");
        assert_eq!(query.song, "风屿");
        assert_eq!(query.start, Position::time(90.0));
        assert!((query.length - 30.0).abs() < f64::EPSILON);
        assert!(query.length_set);
        let half = parse_video_query("风屿 50%").expect("百分比");
        assert!((half.start.percent - 0.5).abs() < f64::EPSILON);
        let range = parse_video_query("风屿 IN 1:30-2:00").expect("区间");
        assert_eq!(range.level, Some(Difficulty::In));
        assert_eq!(range.start, Position::time(90.0));
        assert_eq!(range.end, Position::time(120.0));
        let tail = parse_video_query("风屿 结尾").expect("结尾");
        assert_eq!(tail.start.kind, PosKind::Tail);
        let note = parse_video_query("风屿 #600").expect("物量");
        assert_eq!(note.start.kind, PosKind::Note);
        assert!(parse_video_query("  ").is_err());
        assert_eq!(clock(90.0), "1:30");
        assert_eq!(seconds_text(4.0), "4");
        assert_eq!(seconds_text(2.5), "2.5");
        let duration = 150.0;
        assert_eq!(
            video_plan(&parse_video_query("风屿").expect("整条"), duration),
            Ok((0.0, 150.0, true))
        );
        assert!(video_plan(&parse_video_query("风屿 #600").expect("物量"), duration).is_err());
    }

    #[test]
    fn commands_expose_go_names_and_aliases() {
        assert_eq!(ALIASES_GUESS, ["猜曲绘", "猜歌", "guesssong"]);
        assert_eq!(ALIASES_LETTER, ["猜字母", "猜歌名", "lettergame"]);
        assert_eq!(ALIASES_CLIP, ["猜曲目", "看谱猜歌", "guessclip", "vg"]);
        assert_eq!(ALIASES_VIDEO, ["视频", "看片段", "vid", "peek"]);
        let names = [GUESS, LETTER, CLIP, VIDEO];
        assert_eq!(names.len(), 4);
        assert_eq!(Game::from_command(CLIP), Some(Game::Clip));
        assert_eq!(Game::from_command(VIDEO), None);
        assert_eq!(Game::Guess.timeout(), 165);
        assert_eq!(Game::Letter.timeout(), 900);
        assert_eq!(Game::Clip.timeout(), 300);
        assert_eq!(key_of("group", "g1", "phigros"), "group:g1:phigros");
    }

    #[test]
    fn expired_round_is_revealed_once_and_releases_the_slot() {
        let key = "group:g-expire:phigros";
        save(
            Game::Letter,
            key,
            Round {
                game: Game::Letter,
                start_at: 1_000,
                ..Round::default()
            },
        );
        assert!(matches!(
            live(Game::Letter, key, 1_000 + LETTER_TIMEOUT - 1),
            Live::Alive
        ));
        assert!(matches!(
            live(Game::Letter, key, 1_000 + LETTER_TIMEOUT),
            Live::Expired(_)
        ));
        assert!(matches!(
            live(Game::Letter, key, 1_000 + LETTER_TIMEOUT + 1),
            Live::None
        ));
    }

    #[test]
    fn solved_round_releases_the_slot() {
        let key = "group:g-solved:phigros";
        save(
            Game::Guess,
            key,
            Round {
                game: Game::Guess,
                start_at: 10,
                ..Round::default()
            },
        );
        assert!(take(Game::Guess, key).is_some());
        assert!(matches!(live(Game::Guess, key, 11), Live::None));
    }

    #[test]
    fn a_second_start_in_the_same_chat_is_refused() {
        let key = "group:g-claim:phigros";
        assert!(claim(
            Game::Guess,
            key,
            Round {
                game: Game::Guess,
                ..Round::default()
            }
        ));
        assert!(!claim(
            Game::Guess,
            key,
            Round {
                game: Game::Guess,
                ..Round::default()
            }
        ));
    }

    #[test]
    fn accepted_key_that_opens_nothing_is_not_a_miss() {
        let mut round = Round {
            boards: vec![board_of("光 a").expect("有可猜的格子")],
            ..Round::default()
        };
        assert!(matches!(
            open_verdict(&mut round, &['g']),
            LetterEnd::Redraw { .. }
        ));
        match open_verdict(&mut round, &['u']) {
            LetterEnd::Notice(text) => assert_eq!(text, "u 的位置早就开了"),
            _ => panic!("多音字的第二个读音应当说位置早就开了"),
        }
        match open_verdict(&mut round, &['g']) {
            LetterEnd::Notice(text) => assert_eq!(text, "这些都开过了，换一个"),
            _ => panic!("开过的字符应当说开过了"),
        }
        assert_eq!(open_key(&mut round, 'z'), (0, false));
    }

    #[test]
    fn katakana_and_dakuten_resolve_to_their_row() {
        let round = Round {
            boards: vec![board_of("か").expect("有可猜的格子")],
            ..Round::default()
        };
        assert_eq!(resolve_keys(&round, "カ"), Some(vec!['か']));
        assert_eq!(resolve_keys(&round, "が"), Some(vec!['か']));
        assert_eq!(resolve_keys(&round, "ka"), Some(vec!['か']));
        assert_eq!(resolve_keys(&round, "光"), None);
    }

    #[test]
    fn tied_best_matches_all_count_as_an_answer() {
        let first = Song {
            id: String::from("s1"),
            title: String::from("Crave Wave"),
            ..Song::default()
        };
        let second = Song {
            id: String::from("s2"),
            title: String::from("Crave Wave II"),
            ..Song::default()
        };
        let other = Song {
            id: String::from("s3"),
            title: String::from("Spasmodic"),
            ..Song::default()
        };
        let hits = vec![
            (&first, SCORE_NEAR + 100),
            (&second, SCORE_NEAR + 100),
            (&other, SCORE_NEAR - 9),
        ];
        let ids: Vec<&str> = top_matches(&hits)
            .iter()
            .map(|song| song.id.as_str())
            .collect();
        assert_eq!(ids, vec!["s1", "s2"]);
        assert!(top_matches(&[(&other, SCORE_NEAR - 1)]).is_empty());
    }

    #[test]
    fn letter_cooldown_restarts_after_a_blocked_attempt() {
        let mut guess_at = 100;
        assert_eq!(throttle(&mut guess_at, 101, LETTER_COOLDOWN), Some(2));
        assert_eq!(guess_at, 101);
        assert_eq!(throttle(&mut guess_at, 102, LETTER_COOLDOWN), Some(2));
        assert_eq!(guess_at, 102);
        assert_eq!(throttle(&mut guess_at, 105, LETTER_COOLDOWN), None);
        assert_eq!(guess_at, 105);
    }

    #[test]
    fn letter_hit_mentions_the_guesser_not_the_starter() {
        let out = letter_reply(PhigrosService::notice("猜中了"), true, "guesser");
        assert_eq!(out.mention_user.as_deref(), Some("guesser"));
        let silent = letter_reply(PhigrosService::notice("公布了"), false, "guesser");
        assert_eq!(silent.mention_user, None);
    }
}
