#[cfg(feature = "serde")]
use serde_repr::{Deserialize_repr, Serialize_repr};
#[cfg(feature = "strum")]
use strum::{Display, EnumString};

/// Text alignment in messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize_repr, Serialize_repr))]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[repr(u8)]
pub enum Alignment {
    Left = 0,
    Center = 1,
    Right = 2,
    Justify = 3,
}
/// Message box position relative to the camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize_repr, Serialize_repr))]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[repr(u8)]
pub enum ScreenPosition {
    TopLeft = 0,
    TopCenter = 1,
    TopRight = 2,
    MiddleLeft = 3,
    MiddleCenter = 4,
    MiddleRight = 5,
    BottomLeft = 6,
    BottomCenter = 7,
    BottomRight = 8,
}
