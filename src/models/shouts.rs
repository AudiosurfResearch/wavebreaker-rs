use crate::schema::shouts;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use serde::Serialize;

use super::players::Player;
use super::songs::Song;

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize)]
#[diesel(belongs_to(Player, foreign_key = author_id))]
#[diesel(belongs_to(Song))]
#[diesel(table_name = shouts, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
pub struct Shout {
    pub id: i32,
    pub song_id: i32,
    pub author_id: i32,
    pub posted_at: time::PrimitiveDateTime,
    pub content: String,
}

impl Shout {
    #[must_use]
    pub fn find_by_song_id(target_id: i32) -> shouts::BoxedQuery<'static, diesel::pg::Pg> {
        use crate::schema::shouts::dsl::*;
        shouts.filter(song_id.eq(target_id)).into_boxed()
    }
}

#[derive(Insertable, Debug)]
#[diesel(table_name = shouts)]
pub struct NewShout<'a> {
    pub song_id: i32,
    pub author_id: i32,
    pub content: &'a str,
}

impl<'a> NewShout<'a> {
    #[must_use]
    pub const fn new(song_id: i32, author_id: i32, content: &'a str) -> Self {
        Self {
            song_id,
            author_id,
            content,
        }
    }

    /// Inserts the shout into the database
    ///
    /// # Errors
    /// This fails if:
    /// - Something goes wrong with the database
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::shouts::dsl::*;
        diesel::insert_into(shouts)
            .values(self)
            .execute(conn)
            .await?;
        Ok(())
    }
}
