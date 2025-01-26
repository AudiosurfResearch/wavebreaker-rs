// shamelessly stolen from https://www.shuttle.rs/blog/2024/02/21/using-jwt-auth-rust

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    extract::CookieJar,
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};

use super::errors::{IntoRouteError, RouteError};
use crate::{models::players::Player, AppState};
#[derive(Clone)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub profile: Player,
    pub exp: i64,
}

impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        // Extract the token from the authorization header, if it's not there, try the cookie
        let token = match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(bearer) => bearer.token().to_owned(),
            Err(_) => {
                let jar = parts
                    .extract::<CookieJar>()
                    .await
                    .http_status_error(StatusCode::UNAUTHORIZED)?;

                jar.get("authorization")
                    .map(|cookie| cookie.value().to_owned().replace("Bearer ", ""))
                    .ok_or_else(|| anyhow::anyhow!("No token found"))
                    .http_error("No token found", StatusCode::UNAUTHORIZED)?
            }
        };

        // Decode the user data
        let token_data = decode::<Self>(&token, &state.jwt_keys.decoding, &Validation::default())
            .http_error("Invalid token", StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}
