use anyhow::Context;
use diesel::{
    associations::HasTable,
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    prelude::*,
    serialize,
    serialize::{Output, ToSql},
    sql_types::SmallInt,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use fred::{clients::Pool as RedisPool, prelude::*};
use serde::Serialize;
use time::OffsetDateTime;
use utoipa::ToSchema;

use crate::{
    models::{players::Player, songs::Song},
    schema::scores,
    util::game_types::{Character, League},
};

impl ToSql<SmallInt, Pg> for League
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        <i16 as ToSql<SmallInt, Pg>>::to_sql(&v, &mut out.reborrow())
    }
}

impl<DB> FromSql<SmallInt, DB> for League
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let league_num = i16::from_sql(bytes)?;
        Ok(Self::try_from(league_num)?)
    }
}

impl ToSql<SmallInt, Pg> for Character
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let v = *self as i16;
        <i16 as ToSql<SmallInt, Pg>>::to_sql(&v, &mut out.reborrow())
    }
}

impl<DB> FromSql<SmallInt, DB> for Character
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let league_num = i16::from_sql(bytes)?;
        Ok(Self::try_from(league_num)?)
    }
}

#[derive(
    AsChangeset,
    Identifiable,
    Selectable,
    Queryable,
    Associations,
    Debug,
    Serialize,
    QueryableByName,
    ToSchema,
)]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Song))]
#[diesel(table_name = scores, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[serde(rename_all = "camelCase")]
#[schema(examples(json!(Score {
    id: 1,
    song_id: 2,
    player_id: 1,
    league: League::Elite,
    submitted_at: OffsetDateTime::now_utc(),
    play_count: 12,
    score: 508143,
    // this gets ugly quick because of the sheer amount of values in a track shape
    track_shape: vec![Some(68), Some(65), Some(62), Some(58), Some(54), Some(51), Some(51), Some(49), Some(48), Some(46), Some(44), Some(42), Some(41), Some(40), Some(39), Some(37), Some(37), Some(36), Some(34), Some(32), Some(31), Some(30), Some(29), Some(28), Some(28), Some(27), Some(27), Some(25), Some(26), Some(26), Some(25), Some(22), Some(19), Some(17), Some(16), Some(14), Some(11), Some(8), Some(5), Some(2), Some(0), Some(0), Some(0), Some(1), Some(2), Some(3), Some(3), Some(4), Some(5), Some(6), Some(6), Some(6), Some(6), Some(8), Some(9), Some(9), Some(10), Some(10), Some(11), Some(11), Some(12), Some(13), Some(15), Some(16), Some(18), Some(19), Some(20), Some(22), Some(23), Some(24), Some(26), Some(27), Some(28), Some(30), Some(31), Some(32), Some(33), Some(34), Some(36), Some(37), Some(39), Some(40), Some(41), Some(41), Some(40), Some(39), Some(38), Some(38), Some(37), Some(37), Some(36), Some(36), Some(37), Some(37), Some(37), Some(36), Some(36), Some(35), Some(34), Some(34), Some(34), Some(34), Some(34), Some(34), Some(34), Some(33), Some(33), Some(33), Some(33), Some(33), Some(33), Some(34), Some(35), Some(37), Some(39), Some(40), Some(41), Some(42), Some(44), Some(45), Some(44), Some(41), Some(39), Some(37), Some(36), Some(35), Some(33), Some(30), Some(28), Some(26), Some(23), Some(22), Some(22), Some(23), Some(24), Some(26), Some(26), Some(28), Some(29), Some(30), Some(31), Some(31), Some(32), Some(33), Some(34), Some(36), Some(37), Some(38), Some(39), Some(39), Some(40), Some(41), Some(43), Some(44), Some(46), Some(47), Some(50), Some(51), Some(52), Some(54), Some(55), Some(57), Some(59), Some(60), Some(61), Some(63), Some(64), Some(65), Some(67), Some(68), Some(70), Some(72), Some(74), Some(75), Some(77), Some(78), Some(79), Some(81), Some(82), Some(83), Some(81), Some(78), Some(74), Some(71), Some(67), Some(65), Some(66), Some(66), Some(67), Some(67), Some(66), Some(67), Some(68), Some(70), Some(73), Some(74), Some(73), Some(70), Some(67), Some(65), Some(64), Some(63), Some(60), Some(58), Some(56), Some(53), Some(51), Some(49), Some(49), Some(49), Some(50), Some(50), Some(50), Some(51), Some(52), Some(52), Some(52), Some(52), Some(53), Some(54), Some(54), Some(55), Some(56), Some(56), Some(56), Some(57), Some(57), Some(58), Some(60), Some(62), Some(64), Some(66), Some(67), Some(68), Some(70), Some(72), Some(73), Some(75), Some(76), Some(78), Some(80), Some(81), Some(83), Some(85), Some(87), Some(89), Some(91), Some(92), Some(94), Some(96), Some(98), Some(99), Some(100), Some(102), Some(103), Some(103)],
    xstats: vec![Some(478), Some(4), Some(0), Some(274), Some(2), Some(24612), Some(136), Some(21), Some(9), Some(21), Some(1804), Some(28), Some(1), Some(9), Some(100), Some(7), Some(41), Some(135), Some(16), Some(67), Some(144), Some(34), Some(97), Some(76), Some(27), Some(97), Some(26), Some(16), Some(87), Some(0), Some(4), Some(0), Some(0)],
    density: 994,
    vehicle: Character::PointmanElite,
    feats: vec![Some("Ironmode".to_owned()), Some("Clean Finish".to_owned()), Some("Match21".to_owned()), Some("Butter Ninja".to_owned()), Some("Seeing Red".to_owned())],
    song_length: 28224,
    gold_threshold: 84490,
    iss: 0,
    isj: 0,
})))]
pub struct Score {
    pub id: i32,
    pub song_id: i32,
    pub player_id: i32,
    pub league: League,
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    pub submitted_at: time::OffsetDateTime,
    pub play_count: i32,
    pub score: i32,
    pub track_shape: Vec<Option<i32>>,
    /// Extra data about the play with meaning depending on the character used, sent by the game as a string of x-seperated numbers
    pub xstats: Vec<Option<i32>>,
    pub density: i32,
    pub vehicle: Character,
    /// Bonuses like Clean Finish, Seeing Red, etc.
    pub feats: Vec<Option<String>>,
    pub song_length: i32,
    pub gold_threshold: i32,
    pub iss: i32,
    pub isj: i32,
}

