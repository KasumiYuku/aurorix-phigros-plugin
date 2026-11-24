use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use aurorix_plugin::PluginConfigField;

pub const SAVE_TIMEOUT_MIN: i64 = 1;
pub const SAVE_TIMEOUT_MAX: i64 = 120;
pub const SAVE_MIN_INTERVAL_MIN: i64 = 0;
pub const SAVE_MIN_INTERVAL_MAX: i64 = 3600;
pub const FRESH_BUDGET_MAX: i64 = 5000;
pub const QUERY_CONCURRENCY_MIN: i64 = 1;
pub const QUERY_CONCURRENCY_MAX: i64 = 256;
pub const RENDER_BUDGET_MIN: i64 = 96;
pub const RENDER_BUDGET_MAX: i64 = 4096;
pub const RENDER_CONCURRENCY_MAX: i64 = 64;
pub const RENDER_QUEUE_MIN: i64 = 8;
pub const RENDER_QUEUE_MAX: i64 = 4096;
pub const REPLY_WINDOW_MIN: i64 = 30;
pub const REPLY_WINDOW_MAX: i64 = 3600;
pub const REPLY_MARGIN_MAX: i64 = 3600;
pub const URL_CACHE_MAX: i64 = 86400;
pub const URL_CACHE_RENEWALS_MAX: i64 = 50;
pub const URL_CACHE_PROBE_MIN: i64 = 1;
pub const URL_CACHE_PROBE_MAX: i64 = 100;
pub const SEARCH_LIMIT_MIN: usize = 6;
pub const SEARCH_LIMIT_MAX: usize = 300;
pub const LIST_MAX_MIN: usize = 6;
pub const LIST_MAX_MAX: usize = 300;
pub const PUSH_PER_PAGE_MIN: usize = 1;
pub const PUSH_PER_PAGE_MAX: usize = 200;
pub const LETTER_SONGS_MIN: usize = 1;
pub const LETTER_SONGS_MAX: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub save_timeout_seconds: i64,
    pub save_proxy: String,
    pub save_min_interval_seconds: i64,
    pub fresh_enabled: bool,
    pub fresh_budget_ms: i64,
    pub query_concurrency: i64,
    pub online_rank: bool,
    pub data_file: String,
    pub data_dir: String,
    pub data_remote_url: String,
    pub data_auto_update: bool,
    pub data_update_cron: String,
    pub assets_root: String,
    pub webp_enabled: bool,
    pub render_budget_mb: i64,
    pub render_concurrency: i64,
    pub render_queue: i64,
    pub reply_window_seconds: i64,
    pub reply_margin_seconds: i64,
    pub url_cache_seconds: i64,
    pub url_cache_verify: bool,
    pub url_cache_renewals: i64,
    pub url_cache_probe_percent: i64,
    pub search_limit: usize,
    pub list_limit: usize,
    pub list_max: usize,
    pub letter_songs: usize,
    pub push_per_page: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            save_timeout_seconds: 20,
            save_proxy: String::new(),
            save_min_interval_seconds: 60,
            fresh_enabled: true,
            fresh_budget_ms: 800,
            query_concurrency: 16,
            online_rank: false,
            data_file: String::new(),
            data_dir: String::new(),
            data_remote_url: String::new(),
            data_auto_update: false,
            data_update_cron: String::from("0 4 * * *"),
            assets_root: String::new(),
            webp_enabled: true,
            render_budget_mb: 0,
            render_concurrency: 0,
            render_queue: 0,
            reply_window_seconds: 300,
            reply_margin_seconds: 60,
            url_cache_seconds: 21600,
            url_cache_verify: true,
            url_cache_renewals: 6,
            url_cache_probe_percent: 25,
            search_limit: 300,
            list_limit: 300,
            list_max: 300,
            letter_songs: 8,
            push_per_page: 150,
        }
    }
}

