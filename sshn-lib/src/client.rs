use std::collections::HashMap;

use chrono::{Duration, Utc};
use graphql_client::GraphQLQuery;

use crate::{
    constants::{AUTH_URL, GRAPHQL_URL, LOCALE},
    error::{Error, Result},
    queries::{
        get_publications_list,
        post_application::{self, HousingApplyState},
        GetPublicationsList, GraphqlResponse, PostApplication,
    },
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

    /// Reply to a publication, given that publications id.
    pub async fn reply_to_publication<I: Into<String>>(&self, publication_id: I) -> Result<()> {
        let variables = post_application::Variables {
            publication_id: publication_id.into(),
            locale: Some(String::from(LOCALE)),
        };

        let request_body = PostApplication::build_query(variables);

        let response = self
            .http_client
            .post(&self.base_url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.access_token.as_ref()),
            )
            .json(&request_body)
            .send()
            .await?;

        match response.error_for_status() {
            Ok(response) => {
                let body = response
                    .json::<GraphqlResponse<post_application::ResponseData>>()
                    .await?;

                if let Some(unit) = body.data.housing_apply_to_unit {
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
            Err(err) => Err(Error::HttpRequest(err)),
        }
    }
}
