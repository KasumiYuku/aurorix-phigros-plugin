use std::time::Duration;

use base64::Engine;
use hmac::{Hmac, Mac};
use rand::RngCore;
use md5::{Digest, Md5};
use sha1::Sha1;

use crate::model::Server;

use super::error::{BackendError, Kind};

pub const TICKET_TTL: u64 = 300;
pub const POLL_EVERY: u64 = 3;
pub const POLL_MIN: u64 = 2;
pub const POLL_MAX: u64 = 30;

const CODE_PENDING: &str = "authorization_pending";
const CODE_WAITING: &str = "authorization_waiting";
const CODE_SLOW_DOWN: &str = "slow_down";
const CODE_EXPIRED: &str = "expired_token";
const CODE_DENIED: &str = "access_denied";

const SDK_VERSION: &str = "2.1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeviceState {
    #[default]
    Pending,
    Scanned,
    Ready,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceLogin {
    pub device_code: String,
    pub user_code: String,
    pub url: String,
    pub expires_in: u64,
    pub interval: u64,
    pub device_id: String,
    pub server: Server,
}

impl DeviceLogin {
    pub fn expired(&self) -> bool {
        self.device_code.is_empty() || self.url.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct DeviceFlow {
    http: reqwest::Client,
}

struct Endpoint {
    lc_base: &'static str,
    lc_id: &'static str,
    lc_key: &'static str,
    accounts_base: &'static str,
    open_base: &'static str,
    client_id: &'static str,
}

fn endpoint_of(server: Server) -> Endpoint {
    match server {
        Server::Global => Endpoint {
            lc_base: "https://kviehlel.cloud.ap-sg.tapapis.com/1.1",
            lc_id: "kviehleldgxsagpozb",
            lc_key: "tG9CTm0LDD736k9HMM9lBZrbeBGRmUkjSfNLDNib",
            accounts_base: "https://accounts.tapapis.com",
            open_base: "https://open.tapapis.com",
            client_id: "kviehleldgxsagpozb",
        },
        Server::Cn => Endpoint {
            lc_base: "https://rak3ffdi.cloud.tds1.tapapis.cn/1.1",
            lc_id: "rAK3FfdieFob2Nn8Am",
            lc_key: "Qr9AEqtuoSVS3zeD6iVbM4ZC0AtkJcQ89tywVyi0",
            accounts_base: "https://accounts.tapapis.cn",
            open_base: "https://open.tapapis.cn",
            client_id: "rAK3FfdieFob2Nn8Am",
        },
    }
}

impl Default for DeviceFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceFlow {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .unwrap_or_default();
        Self { http }
    }

    pub async fn start(&self, server: Server) -> Result<DeviceLogin, BackendError> {
        let endpoint = endpoint_of(server);
        let device_id = random_id();
        let info = format!("{{\"device_id\":\"{device_id}\"}}");
        let fields = [
            ("client_id", endpoint.client_id),
            ("response_type", "device_code"),
            ("scope", "public_profile"),
            ("version", SDK_VERSION),
            ("platform", "unity"),
            ("info", info.as_str()),
        ];
        let (body, content_type) = multipart(&fields);
        let url = format!("{}/oauth2/v1/device/code", endpoint.accounts_base);
        let payload = self
            .post(&url, body, &content_type, &[])
            .await?;
        let mut login = parse_device_code(&payload)?;
        login.device_id = device_id;
        login.server = server;
        Ok(login)
    }

    pub async fn poll(
        &self,
        login: &DeviceLogin,
    ) -> Result<(Option<String>, DeviceState), BackendError> {
        let endpoint = endpoint_of(login.server);
        let info = format!("{{\"device_id\":\"{}\"}}", login.device_id);
        let fields = [
            ("grant_type", "device_token"),
            ("client_id", endpoint.client_id),
            ("secret_type", "hmac-sha-1"),
            ("code", login.device_code.as_str()),
            ("version", "1.0"),
            ("platform", "unity"),
            ("info", info.as_str()),
        ];
        let (body, content_type) = multipart(&fields);
        let url = format!("{}/oauth2/v1/token", endpoint.accounts_base);
        let payload = match self.post(&url, body, &content_type, &[]).await {
            Ok(payload) => payload,
            Err(error) => {
                let body = error.message.clone();
                return match poll_outcome(&body) {
                    PollOutcome::Waiting(state) => Ok((None, state)),
                    PollOutcome::Expired => Err(BackendError::new(
                        Kind::TokenInvalid,
                        String::from("授权已过期，请重新发送 /p login"),
                    )),
                    PollOutcome::Denied => Err(BackendError::new(
                        Kind::TokenInvalid,
                        String::from("你取消了这次授权"),
                    )),
                };
            }
        };
        let token = nested_data(&payload);
        let profile = self.profile(&endpoint, &token).await?;
        let session = self.exchange(&endpoint, &profile, &token).await?;
        if session.is_empty() {
            return Err(BackendError::new(Kind::ServiceDown, "云端未返回登录凭据"));
        }
        Ok((Some(session), DeviceState::Ready))
    }

    async fn profile(
        &self,
        endpoint: &Endpoint,
        token: &serde_json::Value,
    ) -> Result<serde_json::Value, BackendError> {
        let kid = token.get("kid").and_then(|item| item.as_str()).unwrap_or("");
        let mac_key = token
            .get("mac_key")
            .and_then(|item| item.as_str())
            .unwrap_or("");
        if kid.is_empty() || mac_key.is_empty() {
            return Err(BackendError::new(Kind::ServiceDown, "授权响应缺少签名材料"));
        }
        let target = format!(
            "{}/account/profile/v1?client_id={}",
            endpoint.open_base,
            urlencode(endpoint.client_id)
        );
        let now = now_seconds();
        let nonce = random_token();
        let authorization = mac_header(&target, "GET", kid, mac_key, now, &nonce);
        let response = self
            .http
            .get(&target)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        if !status.is_success() {
            return Err(BackendError::classify(
                status.as_u16(),
                &String::from_utf8_lossy(&body),
            ));
        }
        Ok(nested_data(&body))
    }

    async fn exchange(
        &self,
        endpoint: &Endpoint,
        profile: &serde_json::Value,
        token: &serde_json::Value,
    ) -> Result<String, BackendError> {
        let mut auth = serde_json::Map::new();
        for layer in [profile, token] {
            if let Some(map) = layer.as_object() {
                for (key, value) in map {
                    if !value.is_null() {
                        auth.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        let body = serde_json::json!({ "authData": { "taptap": auth } });
        let payload = body.to_string();
        let headers = [
            ("X-LC-Id", endpoint.lc_id.to_string()),
            ("X-LC-Sign", lc_sign(endpoint.lc_key, now_seconds())),
        ];
        let url = format!("{}/users", endpoint.lc_base);
        let payload = self
            .post(&url, payload.into_bytes(), "application/json", &headers)
            .await?;
        let value: serde_json::Value = serde_json::from_slice(&payload).unwrap_or_default();
        Ok(value
            .get("sessionToken")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string())
    }

    async fn post(
        &self,
        url: &str,
        body: Vec<u8>,
        content_type: &str,
        headers: &[(&str, String)],
    ) -> Result<Vec<u8>, BackendError> {
        let mut request = self
            .http
            .post(url)
            .header("Content-Type", content_type)
            .body(body);
        for (name, value) in headers {
            request = request.header(*name, value);
        }
        let response = request
            .send()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| BackendError::new(Kind::Network, error.to_string()))?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if !status.is_success() {
            let mut error = BackendError::classify(status.as_u16(), &text);
            error.message = text;
            return Err(error);
        }
        Ok(bytes.to_vec())
    }
}

pub enum PollOutcome {
    Waiting(DeviceState),
    Expired,
    Denied,
}

pub fn poll_outcome(body: &str) -> PollOutcome {
    for code in [CODE_PENDING, CODE_SLOW_DOWN, CODE_WAITING, CODE_EXPIRED, CODE_DENIED] {
        if !body.contains(code) {
            continue;
        }
        return match code {
            CODE_WAITING => PollOutcome::Waiting(DeviceState::Scanned),
            CODE_EXPIRED => PollOutcome::Expired,
            CODE_DENIED => PollOutcome::Denied,
            _ => PollOutcome::Waiting(DeviceState::Pending),
        };
    }
    PollOutcome::Waiting(DeviceState::Pending)
}

pub fn parse_device_code(payload: &[u8]) -> Result<DeviceLogin, BackendError> {
    let value: serde_json::Value = serde_json::from_slice(payload)
        .map_err(|_| BackendError::new(Kind::ServiceDown, "设备码无法解析"))?;
    let value = value
        .get("data")
        .filter(|inner| inner.is_object())
        .cloned()
        .unwrap_or(value);
    let device_code = value
        .get("device_code")
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .to_string();
    let url = value
        .get("qrcode_url")
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .to_string();
    if device_code.is_empty() || url.is_empty() {
        return Err(BackendError::new(Kind::ServiceDown, "设备码响应缺少必要字段"));
    }
    Ok(DeviceLogin {
        device_code,
        user_code: value
            .get("user_code")
            .and_then(|item| item.as_str())
            .unwrap_or("")
            .to_string(),
        url,
        expires_in: number_field(&value, "expires_in", TICKET_TTL),
        interval: clamp_interval(number_field(&value, "interval", POLL_EVERY)),
        device_id: String::new(),
        server: Server::Cn,
    })
}

pub fn mac_header(
    target: &str,
    method: &str,
    kid: &str,
    mac_key: &str,
    timestamp: i64,
    nonce: &str,
) -> String {
    let (scheme, rest) = target.split_once("://").unwrap_or(("https", target));
    let (authority, path) = match rest.find('/') {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|ch| ch.is_ascii_digit()) => (host, port.to_string()),
        _ => (
            authority,
            if scheme == "https" { "443" } else { "80" }.to_string(),
        ),
    };
    let base = format!(
        "{timestamp}\n{nonce}\n{method}\n{path}\n{host}\n{port}\n\n"
    );
    let mut mac = Hmac::<Sha1>::new_from_slice(mac_key.as_bytes()).expect("hmac key");
    mac.update(base.as_bytes());
    let digest = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    format!("MAC id=\"{kid}\", ts=\"{timestamp}\", nonce=\"{nonce}\", mac=\"{digest}\"")
}

pub fn lc_sign(key: &str, timestamp: i64) -> String {
    let mut hasher = Md5::new();
    hasher.update(format!("{timestamp}{key}").as_bytes());
    let digest = hasher.finalize();
    format!("{},{}", hex::encode(digest), timestamp)
}

pub fn clamp_interval(value: u64) -> u64 {
    value.clamp(POLL_MIN, POLL_MAX)
}

fn number_field(value: &serde_json::Value, key: &str, fallback: u64) -> u64 {
    match value.get(key) {
        Some(serde_json::Value::Number(number)) => number.as_u64().unwrap_or(fallback),
        Some(serde_json::Value::String(text)) => text.parse::<u64>().unwrap_or(fallback),
        _ => fallback,
    }
}

fn nested_data(payload: &[u8]) -> serde_json::Value {
    let value: serde_json::Value = serde_json::from_slice(payload).unwrap_or_default();
    match value.get("data") {
        Some(data) if data.is_object() => data.clone(),
        _ => value,
    }
}

fn multipart(fields: &[(&str, &str)]) -> (Vec<u8>, String) {
    let boundary = format!("----Aurorix{}", random_id());
    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (body, format!("multipart/form-data; boundary={boundary}"))
}

fn random_id() -> String {
    let mut raw = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut raw);
    hex::encode(raw)
}

fn random_token() -> String {
    let mut raw = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut raw);
    base64::engine::general_purpose::STANDARD.encode(raw)
}

fn now_seconds() -> i64 {
    chrono::Utc::now().timestamp()
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
    fn mac_header_matches_go_layout() {
        let header = mac_header(
            "https://open.tapapis.cn/account/profile/v1?client_id=abc",
            "GET",
            "kid-1",
            "secret",
            1_700_000_000,
            "nonce-1",
        );
        assert!(header.starts_with("MAC id=\"kid-1\""));
        assert!(header.contains("ts=\"1700000000\""));
        assert!(header.contains("nonce=\"nonce-1\""));
        let base = "1700000000\nnonce-1\nGET\n/account/profile/v1?client_id=abc\nopen.tapapis.cn\n443\n\n";
        let mut mac = Hmac::<Sha1>::new_from_slice(b"secret").unwrap();
        mac.update(base.as_bytes());
        let expect = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        assert!(header.contains(&expect));
    }

    #[test]
    fn mac_header_uses_default_ports() {
        let header = mac_header("http://example.com/x", "GET", "k", "s", 1, "n");
        assert!(header.contains("MAC id=\"k\""));
        let base = "1\nn\nGET\n/x\nexample.com\n80\n\n";
        let mut mac = Hmac::<Sha1>::new_from_slice(b"s").unwrap();
        mac.update(base.as_bytes());
        let expect = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
        assert!(header.contains(&expect));
    }

    #[test]
    fn device_code_unwraps_data_envelope() {
        let payload = br#"{"data":{"device_code":"dc","user_code":"uc","qrcode_url":"https://x","expires_in":300,"interval":3},"success":true}"#;
        let login = parse_device_code(payload).unwrap();
        assert_eq!(login.device_code, "dc");
        assert_eq!(login.user_code, "uc");
        assert_eq!(login.url, "https://x");
        assert_eq!(login.expires_in, 300);
        assert_eq!(login.interval, 3);
        assert!(parse_device_code(br#"{"data":{"device_code":""}}"#).is_err());
    }

    #[test]
    fn interval_is_clamped_like_go() {
        let payload = br#"{"device_code":"d","qrcode_url":"u","interval":0}"#;
        assert_eq!(parse_device_code(payload).unwrap().interval, POLL_MIN);
        let payload = br#"{"device_code":"d","qrcode_url":"u","interval":99}"#;
        assert_eq!(parse_device_code(payload).unwrap().interval, POLL_MAX);
        assert_eq!(clamp_interval(5), 5);
        let payload = br#"{"device_code":"d","qrcode_url":"u"}"#;
        assert_eq!(parse_device_code(payload).unwrap().expires_in, TICKET_TTL);
    }

    #[test]
    fn poll_outcome_maps_upstream_codes() {
        assert!(matches!(
            poll_outcome("{\"error\":\"authorization_pending\"}"),
            PollOutcome::Waiting(DeviceState::Pending)
        ));
        assert!(matches!(
            poll_outcome("{\"error\":\"slow_down\"}"),
            PollOutcome::Waiting(DeviceState::Pending)
        ));
        assert!(matches!(
            poll_outcome("{\"error\":\"authorization_waiting\"}"),
            PollOutcome::Waiting(DeviceState::Scanned)
        ));
        assert!(matches!(poll_outcome("expired_token"), PollOutcome::Expired));
        assert!(matches!(poll_outcome("access_denied"), PollOutcome::Denied));
    }

    #[test]
    fn leancloud_sign_is_md5_of_timestamp_and_key() {
        let sign = lc_sign("key", 1_700_000_000);
        let mut hasher = Md5::new();
        hasher.update(b"1700000000key");
        assert_eq!(
            sign,
            format!("{},1700000000", hex::encode(hasher.finalize()))
        );
    }

    #[test]
    fn multipart_body_is_well_formed() {
        let (body, content_type) = multipart(&[("a", "1"), ("b", "2")]);
        let text = String::from_utf8(body).unwrap();
        assert!(content_type.starts_with("multipart/form-data; boundary="));
        assert!(text.contains("name=\"a\""));
        assert!(text.ends_with("--\r\n"));
    }
}
