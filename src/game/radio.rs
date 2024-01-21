use tracing::instrument;

use crate::util::{errors::RouteError, radio::get_radio_songs};

/// Returns a list of all Audiosurf Radio songs.
/// Only works with clients using an old version of `RadioBrowser.cgr`
/// That version is included with the Wavebreaker mod.
#[instrument]
pub async fn get_radio_list() -> Result<String, RouteError> {
    let radio_songs = match get_radio_songs() {
        Ok(Some(songs)) => songs,
        Ok(None) => {
            return Ok("no radio songs-:*x-This server has-:*x-none-:*x-https://github.com/AudiosurfResearch-:*x-".to_owned());
        }
        Err(e) => {
            tracing::error!("Failed to get radio songs: {}", e);
            return Err(RouteError::new_internal_server());
        }
    };

    // join all songs into a single string with -:*x- as separator
    // ignore the id, we don't need it
    let mut joined_string = String::new();
    for song in radio_songs {
        joined_string.push_str(&format!(
            "{}-:*x-{}-:*x-{}-:*x-{}-:*x-",
            song.artist, song.title, song.cgr_url, song.buy_url
        ));
    }

    Ok(joined_string)
}
