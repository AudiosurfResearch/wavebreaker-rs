use diesel::{prelude::Insertable, query_builder::AsChangeset};
use musicbrainz_rs::{
    entity::{recording::Recording, CoverartResponse},
    FetchCoverart, Search,
};
use tracing::info;

use crate::models::songs::Song;

#[derive(Debug, AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::extra_song_info)]
pub struct MusicBrainzInfo {
    pub cover_url: String,
    pub cover_url_small: String,
    pub mbid: String,
    pub musicbrainz_title: String,
    pub musicbrainz_artist: String,
    pub musicbrainz_length: i32,
}

// TODO: Make this code less bad
pub async fn lookup_metadata(song: &Song, duration: i32) -> anyhow::Result<MusicBrainzInfo> {
    let query = format!(
        "query=(recording:\"{}\" OR alias:\"{0}\") AND artist:\"{}\" AND dur:\"[{} TO {}]\"",
        song.title,
        song.artist,
        duration - 6000,
        duration + 6000
    );

    info!("Searching for recording with query: {:?}", query);

    let query_result = Recording::search(query).execute().await?;
    let query_result = query_result.entities;

    if query_result.is_empty() {
        return Err(anyhow::anyhow!("No recording found"));
    }

    let recording = query_result[0].clone();
    let release = recording.releases.clone();
    let release = if let Some(releases) = release {
        releases[0].clone()
    } else {
        return Err(anyhow::anyhow!("No release found for recording"));
    };

    let cover_url = release.get_coverart().front().res_500().execute().await?;
    let cover_url = match cover_url {
        CoverartResponse::Json(cover) => cover.images[0].image.clone(),
        CoverartResponse::Url(url) => url,
    };

    let cover_url_small = release.get_coverart().front().res_250().execute().await?;
    let cover_url_small = match cover_url_small {
        CoverartResponse::Json(cover) => cover.images[0].image.clone(),
        CoverartResponse::Url(url) => url,
    };

    let mbid = recording.id;
    let musicbrainz_title = recording.title;
    let musicbrainz_artist = recording.artist_credit;
    let musicbrainz_artist = if let Some(artist_credit) = musicbrainz_artist {
        // Join all artists by their join phrase
        let mut artist_string = String::new();
        for artist in artist_credit {
            artist_string.push_str(&artist.name);
            if let Some(join_phrase) = artist.joinphrase {
                artist_string.push_str(&join_phrase);
            }
        }
        artist_string
    } else {
        return Err(anyhow::anyhow!("No artist found for recording"));
    };

    //let's be real, we're not gonna see a song be so long it eclipses i32::MAX
    #[allow(clippy::cast_possible_wrap)]
    let musicbrainz_length = recording.length.map(|length| length as i32);

    Ok(MusicBrainzInfo {
        cover_url,
        cover_url_small,
        mbid,
        musicbrainz_title,
        musicbrainz_artist,
        musicbrainz_length: musicbrainz_length.unwrap_or_default(),
    })
}
