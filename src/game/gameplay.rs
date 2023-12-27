use super::helpers::ticket_auth;
use crate::models::songs::NewSong;
use crate::util::errors::RouteError;
use crate::util::game_types::League;
use crate::util::game_types::{Character, Leaderboard};
use crate::AppState;
use axum::extract::State;
use axum::Form;
use axum_serde::Xml;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize)]
pub struct SongIdRequest {
    artist: String,
    song: String,
    uid: u64,
    league: League,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
pub struct SongIdResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "songid")]
    song_id: i32,
}

/// Attempts to get a song ID from the server.
/// If the song isn't registered on the server yet, it will be created.
///
/// # Errors
/// 
/// This fails if:
/// - The response fails to serialize
/// - The song fails to be created/retrieved
pub async fn fetch_song_id(
    State(state): State<AppState>,
    Form(payload): Form<SongIdRequest>,
) -> Result<Xml<SongIdResponse>, RouteError> {
    let mut conn = state.db.get().await?;
    let song = NewSong::new(&payload.song, &payload.artist).find_or_create(&mut conn).await?;

    info!(
        "Song {} - {} looked up/registered by {}, league {:?}",
        song.artist, song.title, payload.uid, payload.league
    );

    Ok(Xml(SongIdResponse {
        status: "allgood".to_owned(),
        song_id: song.id,
    }))
}

#[derive(Deserialize)]
pub struct SendRideRequest {
    #[serde(rename = "songid")]
    song_id: u64,
    score: u64,
    vehicle: Character,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
pub struct SendRideResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "songid")]
    song_id: u64,
    #[serde(rename = "beatscore")]
    beat_score: BeatScore,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct BeatScore {
    #[serde(rename = "@dethroned")]
    dethroned: bool,
    #[serde(rename = "@friend")]
    friend: bool,
    #[serde(rename = "rivalname")]
    rival_name: String,
    #[serde(rename = "rivalscore")]
    rival_score: u64,
    #[serde(rename = "myscore")]
    my_score: u64,
    #[serde(rename = "reignseconds")]
    reign_seconds: u64,
}

/// Accepts score submissions by the client.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
pub async fn send_ride(
    State(state): State<AppState>,
    Form(payload): Form<SendRideRequest>,
) -> Result<Xml<SendRideResponse>, RouteError> {
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;

    info!(
        "Score received on {} from {} (Steam) with score {}, using {:?}",
        payload.song_id, steam_player, payload.score, payload.vehicle
    );

    Ok(Xml(SendRideResponse {
        status: "allgood".to_owned(),
        song_id: 143,
        beat_score: BeatScore {
            dethroned: true,
            friend: true,
            rival_name: "test".to_owned(),
            rival_score: 143,
            my_score: 143,
            reign_seconds: 143,
        },
    }))
}

#[derive(Deserialize)]
pub struct GetRidesRequest {
    #[serde(rename = "songid")]
    song_id: u64,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
pub struct GetRidesResponse {
    #[serde(rename = "@status")]
    status: String,
    scores: Vec<Score>,
    #[serde(rename = "servertime")]
    server_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Score {
    #[serde(rename = "@scoretype")]
    score_type: Leaderboard,
    league: Vec<LeagueRides>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LeagueRides {
    #[serde(rename = "@leagueid")]
    league_id: League,
    ride: Vec<Ride>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ride {
    username: String,
    score: u64,
    #[serde(rename = "vehicleid")]
    vehicle_id: Character,
    #[serde(rename = "ridetime")]
    time: u64,
    feats: String,
    /// In centiseconds (milliseconds / 10)
    #[serde(rename = "songlength")]
    song_length: u64,
    #[serde(rename = "trafficcount")]
    traffic_count: u64,
}

/// Returns scores for a given song.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
pub async fn get_rides(
    State(state): State<AppState>,
    Form(payload): Form<GetRidesRequest>,
) -> Result<Xml<GetRidesResponse>, RouteError> {
    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;

    info!(
        "Player {} (Steam) requesting rides of song {}",
        steam_player, payload.song_id
    );

    Ok(Xml(GetRidesResponse {
        status: "allgood".to_owned(),
        scores: vec![Score {
            score_type: Leaderboard::Friend,
            league: vec![LeagueRides {
                league_id: League::Casual,
                ride: vec![Ride {
                    username: "frien :)".to_owned(),
                    score: 143,
                    vehicle_id: Character::PointmanElite,
                    time: 143,
                    feats: "Stealth, I guess?".to_owned(),
                    song_length: 14300,
                    traffic_count: 143,
                }],
            }],
        }],
        server_time: 143,
    }))
}
