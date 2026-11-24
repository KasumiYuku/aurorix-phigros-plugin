use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::domain::catalog::Catalog;
use crate::domain::overview;
use crate::model::{Binding, Chapter, Chart, Difficulty, Profile, Scored, Song};
use crate::repo::Repo;
use crate::service::{now_seconds, server_label, PhigrosService};
use crate::settings::SettingsStore;

pub const TOPICS: [&str; 7] = [
    "system.hello",
    "system.gap",
    "catalog.reloaded",
    "config.reloaded",
    "user.synced",
    "alias.changed",
    "metrics.tick",
];

const DEFAULT_BEST_SIZE: usize = 30;
const MAX_BEST_SIZE: usize = 60;
const MAX_USER_PAGE: i64 = 200;
const DEFAULT_SONG_LIMIT: usize = 50;
const MAX_SONG_LIMIT: usize = 500;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebuiRequest {
    pub method: Method,
    pub path: String,
    pub body: String,
}

impl WebuiRequest {
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            body: String::new(),
        }
    }

    pub fn with_body(method: Method, path: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            body: body.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebuiResponse {
    pub status: u16,
    pub json: String,
}

impl WebuiResponse {
    pub fn ok(json: impl Into<String>) -> Self {
        Self {
            status: 200,
            json: json.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub placeholder: &'static str,
}

pub const CONFIG_FIELDS: [Field; 28] = [
    Field {
        key: "save_timeout_seconds",
        label: "存档请求超时（秒）",
        description: "TapTap 存档接口单次超时（秒），默认 20；网络差可调大",
        kind: "text",
        placeholder: "20",
    },
    Field {
        key: "save_proxy",
        label: "存档请求代理",
        description: "拉取存档走的 HTTP(S) 代理，留空直连",
        kind: "text",
        placeholder: "留空 = 直连",
    },
    Field {
        key: "save_min_interval_seconds",
        label: "同步最小间隔（秒）",
        description: "同一账号两次云端拉取的最小间隔（秒），默认 60，防止刷 /p pull 被风控",
        kind: "text",
        placeholder: "60",
    },
    Field {
        key: "fresh_enabled",
        label: "查询时先确认存档有没有变",
        description: "打开后，查分类指令执行前先问一次存档版本戳（几百字节的小请求），变了才下载完整存档；关掉则只在 /p pull 与绑定时联网",
        kind: "boolean",
        placeholder: "",
    },
    Field {
        key: "fresh_budget_ms",
        label: "确认存档最多让用户等（毫秒）",
        description: "上面那次小请求最多挡住指令多久（毫秒），超了先用旧数据出图、同步继续在后台完成。默认 800，0 = 完全不等",
        kind: "text",
        placeholder: "800",
    },
    Field {
        key: "query_concurrency",
        label: "出站并发上限",
        description: "同时在飞的云端请求数上限，默认 16，超出排队",
        kind: "text",
        placeholder: "16",
    },
    Field {
        key: "online_rank",
        label: "线上排行榜数据",
        description: "打开后 /p me 的「全服位置」先问 Phi-Plugin 镜像，拿不到或报错就退回本地榜；默认关闭，纯本地更快",
        kind: "boolean",
        placeholder: "",
    },
    Field {
        key: "data_file",
        label: "曲库文件路径",
        description: "phi.db 的完整路径（绝对路径或相对工作目录）；曲库不再编进二进制。留空则看下面的目录，再留空自动探查 可执行文件旁/源码旁/当前目录 的 data/phi.db",
        kind: "text",
        placeholder: "例如 /srv/phigros/data/phi.db",
    },
    Field {
        key: "data_dir",
        label: "曲库目录",
        description: "曲库所在目录，读其中的 phi.db；填了上面的文件路径时以文件为准，两个都留空则自动探查",
        kind: "text",
        placeholder: "例如 /srv/phigros/data",
    },
    Field {
        key: "data_remote_url",
        label: "曲库更新地址",
        description: "phi.db 的下载地址，用于热更曲库；留空只从本地加载",
        kind: "text",
        placeholder: "留空 = 不远程更新",
    },
    Field {
        key: "data_auto_update",
        label: "自动更新曲库",
        description: "按下面的 cron 定时拉取曲库更新",
        kind: "boolean",
        placeholder: "",
    },
    Field {
        key: "data_update_cron",
        label: "曲库更新计划",
        description: "5 段 cron（分 时 日 月 周），默认 0 4 * * *",
        kind: "text",
        placeholder: "0 4 * * *",
    },
    Field {
        key: "assets_root",
        label: "出图素材目录",
        description: "出图素材目录，需含 manifest.json；留空自动探查，探查不到只回文字卡",
        kind: "text",
        placeholder: "留空 = 自动探查",
    },
    Field {
        key: "webp_enabled",
        label: "出图使用 WebP",
        description: "出图优先用 WebP（需素材目录里有 cwebp）；默认开启，不可用时自动退回 JPEG",
        kind: "boolean",
        placeholder: "",
    },
    Field {
        key: "render_budget_mb",
        label: "出图内存额度（MB）",
        description: "出图可常驻的内存上限（MB），0 = 自动（Go 堆软上限的 1/4）。多插件各占一份，总和需低于 memory_limit_mb",
        kind: "text",
        placeholder: "0 = 自动",
    },
    Field {
        key: "render_concurrency",
        label: "出图并发上限",
        description: "同时渲染张数，0 = 自动（CPU 核数留 2 个）。吃满核的后果是消息处理被饿死",
        kind: "text",
        placeholder: "0 = 自动",
    },
    Field {
        key: "render_queue",
        label: "出图排队深度",
        description: "出图排队深度，0 = 自动（按实测单张耗时与回复窗口反算）。排的是插件自己的 worker，不拖住别人的命令",
        kind: "text",
        placeholder: "0 = 自动",
    },
    Field {
        key: "reply_window_seconds",
        label: "平台回复窗口（秒）",
        description: "平台被动回复窗口（秒），默认 300。超时平台直接拒发；排队深度按它反算",
        kind: "text",
        placeholder: "300",
    },
    Field {
        key: "reply_margin_seconds",
        label: "回复窗口安全余量（秒）",
        description: "从窗口里留出的安全余量（秒），默认 60，给编码、上传与网络抖动合账",
        kind: "text",
        placeholder: "60",
    },
    Field {
        key: "url_cache_seconds",
        label: "出图直链缓存时长（秒）",
        description: "直链复用时长（秒），0 = 关闭。默认 21600（6 小时），上限 86400 且须短于图床签发寿命",
        kind: "text",
        placeholder: "21600",
    },
    Field {
        key: "url_cache_verify",
        label: "直链命中回验",
        description: "直链过半衰期后命中时先探一次（HEAD），探不通就重传。默认开启；关掉会把已失效的直链发出去",
        kind: "boolean",
        placeholder: "",
    },
    Field {
        key: "url_cache_renewals",
        label: "直链到期续期次数上限",
        description: "直链到期探活通过后最多续期几次，0 = 不续期。默认 6（6 小时 × 6 = 最长记一天半）",
        kind: "text",
        placeholder: "6",
    },
    Field {
        key: "url_cache_probe_percent",
        label: "直链回验间隔（缓存时长的 %）",
        description: "两次探活的最小间隔，按缓存时长的百分比算。默认 25（6 小时缓存即每 90 分钟最多探一次）",
        kind: "text",
        placeholder: "25",
    },
    Field {
        key: "search_limit",
        label: "检索结果上限",
        description: "一次曲目检索最多画几首（6~100），默认 100。卡片墙每张都重，调大只会让出图更慢",
        kind: "text",
        placeholder: "100",
    },
    Field {
        key: "list_limit",
        label: "成绩清单默认行数",
        description: "`/p list` 与 `/p table` 不写「前N」时摆几行（1~list_max），默认 170。`/p table` 的默认值就是全部谱面，只决定画几行，不改变命中多少",
        kind: "text",
        placeholder: "170",
    },
    Field {
        key: "list_max",
        label: "成绩清单行数上限",
        description: "`/p list` 与 `/p table` 的「前N」最多能到多少（6~200），默认 200。实测 2 核机器上 120 行 3.0 秒、150 行 4.2 秒、200 行 5.7 秒；再高单张画布就要 200MB 以上，会拖垮整台机器",
        kind: "text",
        placeholder: "200",
    },
    Field {
        key: "letter_songs",
        label: "猜字母题目数",
        description: "一局猜字母（/p letter）抽几首曲名，默认 8；1 首也能玩，多了版面会变长",
        kind: "text",
        placeholder: "8",
    },
    Field {
        key: "push_per_page",
        label: "推分建议整页条数",
        description: "`/p push` · `/p suggest` 整页最多画几条候选（1~200），默认 150；候选不够时有多少画多少，不硬凑。150 条按 5 列是 30 行",
        kind: "text",
        placeholder: "150",
    },
];

pub struct WebuiService {
    data_path: PathBuf,
    service: Option<Arc<PhigrosService>>,
    catalog: RwLock<Option<Arc<Catalog>>>,
    settings: Arc<SettingsStore>,
    started_at: i64,
}

impl WebuiService {
    pub fn new(service: Arc<PhigrosService>) -> Self {
        let data_path = crate::service::catalog_path(service.root());
        let settings = Arc::clone(service.settings_store());
        Self {
            data_path,
            service: Some(service),
            catalog: RwLock::new(None),
            settings,
            started_at: now_seconds(),
        }
    }

    pub fn detached() -> Self {
        Self {
            data_path: PathBuf::from("detached/cmd"),
            service: None,
            catalog: RwLock::new(None),
            settings: Arc::new(SettingsStore::memory()),
            started_at: now_seconds(),
        }
    }

    pub fn with_catalog_path(path: impl AsRef<Path>) -> Self {
        Self {
            data_path: path.as_ref().to_path_buf(),
            service: None,
            catalog: RwLock::new(None),
            settings: Arc::new(SettingsStore::memory()),
            started_at: now_seconds(),
        }
    }

    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    pub fn attached(&self) -> bool {
        self.service.is_some()
    }

    pub fn config_values(&self) -> HashMap<String, String> {
        self.settings.values()
    }

    pub fn catalog(&self) -> Option<Arc<Catalog>> {
        if let Some(cached) = self
            .catalog
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
        {
            return Some(cached);
        }
        let loaded = Arc::new(Catalog::open(&self.data_path).ok()?);
        let mut guard = self
            .catalog
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(Arc::clone(&loaded));
        Some(loaded)
    }

    pub fn reload_catalog(&self) -> Result<Arc<Catalog>, String> {
        let fresh = Arc::new(
            Catalog::open(&self.data_path)
                .map_err(|error| format!("曲库重载失败 {}：{error}", self.data_path.display()))?,
        );
        let mut guard = self
            .catalog
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(Arc::clone(&fresh));
        Ok(fresh)
    }

    pub fn route(&self, req: &WebuiRequest) -> WebuiResponse {
        let (parts, query) = split_target(&req.path);
        let Some(route) = classify(req.method, &parts) else {
            return problem(404, "not_found", "未找到", &format!("{} 没有对应路由", req.path));
        };
        match route {
            Route::Health => self.health(),
            Route::Overview => self.overview(),
            Route::Events => Self::events(),
            Route::Songs => self.songs(&query),
            Route::Chapters => self.chapters(),
            Route::Song(song_id) => self.song(&song_id, &[]),
            Route::AliasList(song_id) => self.alias_list(&song_id, &[]),
            Route::AliasEdit(_) => unavailable("别名写入需要数据库"),
            Route::AliasDelete(_) => unavailable("别名删除需要数据库"),
            Route::CatalogReload => self.catalog_reload(),
            Route::CatalogUpdate => WebuiResponse {
                status: 501,
                json: problem_json("not_implemented", "尚未实现", 501, "catalog update unsupported"),
            },
            Route::Users => WebuiResponse::ok(obj(vec![
                ("items", arr(&[])),
                ("total", int(0)),
            ])),
            Route::User(_) => WebuiResponse::ok(obj(vec![
                ("binding", String::from("null")),
                ("profile", String::from("null")),
            ])),
            Route::UserScores(user_id) => self.scores(&user_id, &query, None),
            Route::UserSync(_) => unavailable("同步需要数据库"),
            Route::UserDelete(_) => unavailable("删除需要数据库"),
            Route::Config => self.config_get(),
            Route::ConfigSchema => WebuiResponse::ok(obj(vec![("fields", fields_json())])),
            Route::ConfigPut => self.config_put(&req.body),
        }
    }

    pub async fn route_async(&self, req: &WebuiRequest) -> WebuiResponse {
        let Some(service) = self.service.clone() else {
            return self.route(req);
        };
        let (parts, query) = split_target(&req.path);
        let Some(route) = classify(req.method, &parts) else {
            return self.route(req);
        };
        match route {
            Route::Overview => self.overview_async(&service).await,
            Route::Users => users_async(&service).await,
            Route::User(user_id) => self.user_async(&service, &user_id).await,
            Route::UserScores(user_id) => self.scores_async(&service, &user_id, &query).await,
            Route::UserSync(user_id) => sync_async(&service, &user_id).await,
            Route::UserDelete(user_id) => delete_async(&service, &user_id).await,
            Route::Song(song_id) => self.song_async(&service, &song_id).await,
            Route::AliasList(song_id) => self.alias_list_async(&service, &song_id).await,
            Route::AliasEdit(song_id) => self.alias_edit_async(&service, &song_id, &req.body).await,
            Route::AliasDelete(song_id) => {
                self.alias_delete_async(&service, &song_id, &query).await
            }
            _ => self.route(req),
        }
    }

    fn health(&self) -> WebuiResponse {
        let ready = self.catalog().is_some();
        WebuiResponse::ok(obj(vec![
            ("ok", boolean(true)),
            ("catalog", boolean(ready)),
        ]))
    }

    fn catalog_stats(&self) -> String {
        let catalog = self.catalog();
        let mut songs = 0_usize;
        let mut charts = 0_usize;
        let mut aliases = 0_usize;
        let mut chapters = 0_usize;
        let mut titles = 0_i64;
        if let Some(catalog) = catalog.as_ref() {
            songs = catalog.songs().len();
            chapters = catalog.chapters().len();
            for song in catalog.songs() {
                charts += song.all_difficulties().len();
                aliases += song.aliases.len();
                titles += 1;
            }
        }
        obj(vec![
            ("songs", int(songs as i64)),
            ("charts", int(charts as i64)),
            ("aliases", int(aliases as i64)),
            ("chapters", int(chapters as i64)),
            ("version", int(titles)),
            ("source", jstr(&self.data_path.display().to_string())),
        ])
    }

    fn runtime_view(&self) -> String {
        let uptime = (now_seconds() - self.started_at).max(0) * 1000;
        obj(vec![
            ("goroutines", int(0)),
            ("heap_bytes", int(0)),
            ("sys_bytes", int(0)),
            ("gc_count", int(0)),
            ("uptime_ms", int(uptime)),
        ])
    }

    fn overview(&self) -> WebuiResponse {
        let (users, records, synced, failed) = (0_i64, 0_i64, 0_i64, 0_i64);
        WebuiResponse::ok(obj(vec![
            ("catalog", self.catalog_stats()),
            ("repo", repo_stats_json(users, records, synced, failed)),
            ("runtime", self.runtime_view()),
            ("chapters", self.chapters_json()),
            ("topics", topics_json()),
        ]))
    }

    async fn overview_async(&self, service: &Arc<PhigrosService>) -> WebuiResponse {
        let stats = service.repo().stats().await.unwrap_or_default();
        WebuiResponse::ok(obj(vec![
            ("catalog", self.catalog_stats()),
            (
                "repo",
                repo_stats_json(stats.users, stats.records, stats.synced, stats.failed),
            ),
            ("runtime", self.runtime_view()),
            ("chapters", self.chapters_json()),
            ("topics", topics_json()),
        ]))
    }

    fn events() -> WebuiResponse {
        let topics: Vec<String> = TOPICS.iter().map(|topic| jstr(topic)).collect();
        WebuiResponse::ok(obj(vec![
            ("stream", boolean(true)),
            ("topics", arr(&topics)),
            ("replay", int(0)),
            ("last_event_id", int(0)),
        ]))
    }

    fn chapters_json(&self) -> String {
        let catalog = match self.catalog() {
            Some(catalog) => catalog,
            None => return arr(&[]),
        };
        let items: Vec<String> = catalog.chapters().iter().map(chapter_json).collect();
        arr(&items)
    }

    fn chapters(&self) -> WebuiResponse {
        WebuiResponse::ok(obj(vec![("items", self.chapters_json())]))
    }

    fn songs(&self, query: &HashMap<String, String>) -> WebuiResponse {
        let filter = match SongFilter::parse(query) {
            Ok(filter) => filter,
            Err(detail) => return problem(400, "invalid_request", "输入不合法", &detail),
        };
        let catalog = match self.catalog() {
            Some(catalog) => catalog,
            None => {
                return WebuiResponse::ok(obj(vec![
                    ("items", arr(&[])),
                    ("total", int(0)),
                    ("limit", int(filter.limit as i64)),
                ]))
            }
        };
        let mut hits: Vec<&Song> = Vec::new();
        let mut total = 0_usize;
        let candidates: Vec<&Song> = if filter.text.is_empty() {
            catalog.songs().iter().collect()
        } else {
            catalog
                .search(&filter.text, catalog.songs().len())
                .into_iter()
                .map(|item| item.0)
                .collect()
        };
        for song in candidates {
            if !filter.matches(song) {
                continue;
            }
            total += 1;
            if hits.len() < filter.limit {
                hits.push(song);
            }
        }
        let items: Vec<String> = hits.iter().map(|song| song_json(song, &[])).collect();
        WebuiResponse::ok(obj(vec![
            ("items", arr(&items)),
            ("total", int(total as i64)),
            ("limit", int(filter.limit as i64)),
        ]))
    }

    fn song(&self, song_id: &str, entries: &[crate::repo::AliasEntry]) -> WebuiResponse {
        let catalog = match self.catalog() {
            Some(catalog) => catalog,
            None => return unavailable("曲库未就绪"),
        };
        let Some(song) = catalog.song(song_id) else {
            return problem(404, "not_found", "未找到", "曲目不存在");
        };
        WebuiResponse::ok(obj(vec![
            ("song", song_json(song, entries)),
            ("aliases", alias_json(&song.id, &song.aliases, entries)),
        ]))
    }

    fn alias_list(&self, song_id: &str, entries: &[crate::repo::AliasEntry]) -> WebuiResponse {
        let catalog = match self.catalog() {
            Some(catalog) => catalog,
            None => return unavailable("曲库未就绪"),
        };
        let Some(song) = catalog.song(song_id) else {
            return problem(404, "not_found", "未找到", "曲目不存在");
        };
        WebuiResponse::ok(alias_json(&song.id, &song.aliases, entries))
    }

    fn catalog_reload(&self) -> WebuiResponse {
        match self.reload_catalog() {
            Ok(_) => WebuiResponse::ok(self.catalog_stats()),
            Err(error) => problem(500, "internal", "服务内部错误", &error),
        }
    }

    fn scores(
        &self,
        user_id: &str,
        query: &HashMap<String, String>,
        best: Option<crate::model::Best>,
    ) -> WebuiResponse {
        let size = match query_int(query, "size", DEFAULT_BEST_SIZE, 1, MAX_BEST_SIZE) {
            Ok(size) => size,
            Err(detail) => return problem(400, "invalid_request", "输入不合法", &detail),
        };
        match best {
            Some(best) => WebuiResponse::ok(best_json(&best)),
            None => WebuiResponse::ok(empty_best_json(size, user_id)),
        }
    }

    async fn scores_async(
        &self,
        service: &Arc<PhigrosService>,
        user_id: &str,
        query: &HashMap<String, String>,
    ) -> WebuiResponse {
        let size = match query_int(query, "size", DEFAULT_BEST_SIZE, 1, MAX_BEST_SIZE) {
            Ok(size) => size,
            Err(detail) => return problem(400, "invalid_request", "输入不合法", &detail),
        };
        let scored = match service.scored(user_id).await {
            Ok(scored) => scored,
            Err(error) => return problem(500, "internal", "服务内部错误", &error),
        };
        let official = service
            .profile(user_id)
            .await
            .ok()
            .flatten()
            .map(|profile| profile.rks)
            .unwrap_or_default();
        let best = service.board(&scored, official, size);
        WebuiResponse::ok(best_json(&best))
    }

    fn config_get(&self) -> WebuiResponse {
        WebuiResponse::ok(obj(vec![
            ("values", self.values_json()),
            ("fields", fields_json()),
        ]))
    }

    fn values_json(&self) -> String {
        let stored = self.config_values();
        let fields: Vec<(&str, String)> = CONFIG_FIELDS
            .iter()
            .map(|field| {
                let value = stored.get(field.key).cloned().unwrap_or_default();
                (field.key, jstr(&value))
            })
            .collect();
        obj(fields)
    }

    fn config_put(&self, body: &str) -> WebuiResponse {
        let Some(pairs) = parse_object(body) else {
            return problem(400, "invalid_request", "输入不合法", "请求体不是合法 JSON");
        };
        for (key, _) in &pairs {
            if !CONFIG_FIELDS.iter().any(|field| field.key == key) {
                return problem(400, "invalid_request", "输入不合法", &format!("未知配置项：{key}"));
            }
        }
        match self.settings.apply(&pairs) {
            Ok(()) => WebuiResponse::ok(obj(vec![("values", self.values_json())])),
            Err(detail) => problem(400, "invalid_request", "输入不合法", &detail),
        }
    }

    async fn user_async(&self, service: &Arc<PhigrosService>, user_id: &str) -> WebuiResponse {
        let repo = service.repo();
        let binding = match repo.binding(user_id).await {
            Ok(binding) => binding,
            Err(error) => return problem(500, "internal", "服务内部错误", &error.to_string()),
        };
        let Some(binding) = binding else {
            return problem(404, "not_found", "未找到", "这个用户没有绑定记录");
        };
        let profile = repo.profile(user_id).await.ok().flatten();
        WebuiResponse::ok(obj(vec![
            ("binding", user_json(&binding)),
            (
                "profile",
                match profile.as_ref() {
                    Some(profile) => profile_json(profile),
                    None => String::from("null"),
                },
            ),
        ]))
    }

    async fn song_async(
        &self,
        service: &Arc<PhigrosService>,
        song_id: &str,
    ) -> WebuiResponse {
        let entries = service
            .repo()
            .alias_entries(song_id)
            .await
            .unwrap_or_default();
        self.song(song_id, &entries)
    }

    async fn alias_list_async(
        &self,
        service: &Arc<PhigrosService>,
        song_id: &str,
    ) -> WebuiResponse {
        let entries = service
            .repo()
            .alias_entries(song_id)
            .await
            .unwrap_or_default();
        self.alias_list(song_id, &entries)
    }

    async fn alias_edit_async(
        &self,
        service: &Arc<PhigrosService>,
        song_id: &str,
        body: &str,
    ) -> WebuiResponse {
        let parsed = match parse_object(body) {
            Some(pairs) => pairs,
            None => return problem(400, "invalid_request", "输入不合法", "请求体不是合法 JSON"),
        };
        if !self.song_exists(song_id) {
            return problem(404, "not_found", "未找到", "曲目不存在");
        }
        let mut alias = String::new();
        let mut allow = true;
        for (key, value) in parsed {
            match key.as_str() {
                "alias" => alias = value,
                "allow" => allow = !matches!(value.as_str(), "false" | "0"),
                _ => {}
            }
        }
        let alias = alias.trim().to_string();
        if alias.is_empty() {
            return problem(400, "invalid_request", "输入不合法", "alias 不能为空");
        }
        if let Err(error) = self.apply_alias(service, song_id, &alias, allow).await {
            return problem(500, "internal", "服务内部错误", &error);
        }
        let repo = service.repo();
        let entries = repo.alias_entries(song_id).await.unwrap_or_default();
        self.alias_list(song_id, &entries)
    }

    async fn alias_delete_async(
        &self,
        service: &Arc<PhigrosService>,
        song_id: &str,
        query: &HashMap<String, String>,
    ) -> WebuiResponse {
        let alias = query
            .get("alias")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if alias.is_empty() {
            return problem(400, "invalid_request", "输入不合法", "alias 不能为空");
        }
        if !self.song_exists(song_id) {
            return problem(404, "not_found", "未找到", "曲目不存在");
        }
        if let Err(error) = self.apply_alias(service, song_id, &alias, false).await {
            return problem(500, "internal", "服务内部错误", &error);
        }
        let repo = service.repo();
        let entries = repo.alias_entries(song_id).await.unwrap_or_default();
        self.alias_list(song_id, &entries)
    }

    fn song_exists(&self, song_id: &str) -> bool {
        match self.catalog() {
            Some(catalog) => catalog.song(song_id).is_some(),
            None => true,
        }
    }

    fn base_alias(&self, song_id: &str, alias: &str) -> bool {
        match self.catalog() {
            Some(catalog) => catalog
                .song(song_id)
                .map(|song| song.aliases.iter().any(|name| name == alias))
                .unwrap_or(false),
            None => false,
        }
    }

    async fn apply_alias(
        &self,
        service: &Arc<PhigrosService>,
        song_id: &str,
        alias: &str,
        allow: bool,
    ) -> Result<(), String> {
        let repo = service.repo();
        let written = match (allow, self.base_alias(song_id, alias)) {
            (true, true) => repo.drop_alias(song_id, alias).await,
            (true, false) => repo.upsert_alias(song_id, alias, now_seconds()).await,
            (false, true) => repo.delete_alias(song_id, alias, now_seconds()).await,
            (false, false) => repo.drop_alias(song_id, alias).await,
        };
        written.map_err(|error| error.to_string())?;
        service.refresh_aliases().await
    }
}

async fn users_async(service: &Arc<PhigrosService>) -> WebuiResponse {
    let repo: Repo = service.repo();
    let board = match repo.leader_board("", MAX_USER_PAGE).await {
        Ok(board) => board,
        Err(error) => return problem(500, "internal", "服务内部错误", &error.to_string()),
    };
    let mut items: Vec<String> = Vec::new();
    for row in &board.top {
        let binding = repo.binding(&row.user_id).await.ok().flatten();
        if let Some(binding) = binding {
            items.push(user_json(&binding));
        }
    }
    WebuiResponse::ok(obj(vec![
        ("items", arr(&items)),
        ("total", int(board.total)),
    ]))
}

async fn sync_async(service: &Arc<PhigrosService>, user_id: &str) -> WebuiResponse {
    let report = match service.sync(user_id).await {
        Ok(report) => report,
        Err(error) => return problem(502, "upstream_error", "上游服务异常", &error),
    };
    WebuiResponse::ok(obj(vec![
        ("added", int(report.events.len() as i64)),
        ("improved", int(report.events.len() as i64)),
        ("records", int(report.scored.len() as i64)),
        ("rks_before", number(0.0)),
        ("rks_after", number(report.profile.rks)),
        ("first_sync", boolean(report.first_run)),
    ]))
}

async fn delete_async(service: &Arc<PhigrosService>, user_id: &str) -> WebuiResponse {
    if let Err(error) = service.repo().delete_user(user_id).await {
        return problem(500, "internal", "服务内部错误", &error.to_string());
    }
    WebuiResponse::ok(obj(vec![("ok", boolean(true))]))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Health,
    Overview,
    Events,
    Songs,
    Chapters,
    Song(String),
    AliasList(String),
    AliasEdit(String),
    AliasDelete(String),
    CatalogReload,
    CatalogUpdate,
    Users,
    User(String),
    UserScores(String),
    UserSync(String),
    UserDelete(String),
    Config,
    ConfigSchema,
    ConfigPut,
}

fn classify(method: Method, parts: &[String]) -> Option<Route> {
    let api = parts.first().map(String::as_str)?;
    if api != "api" {
        return None;
    }
    let head = parts.get(1).map(String::as_str);
    let third = parts.get(2).map(String::as_str);
    let fourth = parts.get(3).map(String::as_str);
    match (method, head, parts.len()) {
        (Method::Get, Some("health"), 2) => Some(Route::Health),
        (Method::Get, Some("overview"), 2) => Some(Route::Overview),
        (Method::Get, Some("events"), 2) => Some(Route::Events),
        (Method::Get, Some("songs"), 2) => Some(Route::Songs),
        (Method::Get, Some("songs"), 3) if third == Some("chapters") => Some(Route::Chapters),
        (Method::Get, Some("songs"), 3) => Some(Route::Song(third?.to_string())),
        (Method::Get, Some("songs"), 4) if fourth == Some("aliases") => {
            Some(Route::AliasList(third?.to_string()))
        }
        (Method::Post, Some("songs"), 4) if fourth == Some("aliases") => {
            Some(Route::AliasEdit(third?.to_string()))
        }
        (Method::Delete, Some("songs"), 4) if fourth == Some("aliases") => {
            Some(Route::AliasDelete(third?.to_string()))
        }
        (Method::Post, Some("catalog"), 3) if third == Some("reload") => Some(Route::CatalogReload),
        (Method::Post, Some("catalog"), 3) if third == Some("update") => Some(Route::CatalogUpdate),
        (Method::Get, Some("users"), 2) => Some(Route::Users),
        (Method::Get, Some("users"), 3) => Some(Route::User(third?.to_string())),
        (Method::Get, Some("users"), 4) if fourth == Some("scores") => {
            Some(Route::UserScores(third?.to_string()))
        }
        (Method::Post, Some("users"), 4) if fourth == Some("sync") => {
            Some(Route::UserSync(third?.to_string()))
        }
        (Method::Delete, Some("users"), 3) => Some(Route::UserDelete(third?.to_string())),
        (Method::Get, Some("config"), 2) => Some(Route::Config),
        (Method::Get, Some("config"), 3) if third == Some("schema") => Some(Route::ConfigSchema),
        (Method::Put, Some("config"), 2) => Some(Route::ConfigPut),
        _ => None,
    }
}

struct SongFilter {
    text: String,
    chapter: String,
    constant: Option<(f64, f64)>,
    bpm: Option<(i32, i32)>,
    difficulties: Vec<Difficulty>,
    limit: usize,
}

impl SongFilter {
    fn parse(query: &HashMap<String, String>) -> Result<Self, String> {
        let text = query
            .get("query")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let chapter = query
            .get("chapter")
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let constant = match query.get("constant").map(|value| value.trim()) {
            Some(raw) if !raw.is_empty() => Some(parse_span_f64(raw).ok_or_else(|| {
                String::from("constant 需为定数或区间, 如 14.5 / 14.5-16.2 / 15(整档)")
            })?),
            _ => None,
        };
        let bpm = match query.get("bpm").map(|value| value.trim()) {
            Some(raw) if !raw.is_empty() => Some(
                parse_span_i32(raw)
                    .ok_or_else(|| String::from("bpm 需为整数或区间, 如 150 / 150-200"))?,
            ),
            _ => None,
        };
        let mut difficulties: Vec<Difficulty> = Vec::new();
        if let Some(raw) = query.get("difficulty") {
            for part in raw.split(',') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let found = Difficulty::ALL
                    .iter()
                    .copied()
                    .find(|item| item.name().eq_ignore_ascii_case(part));
                let Some(found) = found.filter(|item| item.playable()) else {
                    return Err(format!("未知难度：{part}"));
                };
                if !difficulties.contains(&found) {
                    difficulties.push(found);
                }
            }
        }
        let limit = query_int(query, "limit", DEFAULT_SONG_LIMIT, 0, MAX_SONG_LIMIT)?;
        Ok(Self {
            text,
            chapter,
            constant,
            bpm,
            difficulties,
            limit,
        })
    }

    fn matches(&self, song: &Song) -> bool {
        if !self.chapter.is_empty()
            && song.chapter != self.chapter
            && song.version_label != self.chapter
        {
            return false;
        }
        if let Some((low, high)) = self.bpm {
            if song.bpm < low || song.bpm > high {
                return false;
            }
        }
        let levels: Vec<Difficulty> = if self.difficulties.is_empty() {
            Difficulty::playable_all().to_vec()
        } else {
            self.difficulties.clone()
        };
        levels.iter().any(|level| match song.chart(*level) {
            Some(chart) => match self.constant {
                Some((low, high)) => chart.constant >= low && chart.constant <= high,
                None => true,
            },
            None => false,
        })
    }
}

fn parse_span_f64(raw: &str) -> Option<(f64, f64)> {
    if let Some((low, high)) = raw.split_once('-') {
        let low = low.trim().parse::<f64>().ok()?;
        let high = high.trim().parse::<f64>().ok()?;
        if low > high {
            return None;
        }
        return Some((low, high));
    }
    let value = raw.trim().parse::<f64>().ok()?;
    if value.fract() == 0.0 {
        return Some((value, value + 0.9999));
    }
    Some((value, value))
}

fn parse_span_i32(raw: &str) -> Option<(i32, i32)> {
    if let Some((low, high)) = raw.split_once('-') {
        let low = low.trim().parse::<i32>().ok()?;
        let high = high.trim().parse::<i32>().ok()?;
        if low > high {
            return None;
        }
        return Some((low, high));
    }
    let value = raw.trim().parse::<i32>().ok()?;
    Some((value, value))
}

fn query_int(
    query: &HashMap<String, String>,
    key: &str,
    fallback: usize,
    min: usize,
    max: usize,
) -> Result<usize, String> {
    let Some(raw) = query.get(key).map(|value| value.trim()) else {
        return Ok(fallback);
    };
    if raw.is_empty() {
        return Ok(fallback);
    }
    match raw.parse::<usize>() {
        Ok(value) if value >= min && value <= max => Ok(value),
        _ => Err(format!("{key} 需为 {min}~{max} 的整数")),
    }
}

fn split_target(path: &str) -> (Vec<String>, HashMap<String, String>) {
    let (route, query) = match path.split_once('?') {
        Some((route, query)) => (route, query),
        None => (path, ""),
    };
    let parts: Vec<String> = route
        .split('/')
        .filter(|part| !part.is_empty())
        .map(unescape_text)
        .collect();
    let mut params: HashMap<String, String> = HashMap::new();
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };
        params.insert(unescape_text(key), unescape_text(value));
    }
    (parts, params)
}

