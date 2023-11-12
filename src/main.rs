#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used,
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

use crate::game::service::game_config;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::fs;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //Read config
    let filename = "config.toml";
    let contents = fs::read_to_string(filename)?;
    let wavebreaker_config: Config = toml::from_str(&contents).unwrap();

    HttpServer::new(|| {
        App::new()
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
