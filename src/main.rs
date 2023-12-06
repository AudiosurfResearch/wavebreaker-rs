#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
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
#![allow(clippy::no_effect_underscore_binding, clippy::module_name_repetitions)]

mod game;
mod util;

use std::sync::Arc;

use axum::Router;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use game::routes_steam;
use serde::Deserialize;
use steam_rs::Steam;

#[derive(Deserialize)]
struct Config {
    main: Main,
    external: External,
}

#[derive(Deserialize)]
struct Main {
    address: String,
}

#[derive(Deserialize)]
struct External {
    steam_key: String,
}

#[derive(Clone)]
pub struct AppState {
    steam_api: Arc<Steam>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Wavebreaker starting.");

    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .expect("Config should be valid!");

    let state = AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
    };

    let app = Router::new()
        .nest("/as_steamlogin", routes_steam())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(wavebreaker_config.main.address)
        .await
        .expect("Listener should always be able to listen!");
    axum::serve(listener, app)
        .await
        .expect("Server should always be able to... well, serve!");
}
