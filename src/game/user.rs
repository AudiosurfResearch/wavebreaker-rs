use crate::error::{IntoHttp, RouteError};
use crate::util::xml::XmlSerializableResponse;
use rocket::{form::Form, post, response::content::RawXml, FromForm, State};
use serde::{Deserialize, Serialize};
use steam_rs::{steam_id::SteamId, Steam};

#[derive(FromForm)]
pub struct LoginSteamRequest {
    #[field(name = "steamusername")]
    steam_username: String,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct LoginSteamResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "userid")]
    user_id: u64,
    username: String,
    #[serde(rename = "locationid")]
    location_id: u32,
    #[serde(rename = "steamid")]
    steam_id: u32,
}

/// Attempts to authenticate a user through Steam.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
#[post("/game_AttemptLoginSteamVerified.php", data = "<form>")]
pub async fn login_steam(
    form: Form<LoginSteamRequest>,
    steam: &State<Steam>,
) -> Result<RawXml<String>, RouteError> {
    let form = form.into_inner();

    let steam_result = steam
        .authenticate_user_ticket(12900, &form.ticket)
        .await
        .http_internal_error("Failed to authenticate with Steam.")?;
    let player_steam_id = SteamId::from(steam_result.steam_id);

    LoginSteamResponse {
        status: "allgood".to_owned(),
        user_id: 143,
        username: form.steam_username,
        location_id: 143,
        steam_id: player_steam_id.get_account_id(),
    }
    .to_xml_response()
}
