use axum::{
    extract::{Path, State},
    routing::get,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    models::players::{Player, PlayerPublic},
    util::{errors::RouteError, jwt::Claims},
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .route("/:id", get(get_player))
        .route("/self", get(get_self))
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

async fn get_self(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<PlayerResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let player: Player = players::table
        .find(claims.profile.id)
        .first(&mut conn)
        .await?;

    Ok(Json(PlayerResponse {
        player: player.into(),
    }))
}