impl Score {
    /// Calculates and returns the skill points the player earned for this score.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn calc_skill_points(&self) -> i32 {
        let multiplier = (self.league as u32 + 1) * 100;
        ((f64::from(self.score) / f64::from(self.gold_threshold)) * f64::from(multiplier)).round()
            as i32
    }

    /// Deletes the score from the database.
    ///
    /// # Errors
    /// This fails if the database query fails or something goes wrong with Redis.
    pub async fn delete(
        &self,
        conn: &mut AsyncPgConnection,
        redis_pool: &RedisPool,
    ) -> anyhow::Result<()> {
        use crate::schema::scores::dsl::*;

        // Subtract the skill points from the player on Redis
        let sub_amount = 0 - self.calc_skill_points();
        let _: () = redis_pool
            .zincrby("leaderboard", sub_amount.into(), self.player_id)
            .await?;

        diesel::delete(scores.filter(id.eq(self.id)))
            .execute(conn)
            .await?;
        Ok(())
    }

    /// Retrieves the scores for a specific song and league, for display in-game.
    /// **ALL OF THE `game_get_*` FUNCTIONS ARE ONLY FOR IN-GAME LEADERBOARDS.**
    ///  Therefore, the score count is limited to 11.
    ///
    /// # Arguments
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// * `conn` - The database connection.
    pub async fn game_get_global(
        find_song_id: i32,
        find_league: League,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<ScoreWithPlayer>> {
        use crate::schema::{players::dsl::*, scores::dsl::*};

        Ok(scores
            .inner_join(players::table())
            .filter(song_id.eq(find_song_id))
            .filter(league.eq(find_league))
            .order(score.desc())
            .limit(11)
            .load::<(Self, Player)>(conn)
            .await?
            .into_iter()
            .map(|(curr_score, player)| ScoreWithPlayer {
                score: curr_score,
                player,
            })
            .collect::<Vec<ScoreWithPlayer>>())
    }

    /// Gets all rivals' scores for a specific song and league, for display in-game.
    ///
    /// # Arguments
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// *  `rival_ids` - The IDs of the rivals to filter scores by.
    /// * `conn` - The database connection.
    pub async fn game_get_rivals(
        find_song_id: i32,
        find_league: League,
        rival_ids: &Vec<i32>,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<ScoreWithPlayer>> {
        use crate::schema::{players::dsl::*, scores::dsl::*};

        Ok(scores
            .inner_join(players::table())
            .filter(song_id.eq(find_song_id))
            .filter(league.eq(find_league))
            .filter(player_id.eq_any(rival_ids))
            .order(score.desc())
            .limit(11)
            .load::<(Self, Player)>(conn)
            .await?
            .into_iter()
            .map(|(curr_score, player)| ScoreWithPlayer {
                score: curr_score,
                player,
            })
            .collect::<Vec<ScoreWithPlayer>>())
    }

    /// Retrieves the scores of everyone with a certain location
    /// for a specific song and league, for display in-game.
    ///
    /// # Arguments
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// * `conn` - The database connection.
    pub async fn game_get_nearby(
        find_song_id: i32,
        find_league: League,
        find_location_id: i32,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Vec<ScoreWithPlayer>> {
        use crate::schema::{players::dsl::*, scores::dsl::*};

        Ok(scores
            .inner_join(players::table())
            .filter(song_id.eq(find_song_id))
            .filter(league.eq(find_league))
            .filter(location_id.eq(find_location_id))
            .order(score.desc())
            .limit(11)
            .load::<(Self, Player)>(conn)
            .await?
            .into_iter()
            .map(|(curr_score, player)| ScoreWithPlayer {
                score: curr_score,
                player,
            })
            .collect::<Vec<ScoreWithPlayer>>())
    }
}

#[derive(Serialize)]
pub struct ScoreWithPlayer {
    #[serde(flatten)]
    pub score: Score,
    pub player: Player,
}

#[derive(Insertable)]
#[diesel(table_name = scores)]
pub struct NewScore<'a> {
    pub player_id: i32,
    pub song_id: i32,
    pub league: League,
    pub score: i32,
    pub track_shape: &'a [i32],
    pub xstats: &'a [i32],
    pub density: i32,
    pub vehicle: Character,
    pub feats: &'a [&'a str],
    pub song_length: i32,
    pub gold_threshold: i32,
    pub iss: i32,
    pub isj: i32,
}

