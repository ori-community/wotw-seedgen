use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, VariantArray};
use utoipa::ToSchema;
use wotw_seedgen_derive::FromStr;

// TODO should this have a custom PartialOrd implementation?
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
    Woods = 7,
    Reach = 6,
    Depths = 8,
    Pools = 4,
    Wastes = 9,
    Ruins = 10,
    Willow = 11,
    Burrows = 5,
    Spawn = 14,
    Shop = 12,
    Void = 13,
}
