use crate::error::RouteError;
use crate::game::helpers::ticket_auth;
use crate::util::xml::XmlSerializableResponse;
use log::info;
use rocket::{form::Form, post, response::content::RawXml, FromForm, State};
use serde::{Deserialize, Serialize};
use steam_rs::Steam;

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

    let steam_player = ticket_auth(&form.ticket, steam).await?;

    info!("Login request from {} (Steam)", steam_player);

    LoginSteamResponse {
        status: "allgood".to_owned(),
        user_id: 143,
        username: form.steam_username,
        location_id: 143,
        steam_id: steam_player.get_account_id(),
    }
    .to_xml_response()
}
