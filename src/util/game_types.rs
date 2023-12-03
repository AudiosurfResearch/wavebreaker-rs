use num_enum::TryFromPrimitive;
use rocket::form::{self, FromFormField, ValueField};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Represents the three skill levels represented on the leaderboard.
#[derive(Serialize_repr, Deserialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum League {
    Casual,
    Pro,
    Elite,
}

impl<'r> FromFormField<'r> for League {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        let num: u8 = field.value.parse()?;
        let league = match Self::try_from(num) {
            Ok(res) => res,
            _ => Err(form::Error::validation("failed to convert to League enum"))?,
        };
        Ok(league)
    }
}

/// Represents a character/vehicle in the game.
#[derive(Serialize_repr, Deserialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
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

impl<'r> FromFormField<'r> for Character {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        let num: u8 = field.value.parse()?;
        let character = match Self::try_from(num) {
            Ok(res) => res,
            _ => Err(form::Error::validation(
                "failed to convert to Character enum",
            ))?,
        };
        Ok(character)
    }
}

/// Represents the three kinds of leaderboards available in the game.
#[derive(Serialize_repr, Deserialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Leaderboard {
    Friend,
    Global,
    Nearby
}