use axum::{
    extract::{Path, Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use tracing::instrument;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    models::{
        extra_song_info::{ExtraSongInfo, NewExtraSongInfo},
        players::{Player, PlayerPublic},
        scores::Score,
        shouts::Shout,
        songs::Song,
    },
    schema,
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        game_types::{Character, League},
        musicbrainz,
        radio::get_radio_songs as get_radio_songs_util,
        session::Session,
        validator::ValidatedQuery,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_song, delete_song))
        .routes(routes!(get_top_songs))
        .routes(routes!(get_song_scores))
        .routes(routes!(get_radio_songs))
        .routes(routes!(get_song_shouts))
        .routes(routes!(update_song_extra_info))
        .routes(routes!(update_song_extra_info_mbid))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SongResponse {
    #[serde(flatten)]
    song: Song,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_info: Option<ExtraSongInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSongParams {
    #[serde(default)] // default to false
    with_extra_info: bool,
}

/// Get song by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of song to get"),
        ("withExtraInfo" = bool, Query, description = "Include extra info")
    ),
    responses(
        (status = OK, description = "Success", body = SongResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip(state), err(Debug))]
async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    query: Query<GetSongParams>,
) -> Result<Json<SongResponse>, RouteError> {
    use crate::schema::songs;

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;
    if query.with_extra_info {
        let extra_info: Option<ExtraSongInfo> = ExtraSongInfo::belonging_to(&song)
            .first(&mut conn)
            .await
            .optional()?;
        return Ok(Json(SongResponse { song, extra_info }));
    }

    Ok(Json(SongResponse {
        song,
        extra_info: None,
    }))
}

