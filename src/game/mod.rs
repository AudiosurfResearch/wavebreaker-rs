mod gameplay;
mod helpers;
mod misc;
mod radio;
mod user;

use axum::{routing::post, Router};
use tower_http::services::ServeDir;

use self::{
    gameplay::{fetch_song_id, get_rides, send_ride},
    misc::{fetch_shouts, fetch_track_shape, get_custom_news, send_shout},
    radio::get_radio_list,
    user::{login_steam, steam_sync},
};
use crate::AppState;

/// Returns all routes used for everything under ``/as_steamlogin``
pub fn routes_steam() -> Router<AppState> {
    Router::new()
        .route("/game_AttemptLoginSteamVerified.php", post(login_steam))
        .route("/game_SteamSyncSteamVerified.php", post(steam_sync))
        .route("/game_fetchsongid_unicode.php", post(fetch_song_id))
        .route("/game_SendRideSteamVerified.php", post(send_ride))
        .route("/game_GetRidesSteamVerified.php", post(get_rides))
        .route("/game_fetchshouts_unicode.php", post(fetch_shouts))
        .route("/game_sendShoutSteamVerified.php", post(send_shout))
}

/// Returns all routes used for everything under ``//as_steamlogin``
///
/// **beware the double slash**
pub fn routes_steam_doubleslash() -> Router<AppState> {
    Router::new().route("/game_CustomNews.php", post(get_custom_news))
}

/// Returns all routes used for everything under ``/as``
pub fn routes_as(cgr_path: &str) -> Router<AppState> {
    Router::new()
        .route("/game_fetchtrackshape2.php", post(fetch_track_shape))
        .route("/asradio/game_asradiolist5.php", post(get_radio_list))
        .nest_service("/asradio", ServeDir::new(cgr_path))
}
