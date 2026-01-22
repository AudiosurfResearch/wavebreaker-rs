use axum::{extract::State, Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use tracing::instrument;
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        OpenApi,
    },
    Modify, OpenApi as OpenApiTrait, ToSchema,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    util::{
        errors::{RouteError, SimpleRouteErrorOutput},
        query::SortType,
    },
    AppState,
};

mod auth;
mod players;
mod rivals;
mod scores;
mod shouts;
mod songs;

#[derive(OpenApiTrait)]
#[openapi(
    components(schemas(SortType)),
    modifiers(&SecurityAddon),
    servers((url = "/api"), (url = "/rust/api")), security(
    (),
    ("token_jwt" = [])
))]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token_jwt",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub fn routes() -> (Router<AppState>, OpenApi) {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(stats))
        .nest("/songs", songs::routes())
        .nest("/players", players::routes())
        .nest("/auth", auth::routes())
        .nest("/rivals", rivals::routes())
        .nest("/scores", scores::routes())
        .nest("/shouts", shouts::routes())
        .split_for_parts()
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct ServerStats {
    user_count: i64,
    song_count: i64,
    score_count: i64,
    search_supported: bool,
}

/// Get server stats
#[utoipa::path(
    method(get),
    path = "/stats",
    responses(
        (status = OK, description = "Success", body = ServerStats, content_type = "application/json"),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip_all, err(Debug))]
async fn stats(State(state): State<AppState>) -> Result<Json<ServerStats>, RouteError> {
    use crate::schema::{players, scores, songs};

    let mut conn = state.db.get().await?;

    let user_count: i64 = players::table.count().get_result(&mut conn).await?;
    let song_count: i64 = songs::table.count().get_result(&mut conn).await?;
    let score_count: i64 = scores::table.count().get_result(&mut conn).await?;

    Ok(Json(ServerStats {
        user_count,
        song_count,
        score_count,
        search_supported: state.meilisearch.is_some(),
    }))
}
