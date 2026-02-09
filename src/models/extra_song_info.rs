use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

use crate::schema::extra_song_info;

/// Used for storing additional metadata from [MusicBrainz](https://musicbrainz.org).
/// This lets us display fancy stuff™ on the song page.
#[derive(
    Clone,
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    AsChangeset,
    ToSchema,
)]
#[diesel(belongs_to(super::songs::Song))]
#[diesel(table_name = extra_song_info, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[serde(rename_all = "camelCase")]
#[schema(examples(json!(ExtraSongInfo {
    id: 1,
    song_id: 1,
    cover_url: Some("https://ia600309.us.archive.org/13/items/mbid-8a4b4cd1-917c-46bd-94eb-ab9f22a5c46f/mbid-8a4b4cd1-917c-46bd-94eb-ab9f22a5c46f-36122349406_thumb500.jpg".to_owned()),
    cover_url_small: Some("https://ia600309.us.archive.org/13/items/mbid-8a4b4cd1-917c-46bd-94eb-ab9f22a5c46f/mbid-8a4b4cd1-917c-46bd-94eb-ab9f22a5c46f-36122349406_thumb.jpg".to_owned()),
    mbid: Some("51517639-fafe-456a-9d6a-e4a11db00cf5".to_owned()),
    musicbrainz_title: Some("Sendoff".to_owned()),
    musicbrainz_artist: Some("Inverted Silence".to_owned()),
    musicbrainz_length: Some(202559),
    mistag_lock: false,
    aliases_artist: None,
    aliases_title: None,
    updated_at: time::OffsetDateTime::now_utc(),
}), json!(ExtraSongInfo {
    id: 2,
    song_id: 2,
    cover_url: Some("https://i.scdn.co/image/ab67616d0000b27356021e26fd463e7c1d062a9d".to_owned()),
    cover_url_small: Some("https://i.scdn.co/image/ab67616d0000485156021e26fd463e7c1d062a9d".to_owned()),
    mbid: Some("6c7e91a6-9cba-4ddf-b262-b5bf7be72d44".to_owned()),
    musicbrainz_title: Some("Dyad".to_owned()),
    musicbrainz_artist: Some("Jamie Paige".to_owned()),
    musicbrainz_length: Some(282315),
    mistag_lock: false,
    aliases_artist: Some(vec![Some("JamieP".to_owned())]),
    aliases_title: None,
    updated_at: time::OffsetDateTime::now_utc(),
})))]
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
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
    pub updated_at: time::OffsetDateTime,
}

impl Default for ExtraSongInfo {
    fn default() -> Self {
        // i'm going to be real with you chief
        // i am not pulling in a crate with a macro just to make this look better
        Self {
            id: Default::default(),
            song_id: Default::default(),
            cover_url: Default::default(),
            cover_url_small: Default::default(),
            mbid: Default::default(),
            musicbrainz_title: Default::default(),
            musicbrainz_artist: Default::default(),
            musicbrainz_length: Default::default(),
            mistag_lock: Default::default(),
            aliases_artist: Default::default(),
            aliases_title: Default::default(),
            updated_at: time::OffsetDateTime::now_utc(),
        }
    }
}

/// Used for inserting additional metadata from [MusicBrainz](https://musicbrainz.org).
#[derive(
    Insertable, AsChangeset, PartialEq, Eq, Debug, Default, ToSchema, Serialize, Deserialize,
)]
#[diesel(table_name = extra_song_info)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
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
