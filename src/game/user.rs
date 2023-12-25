use crate::models::{NewPlayer, Player, SteamIdWrapper};
use crate::util::errors::{IntoRouteError, RouteError};
use crate::AppState;
use crate::{game::helpers::ticket_auth, schema::players};
use axum::{extract::State, Form};
use axum_serde::Xml;
use diesel_async::RunQueryDsl;
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
    user_id: i32,
    username: String,
    #[serde(rename = "locationid")]
    location_id: i32,
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
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api)
        .await
        .http_internal_error("Failed to authenticate with Steam")?;

    info!("Login request from {} (Steam)", steam_player);

    let summary = &state
        .steam_api
        .get_player_summaries(vec![steam_player])
        .await?;

    let mut conn = state.db.get().await?;

    let new_player = NewPlayer {
        username: &summary[0].persona_name,
        steam_id: SteamIdWrapper(steam_player),
        avatar_url: &summary[0].avatar,
    };

    //Insert new player into table if they don't exist
    let player = diesel::insert_into(players::table)
        .values(&new_player)
        .on_conflict_do_nothing()
        .get_result::<Player>(&mut conn)
        .await?;

    Ok(Xml(LoginSteamResponse {
        status: "allgood".to_owned(),
        user_id: player.id,
        username: player.username,
        location_id: player.location_id,
        steam_id: steam_player.get_account_id(),
    }))
}
