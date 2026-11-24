use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde::Deserialize;

pub const KEYFRAME_MS: u64 = 500;

const MIN_LENGTH_MS: u64 = 500;
const CACHE_BYTES: usize = 256 << 20;
const CUT_TIMEOUT: Duration = Duration::from_secs(20);
const MANIFEST: &str = "manifest.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub song_id: String,
    pub level: String,
    pub path: PathBuf,
    pub duration_ms: u64,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    #[serde(default)]
    file: String,
    #[serde(default)]
    level: String,
    #[serde(default)]
    duration_seconds: f64,
}

pub struct Library {
    index: HashMap<(String, String), Entry>,
}

impl Library {
    fn open(root: &Path) -> Option<Self> {
        let raw = std::fs::read(root.join(MANIFEST)).ok()?;
        let doc: HashMap<String, ManifestEntry> = serde_json::from_slice(&raw).ok()?;
        let mut index: HashMap<(String, String), Entry> = HashMap::new();
        for item in doc.values() {
            if item.file.is_empty() || item.level.is_empty() || item.duration_seconds <= 0.0 {
                continue;
            }
            let Some(song_id) = song_of(item.file.as_str()) else {
                continue;
            };
            let entry = Entry {
                song_id: song_id.clone(),
                level: item.level.clone(),
                path: root.join(item.file.replace('/', "/")),
                duration_ms: (item.duration_seconds * 1000.0).round() as u64,
            };
            index.insert((song_id, item.level.clone()), entry);
        }
        (!index.is_empty()).then_some(Self { index })
    }

    pub fn lookup(&self, song_id: &str, level: &str) -> Option<Entry> {
        if let Some(entry) = self.index.get(&(song_id.to_string(), level.to_string())) {
            return Some(entry.clone());
        }
        self.index
            .values()
            .find(|entry| {
                entry.level == level
                    && (song_id.starts_with(entry.song_id.as_str())
                        || entry.song_id.starts_with(song_id))
            })
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

pub fn library() -> Option<&'static Library> {
    static LIBRARY: OnceLock<Option<Library>> = OnceLock::new();
    LIBRARY
        .get_or_init(|| Library::open(&crate::assets_root().join("video")))
        .as_ref()
}

fn song_of(file: &str) -> Option<String> {
    let name = Path::new(file).file_stem()?.to_str()?;
    let cut = name.rfind('_')?;
    (cut > 0).then(|| name[..cut].to_string())
}

struct Cache {
    slots: HashMap<String, Vec<u8>>,
    order: Vec<String>,
    bytes: usize,
}

impl Cache {
    fn new() -> Self {
        Self {
            slots: HashMap::new(),
            order: Vec::new(),
            bytes: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        let hit = self.slots.get(key).cloned()?;
        self.order.retain(|item| item != key);
        self.order.push(key.to_string());
        Some(hit)
    }

    fn put(&mut self, key: String, data: Vec<u8>) {
        if data.len() > CACHE_BYTES {
            return;
        }
        if let Some(previous) = self.slots.remove(&key) {
            self.bytes = self.bytes.saturating_sub(previous.len());
            self.order.retain(|item| item != &key);
        }
        self.bytes += data.len();
        self.slots.insert(key.clone(), data);
        self.order.push(key);
        while self.bytes > CACHE_BYTES {
            let Some(oldest) = self.order.first().cloned() else {
                break;
            };
            self.order.remove(0);
            if let Some(dropped) = self.slots.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(dropped.len());
            }
        }
    }
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(Cache::new()))
}

pub const LEAD_IN_MS: u64 = 12_000;

pub const TAIL_MARGIN_MS: u64 = 2_000;

pub const MAX_WINDOW_MS: u64 = 30_000;

pub fn window_at(step: usize, narrow: bool) -> u64 {
    let step = step.max(1) as u64;
    let base = match narrow {
        true => 2_000,
        false => 4_000,
    };
    (base + (step - 1) * 3_000).min(MAX_WINDOW_MS)
}

pub fn plan(seed: i64, step: usize, entry: &Entry, narrow: bool) -> (u64, u64) {
    let window = window_at(step, narrow);
    let room = entry
        .duration_ms
        .saturating_sub(LEAD_IN_MS + TAIL_MARGIN_MS + MAX_WINDOW_MS);
    let start = match room {
        0 => LEAD_IN_MS,
        _ => {
            let slots = room / KEYFRAME_MS;
            let offset = mix(seed as u64) % (slots + 1) * KEYFRAME_MS;
            LEAD_IN_MS + offset
        }
    };
    (start, window)
}

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

