use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, SaveChangesDsl};
use serde::Serialize;
use tracing::debug;

use crate::{
    models::{
        extra_song_info::{ExtraSongInfo, NewExtraSongInfo},
        players::{AccountType, Player},
        scores::Score,
    },
    schema::{extra_song_info, songs},
};

#[derive(Identifiable, Selectable, Queryable, Debug, Serialize)]
#[diesel(table_name = songs, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[serde(rename_all = "camelCase")]
pub struct Song {
    // Main info
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub created_at: time::OffsetDateTime,
    pub modifiers: Option<Vec<Option<String>>>,
}

impl Song {
    /// Deletes the song from the database.
    ///
    /// # Errors
    /// Fails if something is wrong with the DB or with Redis.
    pub async fn delete(
        &self,
        conn: &mut AsyncPgConnection,
        redis_conn: &mut deadpool_redis::Connection,
    ) -> anyhow::Result<()> {
        use crate::schema::{
            scores::dsl::{scores, song_id},
            songs::dsl::{id, songs},
        };

        // Manually delete all of the song's scores with our own Score::delete().
        // Necessary because we have to subtract the skill points from Redis
        // Diesel doesn't provide hooks to do it automatically
        let ass_scores: Vec<Score> = scores
            .filter(song_id.eq(self.id))
            .load::<Score>(conn)
            .await?;
        for score in ass_scores {
            score.delete(conn, redis_conn).await?;
        }

        diesel::delete(songs.filter(id.eq(self.id)))
            .execute(conn)
            .await?;
        Ok(())
    }

    /// Merges this song into another one. `self` will be deleted when it's done.
    ///
    /// # Errors
    /// When the merge fails or something is wrong with the database, this fails.
    pub async fn merge_into(
        &self,
        target: i32,
        should_alias: bool,
        conn: &mut AsyncPgConnection,
        redis_conn: &mut deadpool_redis::Connection,
    ) -> anyhow::Result<()> {
        use crate::schema::{scores::dsl::*, songs::dsl::*};

        let target = songs.find(target).first::<Self>(conn).await?;
        let mut target_scores: Vec<Score> = Score::belonging_to(&target)
            .select(Score::as_select())
            .load::<Score>(conn)
            .await?;
        let own_scores: Vec<Score> = Score::belonging_to(&self)
            .select(Score::as_select())
            .load::<Score>(conn)
            .await?;

        debug!("Merging song {} into {}", self.id, target.id);

        for mut own_score in own_scores {
            // Find score with same player and league in the target song
            match target_scores.iter_mut().find(|found_score| {
                found_score.player_id == own_score.player_id
                    && found_score.league == own_score.league
            }) {
                Some(target_score) => {
                    // If the score on the song we want to merge into is lower, we delete that score
                    // then, we add our song's score to the merge target song
                    if target_score.score < own_score.score {
                        target_score.delete(conn, redis_conn).await?;
                        own_score.song_id = target.id;
                        own_score.play_count += target_score.play_count;
                        own_score.save_changes::<Score>(conn).await?;
                    } else {
                        target_score.play_count += own_score.play_count;
                        target_score.save_changes::<Score>(conn).await?;
                        own_score.delete(conn, redis_conn).await?;
                    }
                }
                None => {
                    diesel::update(&own_score)
                        .set(song_id.eq(target.id))
                        .execute(conn)
                        .await?;
                }
            }
        }

        if should_alias {
            let target_extra_info: Option<ExtraSongInfo> = ExtraSongInfo::belonging_to(&target)
                .select(ExtraSongInfo::as_select())
                .first::<ExtraSongInfo>(conn)
                .await
                .optional()?;

            if let Some(target_extra_info) = target_extra_info {
                //This doesn't merge our own alias list into the target's!
                //*Only our artist and title fields* are added to the target's aliases.
                target_extra_info
                    .aliases_artist
                    .clone()
                    .unwrap_or_default()
                    .push(Some(self.artist.clone()));
                target_extra_info
                    .aliases_title
                    .clone()
                    .unwrap_or_default()
                    .push(Some(self.title.clone()));

                target_extra_info
                    .save_changes::<ExtraSongInfo>(conn)
                    .await?;
            } else {
                let new_extra_info = NewExtraSongInfo {
                    song_id: target.id,
                    aliases_artist: Some(vec![self.artist.clone()]),
                    aliases_title: Some(vec![self.title.clone()]),
                    ..Default::default()
                };
                new_extra_info.insert(conn).await?;
            }
        }

        //Delete this song!
        self.delete(conn, redis_conn).await?;

        Ok(())
    }