impl<'a> NewScore<'a> {
    /// Creates a new `NewScore` instance.
    ///
    /// # Arguments
    /// * `player_id` - The ID of the player.
    /// * `song_id` - The ID of the song.
    /// * `league` - The league (Casual, Pro, Elite) the score was set in.
    /// * `score` - The score.
    /// * `track_shape` - Contains the track's elevation at various points.
    /// * `xstats` - The extended stats. The elements' meaning depend on the character.
    /// * `density` - The density value.
    /// * `vehicle` - The character used.
    /// * `feats` - The feats performed.
    /// * `song_length` - The length of the song.
    /// * `gold_threshold` - The score required for the gold meda.
    /// * `iss` - Purpose unknown.
    /// * `isj_value` - Purpose unknown.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        player_id: i32,
        song_id: i32,
        league: League,
        score: i32,
        track_shape: &'a [i32],
        xstats: &'a [i32],
        density: i32,
        vehicle: Character,
        feats: &'a [&'a str],
        song_length: i32,
        gold_threshold: i32,
        iss: i32,
        isj_value: i32,
    ) -> Self {
        Self {
            player_id,
            song_id,
            league,
            score,
            track_shape,
            xstats,
            density,
            vehicle,
            feats,
            song_length,
            gold_threshold,
            iss,
            isj: isj_value,
        }
    }

    /// Creates or updates a score entry in the database.
    ///
    /// # Arguments
    /// * `conn` - The database connection.
    ///
    /// # Returns
    /// The created or updated score entry.
    ///
    /// # Errors
    /// This fails if:
    /// - The database connection fails
    /// - The score fails to be created/retrieved
    pub async fn create_or_update(
        &self,
        conn: &mut AsyncPgConnection,
        redis_conn: &RedisPool,
    ) -> anyhow::Result<Score> {
        use crate::schema::scores::dsl::*;

        let existing_score = scores
            .filter(player_id.eq(self.player_id))
            .filter(song_id.eq(self.song_id))
            .filter(league.eq(self.league))
            .first::<Score>(conn)
            .await
            .optional()?;

        if let Some(existing_score) = existing_score {
            if existing_score.score < self.score {
                // Subtract the skill points of the old score from the Redis leaderboard
                let sub_amount = 0 - existing_score.calc_skill_points();
                let _: () = redis_conn
                    .zincrby("leaderboard", sub_amount.into(), existing_score.player_id)
                    .await?;

                let updated_score = diesel::update(scores)
                    .filter(player_id.eq(self.player_id))
                    .filter(song_id.eq(self.song_id))
                    .filter(league.eq(self.league))
                    .set((
                        score.eq(self.score),
                        track_shape.eq(self.track_shape),
                        xstats.eq(self.xstats),
                        density.eq(self.density),
                        vehicle.eq(self.vehicle),
                        feats.eq(self.feats),
                        song_length.eq(self.song_length),
                        gold_threshold.eq(self.gold_threshold),
                        iss.eq(self.iss),
                        isj.eq(self.isj),
                        play_count.eq(play_count + 1),
                        submitted_at.eq(OffsetDateTime::now_utc()),
                    ))
                    .get_result::<Score>(conn)
                    .await
                    .context("Failed to update score")?;

                // Add the skill points of the new score to the Redis leaderboard
                let add_amount = updated_score.calc_skill_points();
                let _: () = redis_conn
                    .zincrby("leaderboard", add_amount.into(), updated_score.player_id)
                    .await?;

                Ok(updated_score)
            } else {
                Ok(existing_score)
            }
        } else {
            let new_score = diesel::insert_into(scores)
                .values(self)
                .get_result::<Score>(conn)
                .await
                .context("Failed to insert score")?;

            // Add the skill points of the new score to the Redis leaderboard
            let add_amount = new_score.calc_skill_points();
            let _: () = redis_conn
                .zincrby("leaderboard", add_amount.into(), new_score.player_id)
                .await?;

            Ok(new_score)
        }
    }
}
