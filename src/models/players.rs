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
