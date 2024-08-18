use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::{models::songs::Song, util::errors::RouteError, AppState};

pub fn song_routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_song))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SongResponse {
    #[serde(flatten)]
    song: Song,
}

async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SongResponse>, RouteError> {
    use crate::schema::songs;

    let mut conn = state.db.get().await?;

    let song = songs::table.find(id).first(&mut conn).await?;

    Ok(Json(SongResponse { song }))
}
