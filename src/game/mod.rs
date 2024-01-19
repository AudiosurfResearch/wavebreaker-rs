mod gameplay;
mod helpers;
mod radio;
mod user;

use self::gameplay::{fetch_song_id, get_rides, send_ride};
use self::radio::get_radio_list;
use self::user::{login_steam, steam_sync};
use crate::AppState;
use axum::routing::post;
use axum::Router;
use tower_http::services::ServeDir;

/// Returns all routes used for everything under ``/as_steamlogin``
pub fn routes_steam() -> Router<AppState> {
    Router::new()
        .route("/game_AttemptLoginSteamVerified.php", post(login_steam))
        .route("/game_SteamSyncSteamVerified.php", post(steam_sync))
        .route("/game_fetchsongid_unicode.php", post(fetch_song_id))
        .route("/game_SendRideSteamVerified.php", post(send_ride))
        .route("/game_GetRidesSteamVerified.php", post(get_rides))
}

/// Returns all routes used for everything under ``/as``
pub fn routes_as(cgr_path: &str) -> Router<AppState> {
    Router::new()
        .route("/asradio/game_asradiolist5.php", post(get_radio_list))
        .nest_service("/asradio", ServeDir::new(cgr_path))
}
