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

#[derive(Debug, Serialize)]
pub struct Error {
    message: String,
    status: Status,
}

impl Display for Error {
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
    fn http_error(self, message: &str, status_code: Status) -> core::result::Result<T, Error>;

    fn http_internal_error(self, message: &str) -> core::result::Result<T, Error>
    where
        Self: std::marker::Sized,
    {
        self.http_error(message, Status::InternalServerError)
    }

    fn http_internal_error_default(self) -> core::result::Result<T, Error>
    where
        Self: std::marker::Sized,
    {
        self.http_error("An internal error occurred.", Status::InternalServerError)
    }
}

impl<T, E: std::fmt::Debug> IntoHttp<T> for core::result::Result<T, E> {
    fn http_error(self, message: &str, status: Status) -> core::result::Result<T, Error> {
        self.map_err(|err| {
            error!("http_error: {:?}", err);
            Error {
                message: message.to_owned(),
                status,
            }
        })
    }
}

impl<'r> Responder<'r, 'static> for Error {
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
