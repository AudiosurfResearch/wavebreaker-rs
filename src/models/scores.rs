use crate::models::players::Player;
use crate::models::songs::Song;
use crate::schema::scores;
use crate::util::game_types::{Character, League};
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::SmallInt;
use diesel::{prelude::*, serialize};
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use num_enum::TryFromPrimitive;
use time::{OffsetDateTime, PrimitiveDateTime};

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
        Ok(Self::try_from_primitive(league_num)?)
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
        Ok(Self::try_from_primitive(league_num)?)
    }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Player))]
#[diesel(belongs_to(Song))]
#[diesel(table_name = scores)]
#[diesel(primary_key(id))]
pub struct Score {
    pub id: i32,
    pub player_id: i32,
    pub song_id: i32,
    pub league: League,
    pub submitted_at: time::PrimitiveDateTime,
    pub play_count: i32,
    pub score: i32,
    pub track_shape: Vec<Option<i32>>,
    pub xstats: Vec<Option<i32>>,
    pub density: i32,
    pub vehicle: Character,
    pub feats: Vec<Option<String>>,
    pub song_length: i32,
    pub gold_threshold: i32,
    pub iss: i32,
    pub isj: i32,
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
