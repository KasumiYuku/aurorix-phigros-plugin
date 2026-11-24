#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Span {
    pub min: f64,
    pub max: f64,
    pub min_set: bool,
    pub max_set: bool,
    pub min_open: bool,
    pub max_open: bool,
}

impl Span {
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            min_set: true,
            max_set: true,
            ..Self::default()
        }
    }

    pub fn at_least(min: f64) -> Self {
        Self {
            min,
            min_set: true,
            ..Self::default()
        }
    }

    pub fn at_most(max: f64) -> Self {
        Self {
            max,
            max_set: true,
            ..Self::default()
        }
    }

    pub fn greater_than(min: f64) -> Self {
        Self {
            min_open: true,
            ..Self::at_least(min)
        }
    }

    pub fn less_than(max: f64) -> Self {
        Self {
            max_open: true,
            ..Self::at_most(max)
        }
    }

    pub fn equal_to(value: f64) -> Self {
        Self::between(value, value)
    }

    pub fn constant_tier(level: f64) -> Self {
        Self::between(level, level + 0.9)
    }

    pub fn on(self) -> bool {
        self.min_set || self.max_set
    }

    pub fn contains(self, value: f64) -> bool {
        if self.min_set && (value < self.min || (self.min_open && value == self.min)) {
            return false;
        }
        if self.max_set && (value > self.max || (self.max_open && value == self.max)) {
            return false;
        }
        true
    }

    pub fn label(self) -> String {
        let low = if self.min_set {
            Some(if self.min_open {
                format!(">{}", self.min)
            } else {
                format!("≥{}", self.min)
            })
        } else {
            None
        };
        let high = if self.max_set {
            Some(if self.max_open {
                format!("<{}", self.max)
            } else {
                format!("≤{}", self.max)
            })
        } else {
            None
        };
        match (low, high) {
            (Some(low), Some(high)) => {
                if !self.min_open && !self.max_open {
                    format!("{}-{}", self.min, self.max)
                } else {
                    format!("{low} {high}")
                }
            }
            (Some(low), None) => low,
            (None, Some(high)) => high,
            (None, None) => String::new(),
        }
    }
}

const FOLDED: [(&str, &str); 40] = [
    ("＞", ">"),
    ("＜", "<"),
    ("＝", "="),
    ("﹥", ">"),
    ("﹤", "<"),
    ("≧", "≥"),
    ("≦", "≤"),
    ("⩾", "≥"),
    ("⩽", "≤"),
    ("＋", "+"),
    ("－", "-"),
    ("−", "-"),
    ("﹣", "-"),
    ("‑", "-"),
    ("–", "-"),
    ("—", "-"),
    ("―", "-"),
    ("~", "-"),
    ("～", "-"),
    ("〜", "-"),
    ("∼", "-"),
    ("０", "0"),
    ("１", "1"),
    ("２", "2"),
    ("３", "3"),
    ("４", "4"),
    ("５", "5"),
    ("６", "6"),
    ("７", "7"),
    ("８", "8"),
    ("９", "9"),
    ("．", "."),
    ("。", "."),
    ("　", " "),
    ("，", ","),
    ("【", "["),
    ("】", "]"),
    ("（", "("),
    ("）", ")"),
    ("／", "/"),
];

pub fn fold_symbols(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        let mut folded = None;
        for (from, to) in FOLDED {
            if ch.to_string() == from {
                folded = Some(to);
                break;
            }
        }
        match folded {
            Some(to) => result.push_str(to),
            None => result.push(ch),
        }
    }
    result
}

pub fn parse_span(raw: &str) -> Option<Span> {
    let text = fold_symbols(raw.trim());
    if text.is_empty() {
        return None;
    }
    if let Some(span) = parse_comparison(&text) {
        return Some(span);
    }
    if let Some(rest) = text.strip_suffix('+') {
        return parse_f64(rest).map(Span::at_least);
    }
    match text.split_once('-') {
        None => parse_f64(&text).map(|value| Span::between(value, value)),
        Some((head, tail)) => {
            if tail.trim().is_empty() {
                return parse_f64(head.trim()).map(Span::at_most);
            }
            let mut span = Span::default();
            let min = head.trim();
            if !min.is_empty() {
                span.min = parse_f64(min)?;
                span.min_set = true;
            }
            let max = tail.trim();
            if !max.is_empty() {
                span.max = parse_f64(max)?;
                span.max_set = true;
            }
            span.on().then_some(span)
        }
    }
}

fn parse_f64(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok()
}

pub fn parse_constant(raw: &str) -> Option<Span> {
    if let Ok(level) = fold_symbols(raw.trim()).parse::<i64>() {
        if level > 0 {
            return Some(Span::constant_tier(level as f64));
        }
    }
    parse_span(raw)
}

fn parse_comparison(text: &str) -> Option<Span> {
    type SpanOp = fn(f64) -> Span;
    const OPS: [(&str, SpanOp); 8] = [
        (">=", Span::at_least),
        ("≥", Span::at_least),
        ("<=", Span::at_most),
        ("≤", Span::at_most),
        (">", Span::greater_than),
        ("<", Span::less_than),
        ("==", Span::equal_to),
        ("=", Span::equal_to),
    ];
    for (token, build) in OPS {
        if let Some(rest) = text.strip_prefix(token) {
            return parse_f64(rest).map(build);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_documented_form() {
        assert_eq!(parse_span("12.6"), Some(Span::between(12.6, 12.6)));
        assert_eq!(parse_span("12.6-15.2"), Some(Span::between(12.6, 15.2)));
        assert_eq!(parse_span("14+"), Some(Span::at_least(14.0)));
        assert_eq!(parse_span("14-"), Some(Span::at_most(14.0)));
        assert_eq!(parse_span("-15"), Some(Span::at_most(15.0)));
        assert_eq!(parse_span(">10"), Some(Span::greater_than(10.0)));
        assert_eq!(parse_span(">=11"), Some(Span::at_least(11.0)));
        assert_eq!(parse_span("≤14.5"), Some(Span::at_most(14.5)));
        assert_eq!(parse_span("=13"), Some(Span::equal_to(13.0)));
    }

    #[test]
    fn folds_full_width_input() {
        assert_eq!(parse_span("＞１０"), Some(Span::greater_than(10.0)));
        assert_eq!(parse_span("１４＋"), Some(Span::at_least(14.0)));
        assert_eq!(parse_span("１２．６－１５．２"), Some(Span::between(12.6, 15.2)));
    }

    #[test]
    fn bare_integer_constant_reads_whole_tier() {
        assert_eq!(parse_constant("18"), Some(Span::between(18.0, 18.9)));
        assert_eq!(parse_constant("18.5"), Some(Span::between(18.5, 18.5)));
        assert_eq!(parse_constant("17+"), Some(Span::at_least(17.0)));
    }

    #[test]
    fn contains_respects_open_ends() {
        let span = Span::greater_than(10.0);
        assert!(!span.contains(10.0));
        assert!(span.contains(10.1));
        assert!(Span::at_least(10.0).contains(10.0));
        assert!(!parse_span("abc").is_some());
    }

    #[test]
    fn label_round_trips_forms() {
        assert_eq!(Span::between(12.6, 15.2).label(), "12.6-15.2");
        assert_eq!(Span::greater_than(10.0).label(), ">10");
        assert_eq!(Span::at_least(10.0).label(), "≥10");
        assert_eq!(Span::at_most(14.0).label(), "≤14");
    }
}
