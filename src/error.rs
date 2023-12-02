use std::fmt::{Display, Formatter, Result as FmtResult};

use log::error;
use rocket::{
    http::Status,
    response::{self, Responder},
    serde::json::Json,
    Request, Response,
};
use serde::Serialize;
use serde_json::{json, to_string_pretty};
use thiserror::Error;

/// Error type used for Wavebreaker's routes, supplying a message and status to return in the error response.
#[derive(Debug, Serialize, Error)]
pub struct RouteError {
    message: String,
    status: Status,
}

impl Display for RouteError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "{}",
            to_string_pretty(self)
                .expect("The 'Error' type should always be serializable to a string")
        )
    }
}

//shamelessly stolen from https://www.reddit.com/r/rust/comments/ozc0m8/an_actixanyhow_compatible_error_helper_i_found/
pub trait IntoHttp<T> {
    /// Converts any error type to one that causes a response with a custom status and message.
    ///
    /// Useful for any kind of user-facing error.
    fn http_error(self, message: &str, status_code: Status) -> Result<T, RouteError>;

    /// Converts any error type to one that causes a response with status 500 and a custom message.
    ///
    /// Useful for explaining internal errors to the user, like Steam auth failing, without giving them the full details.
    fn http_internal_error(self, message: &str) -> Result<T, RouteError>
    where
        Self: std::marker::Sized,
    {
        self.http_error(message, Status::InternalServerError)
    }

    /// Converts any error type to one that causes a response with status 500 and a generic error message.
    ///
    /// Useful for handling internal errors that shouldn't be exposed to users, like database errors, etc.
    fn http_internal_error_default(self) -> Result<T, RouteError>
    where
        Self: std::marker::Sized,
    {
        self.http_error("An internal error occurred.", Status::InternalServerError)
    }
}

impl<T, E> IntoHttp<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn http_error(self, message: &str, status: Status) -> Result<T, RouteError> {
        self.map_err(|err| {
            let boxed_err = anyhow::Error::new(err);
            error!("Route error: {:?}", boxed_err);
            RouteError {
                message: message.to_owned(),
                status,
            }
        })
    }
}

impl<'r> Responder<'r, 'static> for RouteError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(
            Json(json!({"message": self.message}))
                .respond_to(req)
                .expect("Error handler JSON should never fail to serialize!"),
        )
        .status(self.status)
        .ok()
    }
}
