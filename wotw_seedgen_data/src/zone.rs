#[cfg(feature = "serde")]
use serde_repr::{Deserialize_repr, Serialize_repr};
#[cfg(feature = "strum")]
use strum::{Display, EnumString};

// TODO should this have a custom PartialOrd implementation?
/// World zones as indicated on the map
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize_repr, Serialize_repr))]
#[cfg_attr(feature = "strum", derive(Display, EnumString))]
#[repr(u8)]
pub enum Zone {
    Inkwater = 0,
    Hollow = 1,
    Glades = 2,
    Wellspring = 3,
    Woods = 7,
    Reach = 6,
    Depths = 8,
    Luma = 4,
    Wastes = 9,
    Ruins = 10,
    Willow = 11,
    Burrows = 5,
    Spawn = 14,
    Shop = 12,
    Void = 13,
}