impl Settings {
    pub fn parse(raw: &HashMap<String, String>) -> Result<Self, String> {
        let defaults = Self::default();
        let mut settings = defaults.clone();
        settings.save_timeout_seconds = int_field(
            raw,
            "save_timeout_seconds",
            defaults.save_timeout_seconds,
            SAVE_TIMEOUT_MIN,
            SAVE_TIMEOUT_MAX,
        )?;
        settings.save_min_interval_seconds = int_field(
            raw,
            "save_min_interval_seconds",
            defaults.save_min_interval_seconds,
            SAVE_MIN_INTERVAL_MIN,
            SAVE_MIN_INTERVAL_MAX,
        )?;
        settings.query_concurrency = int_field(
            raw,
            "query_concurrency",
            defaults.query_concurrency,
            QUERY_CONCURRENCY_MIN,
            QUERY_CONCURRENCY_MAX,
        )?;
        settings.fresh_budget_ms = int_field(
            raw,
            "fresh_budget_ms",
            defaults.fresh_budget_ms,
            0,
            FRESH_BUDGET_MAX,
        )?;
        settings.fresh_enabled = bool_field(raw, "fresh_enabled", defaults.fresh_enabled);
        settings.online_rank = bool_field(raw, "online_rank", defaults.online_rank);
        settings.data_auto_update = bool_field(raw, "data_auto_update", defaults.data_auto_update);
        settings.save_proxy = url_field(raw, "save_proxy", true)?;
        settings.data_file = text_field(raw, "data_file");
        settings.data_dir = text_field(raw, "data_dir");
        settings.assets_root = text_field(raw, "assets_root");
        settings.webp_enabled = bool_field(raw, "webp_enabled", defaults.webp_enabled);
        settings.data_remote_url = url_field(raw, "data_remote_url", false)?;
        settings.data_update_cron = cron_field(raw, "data_update_cron", &defaults.data_update_cron)?;
        settings.render_budget_mb = int_field(
            raw,
            "render_budget_mb",
            defaults.render_budget_mb,
            0,
            RENDER_BUDGET_MAX,
        )?;
        if settings.render_budget_mb != 0 && settings.render_budget_mb < RENDER_BUDGET_MIN {
            return Err(format!(
                "render_budget_mb 需为 0(自动) 或 {RENDER_BUDGET_MIN}~{RENDER_BUDGET_MAX} 的整数, 收到 {}",
                settings.render_budget_mb
            ));
        }
        settings.render_concurrency = int_field(
            raw,
            "render_concurrency",
            defaults.render_concurrency,
            0,
            RENDER_CONCURRENCY_MAX,
        )?;
        settings.render_queue = int_field(
            raw,
            "render_queue",
            defaults.render_queue,
            0,
            RENDER_QUEUE_MAX,
        )?;
        if settings.render_queue != 0 && settings.render_queue < RENDER_QUEUE_MIN {
            return Err(format!(
                "render_queue 需为 0(自动) 或 {RENDER_QUEUE_MIN}~{RENDER_QUEUE_MAX} 的整数, 收到 {}",
                settings.render_queue
            ));
        }
        settings.reply_window_seconds = int_field(
            raw,
            "reply_window_seconds",
            defaults.reply_window_seconds,
            REPLY_WINDOW_MIN,
            REPLY_WINDOW_MAX,
        )?;
        settings.reply_margin_seconds = int_field(
            raw,
            "reply_margin_seconds",
            defaults.reply_margin_seconds,
            0,
            REPLY_MARGIN_MAX,
        )?;
        settings.url_cache_seconds = int_field(
            raw,
            "url_cache_seconds",
            defaults.url_cache_seconds,
            0,
            URL_CACHE_MAX,
        )?;
        settings.url_cache_verify = bool_field(raw, "url_cache_verify", defaults.url_cache_verify);
        settings.url_cache_renewals = int_field(
            raw,
            "url_cache_renewals",
            defaults.url_cache_renewals,
            0,
            URL_CACHE_RENEWALS_MAX,
        )?;
        settings.url_cache_probe_percent = int_field(
            raw,
            "url_cache_probe_percent",
            defaults.url_cache_probe_percent,
            URL_CACHE_PROBE_MIN,
            URL_CACHE_PROBE_MAX,
        )?;
        settings.search_limit = int_field(
            raw,
            "search_limit",
            defaults.search_limit as i64,
            SEARCH_LIMIT_MIN as i64,
            SEARCH_LIMIT_MAX as i64,
        )? as usize;
        settings.list_max = int_field(
            raw,
            "list_max",
            defaults.list_max as i64,
            LIST_MAX_MIN as i64,
            LIST_MAX_MAX as i64,
        )? as usize;
        settings.list_limit = int_field(
            raw,
            "list_limit",
            defaults.list_limit as i64,
            1,
            settings.list_max as i64,
        )? as usize;
        settings.push_per_page = int_field(
            raw,
            "push_per_page",
            defaults.push_per_page as i64,
            PUSH_PER_PAGE_MIN as i64,
            PUSH_PER_PAGE_MAX as i64,
        )? as usize;
        settings.letter_songs = int_field(
            raw,
            "letter_songs",
            defaults.letter_songs as i64,
            LETTER_SONGS_MIN as i64,
            LETTER_SONGS_MAX as i64,
        )? as usize;
        Ok(settings)
    }
}

