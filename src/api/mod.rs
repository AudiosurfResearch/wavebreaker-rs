use axum::{Json, Router};
use serde::Serialize;
use utoipa::{openapi::OpenApi, OpenApi as OpenApiTrait, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    util::{errors::RouteError, radio::get_radio_songs},
    AppState,
};

mod auth;
mod players;
mod rivals;
mod songs;

#[derive(OpenApiTrait)]
#[openapi(servers((url = "/api")), security(
    (),
    ("token_jwt" = [])
))]
pub struct ApiDoc;

pub fn routes() -> (Router<AppState>, OpenApi) {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        .nest("/songs", songs::routes())
        .nest("/players", players::routes())
        .nest("/auth", auth::routes())
        .nest("/rivals", rivals::routes())
        .split_for_parts()
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct HealthCheck {
    status: &'static str,
    radio_status: String,
}

/// Get health of the API.
#[utoipa::path(
    method(get),
    path = "/healthCheck",
    responses(
        (status = OK, description = "Success", body = HealthCheck, content_type = "application/json")
    )
)]
async fn health_check() -> Result<Json<HealthCheck>, RouteError> {
    let radio_status: String = get_radio_songs().map_or_else(
        |_| "error".to_owned(),
        |radio_songs| {
            radio_songs.map_or_else(
                || "no songs".to_owned(),
                |songs| format!("{} song(s)", songs.len()),
            )
        },
    );

    Ok(Json(HealthCheck {
        status: "ok",
        radio_status,
    }))
}
