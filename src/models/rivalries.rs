use crate::models::players::Player;
use crate::schema::rivalries;
use diesel::prelude::*;

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Player))]
#[diesel(table_name = rivalries)]
#[diesel(primary_key(player_id, rival_id))]
pub struct Rivalry {
    pub player_id: i32,
    pub rival_id: i32,
    pub established_at: time::PrimitiveDateTime,
}
