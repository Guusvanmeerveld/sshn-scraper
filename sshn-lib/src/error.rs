use std::{fmt::Display, result};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    EncodeFormData(#[from] serde_urlencoded::ser::Error),
    HttpRequest(#[from] reqwest::Error),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

pub type Result<T> = result::Result<T, ClientError>;
