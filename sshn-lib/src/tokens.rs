use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub refresh_token: String,
    // token_type: String,
    // id_token: String,
    // session_state: String,
}

impl Into<Tokens> for LoginResponse {
    fn into(self) -> Tokens {
        Tokens::new(
            Token::new(
                self.refresh_token,
                Utc::now() + Duration::seconds(self.refresh_expires_in),
                TokenType::Refresh,
            ),
            Token::new(
                self.access_token,
                Utc::now() + Duration::seconds(self.expires_in),
                TokenType::Access,
            ),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    r#type: TokenType,
    content: String,
    expires: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
            r#type: TokenType::Access,
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

    pub fn has_expired(&self) -> bool {
        self.expires <= Utc::now()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Tokens {
    refresh_token: Token,
    access_token: Token,
}

impl Tokens {
    pub fn new(refresh_token: Token, access_token: Token) -> Self {
        Self {
            refresh_token,
            access_token,
        }
    }

    pub fn refresh_token(&self) -> &Token {
        &self.refresh_token
    }

    pub fn access_token(&self) -> &Token {
        &self.access_token
    }
}
