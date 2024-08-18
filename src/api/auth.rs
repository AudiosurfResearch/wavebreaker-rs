use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use crate::{util::errors::RouteError, AppState};

pub fn auth_routes() -> Router<AppState> {
    Router::new().route("/return", get(auth_return))
}

async fn auth_return(
    State(state): State<AppState>,
) -> Result<Json<()>, RouteError> {
    unimplemented!();
}
