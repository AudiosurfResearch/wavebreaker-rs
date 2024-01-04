use super::helpers::ticket_auth;
use crate::models::players::Player;
use crate::models::rivalries::Rivalry;
use crate::models::scores::{NewScore, Score, ScoreWithPlayer};
use crate::models::songs::NewSong;
use crate::util::errors::RouteError;
use crate::util::game_types::{split_x_separated, League};
use crate::util::game_types::{Character, Leaderboard};
use crate::AppState;
use axum::extract::State;
use axum::Form;
use axum_serde::Xml;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::try_join;
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
    let song = NewSong::new(&payload.song, &payload.artist)
        .find_or_create(&mut conn)
        .await?;

    info!(
        "Song {} - {} looked up by {}, league {:?}",
        song.artist, song.title, payload.uid, payload.league
    );

    Ok(Xml(SongIdResponse {
        status: "allgood".to_owned(),
        song_id: song.id,
    }))
}

#[derive(Deserialize)]
pub struct SendRideRequest {
    ticket: String,
    #[serde(rename = "songid")]
    song_id: i32,
    score: i32,
    vehicle: Character,
    league: League,
    feats: String,
    #[serde(rename = "songlength")]
    song_length: i32,
    #[serde(rename = "trackshape")]
    track_shape: String,
    density: i32,
    xstats: String,
    #[serde(rename = "goldthreshold")]
    gold_threshold: i32,
    iss: i32,
    isj: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
pub struct SendRideResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "songid")]
    song_id: i32,
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
    rival_score: i32,
    #[serde(rename = "myscore")]
    my_score: i32,
    #[serde(rename = "reignseconds")]
    reign_seconds: i64,
}

/// Accepts score submissions by the client.
///
/// # Errors
/// This fails if:
/// - The response fails to serialize
/// - Authenticating with Steam fails
/// - The score fails to be inserted
pub async fn send_ride(
    State(state): State<AppState>,
    Form(payload): Form<SendRideRequest>,
) -> Result<Xml<SendRideResponse>, RouteError> {
    use crate::schema::players::dsl::*;
    use crate::schema::rivalries::dsl::*;
    use crate::schema::scores::dsl::*;

    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;

    info!(
        "Score received on {} from {} (Steam) with score {}, using {:?}",
        &payload.song_id, &steam_player, &payload.score, &payload.vehicle
    );

    let mut conn = state.db.get().await?;
    let player: Player = Player::find_by_steam_id(steam_player)
        .first::<Player>(&mut conn)
        .await?;

    let current_top: QueryResult<(Score, Player)> = scores
        .inner_join(players::table())
        .filter(song_id.eq(payload.song_id))
        .filter(league.eq(payload.league))
        .filter(player_id.ne(player.id))
        .order(score.desc())
        .first::<(Score, Player)>(&mut conn)
        .await;

    let beat_score = if let Ok(current_top) = current_top {
        if current_top.0.score < payload.score {
            info!(
                "Player {} (Steam) dethroned {} on {} with score {}",
                steam_player, current_top.1.id, current_top.0.song_id, payload.score
            );
        }

        let reign_duration = OffsetDateTime::now_utc() - current_top.0.submitted_at.assume_utc();

        let rivalry = rivalries
            .find((player.id, player.id))
            .first::<Rivalry>(&mut conn)
            .await;
        //If rivalry exists, check if rivalry is mutual
        let mutual = if let Ok(rivalry) = rivalry {
            rivalry.is_mutual(&mut conn).await
        } else {
            false
        };

        BeatScore {
            dethroned: current_top.0.score < payload.score,
            friend: mutual,
            rival_name: current_top.1.username,
            rival_score: current_top.0.score,
            my_score: payload.score,
            reign_seconds: reign_duration.whole_seconds(),
        }
    } else {
        info!(
            "Player {} (Steam) got a new top score of {}",
            steam_player, payload.score
        );
        BeatScore {
            dethroned: false,
            friend: false,
            rival_name: "No one".to_owned(),
            rival_score: 143,
            my_score: 0,
            reign_seconds: 0,
        }
    };

    let new_score = NewScore::new(
        player.id,
        payload.song_id,
        payload.league,
        payload.score,
        &split_x_separated::<i32>(&payload.track_shape)?,
        &payload
            .xstats
            .split(',')
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()?,
        payload.density,
        payload.vehicle,
        &payload.feats.split(", ").collect::<Vec<&str>>(),
        payload.song_length,
        payload.gold_threshold,
        payload.iss,
        payload.isj,
    )
    .create_or_update(&mut conn)
    .await?;

    // TODO: Properly implement dethroning
    Ok(Xml(SendRideResponse {
        status: "allgood".to_owned(),
        song_id: new_score.song_id,
        beat_score,
    }))
}

