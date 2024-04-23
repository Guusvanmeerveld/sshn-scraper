use std::collections::HashMap;

use graphql_client::GraphQLQuery;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    constants::{CLIENT_ID, GRAPHQL_URL, LOCALE, REDIRECT_URI, TOKEN_URL},
    error::{Error, Result},
    queries::{
        get_identity_config, get_publications_list,
        post_application::{self, HousingApplyState},
        GetIdentityConfig, GetPublicationsList, GraphqlResponse, PostApplication,
    },
    tokens::{LoginResponse, Tokens},
};

pub struct Client {
    graphql_url: String,
    http_client: reqwest::Client,
}

pub enum LoginType {
    AuthCode { code: String, verifier: String },
    RefreshToken { token: String },
    Password { username: String, password: String },
}

impl Client {
    pub fn new(graphql_url: Option<String>) -> Self {
        Self {
            graphql_url: graphql_url.unwrap_or(GRAPHQL_URL.to_string()),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn auth(&self, login_type: LoginType) -> Result<Tokens> {
        let mut params = HashMap::new();

        params.insert("client_id", CLIENT_ID);

        match &login_type {
            LoginType::AuthCode { code, verifier } => {
                params.insert("grant_type", "authorization_code");
                params.insert("redirect_uri", REDIRECT_URI);

                params.insert("code_verifier", &verifier);
                params.insert("code", code);
            }
            LoginType::RefreshToken { token } => {
                params.insert("grant_type", "refresh_token");
                params.insert("refresh_token", token);
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

        if let Err(err) = response.error_for_status_ref() {
            log::debug!("{}", response.text().await?);

            return Err(Error::HttpRequest(err));
        };

        let response_data = response.json::<LoginResponse>().await?;

        Ok(response_data.into())
    }

    pub async fn login(self, login_type: LoginType) -> Result<AuthenticatedClient> {
        let tokens = self.auth(login_type).await?;

        let authenticated_client = AuthenticatedClient {
            graphql_url: self.graphql_url.clone(),
            http_client: reqwest::Client::new(),
            client: self,
            tokens,
        };

        Ok(authenticated_client)
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
    http_client: reqwest::Client,
    client: Client,
    tokens: Tokens,
}

impl Into<Tokens> for AuthenticatedClient {
    fn into(self) -> Tokens {
        self.tokens
    }
}

impl AuthenticatedClient {
    pub fn new(graphql_url: Option<String>, tokens: Tokens) -> Self {
        Self {
            graphql_url: graphql_url.clone().unwrap_or(GRAPHQL_URL.to_string()),
            http_client: reqwest::Client::new(),
            client: Client::new(graphql_url),
            tokens,
        }
    }

    async fn check_expiration(&mut self) -> Result<()> {
        if self.tokens.access_token().has_expired() {
            if !self.tokens.refresh_token().has_expired() {
                let token = self.tokens.refresh_token().content().to_string();

                self.tokens = self.client.auth(LoginType::RefreshToken { token }).await?;
            } else {
                return Err(Error::TokenExpired);
            }
        }

        Ok(())
    }

    async fn query<Q: Serialize, T: DeserializeOwned>(&mut self, query: &Q) -> Result<T> {
        self.check_expiration().await?;

        let response = self
            .http_client
            .post(&self.graphql_url)
            .bearer_auth(self.tokens.access_token().as_ref())
            .json(query)
            .send()
            .await?;

        let response = response.error_for_status()?;

        let response_body = response.json::<GraphqlResponse<T>>().await?;

        Ok(response_body.data)
    }

    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn client(&self) -> &Client {
        &self.client
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
