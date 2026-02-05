use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::players::PlayerPublic;
use crate::{
    models::players::{AccountType, Player},
    schema::rivalries,
};

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Player, foreign_key = challenger_id))]
#[diesel(table_name = rivalries, check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(challenger_id, rival_id))]
/// Represents a rivalry between two players.
/// Rivalries on Wavebreaker are similar to certain BEMANI games, where rivalries can be one-sided
/// Challenger is the player who initiated the rivalry.
pub struct Rivalry {
    pub challenger_id: i32,
    pub rival_id: i32,
    pub established_at: time::OffsetDateTime,
}

impl Rivalry {
    /// Find out if the players added *each others* as rivals.
    pub async fn is_mutual(&self, conn: &mut AsyncPgConnection) -> bool {
        use crate::schema::rivalries::dsl::*;

        rivalries
            .filter(challenger_id.eq(self.rival_id))
            .filter(rival_id.eq(self.challenger_id))
            .get_result::<Self>(conn)
            .await
            .is_ok()
    }
}

#[derive(Insertable)]
#[diesel(table_name = rivalries)]
pub struct NewRivalry {
    pub challenger_id: i32,
    pub rival_id: i32,
}

impl NewRivalry {
    /// # Arguments
    /// * `player_id` - The ID of the player.
    /// * `rival_id` - The ID of the rival.
    ///
    /// # Returns
    /// A new `NewRivalry` instance.
    #[must_use]
    pub const fn new(challenger_id: i32, rival_id: i32) -> Self {
        Self {
            challenger_id,
            rival_id,
        }
    }

    /// Creates a new rivalry in the database.
    ///
    /// # Arguments
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Returns
    /// A `QueryResult` containing the created `Rivalry` instance.
    pub async fn create(&self, conn: &mut AsyncPgConnection) -> QueryResult<Rivalry> {
        diesel::insert_into(rivalries::table)
            .values(self)
            .get_result(conn)
            .await
    }
}

#[derive(Queryable, Deserialize, Serialize, ToSchema)]
#[diesel(table_name = rivalries, check_for_backend(diesel::pg::Pg))]
#[serde(rename_all = "camelCase")]
#[schema(examples(json!(RivalryView {
    established_at: time::OffsetDateTime::from_unix_timestamp(1684868184).unwrap(),
    rival: PlayerPublic {
        id: 1,
        username: "m1nt_".to_owned(),
        account_type: AccountType::Team,
        joined_at: time::OffsetDateTime::from_unix_timestamp(1684868184).unwrap(),
        avatar_url: "https://avatars.akamai.steamstatic.com/fadb7d90654a38c196422ed308fc931f96440dde_full.jpg".to_owned()
    }
})))]
pub struct RivalryView {
    #[serde(deserialize_with = "time::serde::iso8601::deserialize")]
    #[serde(serialize_with = "time::serde::iso8601::serialize")]
    pub established_at: time::OffsetDateTime,
    #[diesel(embed)]
    pub rival: PlayerPublic,
}

impl RivalryView {
    /// Creates a `RivalryView` that shows the **rival's** public profile
    pub async fn from_rivalry(rivalry: Rivalry, conn: &mut AsyncPgConnection) -> QueryResult<Self> {
        use crate::schema::players::dsl::*;

        let rival = players
            .find(rivalry.rival_id)
            .first::<Player>(conn)
            .await?
            .into();
        Ok(Self {
            established_at: rivalry.established_at,
            rival,
        })
    }

    /// Creates a `RivalryView` that shows the **challenger's** public profile
    pub async fn from_rivalry_challenger(
        rivalry: Rivalry,
        conn: &mut AsyncPgConnection,
    ) -> QueryResult<Self> {
        use crate::schema::players::dsl::*;

        let challenger = players
            .find(rivalry.challenger_id)
            .first::<Player>(conn)
            .await?
            .into();
        Ok(Self {
            established_at: rivalry.established_at,
            rival: challenger,
        })
    }
}
