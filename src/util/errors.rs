use std::fmt::{Display, Formatter, Result as FmtResult};

use actix_web::{error, http::StatusCode, HttpResponse};
use log::error;
use serde::Serialize;
use serde_json::{json, to_string_pretty};

#[derive(Debug, Serialize)]
struct Error {
    error: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).expect("The 'Error' type should always be serializable to a string"))
    }
}

//shamelessly stolen from https://www.reddit.com/r/rust/comments/ozc0m8/an_actixanyhow_compatible_error_helper_i_found/
pub trait IntoHttpError<T> {
    fn http_error(
        self,
        message: &str,
        status_code: StatusCode,
    ) -> core::result::Result<T, actix_web::Error>;

    fn http_internal_error(self, message: &str) -> core::result::Result<T, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        self.http_error(message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn http_internal_error_default(self) -> core::result::Result<T, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        self.http_error("An internal error occurred.", StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl<T, E: std::fmt::Debug> IntoHttpError<T> for core::result::Result<T, E> {
    fn http_error(
        self,
        message: &str,
        status_code: StatusCode,
    ) -> core::result::Result<T, actix_web::Error> {
        self.map_err(|err| {
            error!("http_error: {:?}", err);
            let err_json = json!({"error": message});
            let response = HttpResponse::build(status_code).json(err_json);
            error::InternalError::from_response(message.to_string(), response).into()
        })
    }
}
