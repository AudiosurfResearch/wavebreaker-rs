#[allow(unused_imports)] //because this import is needed to use from_str(). it's not unused.
use std::str::FromStr;

use crate::{util::errors::IntoHttpError, AppGlobals};
use actix_web::{http::StatusCode, post, web, Result};
use quick_xml::se;
use serde::{Deserialize, Serialize};
use steam_rs::steam_id::SteamId;

#[derive(Deserialize)]
pub struct SteamLoginRequest {
    steamusername: String,
    //snum: u32,
    s64: u64,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SteamLoginResponse {
    #[serde(rename = "@status")]
    status: String,
    userid: u64,
    username: String,
    locationid: u32,
    steamid: u32,
}

/// Endpoint used by the game to log in with a Steam auth ticket.
///
/// # Errors
///
/// This will fail if:
/// - The response fails to serialize
/// - The ticket fails to validate
#[post("/game_AttemptLoginSteamVerified.php")]
pub async fn steam_login(
    web::Form(form): web::Form<SteamLoginRequest>,
    data: web::Data<AppGlobals>,
) -> Result<String, actix_web::Error> {
    log::info!("Log in request from {} ({})", form.steamusername, form.s64);

    let steam_user = data
        .steam_api
        .authenticate_user_ticket(12900, &form.ticket)
        .await
        .http_error(
            "Failed to authenticate with Steam",
            StatusCode::UNAUTHORIZED,
        )?;
    let steam_id = SteamId::from_str(&steam_user.steam_id).http_internal_error_default()?;

    let response = SteamLoginResponse {
        status: "allgood".to_owned(),
        userid: 1,
        username: form.steamusername,
        locationid: 1,
        steamid: steam_id.get_account_id(),
    };

    se::to_string(&response).http_internal_error_default()
}

#[derive(Deserialize)]
pub struct SteamSyncRequest {
    steamusername: String,
    //snum: u32,
    s64: u64,
    ticket: String,
    snums: String, //comma-seperated list of friend SteamID32s
    achstates: String, //comma-seperated list of achievement unlock states
}

//response usually has three root tags and two with THE SAME NAME but I've elected to not care 
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
struct SteamSyncResponse {
    #[serde(rename = "@status")]
    status: String, //usually "added x of y friends"
}

/// Endpoint used by the game to sync the friend list and achievements with the game server.
/// **Not yet implemented!**
#[post("/game_SteamSyncSteamVerified.php")]
pub async fn steam_sync(
    web::Form(form): web::Form<SteamSyncRequest>,
) -> Result<String, actix_web::Error> {
    todo!("Yeah");

    log::info!("Doing steam sync for {} ({})", form.steamusername, form.s64);

    let response = SteamSyncResponse {
        status: "allgood".to_owned(),
    };

    se::to_string(&response).http_internal_error_default()
}