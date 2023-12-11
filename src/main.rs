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

mod game;
mod util;

use anyhow::Context;
use axum::Router;
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

// Stops the client from outputting a huge number of warnings during compilation.
#[allow(warnings, unused)]
mod prisma;
use prisma::PrismaClient;

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
    db: Arc<PrismaClient>,
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
        .context("Failed to load config. Check if it exists and is valid!")?;

    let client = PrismaClient::_builder()
        .build()
        .await
        .context("Failed to initialize Prisma client")?;

    let listener = tokio::net::TcpListener::bind(&wavebreaker_config.main.address)
        .await
        .context("TCP listener failed to bind!")?;

    let state = AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
        config: Arc::new(wavebreaker_config),
        db: Arc::new(client),
    };

    let app = Router::new()
        .nest("/as_steamlogin", routes_steam())
        .with_state(state);

    axum::serve(listener, app)
        .await
        .context("Server failed to... well, serve!")
}