fn unescape_text(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 3 <= bytes.len() => {
                match u8::from_str_radix(&text[index + 1..index + 3], 16) {
                    Ok(value) => {
                        out.push(value);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn chapter_json(chapter: &Chapter) -> String {
    let aliases: Vec<String> = chapter.aliases.iter().map(|alias| jstr(alias)).collect();
    obj(vec![("name", jstr(&chapter.name)), ("aliases", arr(&aliases))])
}

fn song_json(song: &Song, entries: &[crate::repo::AliasEntry]) -> String {
    let mut fields: Vec<(&'static str, String)> = Vec::new();
    fields.push(("id", jstr(&song.id)));
    fields.push(("title", jstr(&song.title)));
    text_field(&mut fields, "composer", &song.composer);
    text_field(&mut fields, "illustrator", &song.illustrator);
    int_optional(&mut fields, "bpm", song.bpm as i64);
    text_field(&mut fields, "length", &song.length);
    text_field(&mut fields, "chapter", &song.chapter);
    int_optional(&mut fields, "version", song.version as i64);
    flag_field(&mut fields, "original", song.original);
    text_field(&mut fields, "sp", &song.sp);
    let merged = merged_aliases(&song.aliases, entries);
    let aliases: Vec<String> = merged.iter().map(|alias| jstr(alias)).collect();
    fields.push(("aliases", arr(&aliases)));
    let mut charts: Vec<String> = Vec::new();
    for difficulty in song.all_difficulties() {
        if let Some(chart) = song.chart(difficulty) {
            charts.push(chart_json(chart, difficulty));
        }
    }
    fields.push(("charts", arr(&charts)));
    obj(fields)
}

fn chart_json(chart: &Chart, difficulty: Difficulty) -> String {
    let mut fields: Vec<(&'static str, String)> = Vec::new();
    fields.push(("difficulty", jstr(difficulty.name())));
    fields.push(("constant", number(chart.constant)));
    text_field(&mut fields, "charter", &chart.charter);
    int_optional(&mut fields, "notes", chart.notes.combo() as i64);
    float_optional(&mut fields, "duration", chart.duration);
    if !chart.history.is_empty() {
        let history: Vec<String> = chart
            .history
            .iter()
            .map(|change| {
                obj(vec![
                    ("version", int(change.version as i64)),
                    ("from", number(change.from)),
                    ("to", number(change.to)),
                ])
            })
            .collect();
        fields.push(("history", arr(&history)));
    }
    obj(fields)
}

fn user_json(binding: &Binding) -> String {
    let mut fields: Vec<(&'static str, String)> = Vec::new();
    fields.push(("user_id", jstr(&binding.user_id)));
    fields.push(("server", jstr(server_label(binding.server))));
    text_field(&mut fields, "token_tail", &token_tail(&binding.token));
    text_field(&mut fields, "nickname", &binding.nickname);
    text_field(&mut fields, "object_id", &binding.object_id);
    text_field(&mut fields, "avatar", &binding.avatar);
    fields.push(("status", jstr(sync_status_label(binding.status))));
    text_field(&mut fields, "error", &binding.error);
    int_optional(&mut fields, "bound_at", millis(binding.bound_at));
    int_optional(&mut fields, "last_sync_at", millis(binding.last_sync_at));
    int_optional(
        &mut fields,
        "save_modified_at",
        millis(binding.save_modified_at),
    );
    obj(fields)
}

fn profile_json(profile: &Profile) -> String {
    let dan = overview::dan_of(profile.challenge_mode_rank as i64);
    let mut fields: Vec<(&'static str, String)> = vec![
        ("rks", number(profile.rks)),
        ("dan_rank", int(dan.level as i64)),
        ("dan_progress", int(dan.progress as i64)),
        ("money", jstr(&money_text(&profile.counters.money))),
        ("cleared", ints(&profile.counters.cleared)),
        ("full_combo", ints(&profile.counters.full_combo)),
        ("phi", ints(&profile.counters.phi)),
    ];
    text_field(&mut fields, "self_intro", &profile.self_intro);
    text_field(&mut fields, "background", &profile.background);
    int_optional(&mut fields, "game_version", profile.game_version as i64);
    int_optional(&mut fields, "save_version", profile.save_version as i64);
    int_optional(&mut fields, "updated_at", millis(profile.updated_at));
    obj(fields)
}

fn scored_json(item: &Scored) -> String {
    let mut fields: Vec<(&'static str, String)> = Vec::new();
    fields.push(("song_id", jstr(&item.record.song_id)));
    text_field(&mut fields, "title", &item.title);
    text_field(&mut fields, "composer", &item.composer);
    fields.push(("difficulty", jstr(item.record.difficulty.name())));
    float_optional(&mut fields, "constant", item.constant);
    fields.push(("known", boolean(item.known)));
    fields.push(("score", int(item.record.score as i64)));
    fields.push(("acc", number(item.record.acc)));
    fields.push(("fc", boolean(item.record.fc)));
    fields.push(("rks", number(item.rks)));
    fields.push(("grade", jstr(item.grade.name())));
    int_optional(&mut fields, "rank", item.rank as i64);
    obj(fields)
}

fn scored_list(items: &[Scored]) -> String {
    let rows: Vec<String> = items.iter().map(scored_json).collect();
    arr(&rows)
}

fn best_json(best: &crate::model::Best) -> String {
    obj(vec![
        ("official", number(best.official)),
        ("computed", number(best.computed)),
        ("avg_all", number(best.avg_all)),
        ("round_up", number(best.round_up)),
        ("records", int(best.records as i64)),
        ("counted", int(best.overflow() as i64)),
        ("unknown", int(best.unknown as i64)),
        ("suspect", int(best.suspect as i64)),
        ("podium", scored_list(&best.podium)),
        ("top", scored_list(&best.top)),
        ("phi", scored_list(&best.phi)),
    ])
}

fn empty_best_json(_size: usize, _user_id: &str) -> String {
    obj(vec![
        ("official", number(0.0)),
        ("computed", number(0.0)),
        ("avg_all", number(0.0)),
        ("round_up", number(0.0)),
        ("records", int(0)),
        ("counted", int(0)),
        ("unknown", int(0)),
        ("suspect", int(0)),
        ("podium", arr(&[])),
        ("top", arr(&[])),
        ("phi", arr(&[])),
    ])
}

fn alias_json(song_id: &str, base: &[String], entries: &[crate::repo::AliasEntry]) -> String {
    let mut added: Vec<String> = Vec::new();
    let mut removed: Vec<String> = Vec::new();
    for entry in entries {
        if entry.song_id != song_id {
            continue;
        }
        if entry.removed {
            removed.push(entry.alias.clone());
        } else {
            added.push(entry.alias.clone());
        }
    }
    let effective = merged_aliases(base, entries);
    let base_items: Vec<String> = base.iter().map(|alias| jstr(alias)).collect();
    let added_items: Vec<String> = added.iter().map(|alias| jstr(alias)).collect();
    let removed_items: Vec<String> = removed.iter().map(|alias| jstr(alias)).collect();
    let effective_items: Vec<String> = effective.iter().map(|alias| jstr(alias)).collect();
    obj(vec![
        ("song_id", jstr(song_id)),
        ("base", arr(&base_items)),
        ("added", arr(&added_items)),
        ("removed", arr(&removed_items)),
        ("effective", arr(&effective_items)),
    ])
}

fn merged_aliases(base: &[String], entries: &[crate::repo::AliasEntry]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for alias in base {
        if alias.trim().is_empty() {
            continue;
        }
        if entries
            .iter()
            .any(|entry| entry.removed && entry.alias == *alias)
        {
            continue;
        }
        if out.iter().any(|item| item == alias) {
            continue;
        }
        out.push(alias.clone());
    }
    for entry in entries {
        if entry.removed || entry.alias.trim().is_empty() {
            continue;
        }
        if out.iter().any(|item| item == &entry.alias) {
            continue;
        }
        out.push(entry.alias.clone());
    }
    out
}

fn token_tail(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 6 {
        return token.to_string();
    }
    chars[chars.len() - 6..].iter().collect()
}

fn sync_status_label(status: i64) -> &'static str {
    match status {
        crate::repo::SYNC_OK => "同步正常",
        crate::repo::SYNC_FAILED => "同步失败",
        _ => "尚未同步",
    }
}

fn millis(seconds: i64) -> i64 {
    if seconds <= 0 {
        return 0;
    }
    seconds.saturating_mul(1000)
}

fn money_text(values: &[i32]) -> String {
    const UNITS: [&str; 5] = ["KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut parts: Vec<String> = Vec::new();
    for index in (0..UNITS.len()).rev() {
        if let Some(value) = values.get(index) {
            if *value != 0 {
                parts.push(format!("{value} {}", UNITS[index]));
            }
        }
    }
    if parts.is_empty() {
        return String::from("0 KiB");
    }
    parts.join(" ")
}

fn repo_stats_json(users: i64, records: i64, synced: i64, failed: i64) -> String {
    obj(vec![
        ("users", int(users)),
        ("records", int(records)),
        ("synced", int(synced)),
        ("failed", int(failed)),
    ])
}

fn topics_json() -> String {
    let items: Vec<String> = TOPICS.iter().map(|topic| jstr(topic)).collect();
    arr(&items)
}

fn fields_json() -> String {
    let items: Vec<String> = CONFIG_FIELDS
        .iter()
        .map(|field| {
            let mut fields: Vec<(&'static str, String)> = Vec::new();
            fields.push(("key", jstr(field.key)));
            fields.push(("label", jstr(field.label)));
            text_field(&mut fields, "description", field.description);
            fields.push(("type", jstr(field.kind)));
            text_field(&mut fields, "placeholder", field.placeholder);
            obj(fields)
        })
        .collect();
    arr(&items)
}

fn text_field(fields: &mut Vec<(&'static str, String)>, key: &'static str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    fields.push((key, jstr(value)));
}

fn int_optional(fields: &mut Vec<(&'static str, String)>, key: &'static str, value: i64) {
    if value == 0 {
        return;
    }
    fields.push((key, int(value)));
}

fn float_optional(fields: &mut Vec<(&'static str, String)>, key: &'static str, value: f64) {
    if value == 0.0 || !value.is_finite() {
        return;
    }
    fields.push((key, number(value)));
}

fn flag_field(fields: &mut Vec<(&'static str, String)>, key: &'static str, value: bool) {
    if !value {
        return;
    }
    fields.push((key, boolean(value)));
}

pub fn esc(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            other if (other as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn jstr(text: &str) -> String {
    format!("\"{}\"", esc(text))
}

fn number(value: f64) -> String {
    if !value.is_finite() {
        return String::from("0");
    }
    let text = format!("{value:.6}");
    let trimmed = text.trim_end_matches('0');
    let trimmed = trimmed.strip_suffix('.').unwrap_or(trimmed);
    if trimmed.is_empty() || trimmed == "-" {
        return String::from("0");
    }
    trimmed.to_string()
}

fn int(value: i64) -> String {
    value.to_string()
}

fn boolean(value: bool) -> String {
    if value {
        return String::from("true");
    }
    String::from("false")
}

fn arr(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

fn ints(items: &[i32]) -> String {
    let rows: Vec<String> = items.iter().map(|value| int(*value as i64)).collect();
    arr(&rows)
}

fn obj(fields: Vec<(&'static str, String)>) -> String {
    let mut out = String::from("{");
    for (index, (key, value)) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&jstr(key));
        out.push(':');
        out.push_str(value);
    }
    out.push('}');
    out
}

fn problem_json(code: &str, title: &str, status: u16, detail: &str) -> String {
    let mut fields: Vec<(&'static str, String)> = Vec::new();
    fields.push(("code", jstr(code)));
    fields.push(("title", jstr(title)));
    fields.push(("status", int(status as i64)));
    text_field(&mut fields, "detail", detail);
    obj(fields)
}

fn problem(status: u16, code: &str, title: &str, detail: &str) -> WebuiResponse {
    WebuiResponse {
        status,
        json: problem_json(code, title, status, detail),
    }
}

fn unavailable(detail: &str) -> WebuiResponse {
    WebuiResponse {
        status: 503,
        json: obj(vec![
            ("ok", boolean(false)),
            ("error", jstr(detail)),
            ("code", jstr("busy")),
        ]),
    }
}

fn parse_object(body: &str) -> Option<Vec<(String, String)>> {
    let chars: Vec<char> = body.chars().collect();
    let mut index = 0;
    skip_ws(&chars, &mut index);
    if chars.get(index) != Some(&'{') {
        return None;
    }
    index += 1;
    let mut out: Vec<(String, String)> = Vec::new();
    loop {
        skip_ws(&chars, &mut index);
        match chars.get(index) {
            Some('}') => {
                index += 1;
                break;
            }
            Some(',') => {
                index += 1;
                continue;
            }
            Some('"') => {}
            _ => return None,
        }
        let key = parse_string(&chars, &mut index)?;
        skip_ws(&chars, &mut index);
        if chars.get(index) != Some(&':') {
            return None;
        }
        index += 1;
        skip_ws(&chars, &mut index);
        let value = parse_value(&chars, &mut index)?;
        out.push((key, value));
    }
    skip_ws(&chars, &mut index);
    if index != chars.len() {
        return None;
    }
    Some(out)
}

fn skip_ws(chars: &[char], index: &mut usize) {
    while let Some(ch) = chars.get(*index) {
        if !ch.is_whitespace() {
            break;
        }
        *index += 1;
    }
}

fn parse_string(chars: &[char], index: &mut usize) -> Option<String> {
    if chars.get(*index) != Some(&'"') {
        return None;
    }
    *index += 1;
    let mut out = String::new();
    while let Some(ch) = chars.get(*index) {
        *index += 1;
        match ch {
            '"' => return Some(out),
            '\\' => {
                let escape = *chars.get(*index)?;
                *index += 1;
                match escape {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let mut code = 0_u32;
                        for _ in 0..4 {
                            let digit = *chars.get(*index)?;
                            *index += 1;
                            code = code * 16 + digit.to_digit(16)?;
                        }
                        out.push(char::from_u32(code).unwrap_or('?'));
                    }
                    _ => return None,
                }
            }
            other => out.push(*other),
        }
    }
    None
}

fn parse_value(chars: &[char], index: &mut usize) -> Option<String> {
    match chars.get(*index) {
        Some('"') => parse_string(chars, index),
        Some('{') | Some('[') => {
            let open = *chars.get(*index)?;
            let close = if open == '{' { '}' } else { ']' };
            let mut depth = 0_usize;
            let started = *index;
            while let Some(ch) = chars.get(*index) {
                if *ch == open {
                    depth += 1;
                } else if *ch == close {
                    depth -= 1;
                    if depth == 0 {
                        *index += 1;
                        let raw: String = chars[started..*index].iter().collect();
                        return Some(raw);
                    }
                }
                *index += 1;
            }
            None
        }
        _ => {
            let started = *index;
            while let Some(ch) = chars.get(*index) {
                if matches!(ch, ',' | '}' | ' ') {
                    break;
                }
                *index += 1;
            }
            if started == *index {
                return None;
            }
            let raw: String = chars[started..*index].iter().collect();
            Some(raw.trim().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Chapter, Chart, Notes};

    fn detached() -> WebuiService {
        WebuiService::detached()
    }

    #[test]
    fn every_documented_route_is_dispatched() {
        let service = detached();
        let get = [
            "/api/health",
            "/api/overview",
            "/api/events",
            "/api/songs",
            "/api/songs/chapters",
            "/api/users",
            "/api/users/10001",
            "/api/users/10001/scores",
            "/api/config",
            "/api/config/schema",
        ];
        for path in get {
            let response = service.route(&WebuiRequest::new(Method::Get, path));
            assert_eq!(response.status, 200, "{path} 应当命中: {}", response.json);
            assert!(response.json.starts_with('{'), "{path} 必须回 JSON 对象");
        }
        for path in ["/api/songs/Chart-1", "/api/songs/Chart-1/aliases"] {
            let response = service.route(&WebuiRequest::new(Method::Get, path));
            assert_eq!(response.status, 503, "{path} 无曲库要降级: {}", response.json);
        }
        let posts = ["/api/catalog/reload", "/api/catalog/update"];
        for path in posts {
            let response = service.route(&WebuiRequest::new(Method::Post, path));
            assert!(response.status == 200 || response.status == 500 || response.status == 501);
            assert!(response.json.starts_with('{'));
        }
        let alias_post = service.route(&WebuiRequest::with_body(
            Method::Post,
            "/api/songs/Chart-1/aliases",
            "{\"alias\":\"neo\"}",
        ));
        assert_eq!(alias_post.status, 503);
        let alias_delete = service.route(&WebuiRequest::new(
            Method::Delete,
            "/api/songs/Chart-1/aliases?alias=neo",
        ));
        assert_eq!(alias_delete.status, 503);
        let sync = service.route(&WebuiRequest::new(Method::Post, "/api/users/10001/sync"));
        assert_eq!(sync.status, 503);
        let unbind = service.route(&WebuiRequest::new(Method::Delete, "/api/users/10001"));
        assert_eq!(unbind.status, 503);
        let config = service.route(&WebuiRequest::with_body(
            Method::Put,
            "/api/config",
            "{\"search_limit\":\"120\"}",
        ));
        assert_eq!(config.status, 200);
        assert!(config.json.contains("\"search_limit\":\"120\""));
        assert_eq!(classify(Method::Get, &["api".to_string()]), None);
    }

    #[test]
    fn unknown_paths_and_wrong_methods_are_not_found() {
        let service = detached();
        for (method, path) in [
            (Method::Get, "/api/nope"),
            (Method::Get, "/"),
            (Method::Get, ""),
            (Method::Post, "/api/health"),
            (Method::Delete, "/api/config"),
            (Method::Put, "/api/users/10001"),
            (Method::Get, "/api/songs/Chart-1/aliases/extra"),
            (Method::Post, "/api/users/10001/nope"),
        ] {
            let response = service.route(&WebuiRequest::new(method, path));
            assert_eq!(response.status, 404, "{path} 应当 404: {}", response.json);
            assert!(response.json.contains("\"code\":\"not_found\""));
        }
    }

    #[test]
    fn users_and_overview_degrade_without_a_database() {
        let service = detached();
        let users = service.route(&WebuiRequest::new(Method::Get, "/api/users"));
        assert_eq!(users.status, 200);
        assert_eq!(users.json, String::from("{\"items\":[],\"total\":0}"));
        let overview = service.route(&WebuiRequest::new(Method::Get, "/api/overview"));
        assert_eq!(overview.status, 200);
        assert!(overview.json.contains("\"repo\":{\"users\":0"));
        assert!(overview.json.contains("\"topics\":[\"system.hello\""));
        let health = service.route(&WebuiRequest::new(Method::Get, "/api/health"));
        assert_eq!(health.json, String::from("{\"ok\":true,\"catalog\":false}"));
        let scores = service.route(&WebuiRequest::new(Method::Get, "/api/users/10001/scores"));
        assert_eq!(scores.status, 200);
        assert!(scores.json.contains("\"podium\":[]"));
        let detail = service.route(&WebuiRequest::new(Method::Get, "/api/users/10001"));
        assert_eq!(detail.status, 200);
        assert!(detail.json.contains("\"binding\":null"));
    }

    #[test]
    fn config_put_then_get_round_trips() {
        let service = detached();
        let before = service.route(&WebuiRequest::new(Method::Get, "/api/config"));
        assert_eq!(before.status, 200);
        assert!(before.json.contains("\"search_limit\":\"\""));
        assert!(before.json.contains("\"key\":\"push_per_page\""));
        let put = service.route(&WebuiRequest::with_body(
            Method::Put,
            "/api/config",
            "{\"search_limit\":\"150\",\"webp_enabled\":true,\"save_proxy\":\"http://127.0.0.1:8080\"}",
        ));
        assert_eq!(put.status, 200);
        let after = service.route(&WebuiRequest::new(Method::Get, "/api/config"));
        assert!(after.json.contains("\"search_limit\":\"150\""));
        assert!(after.json.contains("\"webp_enabled\":\"true\""));
        assert!(after.json.contains("\"save_proxy\":\"http://127.0.0.1:8080\""));
        let values = service.config_values();
        assert_eq!(values.get("search_limit").map(String::as_str), Some("150"));
        let bad = service.route(&WebuiRequest::with_body(
            Method::Put,
            "/api/config",
            "{\"nope\":\"1\"}",
        ));
        assert_eq!(bad.status, 400);
        assert!(bad.json.contains("未知配置项"));
        let broken = service.route(&WebuiRequest::with_body(Method::Put, "/api/config", "[]"));
        assert_eq!(broken.status, 400);
    }

    #[test]
    fn json_helpers_escape_quotes_and_control_characters() {
        assert_eq!(esc("a\"b\\c"), String::from("a\\\"b\\\\c"));
        assert_eq!(esc("line\nnext\ttab\r"), String::from("line\\nnext\\ttab\\r"));
        assert_eq!(esc("\u{1}\u{1f}"), String::from("\\u0001\\u001f"));
        assert_eq!(
            obj(vec![("a", jstr("x\ny")), ("b", int(2))]),
            String::from("{\"a\":\"x\\ny\",\"b\":2}")
        );
        assert_eq!(arr(&[jstr("一"), jstr("二")]), String::from("[\"一\",\"二\"]"));
        assert_eq!(number(15.0), String::from("15"));
        assert_eq!(number(15.25), String::from("15.25"));
        assert_eq!(number(0.0), String::from("0"));
        assert_eq!(number(f64::NAN), String::from("0"));
        assert_eq!(boolean(true), String::from("true"));
        assert_eq!(int(-7), String::from("-7"));
        assert_eq!(
            problem_json("not_found", "未找到", 404, ""),
            String::from("{\"code\":\"not_found\",\"title\":\"未找到\",\"status\":404}")
        );
    }

    #[test]
    fn song_and_chart_views_keep_go_field_names() {
        let mut song = Song {
            id: String::from("Chart-7"),
            title: String::from("Spasmodic"),
            composer: String::from("プラズマ"),
            illustrator: String::from(""),
            bpm: 200,
            length: String::from("2:12"),
            chapter: String::from("霓虹灯牌"),
            version: 3,
            original: true,
            sp: String::from(""),
            aliases: vec![String::from("痉挛")],
            ..Song::default()
        };
        song.charts[0] = Chart {
            difficulty: Difficulty::Ez,
            constant: 7.0,
            charter: String::from("charter"),
            notes: Notes {
                tap: 100,
                drag: 20,
                hold: 5,
                flick: 3,
            },
            duration: 132.0,
            history: Vec::new(),
        };
        let view = song_json(&song, &[]);
        for key in [
            "\"id\":\"Chart-7\"",
            "\"title\":\"Spasmodic\"",
            "\"composer\":\"プラズマ\"",
            "\"bpm\":200",
            "\"chapter\":\"霓虹灯牌\"",
            "\"original\":true",
            "\"aliases\":[\"痉挛\"]",
            "\"charts\":[",
            "\"difficulty\":\"EZ\"",
            "\"constant\":7",
            "\"notes\":128",
            "\"duration\":132",
        ] {
            assert!(view.contains(key), "缺 {key}: {view}");
        }
        assert!(!view.contains("illustrator"), "空字段要省略: {view}");
        assert!(!view.contains("history"), "空历史要省略: {view}");
        let chapter = Chapter {
            name: String::from("霓虹灯牌"),
            display: String::from("霓虹灯牌"),
            aliases: vec![String::from("neon")],
        };
        assert_eq!(
            chapter_json(&chapter),
            String::from("{\"name\":\"霓虹灯牌\",\"aliases\":[\"neon\"]}")
        );
        let merged = merged_aliases(
            &[String::from("a"), String::from("b")],
            &[crate::repo::AliasEntry {
                song_id: String::from("x"),
                alias: String::from("b"),
                removed: true,
                updated_at: 1,
            }],
        );
        assert_eq!(merged, vec![String::from("a")]);
        assert_eq!(token_tail("abcdefgh"), String::from("cdefgh"));
        assert_eq!(sync_status_label(crate::repo::SYNC_OK), "同步正常");
        assert_eq!(sync_status_label(0), "尚未同步");
        assert_eq!(millis(2), 2000);
        assert_eq!(money_text(&[1, 0, 2, 0]), String::from("2 GiB 1 KiB"));
        assert_eq!(money_text(&[0, 0, 0, 0]), String::from("0 KiB"));
    }

    #[test]
    fn filters_accept_spans_and_reject_junk() {
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert(String::from("constant"), String::from("15"));
        let filter = SongFilter::parse(&query).expect("整档定数");
        assert_eq!(filter.constant, Some((15.0, 15.9999)));
        query.insert(String::from("constant"), String::from("14.5-16.2"));
        assert_eq!(
            SongFilter::parse(&query).expect("区间").constant,
            Some((14.5, 16.2))
        );
        query.insert(String::from("constant"), String::from("abc"));
        assert!(SongFilter::parse(&query).is_err());
        query.remove("constant");
        query.insert(String::from("bpm"), String::from("150-200"));
        assert_eq!(
            SongFilter::parse(&query).expect("bpm 区间").bpm,
            Some((150, 200))
        );
        query.remove("bpm");
        query.insert(String::from("difficulty"), String::from("IN,at"));
        let filter = SongFilter::parse(&query).expect("难度");
        assert_eq!(filter.difficulties, vec![Difficulty::In, Difficulty::At]);
        query.insert(String::from("difficulty"), String::from("XX"));
        assert!(SongFilter::parse(&query).is_err());
        query.remove("difficulty");
        query.insert(String::from("limit"), String::from("999"));
        assert!(SongFilter::parse(&query).is_err());
        query.insert(String::from("limit"), String::from("12"));
        assert_eq!(SongFilter::parse(&query).expect("上限").limit, 12);
        let (parts, params) = split_target("/api/songs?query=Spas%20modic&limit=5");
        assert_eq!(parts, vec![String::from("api"), String::from("songs")]);
        assert_eq!(params.get("query").map(String::as_str), Some("Spas modic"));
        assert_eq!(params.get("limit").map(String::as_str), Some("5"));
    }

    #[test]
    fn catalog_is_loaded_lazily_and_reload_reports_errors() {
        let service = WebuiService::with_catalog_path("/definitely/missing/phi.db");
        assert!(service.catalog().is_none());
        assert!(!service.attached());
        assert_eq!(service.data_path(), Path::new("/definitely/missing/phi.db"));
        let reload = service.reload_catalog();
        assert!(reload.is_err());
        let response = service.route(&WebuiRequest::new(Method::Post, "/api/catalog/reload"));
        assert_eq!(response.status, 500);
        assert!(response.json.contains("曲库重载失败"));
        assert_eq!(CONFIG_FIELDS.len(), 28);
        assert!(CONFIG_FIELDS.iter().any(|field| field.key == "data_file"));
    }
}
