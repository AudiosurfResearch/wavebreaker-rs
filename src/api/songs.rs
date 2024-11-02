use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    models::{extra_song_info::ExtraSongInfo, songs::Song},
    util::errors::RouteError,
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().route("/:id", get(get_song))
}

#[derive(Serialize)]
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

async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    query: Query<GetSongParams>,
) -> Result<Json<SongResponse>, RouteError> {
    use crate::schema::songs;

    let mut conn = state.db.get().await?;

    let song: Song = songs::table.find(id).first(&mut conn).await?;
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
