use anyhow::anyhow;
use axum::{
    extract::{RawQuery, State},
    http::{response, StatusCode},
    response::Redirect,
    routing::get,
    Json, Router,
};
use axum_extra::{extract::CookieJar, headers::Cookie};
use diesel_async::RunQueryDsl;
use jsonwebtoken::{encode, Header};
use tracing::info;

use crate::{
    models::players::Player,
    util::{
        errors::{IntoRouteError, RouteError},
        jwt::{AuthBody, Claims},
    },
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/return", get(auth_return))
        .route("/login", get(auth_login))
}

async fn auth_login(State(state): State<AppState>) -> Result<Redirect, RouteError> {
    Ok(Redirect::permanent(state.steam_openid.get_redirect_url()))
}

async fn auth_return(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
) -> Result<(Json<AuthBody>), RouteError> {
    let steamid64 = state
        .steam_openid
        .verify(&query.ok_or_else(|| anyhow!("No query string to verify!"))?)
        .await
        .map_err(|e| anyhow!("OpenID verification failed: {e:?}"))
        .http_error(
            "Couldn't verify Steam OpenID return",
            StatusCode::BAD_REQUEST,
        )?;

    let mut conn = state.db.get().await?;

    let player = Player::find_by_steam_id(steamid64.into())
        .first(&mut conn)
        .await
        .http_error("Profile not found", StatusCode::NOT_FOUND)?;

    info!("Player {} logged in via Steam OpenID", player.id);

    // expiry in 7 days
    let exp = time::OffsetDateTime::now_utc().unix_timestamp() + 60 * 60 * 24 * 7;

    let claims = Claims {
        profile: player,
        exp,
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &state.jwt_keys.encoding)
        .http_internal_error("Failed to create token")?;

    Ok(Json(AuthBody::new(token)))
}
