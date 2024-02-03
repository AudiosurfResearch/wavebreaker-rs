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
    schema::players::steam_account_num,
    util::{
        errors::{IntoRouteError, RouteError},
        game_types::split_x_separated,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct LoginSteamRequest {
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
    steam_id: i32,
}

/// Attempts to authenticate a user through Steam.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
/// - Something goes wrong with the database
#[instrument(skip_all)]
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

    let player = NewPlayer::new(
        &summary[0].persona_name,
        steam_player,
        i32::try_from(steam_player.get_account_id())?,
        &summary[0].avatar_full,
    )
    .create_or_update(&mut conn)
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
#[instrument(skip_all)]
pub async fn steam_sync(
    State(state): State<AppState>,
    Form(payload): Form<SteamSyncRequest>,
) -> Result<Xml<SteamSyncResponse>, RouteError> {
    //Split the string of steam account numbers into a vector
    let friend_nums: Vec<i32> =
        split_x_separated(&payload.snums).http_status_error(axum::http::StatusCode::BAD_REQUEST)?;

    let steam_player = ticket_auth(&payload.ticket, &state.steam_api)
        .await
        .http_internal_error("Failed to authenticate with Steam")?;

    let mut conn = state.db.get().await?;

    //Get all friends
    let friends = players
        .filter(steam_account_num.eq_any(&friend_nums))
        .load::<Player>(&mut conn)
        .await?;

    //Add new rivalry for each friend
    for friend in &friends {
        diesel::insert_into(crate::schema::rivalries::table)
            .values((
                crate::schema::rivalries::challenger_id
                    .eq(i32::try_from(steam_player.get_account_id())?),
                crate::schema::rivalries::rival_id.eq(friend.steam_account_num),
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
