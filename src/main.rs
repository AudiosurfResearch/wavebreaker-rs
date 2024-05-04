#![warn(
    clippy::pedantic,
    clippy::nursery,
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
mod manager;
pub mod models;
pub mod schema;
mod util;

use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{MatchedPath, Request},
    Router,
};
use clap::Parser;
use deadpool_redis::Runtime;
use diesel_async::pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use game::routes_steam;
use serde::Deserialize;
use steam_rs::Steam;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
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
    redis: String,
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
    redis: deadpool_redis::Pool,
}

fn init_state() -> anyhow::Result<AppState> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "wavebreaker=info,tower_http=error,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!("Start init");

    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .context("Config should be valid!")?;

    let diesel_manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &wavebreaker_config.main.database,
    );
    let pool = Pool::builder(diesel_manager)
        .build()
        .context("Failed to build DB pool!")?;

    let redis_cfg = deadpool_redis::Config::from_url(&wavebreaker_config.main.redis);
    let redis_pool = redis_cfg
        .create_pool(Some(Runtime::Tokio1))
        .context("Failed to build Redis pool!")?;

    // Set global user agent so MusicBrainz can contact us if we're messing up
    musicbrainz_rs::config::set_user_agent(
        "wavebreaker-rs/0.1.0 (https://github.com/AudiosurfResearch/wavebreaker-rs)",
    );

    Ok(AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
        config: Arc::new(wavebreaker_config),
        db: pool,
        redis: redis_pool,
    })
}

fn make_router(state: AppState) -> Router {
    Router::new()
        .nest("/as_steamlogin", routes_steam())
        .nest("//as_steamlogin", routes_steam_doubleslash()) // for that one edge case
        .nest("/as", routes_as(&state.config.radio.cgr_location))
        .layer(
            // TAKEN FROM: https://github.com/tokio-rs/axum/blob/d1fb14ead1063efe31ae3202e947ffd569875c0b/examples/error-handling/src/main.rs#L60-L77
            TraceLayer::new_for_http() // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(axum::extract::MatchedPath::as_str);

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = init_state()?;

    // Parse CLI arguments
    // and if we have a management command, don't spin up a server
    let args = manager::Args::parse();
    if args.command.is_some() {
        return manager::parse_command(&args.command.unwrap(), state).await;
    }

    info!("Wavebreaker starting...");

    let listener = tokio::net::TcpListener::bind(&state.config.main.address)
        .await
        .context("Listener should always be able to listen!")?;
    info!("Listening on {}", &state.config.main.address);

    let app = make_router(state);

    axum::serve(listener, app)
        .await
        .context("Server should be able to... well, serve!")
}
