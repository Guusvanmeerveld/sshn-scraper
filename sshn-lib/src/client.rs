use std::collections::HashMap;

use chrono::{Duration, Utc};
use graphql_client::GraphQLQuery;

use crate::{
    constants::{AUTH_URL, GRAPHQL_URL, LOCALE},
    error::Result,
    queries::{get_publications_list, GetPublicationsList, GraphqlResponse},
    tokens::{RefreshTokenResponse, Token},
};

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

    pub async fn login_with_refresh_token<R: AsRef<str>>(
        refresh_token: R,
    ) -> Result<(Client, Token)> {
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

        let access_token = Token::new(
            tokens.access_token,
            Utc::now() + Duration::seconds(tokens.expires_in),
        );

        let client = Client::new(GRAPHQL_URL, access_token);

        let new_refresh_token = Token::new(
            tokens.refresh_token,
            Utc::now() + Duration::seconds(tokens.refresh_expires_in),
        );

        Ok((client, new_refresh_token))
    }

    pub async fn get_publications_list(
        &self,
        max: i64,
    ) -> Result<get_publications_list::ResponseData> {
        let variables = get_publications_list::Variables {
            order_by: Some(get_publications_list::HousingPublicationsOrder::STARTDATE_ASC),
            first: Some(max),
            locale: Some(String::from(LOCALE)),
            after: None,
            where_: None,
        };

        let request_body = GetPublicationsList::build_query(variables);

        let response = self
            .http_client
            .post(&self.base_url)
            .json(&request_body)
            .send()
            .await?;

        let body = response
            .json::<GraphqlResponse<get_publications_list::ResponseData>>()
            .await?;

        Ok(body.data)
    }

    pub async fn reply_to_publication<I: AsRef<str>>(&self, publication_id: I) -> Result<()> {
        Ok(())
    }
}
