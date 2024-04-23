use keyring::Entry;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sshn_lib::{AuthenticatedClient, Tokens};

pub use crate::error::Result;
use crate::{auth, error::Error};

const SERVICE_NAME: &str = "SSHN-cli";

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    pub fn new<U: Into<String>, P: Into<String>>(username: U, password: P) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

pub fn set<I: AsRef<str>, T: Serialize>(identifier: I, data: &T) -> Result<()> {
    let user = whoami::username();

    let entry_name = format!("{}-{}", identifier.as_ref(), SERVICE_NAME);

    let entry = Entry::new(&entry_name, &user)?;

    let data = serde_json::to_string(data)?;

    entry.set_password(&data)?;

    Ok(())
}

pub fn get<I: AsRef<str>, T: DeserializeOwned>(identifier: I) -> Result<T> {
    let user = whoami::username();

    let entry_name = format!("{}-{}", identifier.as_ref(), SERVICE_NAME);

    let entry = Entry::new(&entry_name, &user)?;

    let data = entry.get_password()?;

    let data = serde_json::from_str(&data)?;

    Ok(data)
}

pub async fn get_client() -> Result<AuthenticatedClient> {
    let client = sshn_lib::Client::new(None);

    if let Ok(tokens) = get::<_, Tokens>("tokens") {
        if !tokens.access_token().has_expired() {
            return Ok(AuthenticatedClient::new(None, tokens));
        } else {
            if !tokens.refresh_token().has_expired() {
                return Ok(client
                    .login(sshn_lib::LoginType::RefreshToken {
                        token: tokens.refresh_token().content().to_string(),
                    })
                    .await?);
            }
        }
    }

    log::info!("Tokens expired, logging in using credentials");

    if let Ok(credentials) = get::<_, Credentials>("credentials") {
        return auth::password_login(
            credentials.username,
            credentials.password,
            Default::default(),
        )
        .await;
    }

    Err(Error::MissingCredentials)
}
