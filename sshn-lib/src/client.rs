use std::collections::HashMap;

use chrono::{Duration, Utc};
use graphql_client::GraphQLQuery;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    constants::{CLIENT_ID, GRAPHQL_URL, LOCALE, TOKEN_URL},
    error::{Error, Result},
    queries::{
        get_identity_config, get_publications_list,
        post_application::{self, HousingApplyState},
        GetIdentityConfig, GetPublicationsList, GraphqlResponse, PostApplication,
    },
    tokens::{RefreshTokenResponse, Token},
};

pub struct Client {
    graphql_url: String,
    http_client: reqwest::Client,
}

pub enum LoginType {
    AuthCode(String),
    Password { username: String, password: String },
}

impl Client {
    pub fn new(graphql_url: Option<String>) -> Self {
        Self {
            graphql_url: graphql_url.unwrap_or(GRAPHQL_URL.to_string()),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn login(self, login_type: LoginType) -> Result<AuthenticatedClient> {
        let mut params = HashMap::new();

        params.insert("client_id", CLIENT_ID);

        match &login_type {
            LoginType::AuthCode(code) => {
                params.insert("grant_type", "authorization_code");
                params.insert("authorization_code", code);
            }
            LoginType::Password { username, password } => {
                params.insert("grant_type", "password");

                params.insert("username", &username);
                params.insert("password", &password);
            }
        };

        let body = serde_urlencoded::to_string(&params)?;

        let response = self
            .http_client
            .post(TOKEN_URL)
            .body(body)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let tokens = response.json::<RefreshTokenResponse>().await?;

                let access_token = Token::new(
                    tokens.access_token,
                    Utc::now() + Duration::seconds(tokens.expires_in),
                );

                let refresh_token = Token::new(
                    tokens.refresh_token,
                    Utc::now() + Duration::seconds(tokens.refresh_expires_in),
                );

                let authenticated_client = AuthenticatedClient {
                    graphql_url: self.graphql_url,
                    http_client: self.http_client,
                    token_url: TOKEN_URL.to_string(),
                    access_token,
                    refresh_token,
                };

                Ok(authenticated_client)
            }
            Err(err) => Err(Error::HttpRequest(err)),
        }
    }

    pub async fn get_endpoints(&self) -> Result<get_identity_config::ResponseData> {
        let variables = get_identity_config::Variables {
            realm: String::from("sshn"),
        };

        let request_body = GetIdentityConfig::build_query(variables);

        let response = self
            .http_client
            .post(&self.graphql_url)
            .json(&request_body)
            .send()
            .await?;

        let body = response
            .json::<GraphqlResponse<get_identity_config::ResponseData>>()
            .await?;

        Ok(body.data)
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
            .post(&self.graphql_url)
            .json(&request_body)
            .send()
            .await?;

        let body = response
            .json::<GraphqlResponse<get_publications_list::ResponseData>>()
            .await?;

        Ok(body.data)
    }
}

pub struct AuthenticatedClient {
    graphql_url: String,
    token_url: String,
    http_client: reqwest::Client,
    access_token: Token,
    refresh_token: Token,
}

impl AuthenticatedClient {
    async fn refresh_tokens(&mut self) -> Result<()> {
        if self.refresh_token.expires() < Utc::now() {
            return Err(Error::TokenExpired);
        }

        let mut params = HashMap::new();

        params.insert("client_id", CLIENT_ID);
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", self.refresh_token.as_ref());

        let body = serde_urlencoded::to_string(&params)?;

        let response = self
            .http_client
            .post(&self.token_url)
            .body(body)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .send()
            .await?;

        let tokens = response.json::<RefreshTokenResponse>().await?;

        self.access_token = Token::new(
            tokens.access_token,
            Utc::now() + Duration::seconds(tokens.expires_in),
        );

        self.refresh_token = Token::new(
            tokens.refresh_token,
            Utc::now() + Duration::seconds(tokens.refresh_expires_in),
        );

        Ok(())
    }

    async fn check_expiration(&mut self) -> Result<()> {
        if self.access_token.expires() < Utc::now() {
            self.refresh_tokens().await?;
        }

        Ok(())
    }

    async fn query<Q: Serialize, T: DeserializeOwned>(&mut self, query: &Q) -> Result<T> {
        self.check_expiration().await?;

        let response = self
            .http_client
            .post(&self.graphql_url)
            .bearer_auth(self.access_token.as_ref())
            .json(query)
            .send()
            .await?;

        let response = response.error_for_status()?;

        let response_body = response.json::<GraphqlResponse<T>>().await?;

        Ok(response_body.data)
    }

    /// Reply to a publication, given that publications id.
    pub async fn reply_to_publication<I: Into<String>>(&mut self, publication_id: I) -> Result<()> {
        let variables = post_application::Variables {
            publication_id: publication_id.into(),
            locale: Some(String::from(LOCALE)),
        };

        let request_body = PostApplication::build_query(variables);

        let data: post_application::ResponseData = self.query(&request_body).await?;

        if let Some(unit) = data.housing_apply_to_unit {
            match unit.state {
                HousingApplyState::OK => {}
                _ => {
                    let error = Error::Api(unit.description.unwrap_or(String::new()));

                    return Err(error);
                }
            };
        };

        Ok(())
    }
}
