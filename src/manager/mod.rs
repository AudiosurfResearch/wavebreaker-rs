use clap::{ArgAction, Parser, Subcommand};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use fred::prelude::*;
use tracing::instrument;

use crate::{models::players::AccountType, AppState};

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
    RefreshSkillPoints {
        player_to_refresh: i32,
    },
    RefreshAllSkillPoints,
    ChangeAccountType {
        player_id: i32,
        new_type: AccountType,
    },
    DeletePlayerSessions {
        player_id: i32,
    },
    SyncSongs {
        sync_all: bool,
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

            let to_merge = songs.find(*id_to_merge).first::<Song>(&mut conn).await?;
            to_merge
                .merge_into(*target, *new_alias, &mut conn, &state.redis)
                .await
        }
        Command::DeleteSong { id_to_delete } => {
            use crate::schema::songs::dsl::*;

            let mut conn = state.db.get().await?;

            let song = songs
                .find(*id_to_delete)
                .first::<crate::models::songs::Song>(&mut conn)
                .await?;
            song.delete(&mut conn, &state.redis).await
        }
        Command::DeleteScore { id_to_delete } => {
            use crate::schema::scores::dsl::*;

            let mut conn = state.db.get().await?;

            let score_to_delete = scores
                .find(*id_to_delete)
                .first::<crate::models::scores::Score>(&mut conn)
                .await?;
            score_to_delete.delete(&mut conn, &state.redis).await
        }
        Command::RefreshSkillPoints { player_to_refresh } => {
            use crate::{models::players::Player, schema::players::dsl::*};

            let mut conn = state.db.get().await?;

            let player: Player = players.find(player_to_refresh).first(&mut conn).await?;

            let skill_points = player.calc_skill_points(&mut conn).await?;
            let _: () = state
                .redis
                .zadd(
                    "leaderboard",
                    None,
                    None,
                    false,
                    false,
                    (skill_points.into(), player_to_refresh.to_owned()),
                )
                .await?;

            Ok(())
        }
        Command::RefreshAllSkillPoints => {
            use crate::{models::players::Player, schema::players};

            let mut conn = state.db.get().await?;

            let all_players = players::table.load::<Player>(&mut conn).await?;

            for player in all_players {
                let skill_points = player.calc_skill_points(&mut conn).await?;
                let _: () = state
                    .redis
                    .zadd(
                        "leaderboard",
                        None,
                        None,
                        false,
                        false,
                        (skill_points.into(), player.id),
                    )
                    .await?;
            }

            Ok(())
        }
        Command::ChangeAccountType {
            player_id,
            new_type,
        } => {
            use crate::schema::players::dsl::*;

            let mut conn = state.db.get().await?;

            diesel::update(players.find(player_id))
                .set(account_type.eq(new_type))
                .execute(&mut conn)
                .await?;

            Ok(())
        }
        Command::DeletePlayerSessions { player_id } => {
            use crate::util::session::delete_player_sessions;

            delete_player_sessions(player_id.to_owned(), &state.redis).await?;

            Ok(())
        }
        Command::SyncSongs { sync_all } => {
            use crate::models::extra_song_info::ExtraSongInfo;
            use crate::models::songs::Song;
            use crate::schema::extra_song_info;
            use crate::schema::songs;
            use crate::util::meilisearch::{sync_songs, MeiliSong};

            if *sync_all {
                let mut conn = state.db.get().await?;

                let songs_to_sync: Vec<MeiliSong> = songs::table
                    .left_join(extra_song_info::table)
                    .select((Song::as_select(), extra_song_info::all_columns.nullable()))
                    .load::<(Song, Option<ExtraSongInfo>)>(&mut conn)
                    .await?
                    .iter_mut()
                    .map(|x| {
                        let x = x.clone();
                        MeiliSong {
                            song: x.0,
                            extra_song_info: x.1,
                        }
                    })
                    .collect();

                state
                    .meilisearch
                    .unwrap()
                    .index("songs")
                    .add_documents(&songs_to_sync, Some("id"))
                    .await?;
            } else {
                sync_songs(&state.meilisearch.unwrap(), &state.redis, &state.db).await?;
            }

            Ok(())
        }
    }
}
