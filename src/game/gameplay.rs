use crate::util::errors::IntoHttpError;
use actix_web::{post, web, Result};
use quick_xml::se;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SongIdRequest {
    artist: String,
    song: String,
    uid: u64,
    league: u8,
}

//response usually has three root tags and two with THE SAME NAME but I've elected to not care
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SongIdResponse {
    #[serde(rename = "@status")]
    status: String,
    #[serde(rename = "songid")]
    song_id: u64,
}

#[post("/game_fetchsongid_unicode.php")]
pub async fn fetch_song_id(
    web::Form(form): web::Form<SongIdRequest>,
) -> Result<String, actix_web::Error> {
    se::to_string(&SongIdResponse {
        status: "allgood".to_owned(),
        song_id: 143,
    })
    .http_internal_error_default()
}

#[derive(Deserialize)]
pub struct CustomNewsRequest {
    steamusername: String,
    snum: u32,
    s64: u64,
    artist: String,
    song: String,
    vehicle: u8,
    userid: u64,
    league: u8,
    songid: u64,
    /// Song length in seconds
    songlength: u64,
    ticket: String,
}

//response usually has three root tags and two with THE SAME NAME but I've elected to not care
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULTS")]
struct CustomNewsResponse {
    #[serde(rename = "TEXT")]
    text: String,
}

#[post("/game_CustomNews.php")]
pub async fn custom_news(
    web::Form(form): web::Form<CustomNewsRequest>,
) -> Result<String, actix_web::Error> {
    se::to_string(&CustomNewsResponse {
        text: format!(
            "Welcome to wavebreaker-rs, {}.\nThis ride is gonna be bumpy!",
            form.steamusername
        ),
    }).http_internal_error_default()
}
