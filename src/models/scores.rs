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
use serde::Serialize;
use time::{OffsetDateTime, PrimitiveDateTime};

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

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Serialize)]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Song))]
#[diesel(table_name = scores, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
pub struct Score {
    pub id: i32,
    pub song_id: i32,
    pub player_id: i32,
    pub league: League,
    pub submitted_at: time::PrimitiveDateTime,
    pub play_count: i32,
    pub score: i32,
    pub track_shape: Vec<i32>,
    pub xstats: Vec<i32>,
    pub density: i32,
    pub vehicle: Character,
    pub feats: Vec<String>,
    pub song_length: i32,
    pub gold_threshold: i32,
    pub iss: i32,
    pub isj: i32,
}

impl Score {
    /// Calculates and returns the skill points the player earned for this score.
    pub fn get_skill_points(&self) -> u32 {
        let multiplier = (self.league as u32 + 1) * 100;
        ((self.score as f64 / self.gold_threshold as f64) * multiplier as f64).round() as u32
    }

    /// Retrieves the scores for a specific song and league, for display in-game.
    ///
    /// # Arguments
    ///
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// * `conn` - The database connection.
    ///
    /// # Returns
    ///
    /// A vector of `ScoreWithPlayer` structs.
    ///
    /// # Errors
    ///
    /// This fails if the database query fails.
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

    /// Retrieves all rivals' scores for a specific song and league, for display in-game.
    ///
    /// # Arguments
    ///
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// *  `rival_ids` - The IDs of the rivals to filter scores by.
    /// * `conn` - The database connection.
    ///
    /// # Returns
    ///
    /// A vector of `ScoreWithPlayer` structs.
    ///
    /// # Errors
    ///
    /// This fails if the database query fails.
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
    ///
    /// * `find_song_id` - The ID of the song to find scores for.
    /// * `find_league` - The league to filter scores by.
    /// * `conn` - The database connection.
    ///
    /// # Returns
    ///
    /// A vector of `ScoreWithPlayer` structs.
    ///
    /// # Errors
    ///
    /// This fails if the database query fails.
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
    ///
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
    ///
    /// # Returns
    ///
    /// A new instance of `NewScore`.
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
    ///
    /// * `conn` - The database connection.
    ///
    /// # Returns
    ///
    /// The created or updated score entry.
    ///
    /// # Errors
    ///
    /// This fails if:
    /// - The database connection fails
    /// - The score fails to be created/retrieved
    pub async fn create_or_update(&self, conn: &mut AsyncPgConnection) -> QueryResult<Score> {
        use crate::schema::scores::dsl::*;

        let existing_score = scores
            .filter(player_id.eq(self.player_id))
            .filter(song_id.eq(self.song_id))
            .filter(league.eq(self.league))
            .first::<Score>(conn)
            .await;

        match existing_score {
            Ok(existing_score) => {
                if existing_score.score < self.score {
                    let offset_time = OffsetDateTime::now_utc();

                    diesel::update(scores)
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
                            submitted_at.eq(PrimitiveDateTime::new(
                                offset_time.date(),
                                offset_time.time(),
                            )),
                        ))
                        .get_result::<Score>(conn)
                        .await
                } else {
                    Ok(existing_score)
                }
            }
            Err(_) => {
                diesel::insert_into(scores)
                    .values(self)
                    .get_result::<Score>(conn)
                    .await
            }
        }
    }
}
