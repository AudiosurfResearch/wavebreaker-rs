use super::helpers::ticket_auth;
use crate::models::players::Player;
use crate::models::shouts::Shout;
use crate::util::game_types::join_x_separated;
use crate::AppState;
use crate::{models::scores::Score, util::errors::RouteError};
use axum::extract::State;
use axum::Form;
use axum_extra::extract::Form as ExtraForm;
use axum_serde::Xml;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
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
        text: format!(
            "Hi, {}!\n\nWelcome to wavebreaker-rs,\nthe next generation of Wavebreaker!",
            player.username
        ),
    }))
}

#[derive(Deserialize)]
pub struct GetShoutsRequest {
    ridd: i32,
}

/// Sends track shape to the game as x-seperated string
///
/// # Errors
///
/// This fails if:
/// - The response fails to serialize
/// - Something goes wrong with the database
#[instrument(skip_all)]
pub async fn fetch_track_shape(
    State(state): State<AppState>,
    Form(payload): Form<GetShoutsRequest>,
) -> Result<String, RouteError> {
    use crate::schema::scores::dsl::*;

    let mut conn = state.db.get().await?;

    let ride = scores.find(payload.ridd).first::<Score>(&mut conn).await?;
    let track_shape_string = join_x_separated(&ride.track_shape);

    Ok(track_shape_string)
}

// This literally just exists because the shout fetching endpoint gets two song IDs from the game.
pub fn take_first<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let vec: Vec<T> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().next().unwrap_or_default())
}

#[derive(Deserialize)]
pub struct FetchShoutsRequest {
    #[serde(default, deserialize_with = "take_first")]
    songid: i32,
}

/// Sends track shape to the game as x-seperated string
///
/// # Errors
///
/// This fails if:
/// - The response fails to serialize
/// - Something goes wrong with the database
#[instrument(skip_all)]
pub async fn fetch_shouts(
    State(state): State<AppState>,
    ExtraForm(payload): ExtraForm<FetchShoutsRequest>,
) -> Result<String, RouteError> {
    use crate::schema::shouts::dsl::*;

    let mut conn = state.db.get().await?;

    let shouts_with_player = Shout::find_by_song_id(payload.songid)
        .order(posted_at.desc())
        .inner_join(crate::schema::players::table)
        .select((Shout::as_select(), Player::as_select()))
        .load::<(Shout, Player)>(&mut conn)
        .await?;
    if shouts_with_player.is_empty() {
        return Ok(
            "This song has no shouts yet. Let's change that!\n'Cause we're gonna shout it loud!"
                .to_owned(),
        );
    }

    let mut shout_string = String::new();
    for shout in shouts_with_player {
        shout_string.push_str(&format!(
            "{} (at {}): {}\n",
            shout.1.username, shout.0.posted_at, shout.0.content
        ));
    }

    Ok(shout_string)
}
