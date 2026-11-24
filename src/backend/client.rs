use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{Semaphore, SemaphorePermit};

use crate::model::{Archive, Server};
use crate::settings::{Settings, SettingsStore};

use super::error::{BackendError, Kind};
use super::savefile::{self, SaveError};

const USER_AGENT: &str = "LeanCloud-CSharp-SDK/1.0.3";

const MAX_BODY: usize = 16 << 20;
#[derive(Clone, Copy, Debug)]
struct Endpoint {
    lc_base: &'static str,
    lc_id: &'static str,
    lc_key: &'static str,
}

fn endpoint_of(server: Server) -> Endpoint {
    match server {
        Server::Global => Endpoint {
            lc_base: "https://kviehlel.cloud.ap-sg.tapapis.com/1.1",
            lc_id: "kviehleldgxsagpozb",
            lc_key: "tG9CTm0LDD736k9HMM9lBZrbeBGRmUkjSfNLDNib",
        },
        Server::Cn => Endpoint {
            lc_base: "https://rak3ffdi.cloud.tds1.tapapis.cn/1.1",
            lc_id: "rAK3FfdieFob2Nn8Am",
            lc_key: "Qr9AEqtuoSVS3zeD6iVbM4ZC0AtkJcQ89tywVyi0",
        },
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerInfo {
    pub object_id: String,
    pub nickname: String,
    pub avatar: String,
    pub short_id: String,
    pub joined_at: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SaveStamp {
    pub object_id_len: usize,
    pub modified_at: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveRef {
    pub object_id: String,
    pub modified_at: i64,
    pub summary: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FetchedSave {
    pub archive: Archive,
    pub summary: crate::model::Summary,
    pub player: PlayerInfo,
    pub object_id: String,
    pub modified_at: i64,
}

pub struct SaveClient {
    settings: Arc<SettingsStore>,
    http: Mutex<Option<(String, reqwest::Client)>>,
    permits: Semaphore,
    applied: Mutex<i64>,
}

impl SaveClient {
    pub fn new() -> Result<Self, BackendError> {
        let path = crate::service::settings_path(&crate::assets_root());
        Ok(Self {
            settings: Arc::new(SettingsStore::load(Some(path))),
            http: Mutex::new(None),
            permits: Semaphore::new(0),
            applied: Mutex::new(0),
        })
    }

    pub fn settings_store(&self) -> &Arc<SettingsStore> {
        &self.settings
    }

    pub async fn player(
        &self,
        server: Server,
        token: &str,
    ) -> Result<PlayerInfo, BackendError> {
        let endpoint = endpoint_of(server);
        let url = format!("{}/users/me", endpoint.lc_base);
        let payload = self.get(&url, Some((endpoint, token))).await?;
        Ok(parse_player(&payload))
    }

    pub async fn saves(
        &self,
        server: Server,
        token: &str,
        object_id: &str,
    ) -> Result<Vec<SaveRef>, BackendError> {
        let endpoint = endpoint_of(server);
        let where_clause = format!(
            "{{\"user\":{{\"__type\":\"Pointer\",\"className\":\"_User\",\"objectId\":\"{object_id}\"}}}}"
        );
        let url = format!(
            "{}/gamesaves?skip=0&limit=100&where={}&include=cover,gameFile",
            endpoint.lc_base,
            urlencode(&where_clause)
        );
        let payload = self.get(&url, Some((endpoint, token))).await?;
        parse_saves(&payload)
    }

    pub async fn download(&self, url: &str) -> Result<Vec<u8>, BackendError> {
        if url.is_empty() {
            return Err(BackendError::new(Kind::SaveEmpty, "存档缺少下载地址"));
        }
        self.get(url, None).await
    }

    pub async fn newest(
        &self,
        server: Server,
        token: &str,
        object_id: &str,
    ) -> Result<Option<SaveRef>, BackendError> {
        let mut saves = self.saves(server, token, object_id).await?;
        saves.retain(|item| !item.url.is_empty());
        saves.sort_by_key(|item| std::cmp::Reverse(item.modified_at));
        Ok(saves.into_iter().next())
    }

    pub async fn probe(
        &self,
        server: Server,
        token: &str,
        known_object_id: &str,
    ) -> Result<SaveStamp, BackendError> {
        let object_id = if known_object_id.is_empty() {
            self.player(server, token).await?.object_id
        } else {
            known_object_id.to_string()
        };
        let latest = self
            .newest(server, token, &object_id)
            .await?
            .ok_or_else(|| BackendError::new(Kind::SaveEmpty, "云端没有可用的存档文件"))?;
        Ok(SaveStamp {
            object_id_len: latest.object_id.len(),
            modified_at: latest.modified_at,
        })
    }

    pub async fn archive(
        &self,
        server: Server,
        token: &str,
    ) -> Result<FetchedSave, BackendError> {
        let player = self.player(server, token).await?;
        let latest = self
            .newest(server, token, &player.object_id)
            .await?
            .ok_or_else(|| BackendError::new(Kind::SaveEmpty, "云端没有可用的存档文件"))?;
        let payload = self.download(&latest.url).await?;
        let archive = savefile::parse_zip(&payload).map_err(archive_error)?;
        let summary = savefile::parse_summary(&latest.summary).map_err(|_| {
            BackendError::new(Kind::SaveTampered, "存档摘要无法解析")
        })?;
        Ok(FetchedSave {
            archive,
            summary,
            player,
            object_id: latest.object_id,
            modified_at: latest.modified_at,
        })
    }
    fn http(&self, settings: &Settings) -> Result<reqwest::Client, BackendError> {
        let proxy = settings.save_proxy.trim();
        let mut guard = self
            .http
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((cached, client)) = guard.as_ref() {
            if cached == proxy {
                return Ok(client.clone());
            }
        }
        let mut builder = reqwest::Client::builder().user_agent(USER_AGENT);
        if !proxy.is_empty() {
            let parsed = reqwest::Proxy::all(proxy)
                .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
            builder = builder.proxy(parsed);
        }
        let client = builder
            .build()
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        *guard = Some((proxy.to_string(), client.clone()));
        Ok(client)
    }

    fn apply_limit(&self, want: i64) {
        let want = want.max(1);
        let mut applied = self
            .applied
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *applied == want {
            return;
        }
        if want > *applied {
            self.permits.add_permits((want - *applied) as usize);
        } else {
            self.permits.forget_permits((*applied - want) as usize);
        }
        *applied = want;
    }

    async fn gate(&self, want: i64) -> Result<SemaphorePermit<'_>, BackendError> {
        self.apply_limit(want);
        self.permits
            .acquire()
            .await
            .map_err(|_| BackendError::new(Kind::ServiceDown, "出站并发闸已关闭"))
    }

    async fn get(
        &self,
        url: &str,
        session: Option<(Endpoint, &str)>,
    ) -> Result<Vec<u8>, BackendError> {
        let settings = self.settings.get();
        let client = self.http(&settings)?;
        let _permit = self.gate(settings.query_concurrency).await?;
        let timeout = Duration::from_secs(settings.save_timeout_seconds.max(1) as u64);
        let mut request = client.get(url).timeout(timeout);
        if let Some((endpoint, token)) = session {
            request = request
                .header("X-LC-Id", endpoint.lc_id)
                .header("X-LC-Key", endpoint.lc_key)
                .header("X-LC-Session", token);
        }
        let response = request
            .send()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        if body.len() > MAX_BODY {
            return Err(BackendError::new(Kind::ServiceDown, "云端响应过大"));
        }
        if !status.is_success() {
            let text = String::from_utf8_lossy(&body);
            return Err(BackendError::classify(status.as_u16(), &text));
        }
        Ok(body.to_vec())
    }
}

pub fn archive_error(error: SaveError) -> BackendError {
    if error == SaveError::Version {
        BackendError::new(Kind::SaveVersion, error.to_string())
    } else {
        BackendError::new(Kind::SaveTampered, format!("存档无法解析：{error}"))
    }
}

fn parse_player(payload: &[u8]) -> PlayerInfo {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(payload) else {
        return PlayerInfo::default();
    };
    PlayerInfo {
        object_id: text_field(&value, "objectId"),
        nickname: text_field(&value, "nickname"),
        avatar: text_field(&value, "avatar"),
        short_id: text_field(&value, "shortId"),
        joined_at: parse_time(&text_field(&value, "createdAt")),
    }
}

fn parse_saves(payload: &[u8]) -> Result<Vec<SaveRef>, BackendError> {
    let value: serde_json::Value = serde_json::from_slice(payload)
        .map_err(|_| BackendError::new(Kind::ServiceDown, "存档列表无法解析"))?;
    let results = value
        .get("results")
        .and_then(|item| item.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(results
        .into_iter()
        .map(|item| SaveRef {
            object_id: text_field(&item, "objectId"),
            modified_at: parse_time(
                item.get("modifiedAt")
                    .and_then(|inner| inner.get("iso"))
                    .and_then(|inner| inner.as_str())
                    .unwrap_or(""),
            ),
            summary: text_field(&item, "summary"),
            url: item
                .get("gameFile")
                .and_then(|inner| inner.get("url"))
                .and_then(|inner| inner.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect())
}

fn text_field(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .to_string()
}

pub fn parse_time(text: &str) -> i64 {
    if text.trim().is_empty() {
        return 0;
    }
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|value| value.timestamp())
        .unwrap_or(0)
}

fn urlencode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_match_go_constants() {
        let cn = endpoint_of(Server::Cn);
        assert_eq!(cn.lc_base, "https://rak3ffdi.cloud.tds1.tapapis.cn/1.1");
        assert_eq!(cn.lc_id, "rAK3FfdieFob2Nn8Am");
        let global = endpoint_of(Server::Global);
        assert_eq!(global.lc_base, "https://kviehlel.cloud.ap-sg.tapapis.com/1.1");
        assert_eq!(global.lc_id, "kviehleldgxsagpozb");
    }

    #[test]
    fn player_fields_come_from_json() {
        let payload = br#"{"objectId":"o1","nickname":"n","avatar":"a","shortId":"s","createdAt":"2021-03-04T05:06:07.123Z"}"#;
        let player = parse_player(payload);
        assert_eq!(player.object_id, "o1");
        assert_eq!(player.nickname, "n");
        assert_eq!(player.short_id, "s");
        assert!(player.joined_at > 0);
    }

    #[test]
    fn saves_take_newest_by_modified_at() {
        let payload = br#"{"results":[{"objectId":"a","modifiedAt":{"iso":"2023-01-01T00:00:00Z"},"gameFile":{"url":"u1"}},{"objectId":"b","modifiedAt":{"iso":"2024-01-01T00:00:00Z"},"gameFile":{"url":"u2"}}]}"#;
        let mut saves = parse_saves(payload).unwrap();
        assert_eq!(saves.len(), 2);
        saves.sort_by_key(|item| std::cmp::Reverse(item.modified_at));
        assert_eq!(saves[0].object_id, "b");
        assert_eq!(saves[0].url, "u2");
    }

    #[test]
    fn where_clause_is_percent_encoded() {
        let encoded = urlencode(r#"{"user":{"objectId":"x"}}"#);
        assert!(encoded.contains("%7B") || encoded.contains("%22"));
        assert!(!encoded.contains('"'));
    }
}
