use clap::{ArgAction, Parser, Subcommand};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::instrument;

use crate::AppState;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
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
    DeleteSong {
        id_to_delete: i32,
    },
    DeleteScore {
        id_to_delete: i32,
    },
}

//skip state because it has members that don't implement Debug
#[instrument(name = "cli_command", skip(state))]
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
        Command::DeleteSong { id_to_delete } => {
            use crate::schema::songs::dsl::*;

            let mut conn = state.db.get().await?;
            let mut redis_conn = state.redis.get().await?;

            let song = songs
                .find(*id_to_delete)
                .first::<crate::models::songs::Song>(&mut conn)
                .await?;
            song.delete(&mut conn, &mut redis_conn).await
        }
        Command::DeleteScore { id_to_delete } => {
            use crate::schema::scores::dsl::*;

            let mut conn = state.db.get().await?;
            let mut redis_conn = state.redis.get().await?;

            let score_to_delete = scores
                .find(*id_to_delete)
                .first::<crate::models::scores::Score>(&mut conn)
                .await?;
            score_to_delete.delete(&mut conn, &mut redis_conn).await
        }
    }
}
