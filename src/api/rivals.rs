use axum::{extract::State, routing::get, Json};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    models::{players::Player, rivalries::RivalryView},
    util::{errors::RouteError, jwt::Claims},
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().route("/own", get(get_own_rivals))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RivalryResponse {
    rivalries: Vec<RivalryView>,
}

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
