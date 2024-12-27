use serde::Deserialize;
use utoipa::ToSchema;

/// General type used to specify sort order
#[derive(Debug, Deserialize, ToSchema)]
pub enum SortType {
    #[serde(rename = "asc")]
    #[schema(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    #[schema(rename = "asc")]
    Desc,
}