use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{models::scores::Score, schema::songs};

#[derive(Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name = songs, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
pub struct Song {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub created_at: time::PrimitiveDateTime,
    pub modifiers: Option<Vec<String>>,
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
            songs::dsl::{songs, id},
        };

        // Manually delete all scores associated with this song using Score::delete().
        // This normally wouldn't be necessary, but we have to subtract the skill points from Redis
        // and Diesel doesn't let me hook into the delete operation.
        let ass_scores = scores
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
    ///
    /// * `title` - The title of the song.
    /// * `artist` - The artist of the song.
    ///
    /// # Returns
    ///
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
    ///
    /// * `conn` - The mutable reference to the database connection.
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing the found or created song.
    ///
    /// # Errors
    ///
    /// This fails if the query or DB connection fail.
    pub async fn find_or_create(&self, conn: &mut AsyncPgConnection) -> QueryResult<Song> {
        use crate::schema::songs::dsl::{artist, songs, title};

        match songs
            .filter(title.eq(self.title))
            .filter(artist.eq(self.artist))
            .first(conn)
            .await
        {
            Ok(song) => Ok(song),
            Err(_) => {
                diesel::insert_into(songs)
                    .values(self)
                    .get_result(conn)
                    .await
            }
        }
    }
}
