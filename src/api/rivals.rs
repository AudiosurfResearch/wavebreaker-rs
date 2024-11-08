use axum::{extract::State, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::{players::Player, rivalries::RivalryView},
    util::{errors::RouteError, jwt::Claims},
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_own_rivals))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct RivalryResponse {
    rivalries: Vec<RivalryView>,
}

/// Get own rivals
#[utoipa::path(
    method(get),
    path = "/self",
    responses(
        (status = OK, description = "Success", body = RivalryResponse, content_type = "application/json")
    ),
    security(
        ("token_jwt" = [])
    ))
]
async fn get_own_rivals(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<RivalryResponse>, RouteError> {
    use crate::schema::players::dsl::*;

    let mut conn = state.db.get().await?;

    let player: Player = players.find(claims.profile.id).first(&mut conn).await?;
    let rivalries: Vec<RivalryView> = player.get_rivalry_views(&mut conn).await?;

    Ok(Json(RivalryResponse { rivalries }))
}
