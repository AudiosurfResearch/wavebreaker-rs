use super::helpers::ticket_auth;
use crate::AppState;
use crate::{models::players::Player, util::errors::RouteError};
use axum::extract::State;
use axum::Form;
use axum_serde::Xml;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Deserialize)]
pub struct CustomNewsRequest {
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
pub struct CustomNewsResponse {
    #[serde(rename = "TEXT")]
    text: String,
}

/// Sends text to the game, shown before playing a song
///
/// # Errors
///
/// This fails if:
/// - The response fails to serialize
#[instrument(skip_all)]
pub async fn get_custom_news(
    State(state): State<AppState>,
    Form(payload): Form<CustomNewsRequest>,
) -> Result<Xml<CustomNewsResponse>, RouteError> {
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;

    let mut conn = state.db.get().await?;

    let player: Player = Player::find_by_steam_id(steam_player)
        .first::<Player>(&mut conn)
        .await?;

    Ok(Xml(CustomNewsResponse {
        text: format!("Hi, {}!\n\nWelcome to wavebreaker-rs,\nthe next generation of Wavebreaker!", player.username),
    }))
}
