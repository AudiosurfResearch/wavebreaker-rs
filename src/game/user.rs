use crate::game::helpers::ticket_auth;
use crate::AppState;
use axum_route_error::RouteError;
use axum::{extract::State, Form};
use axum_serde::Xml;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct LoginSteamRequest {
    #[serde(rename = "steamusername")]
    steam_username: String,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
pub struct LoginSteamResponse {
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
pub async fn login_steam(
    State(state): State<AppState>,
    Form(payload): Form<LoginSteamRequest>,
) -> Result<Xml<LoginSteamResponse>, RouteError> {
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;

    info!("Login request from {} (Steam)", steam_player);

    Ok(Xml(LoginSteamResponse {
        status: "allgood".to_owned(),
        user_id: 143,
        username: payload.steam_username,
        location_id: 143,
        steam_id: steam_player.get_account_id(),
    }))
}
