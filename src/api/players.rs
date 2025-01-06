use axum::{
    extract::{Path, Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use fred::prelude::*;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use validator::Validate;

use crate::{
    models::players::{FavoriteCharacter, Player, PlayerPublic},
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_player))
        .routes(routes!(get_self))
        .routes(routes!(get_player_rankings))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PlayerResponse {
    #[serde(flatten)]
    player: PlayerPublic,
    #[serde(skip_serializing_if = "Option::is_none")]
    stats: Option<PlayerStats>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PlayerStats {
    rank: i32,
    skill_points: i32,
    total_plays: i32,
    favorite_character: Option<FavoriteCharacter>,
}

#[serde_inline_default]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPlayerParams {
    #[serde_inline_default(false)]
    with_stats: bool,
}

/// Get player by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of player to get"),
        ("withStats" = Option<bool>, Query, description = "Include player's stats")
    ),
    responses(
        (status = OK, description = "Success", body = PlayerResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Player not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
async fn get_player(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    query: Query<GetPlayerParams>,
) -> Result<Json<PlayerResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let player: Player = players::table
        .find(id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let stats = if query.with_stats {
        Some(PlayerStats {
            rank: player.get_rank(&state.redis).await?,
            skill_points: player.get_skill_points(&state.redis).await?,
            total_plays: player.get_total_plays(&mut conn).await?,
            favorite_character: player.get_favorite_character(&mut conn).await?,
        })
    } else {
        None
    };

    Ok(Json(PlayerResponse {
        player: player.into(),
        stats,
    }))
}

/// Get the player that is currently logged in
#[utoipa::path(
    method(get),
    path = "/self",
    params(
        ("includeStats" = Option<bool>, Query, description = "Include player's stats")
    ),
    responses(
        (status = OK, description = "Success", body = PlayerPublic, content_type = "application/json"),
        (status = UNAUTHORIZED, description = "Not logged in or invalid token", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    ),
    security(
        ("token_jwt" = [])
    )
)]
async fn get_self(
    State(state): State<AppState>,
    claims: Claims,
    query: Query<GetPlayerParams>,
) -> Result<Json<PlayerResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let player: Player = players::table
        .find(claims.profile.id)
        .first(&mut conn)
        .await?;

    let stats = if query.with_stats {
        Some(PlayerStats {
            rank: player.get_rank(&state.redis).await?,
            skill_points: player.get_skill_points(&state.redis).await?,
            total_plays: player.get_total_plays(&mut conn).await?,
            favorite_character: player.get_favorite_character(&mut conn).await?,
        })
    } else {
        None
    };
    Ok(Json(PlayerResponse {
        player: player.into(),
        stats,
    }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PlayerRankingResponse {
    results: Vec<PlayerWithRanking>,
    total: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct PlayerWithRanking {
    player: PlayerPublic,
    skill_points: i32,
}

#[serde_inline_default]
#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct GetRankingsParams {
    #[validate(range(min = 1))]
    #[serde_inline_default(1)]
    page: i64,
    #[validate(range(min = 1, max = 50))]
    #[serde_inline_default(10)]
    page_size: i64,
}

/// Get player rankings
#[utoipa::path(
    method(get),
    path = "/rankings",
    params(
        ("page" = Option<i64>, Query, description = "Page number", minimum = 1),
        ("pageSize" = Option<i64>, Query, description = "Page size", minimum = 1, maximum = 50),
    ),
    responses(
        (status = OK, description = "Success", body = PlayerRankingResponse, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
async fn get_player_rankings(
    State(state): State<AppState>,
    query: Query<GetRankingsParams>,
) -> Result<Json<PlayerRankingResponse>, RouteError> {
    use crate::schema::players;

    let mut conn = state.db.get().await?;

    let leaderboard: Vec<i32> = state
        .redis
        .zrevrange(
            "leaderboard",
            (query.page - 1) * query.page_size,
            query.page * query.page_size - 1,
            false,
        )
        .await?;

    let mut players = players::table
        .filter(players::id.eq_any(&leaderboard))
        .load::<Player>(&mut conn)
        .await?;
    players.sort_by_key(|p| leaderboard.iter().position(|&id| id == p.id).unwrap());

    let mut results: Vec<PlayerWithRanking> = vec![];
    for player in players {
        results.push(PlayerWithRanking {
            player: player.clone().into(),
            skill_points: player.get_skill_points(&state.redis).await?,
        });
    }

    Ok(Json(PlayerRankingResponse {
        results,
        total: state.redis.zcard("leaderboard").await?,
    }))
}
