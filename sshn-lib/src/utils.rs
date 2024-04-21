use base64::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    constants::{CLIENT_ID, LOCALE, REDIRECT_URI},
    error::Result,
};

fn generate_random_string(length: usize) -> String {
    let rng = rand::thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn compute_sha256_with_base64<I: AsRef<str>>(input: I) -> String {
    let mut hasher = Sha256::new();

    hasher.update(input.as_ref());

    let result = hasher.finalize();

    BASE64_URL_SAFE_NO_PAD.encode(result)
}

pub fn get_code_challenge() -> (String, String) {
    let verifier = generate_random_string(43);

    let challenge = compute_sha256_with_base64(&verifier);

    (challenge, verifier)
}

pub fn generate_auth_url<U: AsRef<str>, C: AsRef<str>>(
    base_url: U,
    code_challenge: C,
) -> Result<String> {
    let mut url = Url::parse(base_url.as_ref())?;

    let state = generate_random_string(32);
    let nonce = generate_random_string(32);

    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("state", &state)
        .append_pair("response_mode", "query")
        .append_pair("response_type", "code")
        .append_pair("scope", "openid")
        .append_pair("nonce", &nonce)
        .append_pair("ui_locales", LOCALE)
        .append_pair("code_challenge", code_challenge.as_ref())
        .append_pair("code_challenge_method", "S256");

    Ok(url.to_string())
}
