use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    dsl::sql,
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    serialize::{self, Output, ToSql},
    sql_types::{BigInt, SmallInt, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use fred::{clients::Pool as RedisPool, prelude::*};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use steam_rs::steam_id::SteamId;
use utoipa::ToSchema;

use super::rivalries::RivalryView;
use crate::{
    models::{rivalries::Rivalry, scores::Score},
    schema::players,
    util::game_types::Character,
};

#[derive(Serialize, Deserialize, AsExpression, FromSqlRow, Debug, PartialEq, Eq, Clone)]
#[diesel(sql_type = diesel::sql_types::Text)]
/// Wrapper around `SteamId` so we can use it in Diesel queries.
///
/// Postgres doesn't natively have an uint type, so we have to store it as a string
/// and convert it back to a `SteamId` when we get it from the DB.
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
        Ok(Self(SteamId::from(steam_id_string)))
    }
}

//todo: use OpenAPI extensions to show the names of the numbers in the API docs?
//https://docs.rs/utoipa/latest/utoipa/index.html?search=extensions
//https://openapi-ts.dev/advanced#enum-extensions

/// Represents the type of account a player has.
///
/// 0 = User, 1 = Moderator, 2 = Wavebreaker Team
#[derive(
    AsExpression,
    FromSqlRow,
    Serialize_repr,
    Deserialize_repr,
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    TryFromPrimitive,
    IntoPrimitive,
    ToSchema,
)]
#[diesel(sql_type = diesel::sql_types::SmallInt)]
#[repr(i16)]
pub enum AccountType {
    User,
    Moderator,
    Team,
}

impl ToSql<SmallInt, Pg> for AccountType
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        <i16 as ToSql<SmallInt, Pg>>::to_sql(&v, &mut out.reborrow())
    }
}

impl<DB> FromSql<SmallInt, DB> for AccountType
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let account_type = i16::from_sql(bytes)?;
        Ok(Self::try_from(account_type)?)
    }
}

#[derive(
    Queryable, Selectable, Identifiable, PartialEq, Eq, Debug, Serialize, Deserialize, Clone,
)]
#[diesel(table_name = players, check_for_backend(diesel::pg::Pg))]
pub struct Player {
    pub id: i32,
    pub username: String,
    pub steam_id: SteamIdWrapper,
    pub steam_account_num: i32,
    pub location_id: i32,
    pub account_type: AccountType,
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
    pub joined_at: time::OffsetDateTime,
    pub avatar_url: String,
}

// Types for use with functions that return reusable query fragments
type All = diesel::dsl::Select<players::table, diesel::dsl::AsSelect<Player, diesel::pg::Pg>>;
type WithSteamId = diesel::dsl::Eq<players::steam_id, SteamIdWrapper>;
type BySteamId = diesel::dsl::Filter<All, WithSteamId>;

impl Player {
    /// Get skill points from Redis.
    pub async fn get_skill_points(&self, redis_conn: &RedisPool) -> anyhow::Result<i32> {
        let skill_points: Option<i32> = redis_conn.zscore("leaderboard", self.id).await?;

        Ok(skill_points.unwrap_or(0))
    }

    /// Calculates the total skill points a player has earned with their scores.
    /// This is not the value stored in the Redis leaderboard, this function calculates it again!
    pub async fn calc_skill_points(&self, conn: &mut AsyncPgConnection) -> QueryResult<i32> {
        use crate::schema::scores::dsl::*;

        let player_scores = scores
            .filter(player_id.eq(self.id))
            .load::<Score>(conn)
            .await?;

        let skill_points_sum = player_scores.iter().map(Score::calc_skill_points).sum();

        Ok(skill_points_sum)
    }

    /// Returns the player's global leaderboard rank
    pub async fn get_rank(&self, redis_conn: &RedisPool) -> anyhow::Result<i32> {
        let rank = redis_conn
            .zrevrank::<i32, _, _>("leaderboard", self.id, false)
            .await?
            + 1; // index starts with 0

        Ok(rank)
    }

    /// Returns the total number of the player's plays.
    /// This is the sum of all `play_count`s across all scores, which increments on every score submission (no matter if high score or not).
    pub async fn get_total_plays(&self, conn: &mut AsyncPgConnection) -> QueryResult<i32> {
        use crate::schema::scores::dsl::*;

        let player_scores = scores
            .filter(player_id.eq(self.id))
            .load::<Score>(conn)
            .await?;

        let play_count_sum = player_scores.iter().map(|s| s.play_count).sum();

        Ok(play_count_sum)
    }

