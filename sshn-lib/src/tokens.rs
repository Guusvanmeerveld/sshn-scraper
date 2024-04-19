use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub refresh_token: String,
    // token_type: String,
    // id_token: String,
    // session_state: String,
}

#[derive(Debug)]
pub struct Token {
    content: String,
    expires: DateTime<Utc>,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            content: String::new(),
            expires: Utc::now(),
        }
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl Token {
    pub fn new(content: String, expires: DateTime<Utc>) -> Self {
        Self { content, expires }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}
