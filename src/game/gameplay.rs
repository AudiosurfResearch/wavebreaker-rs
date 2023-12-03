use crate::error::IntoHttp;
use crate::util::game_types::{Character, Leaderboard};
use crate::util::xml::XmlSerializableResponse;
use crate::{error::RouteError, util::game_types::League};
use log::info;
use rocket::State;
use rocket::{form::Form, post, response::content::RawXml, FromForm};
use serde::{Deserialize, Serialize};
use steam_rs::steam_id::SteamId;
use steam_rs::Steam;

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
/// If the song isn't registered on the server yet, it will be created.
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

#[derive(FromForm)]
pub struct SendRideRequest {
    #[field(name = "songid")]
    song_id: u64,
    score: u64,
    vehicle: Character,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SendRideResponse {
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
    rivalname: String,
    rivalscore: u64,
    myscore: u64,
    reignseconds: u64,
}

/// Accepts score submissions by the client.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
#[post("/game_SendRideSteamVerified.php", data = "<form>")]
pub async fn send_ride(
    form: Form<SendRideRequest>,
    steam: &State<Steam>,
) -> Result<RawXml<String>, RouteError> {
    let form = form.into_inner();

    let steam_result = steam
        .authenticate_user_ticket(12900, &form.ticket)
        .await
        .http_internal_error("Failed to authenticate with Steam.")?;
    let player_steam_id = SteamId::from(steam_result.steam_id);

    info!(
        "Score received on {} from {} (Steam) with score {}, using {:?}",
        form.song_id, player_steam_id, form.score, form.vehicle
    );

    SendRideResponse {
        status: "allgood".to_owned(),
        song_id: 143,
        beat_score: BeatScore {
            dethroned: true,
            friend: true,
            rivalname: "test".to_owned(),
            rivalscore: 143,
            myscore: 143,
            reignseconds: 143,
        },
    }
    .to_xml_response()
}

#[derive(FromForm)]
pub struct GetRidesRequest {
    #[field(name = "songid")]
    song_id: u64,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
struct GetRidesResponse {
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
    ride_time: u64,
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
#[post("/game_GetRidesSteamVerified.php", data = "<form>")]
pub async fn get_rides(
    form: Form<GetRidesRequest>,
    steam: &State<Steam>,
) -> Result<RawXml<String>, RouteError> {
    let form = form.into_inner();

    steam
        .authenticate_user_ticket(12900, &form.ticket)
        .await
        .http_internal_error("Failed to authenticate with Steam.")?;

    GetRidesResponse {
        status: "allgood".to_owned(),
        scores: vec![Score {
            score_type: Leaderboard::Friend,
            league: vec![LeagueRides {
                league_id: League::Casual,
                ride: vec![Ride {
                    username: "frien :)".to_owned(),
                    score: 143,
                    vehicle_id: Character::PointmanElite,
                    ride_time: 1_701_633_900,
                    feats: "Stealth, I guess?".to_owned(),
                    song_length: 14300,
                    traffic_count: 143,
                }],
            }],
        }],
        server_time: 1_701_634_291,
    }
    .to_xml_response()
}
