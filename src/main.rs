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

mod error;
mod game;
mod util;

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use game::routes_steam;
use rocket::launch;
use serde::Deserialize;
use steam_rs::Steam;

#[derive(Deserialize)]
struct Config {
    external: External,
}

#[derive(Deserialize)]
struct External {
    steam_key: String,
}

#[launch]
fn rocket() -> _ {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let wavebreaker_config: Config = Figment::new()
        .merge(Toml::file("Wavebreaker.toml"))
        .merge(Env::prefixed("WAVEBREAKER_"))
        .extract()
        .expect("Config should be valid!");

    rocket::build()
        .manage(Steam::new(&wavebreaker_config.external.steam_key))
        .mount("/as_steamlogin", routes_steam())
}
