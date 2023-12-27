use crate::models::players::Player;
use crate::schema::rivalries;
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Player))]
#[diesel(table_name = rivalries)]
#[diesel(primary_key(player_id, rival_id))]
pub struct Rivalry {
    pub player_id: i32,
    pub rival_id: i32,
    pub established_at: time::PrimitiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = rivalries)]
pub struct NewRivalry {
    pub player_id: i32,
    pub rival_id: i32,
}

/// Represents a new rivalry between two players.
///
/// This struct is used to create a new rivalry by specifying the player ID and rival ID.
/// It provides a method to asynchronously create the rivalry in the database.
impl NewRivalry {
    /// Creates a new `NewRivalry` instance.
    ///
    /// # Arguments
    ///
    /// * `player_id` - The ID of the player.
    /// * `rival_id` - The ID of the rival.
    ///
    /// # Returns
    ///
    /// A new `NewRivalry` instance.
    #[must_use]
    pub const fn new(player_id: i32, rival_id: i32) -> Self {
        Self {
            player_id,
            rival_id,
        }
    }

    /// Creates a new rivalry in the database.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Returns
    ///
    /// A `QueryResult` containing the created `Rivalry` instance.
    ///
    /// # Errors
    /// This fails if:
    /// - The query fails
    pub async fn create(&self, conn: &mut AsyncPgConnection) -> QueryResult<Rivalry> {
        diesel::insert_into(rivalries::table)
            .values(self)
            .get_result(conn)
            .await
    }
}
