use std::result;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error from SSHN API: {0}")]
    Api(String),
    #[error("Error encoding form data: {0}")]
    EncodeFormData(#[from] serde_urlencoded::ser::Error),
    #[error("Error sending HTTP request: {0}")]
    HttpRequest(#[from] reqwest::Error),
    #[error("The refresh token expired")]
    TokenExpired,
    #[error("The authentication endpoint is missing")]
    NoAuthUrl,
    #[error("Failed to parse url: {0}")]
    ParseUrl(#[from] url::ParseError),
}

pub type Result<T> = result::Result<T, Error>;
