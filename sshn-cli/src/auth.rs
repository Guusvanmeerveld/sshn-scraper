use std::time::Duration;

use fantoccini::{ClientBuilder, Locator};
use sshn_lib::{generate_auth_url, get_code_challenge, AuthenticatedClient, LoginType};

use crate::{
    error::{Error, Result},
    secrets::{self, Credentials},
    WebDriver,
};

const LOGIN_FORM_ID: &str = "kc-form-login";

#[derive(Debug)]
pub struct AuthOptions {
    webdriver: WebDriver,
    webdriver_port: u16,

    login_base_url: Option<String>,
}

impl AuthOptions {
    pub fn webdriver(self, webdriver: WebDriver) -> Self {
        Self { webdriver, ..self }
    }

    pub fn webdriver_port(self, webdriver_port: u16) -> Self {
        Self {
            webdriver_port,
            ..self
        }
    }

    pub fn login_base_url<L: Into<String>>(self, login_base_url: L) -> Self {
        Self {
            login_base_url: Some(login_base_url.into()),
            ..self
        }
    }
}

impl Default for AuthOptions {
    fn default() -> Self {
        Self {
            login_base_url: None,
            webdriver: WebDriver::Chromium,
            webdriver_port: 4444,
        }
    }
}

/// Starts the given webdriver on the given port, then waits until said driver has started up.
async fn start_web_driver(webdriver: WebDriver, port: u16) -> Result<tokio::process::Child> {
    let process = match webdriver {
        WebDriver::Chromium => tokio::process::Command::new("chromedriver")
            .arg(format!("--port={}", port))
            .arg("--headless")
            .spawn()?,
        WebDriver::Gecko => tokio::process::Command::new("geckodriver")
            .arg("--port")
            .arg(port.to_string())
            .spawn()?,
    };

    let mut attempt = 0;
    let max_attempts = 10;

    while attempt < max_attempts {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            break;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
            attempt += 1;

            log::warn!(
                "Could not connect to web driver, retrying... (Attempt {})",
                attempt
            );
        }
    }

    if attempt >= max_attempts {
        return Err(Error::WebDriverStart);
    }

    Ok(process)
}

pub async fn headless_login<U: AsRef<str>, P: AsRef<str>>(
    username: U,
    password: P,
    options: AuthOptions,
) -> Result<AuthenticatedClient> {
    let client = sshn_lib::UnAuthenticatedClient::new(None);

    let (code_challenge, code_verifier) = get_code_challenge();

    let login_base_url: String = match options.login_base_url.as_ref() {
        Some(url) => url.into(),
        None => {
            let endpoints = client.get_endpoints().await?;

            endpoints
                .identity_config
                .ok_or(Error::MissingLoginUrl)?
                .authorization_endpoint
                .ok_or(Error::MissingLoginUrl)?
        }
    };

    let login_url = generate_auth_url(login_base_url, code_challenge)?;

    let mut driver = start_web_driver(options.webdriver, options.webdriver_port).await?;

    let browser = ClientBuilder::native()
        .connect(&format!("http://localhost:{}", options.webdriver_port))
        .await?;

    log::info!("Logging into SSHN at {}", login_url);

    browser.goto(&login_url).await?;

    let login_form = browser.form(Locator::Id(LOGIN_FORM_ID)).await?;

    login_form
        .set_by_name("username", username.as_ref())
        .await?
        .set_by_name("password", password.as_ref())
        .await?
        .submit_direct()
        .await?;

    let callback_url = browser.current_url().await?;

    driver.kill().await?;

    let authorization_code = callback_url
        .query_pairs()
        .filter(|(key, _value)| key == "code")
        .map(|(_key, value)| value.to_string())
        .next()
        .ok_or(Error::MissingAuthCode)?;

    let auth_client = client
        .login(LoginType::AuthCode {
            code: authorization_code,
            verifier: code_verifier,
        })
        .await?;

    let credentials = Credentials::new(username.as_ref(), password.as_ref());

    secrets::set("credentials", &credentials)?;
    secrets::set("tokens", auth_client.tokens())?;

    Ok(auth_client)
}
