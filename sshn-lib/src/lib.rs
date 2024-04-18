use std::{collections::HashMap, fmt::Debug};

mod client;
mod constants;
pub mod error;

use chrono::{DateTime, Duration, Utc};
use client::Client;
use constants::{AUTH_URL, GRAPHQL_URL};
use error::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct RefreshTokenResponse {
    access_token: String,
    expires_in: i64,
    refresh_expires_in: i64,
    refresh_token: String,
    // token_type: String,
    // id_token: String,
    // session_state: String,
}

#[derive(Debug)]
pub struct Token {
    token: String,
    expires: DateTime<Utc>,
}

impl Token {
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn expires(&self) -> DateTime<Utc> {
        self.expires
    }
}

pub async fn login_with_refresh_token<R: AsRef<str>>(refresh_token: R) -> Result<(Client, Token)> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();

    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token.as_ref());
    params.insert("client_id", "portal-legacy");

    let body = serde_urlencoded::to_string(&params)?;

    let response = client
        .post(AUTH_URL)
        .body(body)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .send()
        .await?;

    let tokens = response.json::<RefreshTokenResponse>().await?;

    let access_token = Token {
        token: tokens.access_token,
        expires: Utc::now() + Duration::seconds(tokens.expires_in),
    };

    let client = Client::new(GRAPHQL_URL, access_token);

    let new_refresh_token = Token {
        token: tokens.refresh_token,
        expires: Utc::now() + Duration::seconds(tokens.refresh_expires_in),
    };

    Ok((client, new_refresh_token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh_token_login() {
        let refresh_token = "";

        let (client, new_refresh_token) = login_with_refresh_token(refresh_token).await.unwrap();

        println!("{:?}", new_refresh_token);
    }
}
