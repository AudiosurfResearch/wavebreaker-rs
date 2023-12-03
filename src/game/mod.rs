mod gameplay;
mod user;

use rocket::{routes, Route};

use crate::game::gameplay::{fetch_song_id, send_ride, get_rides};
use crate::game::user::login_steam;

/// Returns all routes used for everything under ``/as_steamlogin``
pub fn routes_steam() -> Vec<Route> {
    routes![login_steam, fetch_song_id, send_ride, get_rides]
}