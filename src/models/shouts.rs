use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;
use utoipa::ToSchema;

use super::{players::Player, songs::Song};
use crate::{models::players::AccountType, schema::shouts};

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize, ToSchema)]
#[diesel(belongs_to(Player, foreign_key = author_id))]
#[diesel(belongs_to(Song))]
#[diesel(table_name = shouts, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[schema(examples(json!(Shout {
    id: 1,
    song_id: 3,
    author_id: 1,
    content: "i love Reol's No title!!!!! 緩やかに崩れ壊れてく〜".to_owned(),
    posted_at: time::OffsetDateTime::from_unix_timestamp(1458333462).unwrap(),
}), json!(Shout {
    id: 2,
    song_id: 2,
    author_id: 1,
    content: "baby, do you know what you wanna hear?!".to_owned(),
    posted_at: time::OffsetDateTime::now_utc(),
})))]
pub struct Shout {
    pub id: i32,
    pub song_id: i32,
    pub author_id: i32,
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    pub posted_at: time::OffsetDateTime,
    pub content: String,
}

impl Shout {
    #[must_use]
    pub fn find_by_song_id(target_id: i32) -> shouts::BoxedQuery<'static, diesel::pg::Pg> {
        use crate::schema::shouts::dsl::*;
        shouts.filter(song_id.eq(target_id)).into_boxed()
    }

    pub async fn user_can_delete(&self, player: &Player) -> anyhow::Result<bool> {
        if player.id == self.author_id
            || player.account_type == AccountType::Moderator
            || player.account_type == AccountType::Team
        {
            Ok(true)
        } else {
            Ok(false)
        }
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
    /// Fails if something goes wrong with the database
    pub async fn insert(&self, conn: &mut AsyncPgConnection) -> QueryResult<()> {
        use crate::schema::shouts::dsl::*;
        diesel::insert_into(shouts)
            .values(self)
            .execute(conn)
            .await?;
        Ok(())
    }
}
