use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Json, Router,
};
use url::Url;

use crate::{
    util::{
        errors::RouteError,
        steam_openid::{get_redirect_url, verify_return, VerifyForm},
    },
    AppState,
};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/return", get(auth_return))
        .route("/login", get(auth_login))
}

async fn auth_login(State(state): State<AppState>) -> Result<Redirect, RouteError> {
    Ok(Redirect::permanent(&get_redirect_url(
        &state.config.external.steam_realm,
        &state.config.external.steam_return_path,
    )?))
}

async fn auth_return(
    State(state): State<AppState>,
    Query(mut query): Query<VerifyForm>,
) -> Result<Json<()>, RouteError> {
    let steamid64 = verify_return(
        &Url::parse(&state.config.external.steam_realm)?
            .join(&state.config.external.steam_return_path)?
            .to_string(),
        &mut query,
    )
    .await?;

    unimplemented!();
}
