use axum::{
    extract::{Path, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::players::{Player, PlayerPublic},
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    //hack because it'll panic otherwise, see https://github.com/juhaku/utoipa/issues/1183
    OpenApiRouter::new()
        .routes(routes!(get_player))
        .routes(routes!(get_self))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    #[serde(flatten)]
    player: PlayerPublic,
}

/// Get a player by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of player to get"),
    ),
    responses(
        (status = OK, description = "Success",
        body = PlayerPublic, content_type = "application/json",
        example = json!(r#"{
        "id":1,
        "username":"m1nt_",
        "accountType":2,
        "joinedAt":"+002023-05-23T18:56:24.726000000Z",
        "avatarUrl":"https://avatars.steamstatic.com/d72c8ef0f183faf564b9407572d51751794acd15_full.jpg"}"#)),
        (status = NOT_FOUND, description = "Player not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_player(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PlayerResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let player: Player = players::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    Ok(Json(PlayerResponse {
        player: player.into(),
    }))
}

/// Get the player that is currently logged in
#[utoipa::path(
    method(get),
    path = "/self",
    responses(
        (status = OK, description = "Success",
        body = PlayerPublic, content_type = "application/json",
        example = json!(r#"{
        "id":1,
        "username":"m1nt_",
        "accountType":2,
        "joinedAt":"+002023-05-23T18:56:24.726000000Z",
        "avatarUrl":"https://avatars.steamstatic.com/d72c8ef0f183faf564b9407572d51751794acd15_full.jpg"}"#)),
        (status = UNAUTHORIZED, description = "Not logged in or invalid token", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
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
