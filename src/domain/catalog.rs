use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use crate::model::{Chapter, Chart, ConstChange, ExtraChart, Notes, Song};
use crate::repo::AliasEntry;

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("曲库打不开：{0}")]
    Open(String),
    #[error("曲库读取失败：{0}")]
    Sql(String),
    #[error("曲库为空")]
    Empty,
}

pub const SCORE_EXACT: i32 = 1000;
pub const SCORE_PREFIX: i32 = 800;
pub const SCORE_CONTAINS: i32 = 600;
pub const SCORE_SUBSEQUENCE: i32 = 400;
pub const SCORE_NEAR: i32 = 250;

pub struct Catalog {
    songs: Vec<Song>,
    by_id: HashMap<String, usize>,
    chapters: Vec<Chapter>,
    index: RwLock<Vec<(usize, String)>>,
    aliases: RwLock<HashMap<String, Vec<String>>>,
    game_version: i32,
}

impl Catalog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let path = path.as_ref();
        let connection = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|error| CatalogError::Open(error.to_string()))?;
        Self::read(&connection)
    }

    fn read(connection: &rusqlite::Connection) -> Result<Self, CatalogError> {
        let mut orders: HashMap<String, i64> = HashMap::new();
        let mut songs: Vec<Song> = Vec::new();
        let mut by_id: HashMap<String, usize> = HashMap::new();
        let mut statement = connection
            .prepare(
                "SELECT song_id, difficulty, song_ord, title, composer, illustrator, cover_luma, cover_colors, bpm, bpm_min, bpm_max, length, chapter, version, version_label, original, sp, variant_of, constant, charter, notes_tap, notes_drag, notes_hold, notes_flick, duration, history, audio_url, illurl, wikiurl FROM charts",
            )
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, i64>(15)?,
                    row.get::<_, String>(16)?,
                    row.get::<_, String>(17)?,
                    row.get::<_, f64>(18)?,
                    row.get::<_, String>(19)?,
                    row.get::<_, Option<i64>>(20)?,
                    row.get::<_, Option<i64>>(21)?,
                    row.get::<_, Option<i64>>(22)?,
                    row.get::<_, Option<i64>>(23)?,
                    row.get::<_, Option<f64>>(24)?,
                    row.get::<_, String>(25)?,
                    row.get::<_, Option<String>>(26)?,
                    row.get::<_, Option<String>>(27)?,
                    row.get::<_, Option<String>>(28)?,
                ))
            })
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        for row in rows {
            let row = row.map_err(|error| CatalogError::Sql(error.to_string()))?;
            let (song_id, difficulty, song_ord, title, composer, illustrator, cover_luma, cover_colors, bpm, bpm_min, bpm_max, length, chapter, version, version_label, original, sp, variant_of, constant, charter, tap, drag, hold, flick, duration, history, audio, ill, wiki) = row;
            let difficulty = parse_difficulty(&difficulty);
            let position = match by_id.get(&song_id) {
                Some(index) => *index,
                None => {
                    let index = songs.len();
                    by_id.insert(song_id.clone(), index);
                    orders.insert(song_id.clone(), song_ord);
                    songs.push(Song {
                        id: song_id.clone(),
                        title,
                        composer,
                        illustrator,
                        cover_luma: cover_luma.unwrap_or(0) as i32,
                        cover_colors: cover_colors
                            .split([',', '|'])
                            .filter(|item| !item.is_empty())
                            .map(|item| item.trim().to_string())
                            .collect(),
                        bpm: bpm_max as i32,
                        bpm_text: if bpm.is_empty() {
                            if bpm_min == bpm_max {
                                bpm_max.to_string()
                            } else {
                                format!("{bpm_min}~{bpm_max}")
                            }
                        } else {
                            bpm
                        },
                        length,
                        chapter,
                        version: version as i32,
                        version_label,
                        original: original != 0,
                        sp,
                        aliases: Vec::new(),
                        charts: std::array::from_fn(|_| Chart::default()),
                        extra: Vec::new(),
                        variant_of,
                        audio_url: audio.unwrap_or_default(),
                        ill_url: ill.unwrap_or_default(),
                        wiki_url: wiki.unwrap_or_default(),
                    });
                    index
                }
            };
            if let Some(difficulty) = difficulty {
                songs[position].charts[difficulty.index()] = Chart {
                    difficulty,
                    constant,
                    charter,
                    notes: Notes {
                        tap: tap.unwrap_or(0) as i32,
                        drag: drag.unwrap_or(0) as i32,
                        hold: hold.unwrap_or(0) as i32,
                        flick: flick.unwrap_or(0) as i32,
                    },
                    duration: duration.unwrap_or(0.0),
                    history: parse_history(&history),
                };
            }
        }
        if songs.is_empty() {
            return Err(CatalogError::Empty);
        }
        for song in &mut songs {
            song.charts
                .iter_mut()
                .enumerate()
                .for_each(|(index, chart)| {
                    if chart.difficulty.index() != index {
                        chart.difficulty = crate::model::Difficulty::from_index(index)
                            .unwrap_or(crate::model::Difficulty::Ez);
                    }
                });
        }

        let mut statement = connection
            .prepare(
                "SELECT song_id, kind, label, charter, notes_tap, notes_drag, notes_hold, notes_flick FROM extra_charts",
            )
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        let extra_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                ))
            })
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        for row in extra_rows {
            let (song_id, kind, label, charter, tap, drag, hold, flick) =
                row.map_err(|error| CatalogError::Sql(error.to_string()))?;
            if let Some(index) = by_id.get(&song_id) {
                songs[*index].extra.push(ExtraChart {
                    kind,
                    label,
                    charter,
                    notes: Notes {
                        tap: tap.unwrap_or(0) as i32,
                        drag: drag.unwrap_or(0) as i32,
                        hold: hold.unwrap_or(0) as i32,
                        flick: flick.unwrap_or(0) as i32,
                    },
                });
            }
        }

        let mut chapters: Vec<Chapter> = Vec::new();
        let mut statement = connection
            .prepare("SELECT name, display FROM chapters ORDER BY ord")
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        let chapter_rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        for row in chapter_rows {
            let (name, display) =
                row.map_err(|error| CatalogError::Sql(error.to_string()))?;
            chapters.push(Chapter {
                name,
                display,
                aliases: Vec::new(),
            });
        }

        let mut statement = connection
            .prepare("SELECT alias, kind, target FROM aliases ORDER BY ord")
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        let alias_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| CatalogError::Sql(error.to_string()))?;
        for row in alias_rows {
            let (alias, kind, target) =
                row.map_err(|error| CatalogError::Sql(error.to_string()))?;
            if kind == "chapter" {
                if let Some(chapter) = chapters.iter_mut().find(|item| item.name == target) {
                    chapter.aliases.push(alias);
                }
            } else if let Some(index) = by_id.get(&target) {
                songs[*index].aliases.push(alias);
            }
        }

        songs.sort_by_key(|song| orders.get(&song.id).copied().unwrap_or(0));
        by_id.clear();
        let mut index: Vec<(usize, String)> = Vec::new();
        for (position, song) in songs.iter().enumerate() {
            by_id.insert(song.id.clone(), position);
            index.push((position, normalize_title(&song.title)));
            for alias in &song.aliases {
                index.push((position, normalize_title(alias)));
            }
        }
        let game_version = connection
            .query_row(
                "SELECT value FROM meta WHERE key = 'game_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|value| value.trim().parse::<i32>().ok())
            .unwrap_or(0);
        Ok(Self {
            songs,
            by_id,
            chapters,
            game_version,
            index: RwLock::new(index),
            aliases: RwLock::new(HashMap::new()),
        })
    }

    pub fn songs(&self) -> &[Song] {
        &self.songs
    }

    pub fn game_version(&self) -> i32 {
        self.game_version
    }

    pub fn song(&self, id: &str) -> Option<&Song> {
        self.by_id.get(id).map(|index| &self.songs[*index])
    }

    pub fn chapters(&self) -> &[Chapter] {
        &self.chapters
    }

    pub fn chapter_display(&self, name: &str) -> String {
        self.chapters
            .iter()
            .find(|chapter| chapter.name == name)
            .map(|chapter| chapter.display.clone())
            .unwrap_or_else(|| name.to_string())
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<(&Song, i32)> {
        let query = normalize_title(query);
        if query.is_empty() {
            return Vec::new();
        }
        let mut hits: HashMap<usize, i32> = HashMap::new();
        let index = self
            .index
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for (position, name) in index.iter() {
            let score = name_score(&query, name);
            if score <= 0 {
                continue;
            }
            let entry = hits.entry(*position).or_insert(0);
            if score > *entry {
                *entry = score;
            }
        }
        drop(index);
        let mut ranked: Vec<(&Song, i32)> = hits
            .into_iter()
            .map(|(position, score)| (&self.songs[position], score))
            .collect();
        ranked.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.title.cmp(&b.0.title))
                .then_with(|| a.0.id.cmp(&b.0.id))
        });
        ranked.truncate(limit.max(1));
        ranked
    }

    pub fn set_aliases(&self, entries: &[AliasEntry]) {
        let mut overrides: HashMap<String, Vec<String>> = HashMap::new();
        for song in &self.songs {
            let mut names: Vec<String> = song.aliases.clone();
            let mut touched = false;
            for entry in entries.iter().filter(|entry| entry.song_id == song.id) {
                let alias = entry.alias.trim().to_string();
                if alias.is_empty() {
                    continue;
                }
                if entry.removed {
                    let before = names.len();
                    names.retain(|name| name != &alias);
                    touched |= names.len() != before;
                } else if !names.contains(&alias) {
                    names.push(alias);
                    touched = true;
                }
            }
            if touched {
                overrides.insert(song.id.clone(), names);
            }
        }
        let mut index: Vec<(usize, String)> = Vec::new();
        for (position, song) in self.songs.iter().enumerate() {
            index.push((position, normalize_title(&song.title)));
            let names = overrides.get(&song.id).unwrap_or(&song.aliases);
            for alias in names {
                index.push((position, normalize_title(alias)));
            }
        }
        {
            let mut guard = self
                .aliases
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = overrides;
        }
        let mut guard = self
            .index
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = index;
    }

    pub fn resolve(&self, query: &str) -> Option<&Song> {
        let query = normalize_title(query);
        let mut best: Option<(&Song, i32)> = None;
        let overrides = self
            .aliases
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for song in &self.songs {
            let mut score = name_score(&query, &normalize_title(&song.title));
            let names = overrides.get(&song.id).unwrap_or(&song.aliases);
            for alias in names {
                score = score.max(name_score(&query, &normalize_title(alias)));
            }
            if score > best.map(|item| item.1).unwrap_or(0) {
                best = Some((song, score));
            }
        }
        drop(overrides);
        best.filter(|item| item.1 >= SCORE_CONTAINS)
            .map(|item| item.0)
    }
}

