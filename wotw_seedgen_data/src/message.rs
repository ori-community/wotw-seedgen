use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, VariantArray};
use wotw_seedgen_derive::FromStr;

/// Text alignment in messages
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
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum Alignment {
    Left = 0,
    Center = 1,
    Right = 2,
    Justify = 3,
}

/// Horizontal anchor of message boxes
///
/// Note that message boxes are bigger than their visual background, so this can produce unexpected results
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
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum HorizontalAnchor {
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Vertical anchor of message boxes
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
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum VerticalAnchor {
    Top = 0,
    Middle = 1,
    Bottom = 2,
}

/// Abstraction over [`Alignment`], [`HorizontalAnchor`] and [`VerticalAnchor`]
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
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum ScreenPosition {
    /// [`Alignment::Left`], [`HorizontalAnchor::Left`] and [`VerticalAnchor::Top`]
    TopLeft = 0,
    /// [`Alignment::Center`], [`HorizontalAnchor::Center`] and [`VerticalAnchor::Top`]
    TopCenter = 1,
    /// [`Alignment::Right`], [`HorizontalAnchor::Right`] and [`VerticalAnchor::Top`]
    TopRight = 2,
    /// [`Alignment::Left`], [`HorizontalAnchor::Left`] and [`VerticalAnchor::Middle`]
    MiddleLeft = 3,
    /// [`Alignment::Center`], [`HorizontalAnchor::Center`] and [`VerticalAnchor::Middle`]
    MiddleCenter = 4,
    /// [`Alignment::Right`], [`HorizontalAnchor::Right`] and [`VerticalAnchor::Middle`]
    MiddleRight = 5,
    /// [`Alignment::Left`], [`HorizontalAnchor::Left`] and [`VerticalAnchor::Bottom`]
    BottomLeft = 6,
    /// [`Alignment::Center`], [`HorizontalAnchor::Center`] and [`VerticalAnchor::Bottom`]
    BottomCenter = 7,
    /// [`Alignment::Right`], [`HorizontalAnchor::Right`] and [`VerticalAnchor::Bottom`]
    BottomRight = 8,
}

/// Coordinate system of message boxes
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
    Display,
    FromStr,
    VariantArray,
    Default,
)]
#[repr(u8)]
pub enum CoordinateSystem {
    Absolute = 0,
    /// Interpret coordinates on a system with (0,0) being the top left corner and (1,1) the bottom right corner of the view
    #[default]
    Relative = 1,
    /// Interpret coordinates as a world position
    World = 2,
}
