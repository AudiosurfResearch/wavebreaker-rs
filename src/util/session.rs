use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
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
        //let token_data =
        //    verify_session(bearer.token()).http_error("Invalid token", StatusCode::UNAUTHORIZED)?;

        todo!()
    }
}

fn verify_session(token: &str) -> () {
    todo!()
}
