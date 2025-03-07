use axum::{extract::State, Form};
use axum_serde::Xml;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

#[allow(clippy::wildcard_imports)]
use crate::schema::players::dsl::*;
use crate::{
    game::helpers::ticket_auth,
    models::players::{NewPlayer, Player},
    util::{
        errors::{IntoRouteError, RouteError},
        game_types::split_x_separated,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct LoginSteamRequest {
    ticket: String,
    #[serde(rename = "wvbrclientversion")]
    client_version: String,
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
    steam_id: i32,
}

/// Attempts to authenticate a user through Steam.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
/// - Something goes wrong with the database
#[instrument(skip_all, err(Debug), fields(
    steam_id,
    client_version = payload.client_version,
))]
pub async fn login_steam(
    State(state): State<AppState>,
    Form(payload): Form<LoginSteamRequest>,
) -> Result<Xml<LoginSteamResponse>, RouteError> {
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api, &state.redis)
        .await
        .http_internal_error("Failed to authenticate with Steam")?;
    tracing::Span::current().record("steam_id", steam_player.0);

    info!("Login requested");

    let summary = &state
        .steam_api
        .get_player_summaries(vec![steam_player])
        .await?;

    let mut conn = state.db.get().await?;

    let player = NewPlayer::new(
        &summary[0].persona_name,
        steam_player,
        i32::try_from(steam_player.get_account_id())?,
        &summary[0].avatar_full,
    )
    .create_or_update(&mut conn, &state.redis)
    .await?;

    Ok(Xml(LoginSteamResponse {
        status: "allgood".to_owned(),
        user_id: player.id,
        username: player.username,
        location_id: player.location_id,
        steam_id: player.steam_account_num,
    }))
}

#[derive(Deserialize)]
pub struct SteamSyncRequest {
    ticket: String,
    snums: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
pub struct SteamSyncResponse {
    #[serde(rename = "@status")]
    status: String,
}

/// Attempts to sync rivals with user's Steam friends.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
/// - Something goes wrong with the database
#[instrument(
    skip_all,
    err(Debug),
    fields(player, steam_friend_count, found_friend_count)
)]
pub async fn steam_sync(
    State(state): State<AppState>,
    Form(payload): Form<SteamSyncRequest>,
) -> Result<Xml<SteamSyncResponse>, RouteError> {
    //Split the string of steam account numbers into a vector
    //Validating before the steam auth request, because if this is invalid anyway then we don't care about the request
    //This way we have one less Steam API request on the daily limit
    let friend_nums: Vec<i32> =
        split_x_separated(&payload.snums).http_status_error(axum::http::StatusCode::BAD_REQUEST)?;

    let steam_player = ticket_auth(&payload.ticket, &state.steam_api, &state.redis)
        .await
        .http_internal_error("Failed to authenticate with Steam")?;
    let mut conn = state.db.get().await?;

    let player: Player = Player::find_by_steam_id(steam_player)
        .first::<Player>(&mut conn)
        .await?;

    //Get all friends
    let friends = players
        .filter(steam_account_num.eq_any(&friend_nums))
        .load::<Player>(&mut conn)
        .await?;

    tracing::Span::current()
        .record("player", player.id)
        .record("steam_friend_count", friend_nums.len())
        .record("found_friend_count", friends.len());

    info!("Syncing rivals with Steam friends");

    //Add new rivalry for each friend
    for friend in &friends {
        diesel::insert_into(crate::schema::rivalries::table)
            .values((
                crate::schema::rivalries::challenger_id.eq(player.id),
                crate::schema::rivalries::rival_id.eq(friend.id),
            ))
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await?;
    }

    Ok(Xml(SteamSyncResponse {
        //This technically doesn't return the number of friends added
        status: format!("added {} of {} friends", friends.len(), friend_nums.len()),
    }))
}
