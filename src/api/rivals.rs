use axum::{
    extract::{Query, State},
    Json,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::{players::Player, rivalries::{NewRivalry, Rivalry, RivalryView}},
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        jwt::Claims,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_own_rivals)).routes(routes!(add_rival))
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddRivalParams {
    rival_id: i32,
}

/// Get own rivals
#[utoipa::path(
    method(post),
    path = "/add",
    params(
        ("rivalId" = i32, Query, description = "ID of player to add as rival")
    ),
    responses(
        (status = OK, description = "Success"),
        (status = NOT_FOUND, description = "Couldn't find player to rival", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = BAD_REQUEST, description = "Invalid parameters", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = CONFLICT, description = "Rivalry already exists", body = SimpleRouteErrorOutput, content_type = "application/json"),
        (status = UNAUTHORIZED, description = "Unauthorized", body = SimpleRouteErrorOutput, content_type = "application/json")
    ),
    security(
        ("token_jwt" = [])
    ))
]
async fn add_rival(
    State(state): State<AppState>,
    query: Query<AddRivalParams>,
    claims: Claims,
) -> Result<Json<RivalryView>, RouteError> {
    use crate::schema::players::dsl::*;
    use crate::schema::rivalries::dsl::*;

    let mut conn = state.db.get().await?;

    let player: Player = players.find(claims.profile.id).first(&mut conn).await?;
    let rival: Player = players
        .find(query.rival_id)
        .first(&mut conn)
        .await
        .optional()?
        .ok_or_else(RouteError::new_not_found)?;

    let rivalry = rivalries
        .filter(rival_id.eq(rival.id))
        .filter(challenger_id.eq(player.id))
        .first::<Rivalry>(&mut conn)
        .await
        .optional()?;

    if rivalry.is_some() {
        Err(RouteError::new_conflict().set_public_error_message("Rivalry already exists"))
    }
    else {
        let new_rivalry = NewRivalry {
            challenger_id: player.id,
            rival_id: rival.id,
        }.create(&mut conn).await?;

        Ok(Json(RivalryView::from_rivalry(new_rivalry, &mut conn).await?))
    }
}