fn text_field(raw: &HashMap<String, String>, key: &str) -> String {
    raw.get(key)
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn int_field(
    raw: &HashMap<String, String>,
    key: &str,
    fallback: i64,
    min: i64,
    max: i64,
) -> Result<i64, String> {
    let Some(value) = raw.get(key) else {
        return Ok(fallback);
    };
    let text = value.trim();
    if text.is_empty() {
        return Ok(fallback);
    }
    let parsed = text
        .parse::<i64>()
        .map_err(|_| format!("{key} 需为 {min}~{max} 的整数, 收到 \"{text}\""))?;
    if parsed < min || parsed > max {
        return Err(format!("{key} 需为 {min}~{max} 的整数, 收到 \"{text}\""));
    }
    Ok(parsed)
}

fn bool_field(raw: &HashMap<String, String>, key: &str, fallback: bool) -> bool {
    let Some(value) = raw.get(key) else {
        return fallback;
    };
    match value.trim() {
        "true" | "1" | "on" | "yes" => true,
        "false" | "0" | "off" | "no" => false,
        _ => fallback,
    }
}

fn url_field(raw: &HashMap<String, String>, key: &str, proxy: bool) -> Result<String, String> {
    let text = text_field(raw, key);
    if text.is_empty() {
        return Ok(text);
    }
    let rest = text
        .strip_prefix("http://")
        .or_else(|| text.strip_prefix("https://"));
    let host = rest.map(|tail| tail.split(['/', '?', '#']).next().unwrap_or(""));
    let label = if proxy { "代理地址" } else { "地址" };
    match host {
        Some(host) if !host.is_empty() => Ok(text),
        _ => Err(format!(
            "{key} 需为 http:// 或 https:// {label}, 收到 \"{text}\""
        )),
    }
}

fn cron_field(
    raw: &HashMap<String, String>,
    key: &str,
    fallback: &str,
) -> Result<String, String> {
    let text = text_field(raw, key);
    if text.is_empty() {
        return Ok(fallback.to_string());
    }
    if text.split_whitespace().count() != 5 {
        return Err(format!(
            "{key} 需为 5 段 cron(分 时 日 月 周), 收到 \"{text}\""
        ));
    }
    Ok(text)
}

pub struct SettingsStore {
    path: Option<PathBuf>,
    values: Mutex<HashMap<String, String>>,
    snapshot: RwLock<Arc<Settings>>,
}

impl SettingsStore {
    pub fn load(path: Option<PathBuf>) -> Self {
        let stored = path.as_deref().map(read_values).unwrap_or_default();
        let (values, snapshot) = match Settings::parse(&stored) {
            Ok(settings) => (stored, settings),
            Err(_) => (HashMap::new(), Settings::default()),
        };
        Self {
            path,
            values: Mutex::new(values),
            snapshot: RwLock::new(Arc::new(snapshot)),
        }
    }

    pub fn memory() -> Self {
        Self::load(None)
    }

    pub fn get(&self) -> Arc<Settings> {
        Arc::clone(
            &self
                .snapshot
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }

    pub fn values(&self) -> HashMap<String, String> {
        self.values
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn apply(&self, pairs: &[(String, String)]) -> Result<(), String> {
        let mut merged = self.values();
        for (key, value) in pairs {
            merged.insert(key.clone(), value.clone());
        }
        let settings = Settings::parse(&merged)?;
        if let Some(path) = self.path.as_ref() {
            persist(path, &merged)?;
        }
        {
            let mut guard = self
                .values
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard = merged;
        }
        let mut guard = self
            .snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Arc::new(settings);
        Ok(())
    }
}

fn read_values(path: &std::path::Path) -> HashMap<String, String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    serde_json::from_str::<HashMap<String, String>>(&text).unwrap_or_default()
}

fn persist(path: &std::path::Path, values: &HashMap<String, String>) -> Result<(), String> {
    let body = serde_json::to_string_pretty(values)
        .map_err(|error| format!("配置序列化失败：{error}"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("配置目录创建失败 {}：{error}", parent.display()))?;
    }
    std::fs::write(path, body)
        .map_err(|error| format!("配置写入失败 {}：{error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn parse_rejects_out_of_range_values() {
        assert!(Settings::parse(&raw(&[("search_limit", "5")])).is_err());
        assert!(Settings::parse(&raw(&[("search_limit", "301")])).is_err());
        assert!(Settings::parse(&raw(&[("list_max", "5")])).is_err());
        assert!(Settings::parse(&raw(&[("list_limit", "301")])).is_err());
        assert!(Settings::parse(&raw(&[("query_concurrency", "0")])).is_err());
        assert!(Settings::parse(&raw(&[("letter_songs", "21")])).is_err());
        assert!(Settings::parse(&raw(&[("push_per_page", "0")])).is_err());
        assert!(Settings::parse(&raw(&[("render_budget_mb", "95")])).is_err());
        assert!(Settings::parse(&raw(&[("render_queue", "4")])).is_err());
        assert!(Settings::parse(&raw(&[("save_proxy", "127.0.0.1:1")])).is_err());
        assert!(Settings::parse(&raw(&[("data_update_cron", "0 4 * *")])).is_err());
        assert!(Settings::parse(&raw(&[("search_limit", "abc")])).is_err());
    }

    #[test]
    fn parse_falls_back_to_defaults_for_blank_values() {
        let settings = Settings::parse(&raw(&[
            ("search_limit", ""),
            ("list_limit", ""),
            ("fresh_enabled", ""),
        ]))
        .expect("空串按默认值处理");
        assert_eq!(settings.search_limit, 300);
        assert_eq!(settings.list_limit, 300);
        assert!(settings.fresh_enabled);
    }

    #[test]
    fn list_limit_follows_list_max() {
        let settings = Settings::parse(&raw(&[("list_max", "50")])).expect("只写上限要能过");
        assert_eq!(settings.list_max, 50);
        assert_eq!(settings.list_limit, 300);
        let tight = Settings::parse(&raw(&[("list_max", "50"), ("list_limit", "50")]));
        assert_eq!(tight.expect("上限内合法").list_limit, 50);
        assert!(Settings::parse(&raw(&[("list_max", "50"), ("list_limit", "51")])).is_err());
    }

    #[test]
    fn store_applies_valid_values_only() {
        let store = SettingsStore::memory();
        assert_eq!(store.get().search_limit, 300);
        assert!(store
            .apply(&[(String::from("search_limit"), String::from("3"))])
            .is_err());
        assert_eq!(store.get().search_limit, 300);
        store
            .apply(&[(String::from("search_limit"), String::from("42"))])
            .expect("合法值应当落盘");
        assert_eq!(store.get().search_limit, 42);
        assert_eq!(
            store.values().get("search_limit").map(String::as_str),
            Some("42")
        );
    }
}

pub fn fields() -> Vec<PluginConfigField> {
    vec![
        PluginConfigField::number("save_timeout_seconds", "存档超时(秒)").default_value(20),
        PluginConfigField::text("save_proxy", "存档代理"),
        PluginConfigField::number("save_min_interval_seconds", "存档最短间隔(秒)")
            .default_value(60),
        PluginConfigField::boolean("fresh_enabled", "实时查询").default_value(true),
        PluginConfigField::number("fresh_budget_ms", "实时预算(毫秒)").default_value(800),
        PluginConfigField::number("query_concurrency", "查询并发").default_value(16),
        PluginConfigField::boolean("online_rank", "在线排行").default_value(false),
        PluginConfigField::text("data_file", "数据文件"),
        PluginConfigField::text("data_dir", "数据目录"),
        PluginConfigField::text("data_remote_url", "数据远程地址"),
        PluginConfigField::boolean("data_auto_update", "自动更新数据").default_value(false),
        PluginConfigField::text("data_update_cron", "更新计划").default_value("0 4 * * *"),
        PluginConfigField::text("assets_root", "素材目录"),
        PluginConfigField::boolean("webp_enabled", "图片转 WebP").default_value(true),
        PluginConfigField::number("render_budget_mb", "渲染预算(MB)").default_value(512),
        PluginConfigField::number("render_concurrency", "渲染并发").default_value(4),
        PluginConfigField::number("render_queue", "渲染队列").default_value(64),
        PluginConfigField::number("reply_window_seconds", "回复窗口(秒)").default_value(600),
        PluginConfigField::number("reply_margin_seconds", "回复余量(秒)").default_value(60),
        PluginConfigField::number("url_cache_seconds", "图片链接缓存(秒)").default_value(3600),
        PluginConfigField::boolean("url_cache_verify", "图片链接校验").default_value(true),
        PluginConfigField::number("url_cache_renewals", "图片链接续期次数").default_value(3),
        PluginConfigField::number("url_cache_probe_percent", "图片链接探测比例").default_value(5),
        PluginConfigField::number("search_limit", "搜索条数").default_value(20),
        PluginConfigField::number("list_limit", "列表条数").default_value(30),
        PluginConfigField::number("list_max", "列表上限").default_value(120),
        PluginConfigField::number("letter_songs", "看字猜歌曲数").default_value(5),
        PluginConfigField::number("push_per_page", "推送每页条数").default_value(10),
    ]
}
