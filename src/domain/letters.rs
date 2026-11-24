use std::collections::HashMap;
use std::sync::OnceLock;

use super::letters_table::{FOLD_TABLE, PINYIN_GROUPS};

const KANA_ROWS: [(&str, char); 11] = [
    ("あいうえお", 'あ'),
    ("かきくけこ", 'か'),
    ("さしすせそ", 'さ'),
    ("たちつてと", 'た'),
    ("なにぬねの", 'な'),
    ("はひふへほ", 'は'),
    ("まみむめも", 'ま'),
    ("やゆよ", 'や'),
    ("らりるれろ", 'ら'),
    ("わを", 'わ'),
    ("ん", 'ん'),
];

const KANA_ALIAS: [(&str, char); 20] = [
    ("a", 'あ'),
    ("i", 'あ'),
    ("u", 'あ'),
    ("e", 'あ'),
    ("o", 'あ'),
    ("ka", 'か'),
    ("sa", 'さ'),
    ("ta", 'た'),
    ("na", 'な'),
    ("ha", 'は'),
    ("ma", 'ま'),
    ("ya", 'や'),
    ("ra", 'ら'),
    ("wa", 'わ'),
    ("n", 'ん'),
    ("nn", 'ん'),
    ("shi", 'さ'),
    ("chi", 'た'),
    ("tsu", 'た'),
    ("fu", 'は'),
];

const KANA_SMALL: [(char, char); 13] = [
    ('ぁ', 'あ'),
    ('ぃ', 'い'),
    ('ぅ', 'う'),
    ('ぇ', 'え'),
    ('ぉ', 'お'),
    ('っ', 'つ'),
    ('ゃ', 'や'),
    ('ゅ', 'ゆ'),
    ('ょ', 'よ'),
    ('ゎ', 'わ'),
    ('ゕ', 'か'),
    ('ゖ', 'け'),
    ('ゔ', 'う'),
];

const KANA_DAKU: [(char, char); 25] = [
    ('が', 'か'),
    ('ぎ', 'き'),
    ('ぐ', 'く'),
    ('げ', 'け'),
    ('ご', 'こ'),
    ('ざ', 'さ'),
    ('じ', 'し'),
    ('ず', 'す'),
    ('ぜ', 'せ'),
    ('ぞ', 'そ'),
    ('だ', 'た'),
    ('ぢ', 'ち'),
    ('づ', 'つ'),
    ('で', 'て'),
    ('ど', 'と'),
    ('ば', 'は'),
    ('び', 'ひ'),
    ('ぶ', 'ふ'),
    ('べ', 'へ'),
    ('ぼ', 'ほ'),
    ('ぱ', 'は'),
    ('ぴ', 'ひ'),
    ('ぷ', 'ふ'),
    ('ぺ', 'へ'),
    ('ぽ', 'ほ'),
];

const HAN_RANGES: [(char, char); 11] = [
    ('\u{2e80}', '\u{2eff}'),
    ('\u{2f00}', '\u{2fdf}'),
    ('\u{3005}', '\u{3005}'),
    ('\u{3007}', '\u{3007}'),
    ('\u{3021}', '\u{3029}'),
    ('\u{3038}', '\u{303b}'),
    ('\u{3400}', '\u{4dbf}'),
    ('\u{4e00}', '\u{9fff}'),
    ('\u{f900}', '\u{faff}'),
    ('\u{20000}', '\u{2fa1f}'),
    ('\u{30000}', '\u{323af}'),
];

const MARK_RANGES: [(char, char); 6] = [
    ('\u{300}', '\u{36f}'),
    ('\u{1ab0}', '\u{1aff}'),
    ('\u{1dc0}', '\u{1dff}'),
    ('\u{20d0}', '\u{20ff}'),
    ('\u{fe20}', '\u{fe2f}'),
    ('\u{e0100}', '\u{e01ef}'),
];

pub fn is_han(ch: char) -> bool {
    in_ranges(&HAN_RANGES, ch)
}

pub fn is_kana(ch: char) -> bool {
    ('\u{3041}'..='\u{309f}').contains(&ch) || ('\u{30a1}'..='\u{30ff}').contains(&ch)
}

pub fn is_letter_like(ch: char) -> bool {
    ch.is_ascii_digit() || ch.is_alphabetic()
}

pub fn keys_of(ch: char) -> Vec<char> {
    if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
        return vec![ch];
    }
    if ch.is_ascii_uppercase() {
        return vec![ch.to_ascii_lowercase()];
    }
    if ch.is_whitespace() {
        return Vec::new();
    }
    if is_han(ch) {
        return pinyin_keys(ch);
    }
    if is_kana(ch) {
        if let Some(head) = kana_row(&ch.to_string()) {
            return vec![head];
        }
        return vec![ch];
    }
    match fold(ch) {
        Some(folded) => vec![folded],
        None => vec![ch],
    }
}

