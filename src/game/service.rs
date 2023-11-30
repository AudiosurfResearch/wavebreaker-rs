use super::{
    gameplay::{custom_news, fetch_song_id},
    user::{steam_login, steam_sync, update_location},
};
use actix_web::web;

pub fn steamlogin_config(cfg: &mut web::ServiceConfig) {
    cfg.service(steam_login)
        .service(steam_sync)
        .service(update_location)
        .service(fetch_song_id);
}

pub fn game_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/as_steamlogin").configure(steamlogin_config));
    cfg.service(web::scope("//as_steamlogin").service(custom_news)); //Dylan'd. Note the double slash!
}
