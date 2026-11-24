use crate::model::{Difficulty, Grade, PlayRecord, Song};

use super::catalog::Catalog;
use super::span::{fold_symbols, parse_constant, parse_span, Span};

pub const PICK_ROWS: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListOrder {
    #[default]
    Constant,
    Rks,
    Acc,
    Score,
    Rank,
    Title,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ordering {
    pub key: ListOrder,
    pub reverse: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListGroup {
    Difficulty,
    Constant,
    Grade,
    #[default]
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct States {
    pub un_fc: bool,
    pub un_phi: bool,
    pub counted: bool,
    pub unknown: bool,
    pub variant: bool,
}

impl States {
    pub fn any(self) -> bool {
        self.un_fc || self.un_phi || self.counted || self.unknown || self.variant
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filter {
    pub acc: Span,
    pub score: Span,
    pub rks: Span,
    pub grades: Vec<Grade>,
}

impl Filter {
    pub fn any(&self) -> bool {
        self.acc.on() || self.score.on() || self.rks.on() || !self.grades.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    pub constant: Span,
    pub chapter: String,
    pub difficulties: Vec<Difficulty>,
    pub title: String,
    pub played: bool,
    pub filter: Filter,
    pub states: States,
}

impl Query {
    pub fn on(&self) -> bool {
        self.constant.on()
            || !self.chapter.is_empty()
            || !self.difficulties.is_empty()
            || !self.title.is_empty()
            || self.filter.any()
            || self.states.any()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub query: Query,
    pub texts: Vec<String>,
    pub band: bool,
    pub split: bool,
    pub order: Ordering,
    pub group: ListGroup,
    pub limit: usize,
    pub page: usize,
}

impl Default for Parsed {
    fn default() -> Self {
        Self {
            query: Query::default(),
            texts: Vec::new(),
            band: false,
            split: false,
            order: Ordering::default(),
            group: ListGroup::default(),
            limit: 0,
            page: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    Constant,
    Acc,
    Score,
    Rks,
}

const ORDER_ALIASES: [(&str, ListOrder); 14] = [
    ("按rks", ListOrder::Rks),
    ("按acc", ListOrder::Acc),
    ("按准度", ListOrder::Acc),
    ("按分数", ListOrder::Score),
    ("按分", ListOrder::Score),
    ("按定数", ListOrder::Constant),
    ("按名次", ListOrder::Rank),
    ("按曲名", ListOrder::Title),
    ("byrks", ListOrder::Rks),
    ("byacc", ListOrder::Acc),
    ("byscore", ListOrder::Score),
    ("byconst", ListOrder::Constant),
    ("byrank", ListOrder::Rank),
    ("bytitle", ListOrder::Title),
];

const GROUP_ALIASES: [(&str, ListGroup); 8] = [
    ("分难度", ListGroup::Difficulty),
    ("分定数", ListGroup::Constant),
    ("分评级", ListGroup::Grade),
    ("不分", ListGroup::None),
    ("gdiff", ListGroup::Difficulty),
    ("gconst", ListGroup::Constant),
    ("ggrade", ListGroup::Grade),
    ("gnone", ListGroup::None),
];

const GRADE_ALIASES: [(&str, Grade); 9] = [
    ("s", Grade::S),
    ("a", Grade::A),
    ("b", Grade::B),
    ("c", Grade::C),
    ("f", Grade::F),
    ("v", Grade::V),
    ("fc", Grade::Fc),
    ("全连", Grade::Fc),
    ("phi", Grade::Phi),
];

const NUM_KEYWORDS: [(&str, Field); 13] = [
    ("acc", Field::Acc),
    ("准度", Field::Acc),
    ("精确度", Field::Acc),
    ("准确率", Field::Acc),
    ("score", Field::Score),
    ("分数", Field::Score),
    ("成绩", Field::Score),
    ("分", Field::Score),
    ("rks", Field::Rks),
    ("dif", Field::Constant),
    ("difficulty", Field::Constant),
    ("定数", Field::Constant),
    ("定级", Field::Constant),
];

const TEXT_KEYWORDS: [&str; 3] = ["title", "曲名", "曲"];

const STATE_KEYWORDS: [(&str, u8); 14] = [
    ("未fc", 0),
    ("没fc", 0),
    ("nofc", 0),
    ("未phi", 1),
    ("未φ", 1),
    ("未满分", 1),
    ("nophi", 1),
    ("已计入", 2),
    ("计入", 2),
    ("counted", 2),
    ("未收录", 3),
    ("unknown", 3),
    ("变体", 4),
    ("variant", 4),
];

const EXTRA_ALIASES: [(&str, &str); 3] = [("sp", "SP"), ("特殊", "SP"), ("愚人节", "SP")];

fn list_order(token: &str) -> Option<ListOrder> {
    let lower = token.trim().to_lowercase();
    ORDER_ALIASES
        .iter()
        .find(|(key, _)| *key == lower)
        .map(|(_, value)| *value)
}

pub fn order_explicit(raw: &str) -> bool {
    let lower = raw.to_lowercase();
    ORDER_ALIASES
        .iter()
        .any(|(key, _)| lower.contains(key))
}

fn list_group(token: &str) -> Option<ListGroup> {
    let lower = token.trim().to_lowercase();
    GROUP_ALIASES
        .iter()
        .find(|(key, _)| *key == lower)
        .map(|(_, value)| *value)
}

fn parse_grade(token: &str) -> Option<Grade> {
    let lower = fold_symbols(token.trim()).to_lowercase();
    if lower == "ap" || lower == "满分" {
        return Some(Grade::Phi);
    }
    GRADE_ALIASES
        .iter()
        .find(|(key, _)| *key == lower)
        .map(|(_, value)| *value)
}

fn apply_state(states: &mut States, token: &str) -> bool {
    let lower = fold_symbols(token.trim()).to_lowercase();
    let Some((_, slot)) = STATE_KEYWORDS.iter().find(|(key, _)| *key == lower) else {
        return false;
    };
    match slot {
        0 => states.un_fc = true,
        1 => states.un_phi = true,
        2 => states.counted = true,
        3 => states.unknown = true,
        _ => states.variant = true,
    }
    true
}

fn limit_keyword(token: &str) -> Option<usize> {
    let folded = fold_symbols(token.trim()).to_lowercase();
    let rest = folded
        .strip_prefix('前')
        .or_else(|| folded.strip_prefix('n'))?;
    if rest.is_empty() {
        return None;
    }
    rest.parse::<usize>().ok().filter(|value| *value > 0)
}

fn page_keyword(token: &str) -> Option<usize> {
    let folded = fold_symbols(token.trim()).to_lowercase();
    let rest = folded
        .strip_prefix("-p")
        .or_else(|| folded.strip_prefix("p"))?;
    rest.parse::<usize>().ok().filter(|value| *value > 0)
}

fn chapter_keyword(token: &str) -> Option<&str> {
    folded_prefix(token, "章节:")
        .or_else(|| folded_prefix(token, "曲包:"))
        .or_else(|| folded_prefix(token, "ch:"))
}

fn folded_prefix<'a>(token: &'a str, prefix: &str) -> Option<&'a str> {
    if token.len() <= prefix.len() {
        return None;
    }
    let lower = token.to_lowercase();
    lower.starts_with(prefix).then(|| &token[prefix.len()..])
}

fn glued_keyword(token: &str) -> Option<(Field, String)> {
    let lower = token.to_lowercase();
    for (key, field) in NUM_KEYWORDS {
        let Some(rest) = lower.strip_prefix(key) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        let value = rest.trim_start_matches([':', '：', '=', ' ']).to_string();
        if parse_span(&value).is_some() {
            return Some((field, value));
        }
    }
    None
}

fn standalone_keyword(token: &str) -> Option<Option<Field>> {
    let lower = token.to_lowercase();
    if TEXT_KEYWORDS.contains(&lower.as_str()) {
        return Some(None);
    }
    NUM_KEYWORDS
        .iter()
        .find(|(key, _)| *key == lower)
        .map(|(_, field)| Some(*field))
}

fn stop_value(token: &str) -> bool {
    if Difficulty::parse(token).is_some() {
        return true;
    }
    parse_grade(token).is_some()
}

fn bare_level(token: &str) -> Option<f64> {
    let folded = fold_symbols(token);
    if folded
        .chars()
        .any(|ch| ".-+~～—<>=≤≥".contains(ch))
    {
        return None;
    }
    folded.parse::<i64>().ok().map(|value| value as f64)
}

fn extra_kind(token: &str) -> Option<&'static str> {
    let lower = token.trim().to_lowercase();
    EXTRA_ALIASES
        .iter()
        .find(|(key, _)| *key == lower)
        .map(|(_, value)| *value)
}

#[derive(Clone, Debug, Default)]
struct Numbers {
    constant: Span,
    acc: Span,
    score: Span,
    rks: Span,
    seen: [bool; 4],
}

impl Numbers {
    fn slot(&self, field: Field) -> usize {
        match field {
            Field::Constant => 0,
            Field::Acc => 1,
            Field::Score => 2,
            Field::Rks => 3,
        }
    }

    fn apply(&mut self, field: Field, value: &str) -> bool {
        if self.seen[self.slot(field)] {
            return false;
        }
        let parsed = match field {
            Field::Constant => parse_constant(value),
            _ => parse_span(value),
        };
        let Some(span) = parsed else {
            return false;
        };
        match field {
            Field::Constant => self.constant = span,
            Field::Acc => self.acc = span,
            Field::Score => self.score = span,
            Field::Rks => self.rks = span,
        }
        self.seen[self.slot(field)] = true;
        true
    }

    fn set_band(&mut self, level: f64) -> bool {
        if self.seen[0] {
            return false;
        }
        self.constant = Span::between(level, level + 0.9);
        self.seen[0] = true;
        true
    }
}

pub fn parse(raw: &str) -> Option<Parsed> {
    let fields: Vec<&str> = raw.split_whitespace().collect();
    if fields.is_empty() {
        return None;
    }
    let mut out = Parsed::default();
    let mut numbers = Numbers::default();
    let mut grades: Vec<Grade> = Vec::new();
    let mut levels: Vec<Difficulty> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    let mut states = States::default();
    let mut chapter = String::new();
    let mut index = 0;
    while index < fields.len() {
        let token = fields[index];
        if let Some(order) = list_order(token) {
            out.order.key = order;
            index += 1;
            continue;
        }
        if let Some(group) = list_group(token) {
            out.group = group;
            index += 1;
            continue;
        }
        if apply_state(&mut states, token) {
            index += 1;
            continue;
        }
        if token == "-p" && index + 1 < fields.len() {
            if let Some(page) = page_keyword(&format!("p{}", fields[index + 1])) {
                out.page = page;
                index += 2;
                continue;
            }
        }
        if let Some(page) = page_keyword(token) {
            out.page = page;
            index += 1;
            continue;
        }
        if let Some(name) = chapter_keyword(token) {
            chapter = fold_symbols(name.trim()).to_string();
            index += 1;
            continue;
        }
        if let Some(n) = limit_keyword(token) {
            out.limit = n;
            index += 1;
            continue;
        }
        let lower = fold_symbols(token.trim()).to_lowercase();
        if lower == "倒序" || lower == "desc" {
            out.order.reverse = true;
            index += 1;
            continue;
        }
        if lower == "正序" || lower == "asc" {
            out.order.reverse = false;
            index += 1;
            continue;
        }
        if let Some((field, value)) = glued_keyword(token) {
            if !numbers.apply(field, &value) {
                return None;
            }
            index += 1;
            continue;
        }
        if let Some(field) = standalone_keyword(token) {
            index += 1;
            let mut values: Vec<&str> = Vec::new();
            while index < fields.len() && !stop_value(fields[index]) {
                values.push(fields[index]);
                index += 1;
            }
            match field {
                None => texts.extend(values.iter().map(|value| value.to_string())),
                Some(field) => {
                    let joined = values.join("");
                    if joined.is_empty() || !numbers.apply(field, &joined) {
                        return None;
                    }
                }
            }
            continue;
        }
        if let Some(level) = Difficulty::parse(token) {
            levels.push(level);
            out.split = true;
            index += 1;
            continue;
        }
        if let Some(grade) = parse_grade(token) {
            grades.push(grade);
            index += 1;
            continue;
        }
        if let Some(level) = bare_level(token) {
            if !numbers.set_band(level) {
                return None;
            }
            index += 1;
            continue;
        }
        if let Some(span) = parse_span(token) {
            if numbers.seen[0] {
                return None;
            }
            numbers.constant = span;
            numbers.seen[0] = true;
            index += 1;
            continue;
        }
        texts.push(token.to_string());
        index += 1;
    }
    out.query = Query {
        constant: numbers.constant,
        chapter,
        difficulties: levels,
        title: texts.join(" "),
        played: true,
        filter: Filter {
            acc: numbers.acc,
            score: numbers.score,
            rks: numbers.rks,
            grades,
        },
        states,
    };
    out.texts = texts;
    out.band = numbers.constant.min_set
        && numbers.constant.max_set
        && (numbers.constant.max - numbers.constant.min - 0.9).abs() < 1e-6;
    if !out.query.on() {
        return None;
    }
    Some(out)
}

pub fn parse_constant_query(raw: &str) -> Option<Parsed> {
    let parsed = parse(raw)?;
    parsed.query.constant.on().then_some(parsed)
}

fn trim_float(value: f64) -> String {
    format!("{value}")
}

fn span_text(span: Span, band: bool) -> String {
    if band {
        return format!("{:.1}-{:.1}", span.min, span.max);
    }
    if span.min_set && span.max_set && span.min == span.max && !span.min_open && !span.max_open {
        return trim_float(span.min);
    }
    if span.min_set && span.max_set && !span.min_open && !span.max_open {
        return format!("{}-{}", trim_float(span.min), trim_float(span.max));
    }
    span.label()
}

fn int_span_text(span: Span) -> String {
    if span.min_set && span.max_set && span.min == span.max && !span.min_open && !span.max_open {
        return format!("{}", span.min as i64);
    }
    if span.min_set && span.max_set && !span.min_open && !span.max_open {
        return format!("{}-{}", span.min as i64, span.max as i64);
    }
    span.label()
}

fn difficulty_text(levels: &[Difficulty]) -> String {
    levels
        .iter()
        .map(|level| level.name())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn condition_text(parsed: &Parsed) -> String {
    let query = &parsed.query;
    let mut parts: Vec<String> = Vec::new();
    if !query.chapter.is_empty() {
        parts.push(format!("章节「{}」", query.chapter));
    }
    if !query.title.is_empty() {
        parts.push(format!("含「{}」", query.title));
    }
    if query.constant.on() {
        parts.push(format!("定数 {}", span_text(query.constant, parsed.band)));
    }
    if query.filter.score.on() {
        parts.push(format!("分数 {}", int_span_text(query.filter.score)));
    }
    if query.filter.acc.on() {
        parts.push(format!("acc {}%", span_text(query.filter.acc, false)));
    }
    if query.filter.rks.on() {
        parts.push(format!("rks {}", span_text(query.filter.rks, false)));
    }
    if !query.difficulties.is_empty() {
        parts.push(difficulty_text(&query.difficulties));
    }
    if !query.filter.grades.is_empty() {
        parts.push(
            query
                .filter
                .grades
                .iter()
                .map(|grade| grade.name())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }
    parts.join(" · ")
}

pub fn pick_score_difficulty(
    song: &Song,
    records: &[PlayRecord],
    explicit: Option<Difficulty>,
) -> Option<Difficulty> {
    if let Some(level) = explicit {
        if !level.valid() {
            return None;
        }
        if level == Difficulty::Sp {
            return Some(level);
        }
        return song.chart(level).map(|_| level);
    }
    for level in [Difficulty::At, Difficulty::In, Difficulty::Hd, Difficulty::Ez] {
        if song.chart(level).is_none() {
            continue;
        }
        if records
            .iter()
            .any(|record| record.difficulty == level && record.song_id == song.id)
        {
            return Some(level);
        }
    }
    if song.chart(Difficulty::In).is_some() {
        return Some(Difficulty::In);
    }
    song.difficulties().last().copied()
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SongQuery {
    pub text: String,
    pub difficulty: Option<Difficulty>,
    pub extra: String,
    pub index: usize,
}

fn strip_index(token: &str) -> Option<usize> {
    let rest = token.strip_prefix('#')?;
    rest.parse::<usize>().ok().filter(|value| *value > 0)
}

pub fn parse_song_query(raw: &str) -> Option<SongQuery> {
    let mut tokens: Vec<String> = raw.split_whitespace().map(|part| part.to_string()).collect();
    if tokens.is_empty() {
        return None;
    }
    let mut out = SongQuery::default();
    let mut guard = 0;
    while guard < 4 && !tokens.is_empty() {
        guard += 1;
        let last = tokens.len() - 1;
        if let Some(index) = strip_index(&tokens[last]) {
            out.index = index;
            tokens.remove(last);
            continue;
        }
        if let Some(index) = strip_index(&tokens[0]) {
            out.index = index;
            tokens.remove(0);
            continue;
        }
        if tokens.len() < 2 {
            break;
        }
        if let Some(kind) = extra_kind(&tokens[last]) {
            out.extra = kind.to_string();
            tokens.remove(last);
            continue;
        }
        if let Some(level) = Difficulty::parse(&tokens[last]) {
            out.difficulty = Some(level);
            tokens.remove(last);
            continue;
        }
        if let Some(level) = Difficulty::parse(&tokens[0]) {
            out.difficulty = Some(level);
            tokens.remove(0);
            continue;
        }
        break;
    }
    out.text = tokens.join(" ");
    if out.text.is_empty() {
        return None;
    }
    Some(out)
}

pub fn inline_link(label: &str, command: &str) -> String {
    format!(
        "[{}](mqqapi://aio/inlinecmd?command={})",
        label,
        percent_encode(command)
    )
}

pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~');
        if keep {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

pub fn fill_command(prefix: &str, suffix: &str, catalog: &Catalog, song: &Song) -> String {
    let unique = |key: &str| catalog.resolve(key).map(|found| found.id == song.id) == Some(true);
    let mut key = song.title.as_str();
    if !unique(key) {
        if let Some(alias) = song
            .aliases
            .iter()
            .find(|alias| !alias.trim().is_empty() && unique(alias))
        {
            key = alias;
        } else {
            key = song.id.as_str();
        }
    }
    format!("{prefix}{key}{suffix}")
}

fn dash(text: &str) -> String {
    if text.trim().is_empty() {
        return String::from("-");
    }
    text.to_string()
}

pub fn pick_markdown(
    query: &str,
    prefix: &str,
    catalog: &Catalog,
    candidates: &[&Song],
    total: usize,
) -> String {
    pick_markdown_with(query, prefix, "", catalog, candidates, total)
}

pub fn pick_markdown_with(
    query: &str,
    prefix: &str,
    suffix: &str,
    catalog: &Catalog,
    candidates: &[&Song],
    total: usize,
) -> String {
    let mut rows = String::new();
    for (index, song) in candidates.iter().take(PICK_ROWS).enumerate() {
        let label = format!(
            "{} · {} · {}",
            song.title,
            dash(&song.composer),
            dash(&song.chapter)
        );
        let command = fill_command(prefix, suffix, catalog, song);
        rows.push_str(&format!(
            "> {}. {}\n\n",
            index + 1,
            inline_link(&label, &command)
        ));
    }
    format!(
        "\n# $$\\large{{\\;\\boldsymbol{{\\mathcal{{遍历找到多首曲目: {query}}}}}}}$$\n\n{rows}{}",
        format_args!(
            "共 {total} 首匹配 · 点一行把指令填进输入框（同名曲目按曲师 / 章节区分）"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Chart;

    fn song(id: &str, title: &str, levels: &[Difficulty]) -> Song {
        let mut item = Song {
            id: id.to_string(),
            title: title.to_string(),
            composer: String::from("Aurorix"),
            chapter: String::from("单曲精选集"),
            ..Song::default()
        };
        for level in levels {
            item.charts[level.index()] = Chart {
                difficulty: *level,
                constant: 15.0,
                ..Chart::default()
            };
        }
        item
    }

    fn record(song_id: &str, level: Difficulty) -> PlayRecord {
        PlayRecord {
            song_id: song_id.to_string(),
            difficulty: level,
            score: 990_000,
            acc: 99.0,
            fc: false,
        }
    }

    #[test]
    fn parse_accepts_free_token_order() {
        let parsed = parse("14-16 in nofc byrks desc n20 p2").unwrap();
        assert_eq!(parsed.query.constant, Span::between(14.0, 16.0));
        assert_eq!(parsed.query.difficulties, vec![Difficulty::In]);
        assert!(parsed.query.states.un_fc);
        assert_eq!(parsed.order.key, ListOrder::Rks);
        assert!(parsed.order.reverse);
        assert_eq!(parsed.limit, 20);
        assert_eq!(parsed.page, 2);
        assert!(parsed.split);
    }

    #[test]
    fn bare_integer_reads_whole_tier_and_marks_band() {
        let parsed = parse("15").unwrap();
        assert_eq!(parsed.query.constant, Span::between(15.0, 15.9));
        assert!(parsed.band);
        assert_eq!(condition_text(&parsed), "定数 15.0-15.9");
    }

    #[test]
    fn glued_keywords_separate_dimensions() {
        let parsed = parse("acc99+ score990000+ rks15.5").unwrap();
        assert_eq!(parsed.query.filter.acc, Span::at_least(99.0));
        assert_eq!(parsed.query.filter.score, Span::at_least(990_000.0));
        assert_eq!(parsed.query.filter.rks, Span::between(15.5, 15.5));
    }

    #[test]
    fn difficulty_tokens_split_from_title() {
        let parsed = parse("Cthugha AT").unwrap();
        assert_eq!(parsed.query.title, "Cthugha");
        assert_eq!(parsed.query.difficulties, vec![Difficulty::At]);
        assert!(parsed.split);
        assert_eq!(condition_text(&parsed), "含「Cthugha」 · AT");
    }

    #[test]
    fn grades_cover_every_advertised_token() {
        for token in ["phi", "ap", "fc", "v", "s", "a", "b", "c", "f"] {
            assert!(parse_grade(token).is_some(), "{token}");
        }
        let parsed = parse("phi fc").unwrap();
        assert_eq!(parsed.query.filter.grades, vec![Grade::Phi, Grade::Fc]);
    }

    #[test]
    fn page_chapter_and_limit_tokens() {
        assert_eq!(parse("15 p3").unwrap().page, 3);
        assert_eq!(parse("15 -p 4").unwrap().page, 4);
        assert_eq!(parse("15 n7").unwrap().limit, 7);
        assert_eq!(parse("15 前12").unwrap().limit, 12);
        assert_eq!(parse("p3").map(|parsed| parsed.page), None);
        let parsed = parse("章节:霓虹灯牌").unwrap();
        assert_eq!(parsed.query.chapter, "霓虹灯牌");
        assert!(condition_text(&parsed).starts_with("章节「霓虹灯牌」"));
    }

    #[test]
    fn empty_or_conditionless_input_is_rejected() {
        assert!(parse("   ").is_none());
        assert!(parse_constant_query("in").is_none());
        assert!(parse_constant_query("15+").is_some());
    }

    #[test]
    fn pick_prefers_highest_played_level() {
        let item = song("a.0", "A", &[Difficulty::Ez, Difficulty::Hd, Difficulty::In, Difficulty::At]);
        let records = [record("a.0", Difficulty::In), record("a.0", Difficulty::Hd)];
        assert_eq!(
            pick_score_difficulty(&item, &records, None),
            Some(Difficulty::In)
        );
        let records = [
            record("a.0", Difficulty::In),
            record("a.0", Difficulty::At),
        ];
        assert_eq!(
            pick_score_difficulty(&item, &records, None),
            Some(Difficulty::At)
        );
        assert_eq!(
            pick_score_difficulty(&item, &records, Some(Difficulty::Hd)),
            Some(Difficulty::Hd)
        );
        assert_eq!(pick_score_difficulty(&item, &records, Some(Difficulty::Legacy)), None);
    }

    #[test]
    fn pick_falls_back_to_in_then_highest_available() {
        let both = song("b.0", "B", &[Difficulty::Ez, Difficulty::Hd, Difficulty::In]);
        assert_eq!(
            pick_score_difficulty(&both, &[], None),
            Some(Difficulty::In)
        );
        let ez_only = song("c.0", "C", &[Difficulty::Ez]);
        assert_eq!(
            pick_score_difficulty(&ez_only, &[], None),
            Some(Difficulty::Ez)
        );
        let hd_only = song("d.0", "D", &[Difficulty::Hd]);
        assert_eq!(
            pick_score_difficulty(&hd_only, &[], None),
            Some(Difficulty::Hd)
        );
        let none = song("e.0", "E", &[]);
        assert_eq!(pick_score_difficulty(&none, &[], None), None);
    }

    #[test]
    fn explicit_difficulty_must_exist_on_the_chart() {
        let item = song("f.0", "F", &[Difficulty::Ez, Difficulty::In]);
        assert_eq!(
            pick_score_difficulty(&item, &[], Some(Difficulty::In)),
            Some(Difficulty::In)
        );
        assert_eq!(pick_score_difficulty(&item, &[], Some(Difficulty::At)), None);
        assert_eq!(
            pick_score_difficulty(&item, &[], Some(Difficulty::Sp)),
            Some(Difficulty::Sp)
        );
    }

    #[test]
    fn song_query_strips_difficulty_and_index() {
        let parsed = parse_song_query("Spasmodic IN #2").unwrap();
        assert_eq!(parsed.text, "Spasmodic");
        assert_eq!(parsed.difficulty, Some(Difficulty::In));
        assert_eq!(parsed.index, 2);
        let parsed = parse_song_query("at ExoplanetaryMirage").unwrap();
        assert_eq!(parsed.text, "ExoplanetaryMirage");
        assert_eq!(parsed.difficulty, Some(Difficulty::At));
        let parsed = parse_song_query("Message sp").unwrap();
        assert_eq!(parsed.extra, "SP");
        assert_eq!(parsed.text, "Message");
        assert!(parse_song_query("   ").is_none());
    }

    #[test]
    fn inline_links_encode_the_fill_command() {
        let link = inline_link("A · B · C", "/p score Spasmodic");
        assert_eq!(
            link,
            "[A · B · C](mqqapi://aio/inlinecmd?command=%2Fp%20score%20Spasmodic)"
        );
        assert_eq!(percent_encode("AB-c.d_e~f"), "AB-c.d_e~f");
    }

    #[test]
    fn pick_markdown_lists_every_candidate() {
        let Ok(catalog) = Catalog::open(crate::assets_root().join("../data/phi.db")) else {
            return;
        };
        let first = catalog.songs()[0].clone();
        let candidates = [&catalog.songs()[0], &first];
        let body = pick_markdown("chro", "/p score ", &catalog, &candidates, 42);
        assert!(body.contains("遍历找到多首曲目: chro"));
        assert!(body.contains("> 1. ["));
        assert!(body.contains("> 2. ["));
        assert!(body.contains("共 42 首匹配"));
        assert!(body.contains("mqqapi://aio/inlinecmd?command="));
    }

    #[test]
    fn pick_markdown_caps_the_row_count() {
        let Ok(catalog) = Catalog::open(crate::assets_root().join("../data/phi.db")) else {
            return;
        };
        let pool: Vec<crate::model::Song> = catalog.songs().iter().take(15).cloned().collect();
        let candidates: Vec<&crate::model::Song> = pool.iter().collect();
        let body = pick_markdown("x", "/p score ", &catalog, &candidates, 99);
        assert!(body.contains("> 10. ["), "第 10 行要在：{body}");
        assert!(!body.contains("> 11. ["), "候选最多列 {} 行", PICK_ROWS);
        assert!(body.contains("共 99 首匹配"), "总数照实写：{body}");
    }
}
