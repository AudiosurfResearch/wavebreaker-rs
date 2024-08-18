use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::{
    util::{errors::RouteError, radio::get_radio_songs},
    AppState,
};

mod auth;
mod players;
mod songs;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthCheck", get(health_check))
        .nest("/songs", songs::routes())
        .nest("/players", players::routes())
        .nest("/auth", auth::routes())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthCheck {
    status: &'static str,
    radio_status: String,
}

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
