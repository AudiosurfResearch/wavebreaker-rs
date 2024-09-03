use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    routing::get,
    Json, Router,
};
use diesel_async::RunQueryDsl;
use jsonwebtoken::{encode, Header};
use time::{Duration, OffsetDateTime};
use url::Url;

use crate::{
    models::players::Player,
    util::{
        errors::{IntoRouteError, RouteError},
        jwt::{AuthBody, Claims},
        steam_openid::{get_redirect_url, verify_return, VerifyForm},
    },
    AppState,
};

pub fn routes() -> Router<AppState> {
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
) -> Result<Json<AuthBody>, RouteError> {
    let steamid64 = verify_return(
        Url::parse(&state.config.external.steam_realm)?
            .join(&state.config.external.steam_return_path)?
            .as_ref(),
        &mut query,
    )
    .await
    .http_error(
        "Couldn't verify Steam OpenID return",
        StatusCode::BAD_REQUEST,
    )?;

    let mut conn = state.db.get().await?;

    let player = Player::find_by_steam_id(steamid64.into())
        .first(&mut conn)
        .await
        .http_error("Profile not found", StatusCode::NOT_FOUND)?;

    let exp = (OffsetDateTime::now_utc() + Duration::days(31)).unix_timestamp();
    let claims = Claims {
        profile: player,
        exp,
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &state.jwt_keys.encoding)?;

    // Send the token
    Ok(Json(AuthBody::new(token)))
}
