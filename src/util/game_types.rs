use num_enum::TryFromPrimitive;
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Represents the three skill levels represented on the leaderboard.
#[derive(Serialize_repr, Deserialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum League {
    Casual,
    Pro,
    Elite,
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

/// Represents the three kinds of leaderboards available in the game.
#[derive(Serialize_repr, Deserialize_repr, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Leaderboard {
    Friend,
    Global,
    Nearby
}