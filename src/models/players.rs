use std::str::FromStr;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{self, Output, ToSql},
    sql_types::Text,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use redis::AsyncCommands;
use serde::Serialize;
use steam_rs::steam_id::SteamId;
use tracing::info;

use crate::{
    models::{rivalries::Rivalry, scores::Score},
    schema::players,
};

#[derive(Serialize, AsExpression, FromSqlRow, Debug, PartialEq, Eq)]
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

#[derive(Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Serialize)]
#[diesel(table_name = players, check_for_backend(diesel::pg::Pg))]
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

type All = diesel::dsl::Select<players::table, diesel::dsl::AsSelect<Player, diesel::pg::Pg>>;
type WithSteamId = diesel::dsl::Eq<players::steam_id, SteamIdWrapper>;
type BySteamId = diesel::dsl::Filter<All, WithSteamId>;

impl Player {
    /// Returns the total skill points a player has earned with their scores.
    ///
    /// # Errors
    /// This fails if something goes wrong with the database.
    pub async fn get_skill_points(&self, conn: &mut AsyncPgConnection) -> QueryResult<i32> {
        use crate::schema::scores::dsl::*;

        let player_scores = scores
            .filter(player_id.eq(self.id))
            .load::<Score>(conn)
            .await?;

        let skill_points_sum = player_scores.iter().map(Score::get_skill_points).sum();

        Ok(skill_points_sum)
    }

    /// Finds a player by their Steam ID.
    ///
    /// # Arguments
    ///
    /// * `id_to_find` - The Steam ID of the player to find.
    /// * `conn` - A mutable reference to an `AsyncPgConnection`.
    ///
    /// # Returns
    ///
    /// Returns a query fragment
    ///
    /// # Errors
    /// This fails if:
    /// - The player fails to be found in the database
    /// - The database connection fails
    #[must_use]
    pub fn find_by_steam_id(id_to_find: SteamId) -> BySteamId {
        use crate::schema::players::dsl::*;

        Self::all().filter(steam_id.eq(SteamIdWrapper(id_to_find)))
    }

    /// Returns a query fragment that selects all players.
    #[must_use]
    pub fn all() -> All {
        players::table.select(Self::as_select())
    }

    /// Retrieves the rivals of a player.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to an `AsyncPgConnection`.
    ///
    /// # Returns
    ///
    /// Returns a `QueryResult` containing a vector of players.
    ///
    /// # Errors
    ///
    /// This fails if the database connection fails
    pub async fn get_rivals(&self, conn: &mut AsyncPgConnection) -> QueryResult<Vec<Self>> {
        use crate::schema::{players::dsl::*, rivalries::dsl::*};

        let rival_ids = rivalries
            .filter(challenger_id.eq(self.id))
            .load::<Rivalry>(conn)
            .await?
            .into_iter()
            .map(|rivalry| rivalry.rival_id)
            .collect::<Vec<i32>>();

        players
            .filter(id.eq_any(rival_ids))
            .load::<Self>(conn)
            .await
    }
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
    pub async fn create_or_update(
        &self,
        conn: &mut AsyncPgConnection,
        redis_conn: &mut deadpool_redis::Connection,
    ) -> anyhow::Result<Player> {
        // Register player
        // Update info if already registered
        let player_result = diesel::insert_into(players::table)
            .values(self)
            .on_conflict(players::steam_account_num)
            .do_update()
            .set((
                players::username.eq(&self.username),
                players::avatar_url.eq(&self.avatar_url),
            ))
            .get_result::<Player>(conn)
            .await?;

        // If the player doesn't exist in the Redis sorted set, add them
        let player_rank: Option<u32> = redis_conn.zscore("leaderboard", player_result.id).await?;
        if player_rank.is_none() {
            info!("Adding player {} to leaderboard", player_result.id);
            redis_conn.zadd("leaderboard", player_result.id, 0).await?;
        }

        Ok(player_result)
    }
}
