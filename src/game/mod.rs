mod gameplay;
mod helpers;
mod user;

use crate::AppState;
use crate::game::gameplay::{fetch_song_id, get_rides, send_ride};
use crate::game::user::login_steam;
use axum::routing::post;
use axum::Router;

/// Returns all routes used for everything under ``/as_steamlogin``
pub fn routes_steam() -> Router<AppState> {
    Router::new()
        .route("/game_AttemptLoginSteamVerified.php", post(login_steam))
        .route("/game_fetchsongid_unicode.php", post(fetch_song_id))
        .route("/game_SendRideSteamVerified.php", post(send_ride))
        .route("/game_GetRidesSteamVerified.php", post(get_rides))
}
