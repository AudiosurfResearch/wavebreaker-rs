use crate::schema::songs;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

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

#[derive(Insertable)]
#[diesel(table_name = songs)]
/// Represents a new song with a title and artist.
pub struct NewSong<'a> {
    pub title: &'a str,
    pub artist: &'a str,
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
    pub const fn new(title: &'a str, artist: &'a str) -> Self {
        Self { title, artist }
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
