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
    r#type: TokenType,
    content: String,
    expires: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub enum TokenType {
    #[default]
    Access,
    Refresh,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            content: String::new(),
            expires: Utc::now(),
            ..Default::default()
        }
    }
}

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl Token {
    pub fn new<C: Into<String>>(content: C, expires: DateTime<Utc>, token_type: TokenType) -> Self {
        Self {
            content: content.into(),
            expires,
            r#type: token_type,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}
