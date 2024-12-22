use axum::{
    extract::{Path, Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    models::{
        extra_song_info::{self, ExtraSongInfo},
        songs::Song,
    },
    schema,
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
        validator::ValidatedQuery,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_song, delete_song))
        .routes(routes!(get_top_songs))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SongResponse {
    #[serde(flatten)]
    song: Song,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_info: Option<ExtraSongInfo>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSongParams {
    #[serde(default)] // default to false
    with_extra_info: bool,
}

/// Get a song by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of song to get"),
        ("withExtraInfo" = bool, Query, description = "Include extra info")
    ),
    responses(
        (status = OK, description = "Success", body = SongResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
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

/// Delete a song
#[utoipa::path(
    method(delete),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of song to get")
    ),
    responses(
        (status = OK, description = "Success"),
        (status = UNAUTHORIZED, description = "No permission", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = NOT_FOUND, description = "Song not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    ),
    security(
        ("token_jwt" = [])
    )
)]
async fn delete_song(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    claims: Claims,
) -> Result<(), RouteError> {
    use crate::schema::songs;

    let mut conn = state.db.get().await?;

    let song: Song = songs::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    if song.user_can_delete(claims.profile.id, &mut conn).await? {
        diesel::delete(songs::table.filter(songs::id.eq(id)))
            .execute(&mut conn)
            .await?;

        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}

#[serde_inline_default]
#[derive(Deserialize, Validate)]
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
    path = "/top",
    params(
        ("withExtraInfo" = bool, Query, description = "Include extra info"),
        ("page" = i64, Query, description = "Page number", minimum = 1),
        ("pageSize" = i64, Query, description = "Page size", minimum = 1, maximum = 50)
    ),
    responses(
        (status = OK, description = "Success", body = Vec<TopSongResponse>, content_type = "application/json"),
        (status = BAD_REQUEST, description = "Invalid query parameters", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
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
