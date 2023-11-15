use crate::util::errors::IntoHttpError;
use actix_web::{post, web, Result};
use quick_xml::se;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SteamLoginRequest {
    steamusername: String,
    snum: i32,
    s64: i64,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SteamLoginResponse {
    #[serde(rename = "@status")]
    status: String,
    userid: i64,
    username: String,
    locationid: i32,
    steamid: i32,
}

#[post("/game_AttemptLoginSteamVerified.php")]
pub async fn steam_login(
    web::Form(form): web::Form<SteamLoginRequest>,
) -> Result<String, actix_web::Error> {
    log::info!("Log in request from {} ({})", form.steamusername, form.s64);

    let response = SteamLoginResponse {
        status: "allgood".to_owned(),
        userid: 1,
        username: form.steamusername,
        locationid: 1,
        steamid: form.snum,
    };

    se::to_string(&response).http_internal_error("Error serializing response")
}
