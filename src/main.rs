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
    clippy::todo
)]
#![allow(clippy::wildcard_imports)]

mod game;
pub mod models;
pub mod schema;
mod util;

use anyhow::Context;
use axum::Router;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use game::routes_steam;
use serde::Deserialize;
use std::sync::Arc;
use steam_rs::Steam;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::game::{routes_as, routes_steam_doubleslash};

#[derive(Deserialize, Clone)]
struct Config {
    main: Main,
    radio: Radio,
    external: External,
}

#[derive(Deserialize, Clone)]
struct Main {
    address: String,
    database: String,
}

#[derive(Deserialize, Clone)]
struct Radio {
    cgr_location: String,
}

#[derive(Deserialize, Clone)]
struct External {
    steam_key: String,
}

#[derive(Clone)]
pub struct AppState {
    steam_api: Arc<Steam>,
    config: Arc<Config>,
    db: Pool<diesel_async::AsyncPgConnection>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "wavebreaker=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Wavebreaker starting...");

    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .context("Config should be valid!")?;

    let listener = tokio::net::TcpListener::bind(&wavebreaker_config.main.address)
        .await
        .context("Listener should always be able to listen!")?;
    info!("Listening on {}", wavebreaker_config.main.address);

    let diesel_manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &wavebreaker_config.main.database,
    );
    let pool = Pool::builder(diesel_manager)
        .build()
        .context("Failed to build DB pool!")?;

    let state = AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
        config: Arc::new(wavebreaker_config),
        db: pool,
    };

    let app = Router::new()
        .nest("/as_steamlogin", routes_steam())
        .nest("//as_steamlogin", routes_steam_doubleslash()) // for that one edge case
        .nest("/as", routes_as(&state.config.radio.cgr_location))
        .with_state(state);

    axum::serve(listener, app)
        .await
        .context("Server should be able to... well, serve!")
}
