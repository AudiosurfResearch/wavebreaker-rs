use axum::extract::State;
use tracing::instrument;

use crate::{util::errors::RouteError, AppState};

/// Returns a list of all Audiosurf Radio songs.
/// Only works with clients using an old version of `RadioBrowser.cgr`
/// That version is included with the Wavebreaker mod.
#[instrument(skip_all)]
pub async fn get_radio_list(State(state): State<AppState>) -> Result<String, RouteError> {
    // join all songs into a single string with -:*x- as separator
    // ignore the id, we don't need it
    let mut joined_string = String::new();
    for song in &state.config.radio.songs {
        joined_string.push_str(&format!(
            "{}-:*x-{}-:*x-{}-:*x-{}-:*x-",
            song.artist, song.title, song.cgr_url, song.buy_url
        ));
    }

    Ok(joined_string)
}