pub fn normalize_title(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|ch| !matches!(ch, ' ' | '\t' | '\n' | '\r' | '-' | '_' | '　' | '.' | ',' | '\'' | '’'))
        .collect()
}

pub fn name_score(query: &str, target: &str) -> i32 {
    let query_len = query.chars().count();
    let target_len = target.chars().count();
    if query.is_empty() || target.is_empty() {
        return 0;
    }
    if query == target {
        return SCORE_EXACT;
    }
    if target.starts_with(query) {
        return SCORE_PREFIX - target_len.min(99) as i32;
    }
    if query.starts_with(target) {
        return SCORE_PREFIX - 20 - target_len.min(99) as i32;
    }
    if target.contains(query) {
        return SCORE_CONTAINS - target_len.min(99) as i32;
    }
    if let Some(gap) = subsequence_gap(query, target) {
        return SCORE_SUBSEQUENCE - gap.min(99);
    }
    let limit = near_limit(query_len.max(target_len));
    if let Some(distance) = bounded_edit_distance(query, target, limit) {
        return (SCORE_NEAR - distance).max(1);
    }
    0
}

pub fn near_limit(len: usize) -> usize {
    match len {
        0..=4 => 1,
        5..=9 => 2,
        _ => (len / 3).min(5),
    }
}

