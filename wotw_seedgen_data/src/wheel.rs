use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, VariantArray};
use utoipa::ToSchema;
use wotw_seedgen_derive::FromStr;

/// Positioning in a weapon wheel like menu
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize_repr,
    Serialize_repr,
    ToSchema,
    Display,
    FromStr,
    VariantArray,
)]
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

// TODO shifting `All` to 3 would allow seemless conversion from `EquipSlot` to `WheelBind`
/// Possible input configurations for an item in a weapon wheel like menu
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize_repr,
    Serialize_repr,
    ToSchema,
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum WheelBind {
    Ability1 = 0,
    Ability2 = 1,
    Ability3 = 2,
}