#[derive(Deserialize)]
pub struct GetRidesRequest {
    #[serde(rename = "songid")]
    song_id: i32,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
pub struct GetRidesResponse {
    #[serde(rename = "@status")]
    status: String,
    scores: Vec<ResponseScore>,
    #[serde(rename = "servertime")]
    server_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseScore {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Ride {
    username: String,
    score: i32,
    #[serde(rename = "vehicleid")]
    vehicle_id: Character,
    #[serde(rename = "ridetime")]
    time: i64,
    feats: String,
    /// In centiseconds (milliseconds / 10)
    #[serde(rename = "songlength")]
    song_length: i32,
    #[serde(rename = "trafficcount")]
    traffic_count: i32,
}

fn create_league_rides(league: League, scores: Vec<ScoreWithPlayer>) -> LeagueRides {
    let mut league_rides = LeagueRides {
        league_id: league,
        ride: vec![],
    };

    for with_player in scores {
        league_rides.ride.push(Ride {
            username: with_player.player.username,
            score: with_player.score.score,
            vehicle_id: with_player.score.vehicle,
            time: with_player.score.submitted_at.assume_utc().unix_timestamp(),
            feats: with_player
                .score
                .feats
                .iter()
                .filter_map(std::clone::Clone::clone)
                .collect::<Vec<String>>()
                .join(", "),
            song_length: with_player.score.song_length,
            traffic_count: with_player.score.density,
        });
    }

    league_rides
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
    const ALL_LEAGUES: [League; 3] = [League::Casual, League::Pro, League::Elite];

    let steam_player = ticket_auth(&payload.ticket, &state.steam_api).await?;
    info!(
        "Player {} (Steam) requesting rides of song {}",
        steam_player, payload.song_id
    );

    let mut conn = state.db.get().await?;

    let player: Player = Player::find_by_steam_id(steam_player)
        .first::<Player>(&mut conn)
        .await?;

    let mut rival_ids: Vec<i32> = player
        .get_rivals(&mut conn)
        .await?
        .iter()
        .map(|r| r.id)
        .collect();
    rival_ids.push(player.id); // So the player can see themself in rival scores

    let mut global_rides: Vec<LeagueRides> = vec![];
    let mut rival_rides: Vec<LeagueRides> = vec![];
    let mut nearby_rides: Vec<LeagueRides> = vec![];

    for league in ALL_LEAGUES {
        // can't borrow that one connection as mutable multiple times, so we need more
        let mut conn1 = state.db.get().await?;
        let mut conn2 = state.db.get().await?;

        let global_future = Score::game_get_global(payload.song_id, league, &mut conn);
        let rival_future = Score::game_get_rivals(payload.song_id, league, &rival_ids, &mut conn1);
        let nearby_future =
            Score::game_get_nearby(payload.song_id, league, player.location_id, &mut conn2);

        let (global_scores, rival_scores, nearby_scores) =
            try_join!(global_future, rival_future, nearby_future)?;

        global_rides.push(create_league_rides(league, global_scores));
        rival_rides.push(create_league_rides(league, rival_scores));
        nearby_rides.push(create_league_rides(league, nearby_scores));
    }

    Ok(Xml(GetRidesResponse {
        status: "allgood".to_owned(),
        scores: vec![
            ResponseScore {
                score_type: Leaderboard::Global,
                league: global_rides,
            },
            ResponseScore {
                score_type: Leaderboard::Friend,
                league: rival_rides,
            },
            ResponseScore {
                score_type: Leaderboard::Nearby,
                league: nearby_rides,
            },
        ],
        server_time: 143,
    }))
}