pub fn kana_row(token: &str) -> Option<char> {
    if let Some((_, head)) = KANA_ALIAS.iter().find(|(alias, _)| *alias == token) {
        return Some(*head);
    }
    let mut chars = token.chars();
    let first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let mut kana = first;
    if ('\u{30a1}'..='\u{30f6}').contains(&kana) {
        kana = char::from_u32(kana as u32 - 0x60)?;
    }
    if let Some((_, small)) = KANA_SMALL.iter().find(|(from, _)| *from == kana) {
        kana = *small;
    }
    if let Some((_, daku)) = KANA_DAKU.iter().find(|(from, _)| *from == kana) {
        kana = *daku;
    }
    kana_row_of(kana)
}

fn kana_row_of(kana: char) -> Option<char> {
    KANA_ROWS
        .iter()
        .find(|(row, _)| row.contains(kana))
        .map(|(_, head)| *head)
}

pub fn fold(ch: char) -> Option<char> {
    if in_ranges(&MARK_RANGES, ch) {
        return None;
    }
    let base = match FOLD_TABLE.binary_search_by_key(&ch, |entry| entry.0) {
        Ok(index) => FOLD_TABLE[index].1,
        Err(_) => {
            let mut lowered = ch.to_lowercase();
            let first = lowered.next()?;
            if lowered.next().is_some() {
                return None;
            }
            first
        }
    };
    if is_han(base) || is_kana(base) {
        return None;
    }
    if base.is_ascii_digit() || base.is_alphabetic() {
        return Some(base);
    }
    None
}

fn pinyin_keys(ch: char) -> Vec<char> {
    match pinyin_index().get(&ch) {
        Some(letters) => letters.chars().collect(),
        None => Vec::new(),
    }
}

fn pinyin_index() -> &'static HashMap<char, &'static str> {
    static INDEX: OnceLock<HashMap<char, &'static str>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut table: HashMap<char, &'static str> = HashMap::with_capacity(41_923);
        for (letters, chars) in PINYIN_GROUPS.iter() {
            for ch in chars.chars() {
                table.insert(ch, *letters);
            }
        }
        table
    })
}

fn in_ranges(ranges: &[(char, char)], ch: char) -> bool {
    ranges
        .iter()
        .any(|(start, end)| ch >= *start && ch <= *end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanzi_keys_expand_to_every_pinyin_letter() {
        let keys = keys_of('光');
        for letter in ['g', 'u', 'a', 'n'] {
            assert!(keys.contains(&letter), "光 应当认 {letter}: {keys:?}");
        }
        assert!(!keys.contains(&'光'));
        let polyphone = keys_of('行');
        for letter in ['x', 'i', 'n', 'g', 'h', 'a'] {
            assert!(polyphone.contains(&letter), "行 应当认 {letter}: {polyphone:?}");
        }
    }

    #[test]
    fn kana_rows_cover_go_aliases() {
        assert_eq!(kana_row("か"), Some('か'));
        assert_eq!(kana_row("が"), Some('か'));
        assert_eq!(kana_row("カ"), Some('か'));
        assert_eq!(kana_row("ガ"), Some('か'));
        assert_eq!(kana_row("ka"), Some('か'));
        assert_eq!(kana_row("shi"), Some('さ'));
        assert_eq!(kana_row("chi"), Some('た'));
        assert_eq!(kana_row("nn"), Some('ん'));
        assert_eq!(kana_row("ん"), Some('ん'));
        assert_eq!(kana_row("abc"), None);
        assert_eq!(keys_of('か'), vec!['か']);
        assert_eq!(keys_of('ヴ'), vec!['あ']);
    }

    #[test]
    fn fold_handles_fullwidth_and_diacritics() {
        assert_eq!(fold('Ａ'), Some('a'));
        assert_eq!(fold('é'), Some('e'));
        assert_eq!(fold('ώ'), Some('ω'));
        assert_eq!(fold('ς'), Some('σ'));
        assert_eq!(fold('一'), None);
        assert_eq!(fold('\u{301}'), None);
        assert_eq!(keys_of('Ａ'), vec!['a']);
        assert_eq!(keys_of('é'), vec!['e']);
        assert_eq!(keys_of('-'), vec!['-']);
        assert_eq!(keys_of(' '), Vec::<char>::new());
    }
}
