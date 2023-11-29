#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    //clippy::expect_used,
    clippy::correctness,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::cognitive_complexity,
    clippy::double_parens,
    clippy::len_zero,
    clippy::question_mark,
    clippy::suspicious,
    clippy::todo,
    //clippy::all  //for extra anger
)]

mod game;
mod util;

use crate::game::service::game_config;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::fs;
use steam_rs::Steam;

#[derive(Deserialize)]
struct Config {
    main: Main,
    external: External,
}

#[derive(Deserialize)]
struct Main {
    ip: String,
    port: u16,
}

#[derive(Deserialize)]
struct External {
    steam_key: String,
}

pub struct AppGlobals {
    steam_api: Steam,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Read config
    let contents = fs::read_to_string("config.toml")?;
    let wavebreaker_config: Config =
        toml::from_str(&contents).expect("The config should be in a valid format");

    HttpServer::new(move || {
        App::new()
            //Initialize global app stuff, like the Steam API
            .app_data(web::Data::new(AppGlobals {
                steam_api: Steam::new(&wavebreaker_config.external.steam_key),
            }))
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Ok().body("I've never felt so incredible going my way!")
                }),
            )
            .configure(game_config)
    })
    .bind((wavebreaker_config.main.ip, wavebreaker_config.main.port))?
    .run()
    .await
}