/// Delete song by ID
#[utoipa::path(
    method(delete),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of song to get")
    ),
    responses(
        (status = OK, description = "Success"),
        (status = UNAUTHORIZED, description = "No permission", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    ),
    security(
        ("token_jwt" = [])
    )
)]
#[instrument(skip(state, session), err(Debug))]
async fn delete_song(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    session: Session,
) -> Result<(), RouteError> {
    use crate::schema::songs;

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    if song.user_can_delete(&session.player, &mut conn).await? {
        song.delete(&mut conn, &state.redis).await?;

        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}

#[serde_inline_default]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct GetTopSongParams {
    #[serde(default)] // default to false
    with_extra_info: bool,
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TopSongResponse {
    song_data: SongResponse,
    times_played: i64,
}

allow_columns_to_appear_in_same_group_by_clause!(
    schema::songs::id,
    schema::songs::title,
    schema::songs::artist,
    schema::songs::created_at,
    schema::songs::modifiers,
    schema::extra_song_info::id,
    schema::extra_song_info::song_id,
    schema::extra_song_info::cover_url,
    schema::extra_song_info::cover_url_small,
    schema::extra_song_info::mbid,
    schema::extra_song_info::musicbrainz_title,
    schema::extra_song_info::musicbrainz_artist,
    schema::extra_song_info::musicbrainz_length,
    schema::extra_song_info::mistag_lock,
    schema::extra_song_info::aliases_artist,
    schema::extra_song_info::aliases_title,
);

/// Get global most played songs
#[utoipa::path(
    method(get),
    path = "/rankings",
    params(
        ("withExtraInfo" = Option<bool>, Query, description = "Include extra info"),
        ("page" = Option<i64>, Query, description = "Page number", minimum = 1),
        ("pageSize" = Option<i64>, Query, description = "Page size", minimum = 1, maximum = 50)
    ),
    responses(
        (status = OK, description = "Success", body = Vec<TopSongResponse>, content_type = "application/json"),
        (status = BAD_REQUEST, description = "Invalid query parameters", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip(state), err(Debug))]
async fn get_top_songs(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<GetTopSongParams>,
) -> Result<Json<Vec<TopSongResponse>>, RouteError> {
    use diesel::{dsl::sql, sql_types::BigInt};

    use crate::schema::{extra_song_info, scores, songs};

    let mut conn = state.db.get().await?;

    if query.with_extra_info {
        let songs_with_extra: Vec<(Song, i64, Option<ExtraSongInfo>)> = songs::table
            .left_join(scores::table)
            .left_join(extra_song_info::table)
            .group_by((
                songs::id,
                songs::title,
                songs::artist,
                songs::created_at,
                songs::modifiers,
                schema::extra_song_info::id,
                schema::extra_song_info::song_id,
                schema::extra_song_info::cover_url,
                schema::extra_song_info::cover_url_small,
                schema::extra_song_info::mbid,
                schema::extra_song_info::musicbrainz_title,
                schema::extra_song_info::musicbrainz_artist,
                schema::extra_song_info::musicbrainz_length,
                schema::extra_song_info::mistag_lock,
                schema::extra_song_info::aliases_artist,
                schema::extra_song_info::aliases_title,
            ))
            .select((
                Song::as_select(),
                sql::<BigInt>("COUNT(scores.song_id) AS score_count"),
                extra_song_info::all_columns.nullable(),
            ))
            .order_by(sql::<BigInt>("score_count DESC"))
            .offset((query.page - 1) * query.page_size)
            .limit(query.page_size)
            .load::<(Song, i64, Option<ExtraSongInfo>)>(&mut conn)
            .await?;

        let songs: Vec<TopSongResponse> = songs_with_extra
            .into_iter()
            .map(|(song, times_played, extra_info)| TopSongResponse {
                song_data: SongResponse { song, extra_info },
                times_played,
            })
            .collect();

        Ok(Json(songs))
    } else {
        let songs: Vec<(Song, i64)> = songs::table
            .left_join(scores::table)
            .select((
                Song::as_select(),
                sql::<BigInt>("COUNT(scores.song_id) AS score_count"),
            ))
            .group_by(songs::id)
            .order_by(sql::<BigInt>("score_count DESC"))
            .offset((query.page - 1) * query.page_size)
            .limit(query.page_size)
            .load::<(Song, i64)>(&mut conn)
            .await?;

        let songs: Vec<TopSongResponse> = songs
            .into_iter()
            .map(|(song, times_played)| TopSongResponse {
                song_data: SongResponse {
                    song,
                    extra_info: None,
                },
                times_played,
            })
            .collect();

        Ok(Json(songs))
    }
}

#[serde_inline_default]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct GetSongScoresParams {
    #[serde_inline_default(true)]
    with_player: bool,
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i64,
    league: Option<League>,
    character: Option<Character>,
    player_id: Option<i32>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ScoreResponse {
    #[serde(flatten)]
    score: Score,
    #[serde(skip_serializing_if = "Option::is_none")]
    player: Option<PlayerPublic>,
}

/// Get song's scores
#[utoipa::path(
    method(get),
    path = "/{id}/scores",
    params(
        ("id" = i32, Path, description = "ID of song to get"),
        ("withPlayer" = Option<bool>, Query, description = "Include player info"),
        ("page" = Option<i64>, Query, description = "Page number", minimum = 1),
        ("pageSize" = Option<i64>, Query, description = "Page size", minimum = 1, maximum = 50),
        ("league" = Option<League>, Query, description = "League to filter by"),
        ("character" = Option<Character>, Query, description = "Character to filter by"),
        ("playerId" = Option<i32>, Query, description = "Player ID to filter by"),
    ),
    responses(
        (status = OK, description = "Success", body = Vec<ScoreResponse>, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip(state), err(Debug))]
async fn get_song_scores(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedQuery(query): ValidatedQuery<GetSongScoresParams>,
) -> Result<Json<Vec<ScoreResponse>>, RouteError> {
    use crate::schema::{players, scores, songs};

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let mut db_query = scores::table
        .filter(scores::song_id.eq(song.id))
        .into_boxed();
    if let Some(league) = query.league {
        db_query = db_query.filter(scores::league.eq(league));
    }
    if let Some(character) = query.character {
        db_query = db_query.filter(scores::vehicle.eq(character));
    }
    if let Some(player_id) = query.player_id {
        db_query = db_query.filter(scores::player_id.eq(player_id));
    }
    db_query = db_query
        .order(scores::score.desc())
        .offset((query.page - 1) * query.page_size)
        .limit(query.page_size);
    if query.with_player {
        let scores_with_player: Vec<(Score, Player)> = db_query
            .inner_join(players::table)
            .select((Score::as_select(), Player::as_select()))
            .load::<(Score, Player)>(&mut conn)
            .await?;

        let scores: Vec<ScoreResponse> = scores_with_player
            .into_iter()
            .map(|(score, player)| ScoreResponse {
                score,
                player: Some(player.into()),
            })
            .collect();

        Ok(Json(scores))
    } else {
        let scores: Vec<Score> = db_query.load::<Score>(&mut conn).await?;

        let scores: Vec<ScoreResponse> = scores
            .into_iter()
            .map(|score| ScoreResponse {
                score,
                player: None,
            })
            .collect();

        Ok(Json(scores))
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct RadioSongResponse {
    song: Song,
    extra_info: Option<ExtraSongInfo>,
    external_url: String,
}

/// Get radio songs
#[utoipa::path(
    method(get),
    path = "/radio",
    params(
        ("withExtraInfo" = Option<bool>, Query, description = "Include extra info")
    ),
    responses(
        (status = OK, description = "Success", body = Vec<RadioSongResponse>, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip(state), err(Debug))]
async fn get_radio_songs(
    State(state): State<AppState>,
    query: Query<GetSongParams>,
) -> Result<Json<Vec<RadioSongResponse>>, RouteError> {
    use crate::schema::{extra_song_info, songs};

    let mut conn = state.db.get().await?;

    let radio_songs = get_radio_songs_util()?;
    match radio_songs {
        Some(radio_songs) => {
            let ids = radio_songs.iter().map(|song| song.id).collect::<Vec<_>>();
            let external_urls = radio_songs
                .iter()
                .map(|song| song.external_url.clone())
                .collect::<Vec<_>>();
            if query.with_extra_info {
                let songs_with_extra: Vec<(Song, Option<ExtraSongInfo>)> = songs::table
                    .filter(songs::id.eq_any(ids))
                    .left_join(extra_song_info::table)
                    .select((Song::as_select(), extra_song_info::all_columns.nullable()))
                    .load::<(Song, Option<ExtraSongInfo>)>(&mut conn)
                    .await?;
                let radio_song_responses: Vec<RadioSongResponse> = songs_with_extra
                    .into_iter()
                    .zip(external_urls)
                    .map(|((song, extra_info), external_url)| RadioSongResponse {
                        song,
                        extra_info,
                        external_url,
                    })
                    .collect();
                Ok(Json(radio_song_responses))
            } else {
                let songs: Vec<Song> = songs::table
                    .filter(songs::id.eq_any(ids))
                    .load::<Song>(&mut conn)
                    .await?;
                let radio_song_responses: Vec<RadioSongResponse> = songs
                    .into_iter()
                    .zip(external_urls)
                    .map(|(song, external_url)| RadioSongResponse {
                        song,
                        extra_info: None,
                        external_url,
                    })
                    .collect();

                Ok(Json(radio_song_responses))
            }
        }
        None => Ok(Json(vec![])),
    }
}

#[serde_inline_default]
#[derive(Deserialize, Validate)]
struct GetSongShoutsParams {
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i32,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i32,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SongShoutsResult {
    shout: Shout,
    author: PlayerPublic,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SongShoutsResponse {
    results: Vec<SongShoutsResult>,
    total: i64,
}

/// Get song's shouts
#[utoipa::path(
    method(get),
    path = "/{id}/shouts",
    params(
        ("id" = i32, Path, description = "ID of song to get"),
        ("page" = i32, Query, description = "Page number", minimum = 1),
        ("pageSize" = i32, Query, description = "Page size", minimum = 1, maximum = 50)
    ),
    responses(
        (status = OK, description = "Success", body = SongResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip(state), err(Debug))]
async fn get_song_shouts(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    ValidatedQuery(query): ValidatedQuery<GetSongScoresParams>,
) -> Result<Json<SongShoutsResponse>, RouteError> {
    use crate::schema::{players, shouts, songs};

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let shouts: Vec<(Shout, Player)> = shouts::table
        .filter(shouts::song_id.eq(song.id))
        .inner_join(players::table)
        .select((shouts::all_columns, players::all_columns))
        .order(shouts::posted_at.desc())
        .offset((query.page - 1) * query.page_size)
        .limit(query.page_size)
        .load::<(Shout, Player)>(&mut conn)
        .await?;

    let results = shouts
        .into_iter()
        .map(|(shout, author)| SongShoutsResult {
            shout,
            author: author.into(),
        })
        .collect();
    let total: i64 = shouts::table
        .filter(shouts::song_id.eq(song.id))
        .count()
        .get_result(&mut conn)
        .await?;

    Ok(Json(SongShoutsResponse { results, total }))
}

/// Manually update song extra info
#[utoipa::path(
    method(put),
    path = "/{id}/extraInfo",
    params(
        ("id" = i32, Path, description = "ID of song to update")
    ),
    responses(
        (status = OK, description = "Success"),
        (status = UNAUTHORIZED, description = "No permission", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    ),
    security(
        ("token_jwt" = [])
    )
)]
#[instrument(skip(state, session), err(Debug))]
async fn update_song_extra_info(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    session: Session,
    Json(extra_info): Json<NewExtraSongInfo>,
) -> Result<(), RouteError> {
    use diesel::insert_into;

    use crate::schema::{extra_song_info, songs};

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    if song.user_can_edit(&session.player, &mut conn).await? {
        let new_extra_song_info = NewExtraSongInfo::new(
            id,
            extra_info.cover_url,
            extra_info.cover_url_small,
            extra_info.mbid,
            extra_info.musicbrainz_title,
            extra_info.musicbrainz_artist,
            extra_info.musicbrainz_length,
            extra_info.aliases_title,
            extra_info.aliases_artist,
        );

        insert_into(extra_song_info::table)
            .values(&new_extra_song_info)
            .on_conflict(extra_song_info::song_id)
            .do_update()
            .set(&new_extra_song_info)
            .execute(&mut conn)
            .await?;
        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}

#[derive(Debug, Deserialize, ToSchema)]
struct MbidRefreshBody {
    recording_mbid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    release_mbid: Option<String>,
}

/// Update song extra info by MBID
#[utoipa::path(
    method(put),
    path = "/{id}/extraInfoByMbid",
    params(
        ("id" = i32, Path, description = "ID of song to update")
    ),
    responses(
        (status = OK, description = "Success"),
        (status = UNAUTHORIZED, description = "No permission", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    ),
    security(
        ("token_jwt" = [])
    )
)]
#[instrument(skip(state, session), err(Debug))]
async fn update_song_extra_info_mbid(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    session: Session,
    Json(payload): Json<MbidRefreshBody>,
) -> Result<(), RouteError> {
    use diesel::insert_into;

    use crate::schema::{extra_song_info, songs};

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    if song.user_can_edit(&session.player, &mut conn).await? {
        let mb_info = musicbrainz::lookup_mbid(
            &payload.recording_mbid,
            payload.release_mbid.as_deref(),
            &state.musicbrainz,
        )
        .await?;

        insert_into(extra_song_info::table)
            .values(&mb_info)
            .on_conflict(extra_song_info::song_id)
            .do_update()
            .set(&mb_info)
            .execute(&mut conn)
            .await?;

        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}
