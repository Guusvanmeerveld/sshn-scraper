use std::result;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SSHN Api returned an error: {0}")]
    SshnLib(#[from] sshn_lib::error::Error),

    #[error("SSHN Api did not return a valid login url")]
    MissingLoginUrl,

    #[error("SSHN Api did not return valid authorization code")]
    MissingAuthCode,

    #[error("Failed to start web driver")]
    WebDriverStart,

    #[error("Failed to create new web driver session: {0}")]
    NewWebDriverSession(#[from] fantoccini::error::NewSessionError),

    #[error("Failed communicating with browser: {0}")]
    HeadlessBrowser(#[from] fantoccini::error::CmdError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = result::Result<T, Error>;
