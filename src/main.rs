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

use anyhow::Context;
use axum::Router;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use game::routes_steam;
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use std::sync::Arc;
use steam_rs::Steam;

#[derive(Deserialize, Clone)]
struct Config {
    main: Main,
    external: External,
}

#[derive(Deserialize, Clone)]
struct Main {
    address: String,
    database: String,
}

#[derive(Deserialize, Clone)]
struct External {
    steam_key: String,
}

#[derive(Clone)]
pub struct AppState {
    steam_api: Arc<Steam>,
    config: Arc<Config>,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .context("Failed to load config!")?;

    let pool = Database::connect(&wavebreaker_config.main.database)
        .await
        .context("Failed to connect to database!")?;

    let state = AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
        config: Arc::new(wavebreaker_config.clone()),
        db: pool,
    };

    let app = Router::new()
        .nest("/as_steamlogin", routes_steam())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&wavebreaker_config.main.address)
        .await
        .context("Failed to bind to address")?;
    axum::serve(listener, app)
        .await
        .context("Server should always be able to... well, serve!")
}
