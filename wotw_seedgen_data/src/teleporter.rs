use std::fmt::{self, Display};

use crate::UberIdentifier;
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
    pub const MARSH_ID: UberIdentifier = UberIdentifier::new(21786, 10185);
    pub const DEN_ID: UberIdentifier = UberIdentifier::new(11666, 61594);
    pub const HOLLOW_ID: UberIdentifier = UberIdentifier::new(937, 26601);
    pub const GLADES_ID: UberIdentifier = UberIdentifier::new(42178, 42096);
    pub const WELLSPRING_ID: UberIdentifier = UberIdentifier::new(53632, 18181);
    pub const BURROWS_ID: UberIdentifier = UberIdentifier::new(24922, 42531);
    pub const WOODS_ENTRANCE_ID: UberIdentifier = UberIdentifier::new(58674, 7071);
    pub const WOODS_EXIT_ID: UberIdentifier = UberIdentifier::new(58674, 1965);
    pub const REACH_ID: UberIdentifier = UberIdentifier::new(28895, 54235);
    pub const DEPTHS_ID: UberIdentifier = UberIdentifier::new(18793, 38871);
    pub const CENTRAL_POOLS_ID: UberIdentifier = UberIdentifier::new(945, 58183);
    pub const POOLS_BOSS_ID: UberIdentifier = UberIdentifier::new(945, 1370);
    pub const FEEDING_GROUNDS_ID: UberIdentifier = UberIdentifier::new(58674, 10029);
    pub const CENTRAL_WASTES_ID: UberIdentifier = UberIdentifier::new(20120, 49994);
    pub const OUTER_RUINS_ID: UberIdentifier = UberIdentifier::new(20120, 41398);
    pub const INNER_RUINS_ID: UberIdentifier = UberIdentifier::new(10289, 4928);
    pub const WILLOW_ID: UberIdentifier = UberIdentifier::new(16155, 41465);
    pub const SHRIEK_ID: UberIdentifier = UberIdentifier::new(16155, 50867);

    /// Returns the [`UberIdentifier`] tracking whether the player has activated this `Teleporter`
    pub const fn uber_identifier(self) -> UberIdentifier {
        match self {
            Self::Marsh => Self::MARSH_ID,
            Self::Den => Self::DEN_ID,
            Self::Hollow => Self::HOLLOW_ID,
            Self::Glades => Self::GLADES_ID,
            Self::Wellspring => Self::WELLSPRING_ID,
            Self::Burrows => Self::BURROWS_ID,
            Self::WoodsEntrance => Self::WOODS_ENTRANCE_ID,
            Self::WoodsExit => Self::WOODS_EXIT_ID,
            Self::Reach => Self::REACH_ID,
            Self::Depths => Self::DEPTHS_ID,
            Self::CentralPools => Self::CENTRAL_POOLS_ID,
            Self::PoolsBoss => Self::POOLS_BOSS_ID,
            Self::FeedingGrounds => Self::FEEDING_GROUNDS_ID,
            Self::CentralWastes => Self::CENTRAL_WASTES_ID,
            Self::OuterRuins => Self::OUTER_RUINS_ID,
            Self::InnerRuins => Self::INNER_RUINS_ID,
            Self::Willow => Self::WILLOW_ID,
            Self::Shriek => Self::SHRIEK_ID,
        }
    }
    /// Returns the `Teleporter` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        match uber_identifier {
            Self::MARSH_ID => Some(Self::Marsh),
            Self::DEN_ID => Some(Self::Den),
            Self::HOLLOW_ID => Some(Self::Hollow),
            Self::GLADES_ID => Some(Self::Glades),
            Self::WELLSPRING_ID => Some(Self::Wellspring),
            Self::BURROWS_ID => Some(Self::Burrows),
            Self::WOODS_ENTRANCE_ID => Some(Self::WoodsEntrance),
            Self::WOODS_EXIT_ID => Some(Self::WoodsExit),
            Self::REACH_ID => Some(Self::Reach),
            Self::DEPTHS_ID => Some(Self::Depths),
            Self::CENTRAL_POOLS_ID => Some(Self::CentralPools),
            Self::POOLS_BOSS_ID => Some(Self::PoolsBoss),
            Self::FEEDING_GROUNDS_ID => Some(Self::FeedingGrounds),
            Self::CENTRAL_WASTES_ID => Some(Self::CentralWastes),
            Self::OUTER_RUINS_ID => Some(Self::OuterRuins),
            Self::INNER_RUINS_ID => Some(Self::InnerRuins),
            Self::WILLOW_ID => Some(Self::Willow),
            Self::SHRIEK_ID => Some(Self::Shriek),
            _ => None,
        }
    }
}