fn bounded_edit_distance(query: &str, target: &str, limit: usize) -> Option<i32> {
    let left: Vec<char> = query.chars().collect();
    let right: Vec<char> = target.chars().collect();
    let (rows, columns) = (left.len(), right.len());
    if rows.abs_diff(columns) > limit {
        return None;
    }
    let mut before: Vec<usize> = Vec::new();
    let mut previous: Vec<usize> = (0..=columns).collect();
    for row in 1..=rows {
        let mut current = vec![row; columns + 1];
        let mut best = current[0];
        for column in 1..=columns {
            let cost = usize::from(left[row - 1] != right[column - 1]);
            let mut value = (previous[column] + 1)
                .min(current[column - 1] + 1)
                .min(previous[column - 1] + cost);
            if row > 1
                && column > 1
                && left[row - 1] == right[column - 2]
                && left[row - 2] == right[column - 1]
            {
                value = value.min(before[column - 2] + 1);
            }
            current[column] = value;
            best = best.min(value);
        }
        if best > limit {
            return None;
        }
        before = previous;
        previous = current;
    }
    let distance = previous[columns];
    (distance <= limit).then_some(distance as i32)
}

pub fn name_hits(query: &str, song: &Song) -> bool {
    let query = normalize_title(query);
    if query.chars().count() < 2 {
        return false;
    }
    let hit = |name: &str| {
        let target = normalize_title(name);
        !target.is_empty() && target.starts_with(&query)
    };
    if hit(&song.title) {
        return true;
    }
    song.aliases.iter().any(|alias| hit(alias))
}

