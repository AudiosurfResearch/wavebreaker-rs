use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use fred::{
    prelude::{KeysInterface, Pool},
    types::Expiration,
};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::errors::{IntoRouteError, RouteError};
use crate::{models::players::Player, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub player: Player,
}

impl<S> FromRequestParts<S> for Session
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        // Extract the token from the authorization header, if it's not there, try the cookie
        let bearer = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .http_status_error(StatusCode::UNAUTHORIZED)?;

        // Decode the user data
        let token_data = verify_token(bearer.token(), &state.redis)
            .await
            .http_error("Invalid token", StatusCode::UNAUTHORIZED)?;

        todo!()
    }
}

async fn verify_token(token: &str, redis: &Pool) -> anyhow::Result<Session> {
    todo!()
}

async fn create_session(player: &Player, redis: &Pool) -> anyhow::Result<String> {
    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    let session = Session {
        player: player.clone(),
    };

    let serialized = serde_json::to_string(&session)?;
    let _: () = redis
        .set(
            format!("session:{}", token),
            serialized,
            Some(Expiration::EX(60 * 60 * 24 * 30)),
            None,
            false,
        )
        .await?;

    Ok(token)
}
