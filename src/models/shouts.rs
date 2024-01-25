use crate::schema::shouts;
use diesel::prelude::*;
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
