use std::fmt::{self, Display};

use crate::{uber_identifier::teleporter, UberIdentifier};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::VariantArray;
use wotw_seedgen_derive::FromStr;

/// Spirit Wells which exist in the base game
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
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum Teleporter {
    Marsh = 16,
    Den = 1,
    Hollow = 5,
    Glades = 17,
    Wellspring = 3,
    Burrows = 0,
    WoodsEntrance = 7,
    WoodsExit = 8,
    Reach = 4,
    Depths = 6,
    CentralPools = 2,
    PoolsBoss = 13,
    FeedingGrounds = 9,
    CentralWastes = 10,
    OuterRuins = 11,
    InnerRuins = 14,
    Willow = 12,
    Shriek = 15,
}
impl Display for Teleporter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Teleporter::Marsh => write!(f, "Marsh")?,
            Teleporter::Den => write!(f, "Den")?,
            Teleporter::Hollow => write!(f, "Hollow")?,
            Teleporter::Glades => write!(f, "Glades")?,
            Teleporter::Wellspring => write!(f, "Wellspring")?,
            Teleporter::Burrows => write!(f, "Burrows")?,
            Teleporter::WoodsEntrance => write!(f, "WoodsEntrance")?,
            Teleporter::WoodsExit => write!(f, "WoodsExit")?,
            Teleporter::Reach => write!(f, "Reach")?,
            Teleporter::Depths => write!(f, "Depths")?,
            Teleporter::CentralPools => write!(f, "CentralPools")?,
            Teleporter::PoolsBoss => write!(f, "PoolsBoss")?,
            Teleporter::FeedingGrounds => write!(f, "FeedingGrounds")?,
            Teleporter::CentralWastes => write!(f, "CentralWastes")?,
            Teleporter::OuterRuins => write!(f, "OuterRuins")?,
            Teleporter::InnerRuins => write!(f, "InnerRuins")?,
            Teleporter::Willow => write!(f, "Willow")?,
            Teleporter::Shriek => write!(f, "Shriek")?,
        }
        write!(f, "TP")
    }
}
impl Teleporter {
    /// Returns the [`UberIdentifier`] tracking whether the player has activated this `Teleporter`
    pub const fn uber_identifier(self) -> UberIdentifier {
        match self {
            Teleporter::Marsh => teleporter::MARSH,
            Teleporter::Den => teleporter::DEN,
            Teleporter::Hollow => teleporter::HOLLOW,
            Teleporter::Glades => teleporter::GLADES,
            Teleporter::Wellspring => teleporter::WELLSPRING,
            Teleporter::Burrows => teleporter::BURROWS,
            Teleporter::WoodsEntrance => teleporter::WOODS_ENTRANCE,
            Teleporter::WoodsExit => teleporter::WOODS_EXIT,
            Teleporter::Reach => teleporter::REACH,
            Teleporter::Depths => teleporter::DEPTHS,
            Teleporter::CentralPools => teleporter::CENTRAL_POOLS,
            Teleporter::PoolsBoss => teleporter::POOLS_BOSS,
            Teleporter::FeedingGrounds => teleporter::FEEDING_GROUNDS,
            Teleporter::CentralWastes => teleporter::CENTRAL_WASTES,
            Teleporter::OuterRuins => teleporter::OUTER_RUINS,
            Teleporter::InnerRuins => teleporter::INNER_RUINS,
            Teleporter::Willow => teleporter::WILLOW,
            Teleporter::Shriek => teleporter::SHRIEK,
        }
    }
    /// Returns the `Teleporter` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        match uber_identifier {
            teleporter::MARSH => Some(Teleporter::Marsh),
            teleporter::DEN => Some(Teleporter::Den),
            teleporter::HOLLOW => Some(Teleporter::Hollow),
            teleporter::GLADES => Some(Teleporter::Glades),
            teleporter::WELLSPRING => Some(Teleporter::Wellspring),
            teleporter::BURROWS => Some(Teleporter::Burrows),
            teleporter::WOODS_ENTRANCE => Some(Teleporter::WoodsEntrance),
            teleporter::WOODS_EXIT => Some(Teleporter::WoodsExit),
            teleporter::REACH => Some(Teleporter::Reach),
            teleporter::DEPTHS => Some(Teleporter::Depths),
            teleporter::CENTRAL_POOLS => Some(Teleporter::CentralPools),
            teleporter::POOLS_BOSS => Some(Teleporter::PoolsBoss),
            teleporter::FEEDING_GROUNDS => Some(Teleporter::FeedingGrounds),
            teleporter::CENTRAL_WASTES => Some(Teleporter::CentralWastes),
            teleporter::OUTER_RUINS => Some(Teleporter::OuterRuins),
            teleporter::INNER_RUINS => Some(Teleporter::InnerRuins),
            teleporter::WILLOW => Some(Teleporter::Willow),
            teleporter::SHRIEK => Some(Teleporter::Shriek),
            _ => None,
        }
    }
}
