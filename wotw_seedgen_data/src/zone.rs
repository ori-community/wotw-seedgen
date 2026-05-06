use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, VariantArray};
use utoipa::ToSchema;
use wotw_seedgen_derive::FromStr;

/// World zones as indicated on the map
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
pub enum Zone {
    Marsh = 0,
    Hollow = 1,
    Glades = 2,
    Wellspring = 3,
    Woods = 4,
    Reach = 5,
    Depths = 6,
    Pools = 7,
    Wastes = 8,
    Ruins = 9,
    Willow = 10,
    Burrows = 11,
    Shop = 12,
    Void = 13,
    Spawn = 14,
}
