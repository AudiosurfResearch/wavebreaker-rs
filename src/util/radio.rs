use std::fs;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct RadioConfig {
    radio_songs: Option<Vec<RadioSong>>,
}

#[derive(Deserialize, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct RadioSong {
    pub id: u32,
    pub title: String,
    pub artist: String,
    pub external_url: String,
    pub cgr_url: String,
}

pub fn get_radio_songs() -> anyhow::Result<Option<Vec<RadioSong>>> {
    let config_string = fs::read_to_string("WavebreakerRadio.toml")?;
    let radio_config: RadioConfig = toml::from_str(&config_string)?;
    Ok(radio_config.radio_songs)
}
