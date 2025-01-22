use axum::{
    extract::{
        rejection::{FormRejection, QueryRejection},
        Form, FromRequest, FromRequestParts, Query, Request,
    },
    http::request::Parts,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::errors::RouteError;

// ValidatedForm is from https://github.com/tokio-rs/axum/blob/main/examples/validator/src/main.rs

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = RouteError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate().map_err(|e| {
            let message = format!("Form validation error: [{e}]").replace('\n', ", ");
            RouteError::new_bad_request().set_public_error_message(&message)
        })?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
{
    type Rejection = RouteError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
        value.validate().map_err(|e| {
            let message = format!("Query validation error: [{e}]").replace('\n', ", ");
            RouteError::new_bad_request().set_public_error_message(&message)
        })?;
        Ok(Self(value))
    }
}
