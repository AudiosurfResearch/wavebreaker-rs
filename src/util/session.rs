use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use fred::prelude::*;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{IntoRouteError, RouteError};
use crate::{models::players::Player, AppState};

// that's 21 days
const EXPIRE_IN_SECS: i64 = 60 * 60 * 24 * 21;

/// To return the created session after successfully authenticating with Steam OpenID
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
pub struct Session {
    pub player: Player,
}

/// Representation of a player's session as it's stored in Valkey
#[derive(Debug, Serialize, Deserialize)]
struct StoredSession {
    player_id: i32,
}

impl<S> FromRequestParts<S> for Session
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let mut conn = state.db.get().await?;

        // Extract the token from the authorization header
        let bearer = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .http_status_error(StatusCode::UNAUTHORIZED)?;

        // Decode the user data
        let session = verify_token(bearer.token(), &mut conn, &state.redis)
            .await
            .http_error("Invalid token", StatusCode::UNAUTHORIZED)?;

        Ok(session)
    }
}

/// Verify a session token, returning the `Session` and resetting its expiry. This can fail if something goes wrong with the database or Valkey.
pub async fn verify_token(
    token: &str,
    conn: &mut AsyncPgConnection,
    redis: &Pool,
) -> anyhow::Result<Session> {
    use crate::schema::players::dsl::*;

    let stored_session_json: Value = redis.get(format!("session:{}", token)).await?;
    // refresh expiry
    let _: () = redis
        .expire(format!("session:{}", token), EXPIRE_IN_SECS, None)
        .await?;
    let stored_session: StoredSession = serde_json::from_value(stored_session_json)?;

    // Get info needed to translate the session info at rest to a regular Session
    let player = players.find(stored_session.player_id).first(conn).await?;

    Ok(Session { player })
}

/// Create a session and return the token. This can fail if there's something wrong with Valkey.
pub async fn create_session(player: &Player, redis: &Pool) -> anyhow::Result<String> {
    let token: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();

    let session = StoredSession {
        player_id: player.id,
    };

    let serialized = serde_json::to_string(&session)?;
    let _: () = redis
        .set(
            format!("session:{}", token),
            serialized,
            Some(Expiration::EX(EXPIRE_IN_SECS)),
            None,
            false,
        )
        .await?;

    Ok(token)
}
