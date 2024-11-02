use axum::{Json, Router};
use serde::Serialize;
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        OpenApi,
    },
    Modify, OpenApi as OpenApiTrait, ToSchema,
};
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
#[openapi(
    modifiers(&SecurityAddon),
    servers((url = "/api")), security(
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
            )
        }
    }
}

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

/// Get health of the server
#[utoipa::path(
    method(get),
    path = "/healthCheck",
    responses(
        (status = OK, description = "Success",
        body = HealthCheck, content_type = "application/json",
        examples = (json!(r#"{ "status": "ok", "radioStatus": "2 song(s)" }"#))),
        (status = OK, description = "Server works, but Radio is broken",
        body = HealthCheck, content_type = "application/json",
        example = json!(r#"{ "status": "ok", "radioStatus": "error" }"#)),
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
