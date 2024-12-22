use axum::{
    extract::{Path, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::{players::AccountType, scores::Score},
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_score, delete_score))
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ScoreResponse {
    #[serde(flatten)]
    score: Score,
}

/// Get a score by ID
#[utoipa::path(
    method(get),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of score to get"),
    ),
    responses(
        (status = OK, description = "Success", body = ScoreResponse, content_type = "application/json"),
        (status = NOT_FOUND, description = "Score not found", body = SimpleRouteErrorOutput, content_type = "application/json")
    )
)]
async fn get_score(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ScoreResponse>, RouteError> {
    use crate::schema::scores;

    let mut conn = state.db.get().await?;

    let score: Score = scores::table
        .find(id)
        .first::<Score>(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    Ok(Json(ScoreResponse { score }))
}

///Delete a score by ID
#[utoipa::path(
    method(delete),
    path = "/{id}",
    params(
        ("id" = i32, Path, description = "ID of score to delete"),
    ),
    responses(
        (status = OK, description = "Success", content_type = "application/json"),
        (status = NOT_FOUND, description = "Score not found", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = UNAUTHORIZED, description = "Unauthorized", body = SimpleRouteErrorOutput, content_type = "application/json"),
    ),
    security(
        ("token_jwt" = [])
    )
)]
async fn delete_score(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    claims: Claims,
) -> Result<(), RouteError> {
    use crate::schema::scores;

    if claims.profile.account_type == AccountType::Moderator
        || claims.profile.account_type == AccountType::Team
    {
        let mut conn = state.db.get().await?;

        let score: Score = scores::table
            .find(id)
            .first(&mut conn)
            .await
            .optional()?
            .ok_or_else(RouteError::new_not_found)?;
        score.delete(&mut conn, &state.redis).await?;

        Ok(())
    } else {
        Err(RouteError::new_unauthorized())
    }
}