    #[allow(clippy::doc_markdown)]
    /// Automatically adds extra metadata from [MusicBrainz](https://musicbrainz.org) to the song if it doesn't have any.
    ///
    /// This function doesn't check if an existing `ExtraSongInfo` struct lacks info.
    /// It bails if it finds an existing struct *at all.*
    ///
    /// # Errors
    /// Fails on database error or if the MusicBrainz lookup fails.
    pub async fn auto_add_metadata(
        &self,
        duration: i32,
        conn: &mut AsyncPgConnection,
    ) -> anyhow::Result<()> {
        use crate::util::musicbrainz::lookup_metadata;

        let extra_info = ExtraSongInfo::belonging_to(self)
            .select(ExtraSongInfo::as_select())
            .first::<ExtraSongInfo>(conn)
            .await
            .optional()?;

        if extra_info.is_none() {
            let metadata = lookup_metadata(self, duration).await?;

            diesel::insert_into(extra_song_info::table)
                .values((metadata, extra_song_info::song_id.eq(self.id)))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    #[allow(clippy::doc_markdown)]
    /// Gets and adds metadata to a song from a [MusicBrainz ID](https://musicbrainz.org/doc/MusicBrainz_Identifier).
    /// It updates all relevant fields on the `ExtraSongInfo` struct, if there is one already.
    /// If there isn't, it creates a new one.
    ///
    /// # Errors
    /// Fails on database error or if the MusicBrainz lookup fails.
    pub async fn add_metadata_mbid(
        &self,
        mbid: &str,
        release_mbid: Option<&str>,
        conn: &mut AsyncPgConnection,
    ) -> anyhow::Result<()> {
        use crate::util::musicbrainz::lookup_mbid;

        let existing_info = ExtraSongInfo::belonging_to(self)
            .select(ExtraSongInfo::as_select())
            .first::<ExtraSongInfo>(conn)
            .await
            .optional()?;

        let mb_info = lookup_mbid(mbid, release_mbid).await?;

        if let Some(existing_info) = existing_info {
            diesel::update(&existing_info)
                .set(mb_info)
                .execute(conn)
                .await?;
        } else {
            diesel::insert_into(extra_song_info::table)
                .values((mb_info, extra_song_info::song_id.eq(self.id)))
                .execute(conn)
                .await?;
        }

        Ok(())
    }

    /// Checks if a user is allowed to edit a song's metadata.
    /// This is allowed if the user is a moderator/Wavebreaker team member, or if they set the first score on the song.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player.
    /// * `conn` - The mutable reference to the database connection.
    ///
    /// # Errors
    /// If something is wrong with the database or a player with the ID doesn't exist, this fails.
    pub async fn user_can_edit(
        &self,
        player_id: i32,
        conn: &mut AsyncPgConnection,
    ) -> anyhow::Result<bool> {
        use crate::schema::{
            players::dsl::players,
            scores::dsl::{scores, song_id, submitted_at},
        };

        let player = players.find(player_id).first::<Player>(conn).await?;

        if player.account_type == AccountType::Moderator || player.account_type == AccountType::Team
        {
            return Ok(true);
        }

        //Get first score of song
        let first_score = scores
            .filter(song_id.eq(self.id))
            .order(submitted_at.asc())
            .first::<Score>(conn)
            .await
            .optional()?;

        // If the first score was set by the player, they can edit the song
        match first_score {
            Some(score) => Ok(score.player_id == player.id),
            None => Ok(false),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = songs)]
/// Represents a new song with a title and artist.
pub struct NewSong<'a> {
    pub title: &'a str,
    pub artist: &'a str,
    pub modifiers: Option<Vec<&'a str>>,
}

impl<'a> NewSong<'a> {
    /// Creates a new `NewSong` instance with the given title and artist.
    ///
    /// # Arguments
    /// * `title` - The title of the song.
    /// * `artist` - The artist of the song.
    ///
    /// # Returns
    /// A new `NewSong` instance.
    #[must_use]
    pub const fn new(title: &'a str, artist: &'a str, modifiers: Option<Vec<&'a str>>) -> Self {
        Self {
            title,
            artist,
            modifiers,
        }
    }

    /// Finds or creates a song in the database.
    ///
    /// # Arguments
    /// * `conn` - The mutable reference to the database connection.
    ///
    /// # Returns
    /// A `QueryResult` containing the found or created song.
    ///
    /// # Errors
    /// This fails if the query or DB connection fail.
    pub async fn find_or_create(&self, conn: &mut AsyncPgConnection) -> QueryResult<Song> {
        use diesel::sql_types::{Nullable, Text};

        use crate::schema::{
            extra_song_info::dsl::{
                aliases_artist, aliases_title, musicbrainz_artist, musicbrainz_title,
            },
            songs::dsl::{artist, title},
        };

        // diesel doesn't have support for the lower function out of the box
        define_sql_function!(fn lower(x: Nullable<Text> ) -> Nullable<Text>);

        // the alias arrays and the musicbrainz data have to play by the game's rules
        // or else we can never match them with what the game sends!
        // for arrays: lowercase (the lower function wont work on arrays)
        // for all of them: "&" replaced with "and", potentially other changes by the client too!
        // can we fix this in the hook? what do we do?!
        let title_predicate = title.eq(self.title).or(lower(musicbrainz_title)
            .eq(self.title)
            .or(aliases_title.contains(vec![self.title])));
        let artist_predicate = artist.eq(self.artist).or(lower(musicbrainz_artist)
            .eq(self.artist)
            .or(aliases_artist.contains(vec![self.artist])));

        match songs::table
            .left_join(extra_song_info::table)
            .select((Song::as_select(), Option::<ExtraSongInfo>::as_select()))
            .filter(title_predicate.and(artist_predicate))
            .first::<(Song, Option<ExtraSongInfo>)>(conn)
            .await
            .optional()?
        {
            Some(song_extended) => Ok(song_extended.0),
            None => {
                diesel::insert_into(songs::table)
                    .values(self)
                    .get_result(conn)
                    .await
            }
        }
    }
}
