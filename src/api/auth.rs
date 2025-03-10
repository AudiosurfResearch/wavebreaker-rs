use anyhow::anyhow;
use axum::{
    extract::{RawQuery, State},
    http::StatusCode,
    response::Redirect,
    Json,
};
use diesel_async::RunQueryDsl;
use jsonwebtoken::{encode, Header};
use tracing::{info, instrument};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    models::players::Player,
    util::{
        errors::{IntoRouteError, RouteError, SimpleRouteErrorOutput},
        jwt::AuthBody, session::create_session,
    },
    AppState,
};

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(auth_login))
        .routes(routes!(auth_return))
}

/// Start login
#[utoipa::path(
    method(get),
    path = "/login",
    responses(
        (status = 308, description = "Redirect to Steam", body = ())
    )
)]
#[instrument(skip_all, err(Debug))]
async fn auth_login(State(state): State<AppState>) -> Result<Redirect, RouteError> {
    Ok(Redirect::permanent(state.steam_openid.get_redirect_url()))
}

/// Wrapper for jwt crate's AuthBody because it doesn't implement ToSchema
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct AuthBodySchema {
    access_token: String,
    token_type: String,
}

/// Return after Steam login
#[utoipa::path(
    method(get),
    path = "/return",
    responses(
        (status = OK, description = "Success", body = AuthBodySchema),
        (status = BAD_REQUEST, description = "OpenID verification failed", body = SimpleRouteErrorOutput),
        (status = NOT_FOUND, description = "Profile not found", body = SimpleRouteErrorOutput),
        (status = INTERNAL_SERVER_ERROR, description = "Miscellaneous error", body = SimpleRouteErrorOutput)
    )
)]
#[instrument(skip_all, err(Debug), fields(player))]
async fn auth_return(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
) -> Result<Json<AuthBody>, RouteError> {
    let steamid64 = state
        .steam_openid
        .verify(
            &query
                .ok_or_else(|| anyhow!("No query string to verify!"))
                .http_error("Query string is empty", StatusCode::BAD_REQUEST)?,
        )
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
    tracing::Span::current().record("player", player.id);

    info!("Player {} logged in via Steam OpenID", player.id);

    // Create the session token
    let token = create_session(&player, &state.redis).await
        .http_internal_error("Failed to create token")?;

    Ok(Json(AuthBody::new(token)))
}
