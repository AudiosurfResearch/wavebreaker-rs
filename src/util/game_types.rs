use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::ToSchema;

/// Represents the three skill levels represented on the leaderboard.
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
pub enum League {
    Casual,
    Pro,
    Elite,
}

/// Represents a character/vehicle in the game.
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
pub enum Character {
    PointmanPro = 0,
    DoubleVisionPro = 1,
    Vegas = 2,
    Pusher = 3,
    Eraser = 4,
    //5-8 are unused
    DoubleVision = 9,
    PointmanElite = 10,
    MonoPro = 11,
    EraserElite = 12,
    NinjaMono = 13,
    DoubleVisionElite = 14,
    Pointman = 15,
    PusherElite = 16,
    Mono = 17,
}

/// Represents the three kinds of leaderboards available in the game.
#[derive(Deserialize_repr, Serialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Leaderboard {
    Friend,
    Global,
    Nearby,
}

/// Split a string with values separated by 'x' into a vector of the values.
pub fn split_x_separated<T>(s: &str) -> Result<Vec<T>, T::Err>
where
    T: std::str::FromStr,
{
    if s.is_empty() {
        return Ok(vec![]);
    }

    //If string ends with 'x', remove it
    let s = s.strip_suffix('x').unwrap_or(s);
    s.split('x')
        .map(str::parse::<T>)
        .collect::<Result<Vec<T>, T::Err>>()
}

pub fn join_x_separated<T>(v: &[T]) -> String
where
    T: std::fmt::Display,
{
    let mut result = v
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>()
        .join("x");
    result.push('x');
    result
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_x_separated() {
        // Test case 1: Valid input
        let input1 = "1x2x3x4x";
        let expected1 = vec![1, 2, 3, 4];
        assert_eq!(split_x_separated::<i32>(input1).unwrap(), expected1);

        // Test case 2: Empty input
        let input2 = "";
        let expected2: Vec<i32> = vec![];
        assert_eq!(split_x_separated::<i32>(input2).unwrap(), expected2);

        // Test case 3: Invalid input
        let input3 = "1x2x3xAAAx";
        assert!(split_x_separated::<i32>(input3).is_err());
    }

    #[test]
    fn test_join_x_separated() {
        // Test case 1: Valid input
        let input1 = vec![1, 2, 3, 4];
        let expected1 = "1x2x3x4x";
        assert_eq!(join_x_separated(&input1), expected1);

        // Test case 2: Empty input
        let input2: Vec<i32> = vec![];
        let expected2 = "x";
        assert_eq!(join_x_separated(&input2), expected2);
    }
}
