use crate::error::{IntoHttp, RouteError};
use quick_xml::se;
use rocket::response::content::{self, RawXml};
use serde::Serialize;

pub trait XmlSerializableResponse {
    fn to_xml_response(&self) -> Result<RawXml<String>, RouteError>;
}

impl<T> XmlSerializableResponse for T
where
    T: Serialize,
{
    fn to_xml_response(&self) -> Result<RawXml<String>, RouteError> {
        let response_string = se::to_string(self).http_internal_error_default()?;
        Ok(content::RawXml(response_string))
    }
}
