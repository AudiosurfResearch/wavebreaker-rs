use clap::{ArgAction, Parser, Subcommand};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

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
        #[clap(action=ArgAction::Set)]
        new_alias: bool,
    },
}

pub async fn parse_command(command: &Command, state: AppState) -> anyhow::Result<()> {
    match command {
        Command::MergeSongs {
            id_to_merge,
            target,
            new_alias,
        } => {
            use crate::{models::songs::Song, schema::songs::dsl::*};

            let mut conn = state.db.get().await?;
            let mut redis_conn = state.redis.get().await?;

            let to_merge = songs.find(*id_to_merge).first::<Song>(&mut conn).await?;
            to_merge
                .merge_into(*target, *new_alias, &mut conn, &mut redis_conn)
                .await
        }
    }
}
