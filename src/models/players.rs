use crate::schema::players;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::sql_types::Text;
use diesel::{
    prelude::*,
    serialize::{self, Output, ToSql},
};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use std::str::FromStr;
use steam_rs::steam_id::SteamId;

#[derive(AsExpression, FromSqlRow, Debug, PartialEq, Eq)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct SteamIdWrapper(pub SteamId);

impl ToSql<Text, Pg> for SteamIdWrapper
where
    String: ToSql<Text, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let steam_id_string = self.0.to_string();
        <String as ToSql<Text, Pg>>::to_sql(&steam_id_string, &mut out.reborrow())
    }
}

impl<DB> FromSql<Text, DB> for SteamIdWrapper
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let steam_id_string = String::from_sql(bytes)?;
        Ok(Self(SteamId::from_str(&steam_id_string)?))
    }
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug)]
#[diesel(table_name = players)]
pub struct Player {
    pub id: i32,
    pub username: String,
    pub steam_id: SteamIdWrapper,
    pub steam_account_num: i32,
    pub location_id: i32,
    pub account_type: i16,
    pub joined_at: time::PrimitiveDateTime,
    pub avatar_url: String,
}

#[derive(Insertable)]
#[diesel(table_name = players)]
pub struct NewPlayer<'a> {
    pub username: &'a str,
    pub steam_id: SteamIdWrapper,
    pub steam_account_num: i32,
    pub avatar_url: &'a str,
}

/// Represents a new player.
///
/// This struct is used to create a new player with the specified username, Steam ID, Steam account number, and avatar URL.
/// It provides a method to create or update the player in the database.
impl<'a> NewPlayer<'a> {
    /// Creates a new player with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the player.
    /// * `steam_id` - The `SteamId` of the player.
    /// * `steam_account_num` - The Steam account number of the player.
    /// * `avatar_url` - The avatar URL of the player.
    ///
    /// # Returns
    ///
    /// A new `NewPlayer` instance.
    #[must_use]
    pub const fn new(
        username: &'a str,
        steam_id: SteamId,
        steam_account_num: i32,
        avatar_url: &'a str,
    ) -> Self {
        Self {
            username,
            steam_id: SteamIdWrapper(steam_id),
            steam_account_num,
            avatar_url,
        }
    }

    /// Creates or updates the player in the database.
    ///
    /// # Errors
    ///
    /// This fails if:
    /// - The player fails to be inserted/updated in the database
    pub async fn create_or_update(&self, conn: &mut AsyncPgConnection) -> QueryResult<Player> {
        diesel::insert_into(players::table)
            .values(self)
            .on_conflict(players::steam_account_num)
            .do_update()
            .set((
                players::username.eq(&self.username),
                players::avatar_url.eq(&self.avatar_url),
            ))
            .get_result::<Player>(conn)
            .await
    }
}
