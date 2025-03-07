use diesel::{prelude::Insertable, query_builder::AsChangeset};
use musicbrainz_rs::{
    client::MusicBrainzClient,
    entity::{recording::Recording, release::Release, CoverartResponse},
    Fetch, FetchCoverart, Search,
};
use tracing::{error, info, instrument, warn};

use crate::models::songs::Song;

#[derive(Debug, AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::extra_song_info)]
pub struct MusicBrainzInfo {
    pub cover_url: Option<String>,
    pub cover_url_small: Option<String>,
    pub mbid: String,
    pub musicbrainz_title: String,
    pub musicbrainz_artist: String,
    pub musicbrainz_length: i32,
}

// TODO: Make this code less bad
/// Tries automatically finding song on MB with title, artist and duration
///
/// # Errors
/// Fails if no song is found or lookup errors
#[instrument(skip(song, client), err, fields(title = song.title, artist = song.artist))]
pub async fn lookup_metadata(
    song: &Song,
    duration: i32,
    client: &MusicBrainzClient,
) -> anyhow::Result<Option<MusicBrainzInfo>> {
    let query = format!(
        "query=(recording:\"{}\" OR alias:\"{0}\") AND artist:\"{}\" AND dur:\"[{} TO {}]\"",
        song.title,
        song.artist,
        duration - 6000,
        duration + 6000
    );

    info!("Searching for recording with query: {:?}", query);

    let query_result = Recording::search(query)
        .execute_with_client(&client)
        .await?
        .entities;

    if query_result.is_empty() {
        info!(
            "No recording found for {} - {} (ID {})",
            song.artist, song.title, song.id
        );
        return Ok(None);
    }

    let recording = query_result[0].clone();
    info!("Found recording with ID {}", recording.id);
    let release = match recording.releases.clone() {
        Some(releases) => releases[0].clone(),
        None => return Err(anyhow::anyhow!("No release found for recording")),
    };
    info!("Found release with ID {}", release.id);

    let cover_url = match release
        .get_coverart()
        .front()
        .res_500()
        .execute_with_client(&client)
        .await
    {
        Ok(cover_resp) => match cover_resp {
            CoverartResponse::Json(cover) => Some(cover.images[0].image.clone()),
            CoverartResponse::Url(url) => Some(url),
        },
        Err(e) => {
            warn!("Failed to fetch cover: {:?}", e);
            None
        }
    };
    if cover_url.is_some() {
        info!("Found cover {:?}", cover_url);
    }

    let cover_url_small = match release
        .get_coverart()
        .front()
        .res_250()
        .execute_with_client(&client)
        .await
    {
        Ok(cover_resp) => match cover_resp {
            CoverartResponse::Json(cover) => Some(cover.images[0].image.clone()),
            CoverartResponse::Url(url) => Some(url),
        },
        Err(e) => {
            warn!("Failed to fetch small cover: {:?}", e);
            None
        }
    };
    if cover_url_small.is_some() {
        info!("Found small cover {:?}", cover_url_small);
    }

    let mbid = recording.id;
    let musicbrainz_title = recording.title;
    let musicbrainz_artist = match recording.artist_credit {
        Some(artist_credit) => {
            // Join all artists by their join phrase
            let mut artist_string = String::new();
            for artist in artist_credit {
                artist_string.push_str(&artist.name);
                if let Some(join_phrase) = artist.joinphrase {
                    artist_string.push_str(&join_phrase);
                }
            }
            artist_string
        }
        None => return Err(anyhow::anyhow!("No artist found for recording")),
    };
    info!("Merged artist credits to: {}", musicbrainz_artist);

    //let's be real, we're not gonna see a song be so long it eclipses i32::MAX
    #[allow(clippy::cast_possible_wrap)]
    let musicbrainz_length = recording.length.map(|length| length as i32);

    Ok(Some(MusicBrainzInfo {
        cover_url,
        cover_url_small,
        mbid,
        musicbrainz_title,
        musicbrainz_artist,
        musicbrainz_length: musicbrainz_length.unwrap_or_default(),
    }))
}

/// Fetches song metadata using recording and release MBIDs
///
/// # Errors
/// Fails if no song is found or lookup fails
#[instrument(skip(client), err)]
pub async fn lookup_mbid(
    mbid: &str,
    release_mbid: Option<&str>,
    client: &MusicBrainzClient,
) -> anyhow::Result<MusicBrainzInfo> {
    let recording = Recording::fetch()
        .id(mbid)
        .with_releases()
        .with_artists()
        .execute_with_client(&client)
        .await?;

    // get cover from user-supplied release, if present
    let release = match release_mbid {
        Some(release_mbid) => {
            info!("Fetching release from MBID");
            match Release::fetch()
                .id(release_mbid)
                .execute_with_client(&client)
                .await
            {
                Ok(release_result) => release_result,
                Err(_) => {
                    return Err(anyhow::anyhow!("Failed to fetch release from MBID"));
                }
            }
        }
        None => match recording.releases.clone() {
            Some(releases) => releases[0].clone(),
            None => return Err(anyhow::anyhow!("No release found for recording")),
        },
    };

    let cover_url = match release
        .get_coverart()
        .front()
        .res_500()
        .execute_with_client(&client)
        .await
    {
        Ok(cover_resp) => match cover_resp {
            CoverartResponse::Json(cover) => Some(cover.images[0].image.clone()),
            CoverartResponse::Url(url) => Some(url),
        },
        Err(e) => {
            error!("Failed to fetch cover: {:?}", e);
            None
        }
    };
    if cover_url.is_some() {
        info!("Found cover {:?}", cover_url);
    }

    let cover_url_small = match release
        .get_coverart()
        .front()
        .res_250()
        .execute_with_client(&client)
        .await
    {
        Ok(cover_resp) => match cover_resp {
            CoverartResponse::Json(cover) => Some(cover.images[0].image.clone()),
            CoverartResponse::Url(url) => Some(url),
        },
        Err(e) => {
            error!("Failed to fetch small cover: {:?}", e);
            None
        }
    };
    if cover_url_small.is_some() {
        info!("Found small cover {:?}", cover_url_small);
    }

    let mbid = recording.id;
    let musicbrainz_title = recording.title;
    let musicbrainz_artist = match recording.artist_credit {
        Some(artist_credit) => {
            // Join all artists by their join phrase
            let mut artist_string = String::new();
            for artist in artist_credit {
                artist_string.push_str(&artist.name);
                if let Some(join_phrase) = artist.joinphrase {
                    artist_string.push_str(&join_phrase);
                }
            }
            artist_string
        }
        None => return Err(anyhow::anyhow!("No artist found for recording")),
    };
    info!("Merged artist credits to: {}", musicbrainz_artist);

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