    /// Returns the player's favorite character.
    /// This is the character that they have set the most scores with. Unlike `get_total_plays`, this only counts high scores,
    /// since we do not track the character for submissions that aren't high scores.
    pub async fn get_favorite_character(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Option<FavoriteCharacter>> {
        use crate::schema::scores::dsl::*;

        let result: Option<(Character, i64)> = scores
            .filter(player_id.eq(self.id))
            .select((
                vehicle,
                sql::<BigInt>("COUNT(scores.vehicle) AS play_count"),
            ))
            .group_by(vehicle)
            .order_by(sql::<BigInt>("play_count DESC"))
            .first::<(Character, i64)>(conn)
            .await
            .optional()?;

        Ok(result.map(|(character, times_used)| FavoriteCharacter {
            character,
            times_used,
        }))
    }

    /// Finds a player by their Steam ID.
    ///
    /// # Arguments
    /// * `id_to_find` - The Steam ID of the player to find.
    ///
    /// # Returns
    /// Returns a query fragment, not the player!
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

    /// Retrieves the challengers of a player.
    pub async fn get_challengers(&self, conn: &mut AsyncPgConnection) -> QueryResult<Vec<Self>> {
        use crate::schema::{players::dsl::*, rivalries::dsl::*};

        let challenger_ids = rivalries
            .filter(rival_id.eq(self.id))
            .load::<Rivalry>(conn)
            .await?
            .into_iter()
            .map(|rivalry| rivalry.challenger_id)
            .collect::<Vec<i32>>();

        players
            .filter(id.eq_any(challenger_ids))
            .load::<Self>(conn)
            .await
    }

    /// Retrieves rivalries, with the date they were established, and the profiles of the rivals.
    /// This is **not** like `get_rivals`, which only returns a `Vec<Player>` of the rivals and nothing else.
    pub async fn get_rivalry_views(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<RivalryView>> {
        use crate::schema::rivalries::dsl::*;

        rivalries
            .inner_join(
                crate::schema::players::table.on(rival_id.eq(crate::schema::players::dsl::id)),
            )
            .filter(challenger_id.eq(self.id))
            .select((established_at, PlayerPublic::as_select()))
            .load::<RivalryView>(conn)
            .await
    }

    /// Retrieves rivalries, with the date they were established, and the profiles of the **challengers**.
    pub async fn get_challenger_rivalry_views(
        &self,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<RivalryView>> {
        use crate::schema::rivalries::dsl::*;

        rivalries
            .inner_join(
                crate::schema::players::table.on(challenger_id.eq(crate::schema::players::dsl::id)),
            )
            .filter(rival_id.eq(self.id))
            .select((established_at, PlayerPublic::as_select()))
            .load::<RivalryView>(conn)
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
    /// * `username` - The username of the player.
    /// * `steam_id` - The `SteamId` of the player.
    /// * `steam_account_num` - The Steam account number of the player.
    /// * `avatar_url` - The avatar URL of the player.
    ///
    /// # Returns
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
    pub async fn create_or_update(
        &self,
        conn: &mut AsyncPgConnection,
        redis_conn: &RedisPool,
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

        // If the player doesn't exist in the Redis sorted set, add them with a score of 0
        redis_conn
            .zadd::<(), _, _>(
                "leaderboard",
                Some(SetOptions::NX),
                None,
                false,
                false,
                (0f64, player_result.id),
            )
            .await?;

        Ok(player_result)
    }
}

#[derive(Selectable, Queryable, Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name = players, check_for_backend(diesel::pg::Pg))]
pub struct PlayerPublic {
    pub id: i32,
    pub username: String,
    pub account_type: AccountType,
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    pub joined_at: time::OffsetDateTime,
    pub avatar_url: String,
}

impl From<Player> for PlayerPublic {
    fn from(player: Player) -> Self {
        Self {
            id: player.id,
            username: player.username,
            account_type: player.account_type,
            joined_at: player.joined_at,
            avatar_url: player.avatar_url,
        }
    }
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteCharacter {
    pub character: Character,
    pub times_used: i64,
}
