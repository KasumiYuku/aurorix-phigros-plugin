#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    TokenInvalid,
    RateLimited,
    ServiceDown,
    SaveEmpty,
    SaveVersion,
    SaveTampered,
    Network,
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("{kind:?}: {message}")]
pub struct BackendError {
    pub kind: Kind,
    pub message: String,
    pub status: Option<u16>,
}

impl BackendError {
    pub fn new(kind: Kind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status: None,
        }
    }

    pub fn with_status(kind: Kind, message: impl Into<String>, status: u16) -> Self {
        Self {
            kind,
            message: message.into(),
            status: Some(status),
        }
    }

    pub fn classify(status: u16, body: &str) -> Self {
        let code = leancloud_code(body);
        let kind = match status {
            401 => Kind::TokenInvalid,
            429 => Kind::RateLimited,
            500..=599 => Kind::ServiceDown,
            _ if code == Some(211) => Kind::TokenInvalid,
            _ => Kind::ServiceDown,
        };
        let message = match kind {
            Kind::TokenInvalid => String::from(
                "登录凭据已失效，请重新发送 /p bind 获取授权链接，或换一个有效的 sessionToken",
            ),
            Kind::RateLimited => String::from("云端限流了，稍后再试"),
            _ => format!("云端返回 {status}"),
        };
        Self {
            kind,
            message,
            status: Some(status),
        }
    }

    pub fn token_invalid(&self) -> bool {
        self.kind == Kind::TokenInvalid
    }
}

fn leancloud_code(body: &str) -> Option<i64> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("code")?.as_i64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_statuses_like_go() {
        assert_eq!(BackendError::classify(401, "").kind, Kind::TokenInvalid);
        assert_eq!(BackendError::classify(429, "").kind, Kind::RateLimited);
        assert_eq!(BackendError::classify(503, "").kind, Kind::ServiceDown);
        assert_eq!(
            BackendError::classify(400, r#"{"code":211}"#).kind,
            Kind::TokenInvalid
        );
    }

    #[test]
    fn token_message_matches_go_copy() {
        let error = BackendError::classify(401, "");
        assert!(error.message.contains("/p bind"));
        assert!(error.token_invalid());
    }
}
