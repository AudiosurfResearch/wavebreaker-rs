use actix_web::{web, HttpResponse, Responder};
use quick_xml::se;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SteamLoginRequest {
    steamusername: String,
    snum: i32,
    s64: i64,
    ticket: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "RESULT")]
struct SteamLoginResponse {
    #[serde(rename = "@status")]
    status: String,
    userid: i64,
    username: String,
    locationid: i32,
    steamid: i32,
}

pub async fn steam_login(web::Form(form): web::Form<SteamLoginRequest>) -> impl Responder {
    let response = SteamLoginResponse {
        status: "allgood".to_owned(),
        userid: 1,
        username: form.steamusername,
        locationid: 1,
        steamid: form.snum,
    };

    HttpResponse::Ok().body(se::to_string(&response).unwrap())
}
