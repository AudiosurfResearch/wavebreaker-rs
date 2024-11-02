use axum::{
    extract::{Path, State},
    routing::get,
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::{
    models::players::{Player, PlayerPublic},
    util::{errors::RouteError, jwt::Claims},
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_player))
        .route("/self", get(get_self))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    #[serde(flatten)]
    player: PlayerPublic,
}

//todo: move example for PlayerPublic to that struct itself

/// Get a player by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    responses(
        (status = OK, description = "Success",
        body = PlayerResponse, content_type = "application/json",
        example = json!(r#"{
        "id":1,
        "username":"m1nt_",
        "accountType":2,
        "joinedAt":"+002023-05-23T18:56:24.726000000Z",
        "avatarUrl":"https://avatars.steamstatic.com/d72c8ef0f183faf564b9407572d51751794acd15_full.jpg"}"#))
    )
)]
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
