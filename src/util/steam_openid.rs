use std::{error::Error, fmt};

use lazy_static::lazy_static;
//taken from https://crates.io/crates/steam-openid
use regex::Regex;
use url::Url;

pub fn get_redirect_url(realm: &str, path: &str) -> std::result::Result<String, NewError> {
    let return_to = Url::parse(realm).map_err(|_| NewError::BadUrl)?;
    let return_to = return_to
        .join(path)
        .map_err(|_| NewError::BadUrl)?
        .to_string();

    let form = RedirectForm {
        ns: "http://specs.openid.net/auth/2.0",
        identity: "http://specs.openid.net/auth/2.0/identifier_select",
        claimed_id: "http://specs.openid.net/auth/2.0/identifier_select",
        mode: "checkid_setup",
        realm,
        return_to: &return_to,
    };
    let form_str = serde_urlencoded::to_string(&form).map_err(|_| NewError::BadUrl)?;

    let mut redirect_url = Url::parse("https://steamcommunity.com/openid/login").unwrap();
    redirect_url.set_query(Some(&form_str));

    Ok(redirect_url.to_string())
}

pub async fn verify_return(
    return_to: &str,
    form: &mut VerifyForm,
) -> std::result::Result<u64, VerifyError> {
    lazy_static! {
        static ref STEAMID_REGEX: Regex =
            Regex::new("^https://steamcommunity.com/openid/id/([0-9]{17})$").unwrap();
    }

    if form.return_to != return_to {
        return Err(VerifyError::Denied);
    }

    form.mode = "check_authentication".to_string();
    let form_str = serde_urlencoded::to_string(&form).map_err(|_| VerifyError::BadQuery)?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://steamcommunity.com/openid/login")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form_str)
        .send()
        .await
        .map_err(|_| VerifyError::RequestFailed)?
        .text()
        .await
        .map_err(|_| VerifyError::BadResponse)?;

    let is_valid = response.split('\n').any(|line| line == "is_valid:true");
    if !is_valid {
        return Err(VerifyError::Denied);
    }

    let captures = STEAMID_REGEX
        .captures(&form.claimed_id)
        .ok_or(VerifyError::BadQuery)?;
    let steamid64_str = captures.get(1).ok_or(VerifyError::BadQuery)?.as_str();
    let steamid64 = steamid64_str
        .parse::<u64>()
        .map_err(|_| VerifyError::BadQuery)?;

    Ok(steamid64)
}

#[derive(Debug)]
pub enum NewError {
    BadUrl,
}
impl Error for NewError {}
impl fmt::Display for NewError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub enum VerifyError {
    Denied,
    BadQuery,
    RequestFailed,
    BadResponse,
}
impl Error for VerifyError {}
impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(serde::Serialize)]
struct RedirectForm<'a> {
    #[serde(rename = "openid.ns")]
    ns: &'static str,
    #[serde(rename = "openid.identity")]
    identity: &'static str,
    #[serde(rename = "openid.claimed_id")]
    claimed_id: &'static str,
    #[serde(rename = "openid.mode")]
    mode: &'static str,
    #[serde(rename = "openid.return_to")]
    return_to: &'a str,
    #[serde(rename = "openid.realm")]
    realm: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerifyForm {
    #[serde(rename = "openid.ns")]
    ns: String,
    #[serde(rename = "openid.mode")]
    mode: String,
    #[serde(rename = "openid.op_endpoint")]
    op_endpoint: String,
    #[serde(rename = "openid.claimed_id")]
    claimed_id: String,
    #[serde(rename = "openid.identity")]
    identity: Option<String>,
    #[serde(rename = "openid.return_to")]
    return_to: String,
    #[serde(rename = "openid.response_nonce")]
    response_nonce: String,
    #[serde(rename = "openid.invalidate_handle")]
    invalidate_handle: Option<String>,
    #[serde(rename = "openid.assoc_handle")]
    assoc_handle: String,
    #[serde(rename = "openid.signed")]
    signed: String,
    #[serde(rename = "openid.sig")]
    sig: String,
}
