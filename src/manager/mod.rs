use anyhow::anyhow;
use clap::{Parser, Subcommand};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::debug;

use crate::AppState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    MergeSongs {
        id_to_merge: i32,
        target: i32,
    },
}

pub async fn parse_command(command: &Command, state: AppState) -> anyhow::Result<()> {
    match command {
        Command::MergeSongs { id_to_merge, target } => {
            let conn = state.db.get().await?;
            Err(anyhow!("Not implemented"))
        }
    }
}
