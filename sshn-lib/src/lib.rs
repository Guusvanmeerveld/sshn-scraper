mod client;
mod constants;
pub mod error;
mod queries;
mod tokens;

pub use crate::client::Client;

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::{constants::GRAPHQL_URL, tokens::Token};

    use super::*;

    #[tokio::test]
    async fn test_refresh_token_login() {
        let refresh_token = "";

        let (client, _new_refresh_token) = Client::login_with_refresh_token(refresh_token)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_publications() {
        let client = Client::new(GRAPHQL_URL, Token::default());

        let data = client.get_publications_list(30).await.unwrap();

        println!("{:?}", data);
    }

    #[tokio::test]
    async fn test_post_application() {
        let client = Client::new(GRAPHQL_URL, Token::new("", Utc::now()));

        client.reply_to_publication("").await.unwrap();
    }
}
