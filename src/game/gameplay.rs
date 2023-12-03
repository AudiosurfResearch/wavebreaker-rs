use crate::{error::RouteError, util::game_types::League};
use crate::util::xml::XmlSerializableResponse;
use log::info;
use rocket::{form::Form, post, response::content::RawXml, FromForm};
use serde::{Deserialize, Serialize};

#[derive(FromForm)]
pub struct SongIdRequest {
    artist: String,
    song: String,
    uid: u64,
    league: League,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SongIdResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "songid")]
    song_id: u64,
}

/// Attempts to get a song ID from the server.
/// If there is the song isn't registered on the server yet, it will be created.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
#[post("/game_fetchsongid_unicode.php", data = "<form>")]
pub async fn fetch_song_id(form: Form<SongIdRequest>) -> Result<RawXml<String>, RouteError> {
    let form = form.into_inner();

    info!(
        "Song {} - {} registered by {}, league {:?}",
        form.artist, form.song, form.uid, form.league
    );

    SongIdResponse {
        status: "allgood".to_owned(),
        song_id: 143,
    }
    .to_xml_response()
}
