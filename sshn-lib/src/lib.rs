mod api;
mod client;
mod constants;
pub mod error;
mod queries;
mod tokens;
mod utils;

pub use api::*;

pub use {
    client::{AuthenticatedClient, Client, LoginType, UnAuthenticatedClient},
    tokens::{Token, TokenType, Tokens},
    utils::{generate_auth_url, get_code_challenge},
};

#[cfg(test)]
mod tests {

    use crate::client::LoginType;

    use super::*;

    #[tokio::test]
    async fn test_get_publications() {
        let client = UnAuthenticatedClient::new(None);

        let data = client.get_endpoints().await.unwrap();

        println!("{:?}", data);
    }

    #[tokio::test]
    async fn test_login() {
        dotenv::dotenv().ok();

        let username = std::env::var("SSHN_USERNAME").unwrap();
        let password = std::env::var("SSHN_PASSWORD").unwrap();

        let client = UnAuthenticatedClient::new(None);

        let _auth_client = client
            .login(LoginType::Password { username, password })
            .await
            .unwrap();
    }

    // #[tokio::test]
    // async fn test_post_application() {
    //     let client = Client::new(None);

    //     client.reply_to_publication("").await.unwrap();
    // }
}
