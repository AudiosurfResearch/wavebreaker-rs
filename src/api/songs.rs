use axum::{
    extract::{Path, Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::{extra_song_info::ExtraSongInfo, songs::Song},
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_song, delete_song))
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
        (status = OK, description = "Success", body = ()),
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
