use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use aurorix_render::skia_safe::Image;
use aurorix_render::{CanvasKit, Color, SharedImage, Texture};

static TEXTURE_TABLE: OnceLock<Mutex<HashMap<String, Arc<Texture>>>> = OnceLock::new();

static IMAGE_TABLE: OnceLock<Mutex<HashMap<String, Arc<SharedImage>>>> = OnceLock::new();

fn texture_guard() -> MutexGuard<'static, HashMap<String, Arc<Texture>>> {
    TEXTURE_TABLE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn texture(kit: &CanvasKit<'_>, key: &str) -> Option<Arc<Texture>> {
    if let Some(hit) = texture_guard().get(key).cloned() {
        return Some(hit);
    }
    let texture = kit.asset(key).ok()?;
    let mut table = texture_guard();
    let entry = table
        .entry(key.to_string())
        .or_insert_with(|| Arc::clone(&texture))
        .clone();
    Some(entry)
}

fn image_guard() -> MutexGuard<'static, HashMap<String, Arc<SharedImage>>> {
    IMAGE_TABLE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn image(key: &str) -> Option<Arc<SharedImage>> {
    image_guard().get(key).cloned()
}

pub fn keep_image(key: &str, image: Image) -> Arc<SharedImage> {
    let mut table = image_guard();
    table
        .entry(key.to_string())
        .or_insert_with(|| Arc::new(SharedImage::from_image(image)))
        .clone()
}

pub fn image_cached(key: &str) -> bool {
    image_guard().contains_key(key)
}

pub fn texture_cached(key: &str) -> bool {
    texture_guard().contains_key(key)
}

pub fn cache_len() -> usize {
    texture_guard().len()
}

pub const GRADE_TABLE: [(&str, &str); 10] = [
    ("F", "grade_f"),
    ("f", "grade_f"),
    ("C", "grade_c"),
    ("c", "grade_c"),
    ("B", "grade_b"),
    ("b", "grade_b"),
    ("A", "grade_a"),
    ("a", "grade_a"),
    ("S", "grade_s"),
    ("s", "grade_s"),
];

pub const GRADE_EXT_TABLE: [(&str, &str); 5] = [
    ("V", "grade_v"),
    ("v", "grade_v"),
    ("FC", "grade_fc"),
    ("fc", "grade_fc"),
    ("phi", "grade_phi"),
];

pub fn grade_name(grade: &str) -> Option<&'static str> {
    if let Some((_, name)) = GRADE_TABLE.iter().find(|(label, _)| *label == grade) {
        return Some(name);
    }
    if let Some((_, name)) = GRADE_EXT_TABLE.iter().find(|(label, _)| *label == grade) {
        return Some(name);
    }
    match grade {
        "Phi" | "PHI" => Some("grade_phi"),
        "FC" | "Fc" => Some("grade_fc"),
        _ => None,
    }
}

pub const RANK_BADGE_TABLE: [&str; 6] = [
    "rank_white",
    "rank_green",
    "rank_blue",
    "rank_red",
    "rank_gold",
    "rank_colorful",
];

pub fn rank_badge_name(level: i32) -> &'static str {
    RANK_BADGE_TABLE[level.clamp(0, RANK_BADGE_TABLE.len() as i32 - 1) as usize]
}

pub const DIFFICULTY_COLORS: [(&str, Color); 4] = [
    ("EZ", Color::new(0xA0, 0xE0, 0x57, 0xFF)),
    ("HD", Color::new(0x45, 0xE0, 0xA8, 0xFF)),
    ("IN", Color::new(0x7F, 0xA3, 0xD4, 0xFF)),
    ("AT", Color::new(0xFF, 0x5F, 0x68, 0xFF)),
];

pub const DIFFICULTY_NEUTRAL: Color = Color::new(0xB9, 0xC0, 0xCA, 0xFF);

pub fn difficulty_chip(label: &str) -> Color {
    DIFFICULTY_COLORS
        .iter()
        .find(|(name, _)| *name == label)
        .map(|(_, ink)| *ink)
        .unwrap_or(DIFFICULTY_NEUTRAL)
}

pub const DEFAULT_AVATAR: &str = "bar_avatar_default";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_name_covers_eight_tiers() {
        for grade in ["F", "C", "B", "A", "S", "V", "FC", "PHI"] {
            assert!(grade_name(grade).is_some(), "评级 {grade} 必须能落到素材名");
        }
        assert_eq!(grade_name("PHI"), Some("grade_phi"));
        assert_eq!(grade_name("FC"), Some("grade_fc"));
        assert_eq!(grade_name("X"), None);
    }

    #[test]
    fn rank_badge_clamps_six_tiers() {
        assert_eq!(rank_badge_name(0), "rank_white");
        assert_eq!(rank_badge_name(5), "rank_colorful");
        assert_eq!(rank_badge_name(-3), "rank_white");
        assert_eq!(rank_badge_name(9), "rank_colorful");
    }

    #[test]
    fn difficulty_chip_has_four_colors() {
        assert_eq!(difficulty_chip("AT"), Color::new(0xFF, 0x5F, 0x68, 0xFF));
        assert_eq!(difficulty_chip("IN"), Color::new(0x7F, 0xA3, 0xD4, 0xFF));
        assert_eq!(difficulty_chip("SP"), DIFFICULTY_NEUTRAL);
    }
}