pub fn subsequence_gap(query: &str, target: &str) -> Option<i32> {
    let query: Vec<char> = query.chars().collect();
    let target: Vec<char> = target.chars().collect();
    let mut position = 0;
    for ch in &target {
        if position < query.len() && *ch == query[position] {
            position += 1;
        }
    }
    if position == query.len() {
        Some((target.len() - query.len()) as i32)
    } else {
        None
    }
}

fn parse_difficulty(text: &str) -> Option<crate::model::Difficulty> {
    if text.is_empty() {
        return None;
    }
    crate::model::Difficulty::parse(text)
}

fn parse_history(raw: &str) -> Vec<ConstChange> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Vec::new();
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    if let Some(items) = value.as_array() {
        for item in items {
            let version = item
                .get("version")
                .or_else(|| item.get("v"))
                .and_then(|inner| inner.as_i64())
                .unwrap_or(0) as i32;
            let from = item
                .get("from")
                .or_else(|| item.get("old"))
                .and_then(|inner| inner.as_f64())
                .unwrap_or(0.0);
            let to = item
                .get("to")
                .or_else(|| item.get("new"))
                .and_then(|inner| inner.as_f64())
                .unwrap_or(0.0);
            let version_label = item
                .get("versionLabel")
                .or_else(|| item.get("label"))
                .and_then(|inner| inner.as_str())
                .unwrap_or("")
                .to_string();
            result.push(ConstChange {
                version,
                version_label,
                from,
                to,
            });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Song {
        Song {
            id: String::from("song"),
            title: String::from("Spasmodic"),
            aliases: vec![String::from("痉挛")],
            ..Song::default()
        }
    }

    #[test]
    fn normalization_drops_go_separators() {
        assert_eq!(normalize_title("Spas-Modic_1.0"), "spasmodic10");
        assert_eq!(normalize_title("  Stasis\t"), "stasis");
        assert_eq!(normalize_title("ＡＢ"), "ａｂ");
    }

    #[test]
    fn name_score_matches_go_ladder() {
        assert_eq!(name_score("spasmodic", "spasmodic"), SCORE_EXACT);
        assert_eq!(name_score("spas", "spasmodic"), SCORE_PREFIX - 9);
        assert_eq!(name_score("spasmodicx", "spasmodic"), SCORE_PREFIX - 20 - 9);
        assert_eq!(name_score("asmod", "spasmodic"), SCORE_CONTAINS - 9);
        assert!(name_score("spmd", "spasmodic") > 0);
        assert_eq!(name_score("zzzz", "spasmodic"), 0);
        assert_eq!(name_score("", "spasmodic"), 0);
    }

    #[test]
    fn name_hits_needs_two_chars_and_prefix() {
        let song = sample();
        assert!(name_hits("spas", &song));
        assert!(name_hits("痉挛", &song));
        assert!(!name_hits("s", &song));
        assert!(!name_hits("modic", &song));
    }

    #[test]
    fn subsequence_gap_counts_skipped_chars() {
        assert_eq!(subsequence_gap("spm", "spasmodic"), Some(6));
        assert_eq!(subsequence_gap("zzz", "spasmodic"), None);
    }

    fn real_catalog() -> Catalog {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let primary = root.join("assets/data/phi.db");
        let sibling = root.join("assets/../data/phi.db");
        let path = if primary.exists() { primary } else { sibling };
        Catalog::open(path).unwrap()
    }

    #[test]
    fn real_phi_db_reads_songs_chapters_and_alias() {
        let started = std::time::Instant::now();
        let catalog = real_catalog();
        assert_eq!(catalog.songs().len(), 349);
        assert_eq!(catalog.chapters().len(), 35);
        let id = "Spasmodic.姜米條颶風元力上人.0";
        let song = catalog.song(id).unwrap();
        assert_eq!(song.title, "Spasmodic");
        assert_eq!(song.composer, "姜米條☆颶風 ~ ♫元力上人♫");
        assert_eq!(song.chapter, "Chapter 5 霓虹灯牌");
        assert_eq!(song.charts[0].constant, 8.2);
        assert_eq!(song.charts[1].constant, 12.9);
        assert_eq!(song.charts[2].constant, 15.5);
        assert_eq!(song.charts[3].constant, 16.7);
        assert_eq!(song.charts[2].notes.combo(), 747 + 495 + 119 + 28);
        assert_eq!(song.charts[2].duration, 150.0);
        assert_eq!(catalog.chapter_display("Chapter 5 霓虹灯牌"), "霓虹灯牌");
        let hits = catalog.search("Spasmodic", 8);
        assert!(hits.iter().any(|(hit, score)| {
            hit.id == id && *score >= SCORE_PREFIX - 99
        }));
        let resolved = catalog.resolve("痉挛").unwrap();
        assert_eq!(resolved.id, id);
        assert!(started.elapsed() < std::time::Duration::from_millis(500));
    }
}
