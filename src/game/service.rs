use super::user::{steam_login, steam_sync};
use actix_web::web;

pub fn steamlogin_config(cfg: &mut web::ServiceConfig) {
    //auth
    cfg.service(steam_login).service(steam_sync);
}

pub fn game_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/as_steamlogin").configure(steamlogin_config));
}
