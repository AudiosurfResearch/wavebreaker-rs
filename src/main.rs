#![warn(
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

mod api;
mod game;
mod manager;
pub mod models;
pub mod schema;
mod util;

use std::{io::stdout, str::FromStr, sync::Arc};

use anyhow::{anyhow, Context};
use axum::{body::Body, http::Request, Router};
use clap::Parser;
use diesel::pg::Pg;
use diesel_async::{
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use fred::{clients::Pool as RedisPool, prelude::*, types::config::Config as RedisConfig};
use musicbrainz_rs::client::MusicBrainzClient;
use sentry::{integrations::tower::NewSentryLayer, types::Dsn};
use serde::Deserialize;
use steam_openid::SteamOpenId;
use steam_rs::Steam;
use tower::ServiceBuilder;
use tracing::{debug, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt,
};
use utoipa_scalar::{Scalar, Servable};

use crate::game::{routes_as, routes_steam, routes_steam_doubleslash};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Wavebreaker-specific user agent
pub const WAVEBREAKER_USER_AGENT: &str = concat!(
    concat!(
        concat!(env!("CARGO_PKG_NAME"), "/"),
        env!("CARGO_PKG_VERSION")
    ),
    concat!(concat!(" (", env!("CARGO_PKG_REPOSITORY")), ")")
);

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
    steam_realm: String,
    steam_return_path: String,
    sentry_dsn: Option<String>,
    //meilisearch_url: String,
    //meilisearch_key: String,
}

#[derive(Clone)]
pub struct AppState {
    steam_api: Arc<Steam>,
    steam_openid: Arc<SteamOpenId>,
    config: Arc<Config>,
    db: Pool<diesel_async::AsyncPgConnection>,
    redis: Arc<RedisPool>,
    musicbrainz: Arc<MusicBrainzClient>,
}

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

/// Reads the config, initializes database connections and the Steam API client
///
/// # Returns
/// An `AppState` struct with all the necessary members
///
/// # Errors
/// This function can fail if the config file is missing or invalid, the connection to Postgres or Redis fails, or the Steam API key is invalid
async fn init_state(wavebreaker_config: Config) -> anyhow::Result<AppState> {
    let diesel_manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
        &wavebreaker_config.main.database,
    );
    let pool = Pool::builder(diesel_manager)
        .build()
        .context("Failed to build DB pool!")?;

    // clone the url because moving the value will screw things up
    let pg_url = wavebreaker_config.main.database.clone();
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::Connection;
        use diesel_async::pg::AsyncPgConnection;
        let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&pg_url)
            .expect("Failed to establish DB connection for migrations!");

        run_migrations(&mut conn).expect("Failed to run migrations!");
    })
    .await?;

    let redis_cfg = RedisConfig::from_url(&wavebreaker_config.main.redis)?;
    let redis_builder = Builder::from_config(redis_cfg);

    let redis_pool = redis_builder
        .build_pool(3)
        .context("Failed to build Redis pool!")?;

    redis_pool
        .init()
        .await
        .context("Clients failed to connect to Redis!")?;

    let mut client = MusicBrainzClient::default();
    client
        .set_user_agent(WAVEBREAKER_USER_AGENT)
        .expect("Setting the MusicBrainz client's user agent should not fail.");

    let steam_openid = SteamOpenId::new(
        &wavebreaker_config.external.steam_realm,
        &wavebreaker_config.external.steam_return_path,
    )
    .map_err(|e| anyhow!("Failed to construct SteamOpenId: {e:?}"))?;

    Ok(AppState {
        steam_api: Arc::new(Steam::new(&wavebreaker_config.external.steam_key)),
        steam_openid: Arc::new(steam_openid),
        db: pool,
        redis: Arc::new(redis_pool),
        config: Arc::new(wavebreaker_config),
        musicbrainz: Arc::new(client),
    })
}

fn make_router(state: AppState) -> Router {
    let (api_router, openapi) = api::routes();

    let sentry_layer = if state.config.external.sentry_dsn.is_some() {
        Some(NewSentryLayer::<Request<Body>>::new_from_top())
    } else {
        None
    };

    Router::new()
        .nest("/as_steamlogin", routes_steam())
        .nest("//as_steamlogin", routes_steam_doubleslash()) // for that one edge case
        .nest("/as", routes_as(&state.config.radio.cgr_location))
        .nest("/api", api_router)
        .merge(Scalar::with_url("/api/docs", openapi))
        .layer(ServiceBuilder::new().option_layer(sentry_layer))
        .with_state(state)
}

fn main() -> anyhow::Result<()> {
    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .context("Config should be valid!")?;

    let dsn: Option<Dsn> = match &wavebreaker_config.external.sentry_dsn {
        Some(dsn) => Some(Dsn::from_str(dsn).expect("Sentry DSN should be parseable!")),
        None => None,
    };
    let sentry = sentry::init(sentry::ClientOptions {
        dsn,
        release: sentry::release_name!(),
        ..sentry::ClientOptions::default()
    });

    let file_appender = RollingFileAppender::builder()
        .filename_suffix("wavebreaker.log")
        .rotation(Rotation::DAILY)
        .build("./logs")
        .expect("Initializing logging failed");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let sentry_layer = if sentry.is_enabled() {
        Some(sentry::integrations::tracing::layer())
    } else {
        None
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "wavebreaker=info,tower_http=error,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(stdout.and(non_blocking)))
        .with(sentry_layer)
        .init();

    debug!("Start init");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let state = init_state(wavebreaker_config.clone()).await?;

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

            axum::serve(listener, app.into_make_service())
                .await
                .context("Server should be able to... well, serve!")
        })
}
