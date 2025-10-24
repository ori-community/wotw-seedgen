use crate::UberIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, FromRepr, VariantArray};
use wotw_seedgen_derive::FromStr;

// TODO maybe descriptions should exist for random placements as well? Supposedly these are the weapon upgrade descriptions:
// "Increases Sentry attack speed"
// "Drop attacks with Hammer create a shockwave"
// "Tap to pause the Shuriken's flight and spin it in place"
// "Spike explodes on hit"
// "Charge up a flame to damage and set all enemies on fire"
/// Opher weapon upgrades
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
    FromRepr,
    VariantArray,
)]
#[repr(u8)]
pub enum WeaponUpgrade {
    ExplodingSpear = 0,
    HammerShockwave = 1,
    StaticShuriken = 2,
    ChargeBlaze = 3,
    RapidSentry = 4,
}

impl WeaponUpgrade {
    pub const EXPLODING_SPEAR_ID: UberIdentifier = UberIdentifier::new(3440, 5687);
    pub const SHOCK_HAMMER_ID: UberIdentifier = UberIdentifier::new(3440, 46488);
    pub const STATIC_SHURIKEN_ID: UberIdentifier = UberIdentifier::new(3440, 10776);
    pub const CHARGE_BLAZE_ID: UberIdentifier = UberIdentifier::new(3440, 61898);
    pub const RAPID_SENTRY_ID: UberIdentifier = UberIdentifier::new(3440, 57376);

    /// Returns the [`UberIdentifier`] tracking whether the player has this `WeaponUpgrade`
    pub const fn uber_identifier(self) -> UberIdentifier {
        match self {
            Self::ExplodingSpear => Self::EXPLODING_SPEAR_ID,
            Self::HammerShockwave => Self::SHOCK_HAMMER_ID,
            Self::StaticShuriken => Self::STATIC_SHURIKEN_ID,
            Self::ChargeBlaze => Self::CHARGE_BLAZE_ID,
            Self::RapidSentry => Self::RAPID_SENTRY_ID,
        }
    }

    /// Returns the `WeaponUpgrade` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        match uber_identifier {
            Self::EXPLODING_SPEAR_ID => Some(Self::ExplodingSpear),
            Self::SHOCK_HAMMER_ID => Some(Self::HammerShockwave),
            Self::STATIC_SHURIKEN_ID => Some(Self::StaticShuriken),
            Self::CHARGE_BLAZE_ID => Some(Self::ChargeBlaze),
            Self::RAPID_SENTRY_ID => Some(Self::RapidSentry),
            _ => None,
        }
    }
}
