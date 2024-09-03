use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;

use crate::{
    models::players::{Player, PlayerPublic},
    util::errors::RouteError,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_player))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    #[serde(flatten)]
    player: PlayerPublic,
}

async fn get_player(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PlayerResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let player: Player = players::table.find(id).first(&mut conn).await?;

    Ok(Json(PlayerResponse {
        player: player.into(),
    }))
}
