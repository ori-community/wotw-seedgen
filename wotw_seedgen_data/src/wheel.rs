#[cfg(feature = "serde")]
use serde_repr::{Deserialize_repr, Serialize_repr};
#[cfg(feature = "strum")]
use strum::{Display, EnumString};

/// Positioning in a weapon wheel like menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize_repr, Serialize_repr))]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[repr(u8)]
pub enum WheelItemPosition {
    Top = 0,
    TopRight = 1,
    RightTop = 2,
    Right = 3,
    RightBottom = 4,
    BottomRight = 5,
    Bottom = 6,
    BottomLeft = 7,
    LeftBottom = 8,
    Left = 9,
    LeftTop = 10,
    TopLeft = 11,
}
/// Possible input configurations for an item in a weapon wheel like menu
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize_repr, Serialize_repr))]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[repr(u8)]
pub enum WheelBind {
    All = 0,
    Ability1 = 1,
    Ability2 = 2,
    Ability3 = 3,
}
