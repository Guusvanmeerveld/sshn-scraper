use crate::{error::Result, Token};

pub struct Client {
    base_url: String,
    access_token: Token,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new<U: Into<String>>(base_url: U, access_token: Token) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            base_url: base_url.into(),
            access_token,
        }
    }

    pub async fn get_publications(&self) -> Result<()> {
        Ok(())
    }
}
