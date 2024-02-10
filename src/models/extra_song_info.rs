use diesel::prelude::*;
use serde::Serialize;

use crate::schema::extra_song_info;

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Serialize)]
#[diesel(table_name = extra_song_info, check_for_backend(diesel::pg::Pg))]
pub struct ExtraSongInfo {
    // Extended data
    pub id: i32,
    pub song_id: i32,
    pub cover_url: Option<String>,
    pub cover_url_small: Option<String>,
    pub mbid: Option<String>,
    pub musicbrainz_title: Option<String>,
    pub musicbrainz_artist: Option<String>,
    pub musicbrainz_length: Option<i32>,
    pub mistag_lock: bool,
    pub aliases_artist: Option<Vec<String>>,
    pub aliases_title: Option<Vec<String>>,
}
