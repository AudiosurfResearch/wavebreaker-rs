use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;
use utoipa::ToSchema;

use crate::schema::extra_song_info;

/// Used for storing additional metadata from [MusicBrainz](https://musicbrainz.org).
/// This lets us display fancy stuff™ on the song page.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Default,
    AsChangeset,
    ToSchema,
)]
#[diesel(belongs_to(super::songs::Song))]
#[diesel(table_name = extra_song_info, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[serde(rename_all = "camelCase")]
pub struct ExtraSongInfo {
    pub id: i32,
    pub song_id: i32,
    pub cover_url: Option<String>,
    pub cover_url_small: Option<String>,
    pub mbid: Option<String>,
    pub musicbrainz_title: Option<String>,
    pub musicbrainz_artist: Option<String>,
    pub musicbrainz_length: Option<i32>,
    /// For songs that have been mistagged by the automatic lookup.
    /// A value of `true` prevents any new metadata lookups by title
    pub mistag_lock: bool,
    /// Alternative artist tags that can be matched to this song
    pub aliases_artist: Option<Vec<Option<String>>>,
    /// Alternative title tags that can be matched to this song
    pub aliases_title: Option<Vec<Option<String>>>,
}

/// Used for inserting additional metadata from [MusicBrainz](https://musicbrainz.org).
#[derive(Insertable, PartialEq, Eq, Debug, Default)]
#[diesel(table_name = extra_song_info)]
#[allow(clippy::module_name_repetitions)]
pub struct NewExtraSongInfo {
    pub song_id: i32,
    pub cover_url: Option<String>,
    pub cover_url_small: Option<String>,
    pub mbid: Option<String>,
    pub musicbrainz_title: Option<String>,
    pub musicbrainz_artist: Option<String>,
    pub musicbrainz_length: Option<i32>,
    pub aliases_title: Option<Vec<String>>,
    pub aliases_artist: Option<Vec<String>>,
}

impl NewExtraSongInfo {
    #[must_use]
    #[allow(clippy::too_many_arguments)] // Too bad, I don't care!
    pub const fn new(
        song_id: i32,
        cover_url: Option<String>,
        cover_url_small: Option<String>,
        mbid: Option<String>,
        musicbrainz_title: Option<String>,
        musicbrainz_artist: Option<String>,
        musicbrainz_length: Option<i32>,
        aliases_title: Option<Vec<String>>,
        aliases_artist: Option<Vec<String>>,
    ) -> Self {
        Self {
            song_id,
            cover_url,
            cover_url_small,
            mbid,
            musicbrainz_title,
            musicbrainz_artist,
            musicbrainz_length,
            aliases_title,
            aliases_artist,
        }
    }

    /// Creates a new `ExtraSongInfo` record in the database.
    ///
    /// # Arguments
    /// * `connection` - The database connection.
    ///
    /// # Errors
    /// Fails if something is wrong with the database.
    pub async fn insert(&self, connection: &mut AsyncPgConnection) -> QueryResult<ExtraSongInfo> {
        diesel::insert_into(extra_song_info::table)
            .values(self)
            .get_result(connection)
            .await
    }
}