pub async fn cut(
    entry: &Entry,
    start_ms: u64,
    length_ms: u64,
    audio: bool,
) -> Result<Vec<u8>, String> {
    let start = start_ms / KEYFRAME_MS * KEYFRAME_MS;
    if start >= entry.duration_ms || length_ms < MIN_LENGTH_MS {
        return Err(String::from("这一段已经越过视频末尾"));
    }
    let room = entry.duration_ms - start;
    let length = match length_ms > room {
        true => room / KEYFRAME_MS * KEYFRAME_MS,
        false => length_ms,
    };
    if length < MIN_LENGTH_MS {
        return Err(String::from("这一段太短了，多给几秒"));
    }
    let key = format!("{}|{start}|{length}|{audio}", entry.path.display());
    if let Ok(mut cache) = cache().lock() {
        if let Some(hit) = cache.get(&key) {
            return Ok(hit);
        }
    }
    let data = run(entry, start, length, audio).await?;
    if let Ok(mut cache) = cache().lock() {
        cache.put(key, data.clone());
    }
    Ok(data)
}

async fn run(entry: &Entry, start: u64, length: u64, audio: bool) -> Result<Vec<u8>, String> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!(
        "aurorix-clip-{}-{stamp}.mp4",
        std::process::id()
    ));
    let mut command = tokio::process::Command::new("ffmpeg");
    command
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-nostdin")
        .arg("-y")
        .arg("-ss")
        .arg(seconds(start))
        .arg("-t")
        .arg(seconds(length))
        .arg("-i")
        .arg(entry.path.as_os_str())
        .arg("-map")
        .arg("0:v:0");
    if audio {
        command.arg("-map").arg("0:a:0?");
    } else {
        command.arg("-an");
    }
    command
        .arg("-c")
        .arg("copy")
        .arg("-avoid_negative_ts")
        .arg("make_zero")
        .arg("-movflags")
        .arg("+faststart")
        .arg("-f")
        .arg("mp4")
        .arg(path.as_os_str())
        .kill_on_drop(true);
    let outcome = tokio::time::timeout(CUT_TIMEOUT, command.output()).await;
    let result = match outcome {
        Ok(Ok(output)) => match output.status.success() {
            true => Ok(()),
            false => Err(format!(
                "切片失败: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )),
        },
        Ok(Err(error)) => Err(format!("切片失败: {error}")),
        Err(_) => Err(String::from("切片超时")),
    };
    let data = result.and_then(|()| {
        std::fs::read(&path).map_err(|error| format!("读切片失败: {error}"))
    });
    let _ = std::fs::remove_file(&path);
    let data = data?;
    if data.is_empty() {
        return Err(String::from("切出空片段"));
    }
    Ok(data)
}

fn seconds(millis: u64) -> String {
    format!("{:.3}", millis as f64 / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_of_reads_the_stem_before_the_level() {
        assert_eq!(
            song_of("IN/000AinSophAur.Yumeji.0_IN.mp4").as_deref(),
            Some("000AinSophAur.Yumeji.0")
        );
        assert_eq!(song_of("novideo.mp4"), None);
    }

    #[test]
    fn cut_api_snaps_to_the_keyframe_grid() {
        let entry = Entry {
            song_id: String::from("s"),
            level: String::from("IN"),
            path: PathBuf::from("/tmp/none.mp4"),
            duration_ms: 3_140,
        };
        let start = 1_260u64 / KEYFRAME_MS * KEYFRAME_MS;
        assert_eq!(start, 1_000);
        let room = entry.duration_ms - start;
        assert_eq!(room / KEYFRAME_MS * KEYFRAME_MS, 2_000);
    }

    #[test]
    fn cache_drops_the_oldest_entry_when_full() {
        let mut cache = Cache::new();
        cache.put(String::from("a"), vec![1u8; 8]);
        cache.put(String::from("b"), vec![2u8; 8]);
        assert!(cache.get("a").is_some());
        cache.put(String::from("c"), vec![3u8; 8]);
        assert_eq!(cache.bytes, 24);
    }
}
