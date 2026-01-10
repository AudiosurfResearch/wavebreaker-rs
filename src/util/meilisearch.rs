use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool as PostgresPool;
use diesel_async::RunQueryDsl;
use fred::clients::Pool as RedisPool;
use fred::prelude::*;
use meilisearch_sdk::client::Client as MeiliClient;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::info;

use crate::models::extra_song_info::ExtraSongInfo;
use crate::models::songs::Song;
use crate::schema::extra_song_info;
use crate::schema::songs;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeiliSong {
    #[serde(flatten)]
    pub song: Song,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_song_info: Option<ExtraSongInfo>,
}

#[tracing::instrument(skip_all)]
pub async fn sync_songs(
    meili: &MeiliClient,
    redis: &RedisPool,
    db: &PostgresPool<diesel_async::AsyncPgConnection>,
) -> anyhow::Result<()> {
    info!("Syncing songs to Meilisearch");

    let mut conn = db.get().await?;

    let _: () = redis
        .set(
            "last_meilisearch_sync",
            OffsetDateTime::now_utc().unix_timestamp(),
            None,
            Some(SetOptions::NX),
            false,
        )
        .await?;

    let last_sync: OffsetDateTime = redis.get("last_meilisearch_sync").await.map(|i| {
        OffsetDateTime::from_unix_timestamp(i)
            .expect("UNIX timestamp should always be valid since it's controlled by the song sync")
    })?;
    info!("Last sync done at: {:?}", last_sync);

    let songs_to_sync: Vec<MeiliSong> = songs::table
        .filter(
            songs::updated_at
                .le(last_sync)
                .or(extra_song_info::updated_at.le(last_sync)),
        )
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

    meili
        .index("songs")
        .add_documents(&songs_to_sync, Some("id"))
        .await?;

    let _: () = redis
        .set(
            "last_meilisearch_sync",
            OffsetDateTime::now_utc().unix_timestamp(),
            None,
            None,
            false,
        )
        .await?;

    Ok(())
}
