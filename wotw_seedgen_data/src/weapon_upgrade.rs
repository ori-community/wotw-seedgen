use crate::{uber_identifier::weapon_upgrade, UberIdentifier};
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
    /// Returns the [`UberIdentifier`] tracking whether the player has this `WeaponUpgrade`
    pub const fn uber_identifier(self) -> UberIdentifier {
        match self {
            WeaponUpgrade::ExplodingSpear => weapon_upgrade::EXPLODING_SPEAR,
            WeaponUpgrade::HammerShockwave => weapon_upgrade::SHOCK_HAMMER,
            WeaponUpgrade::StaticShuriken => weapon_upgrade::STATIC_SHURIKEN,
            WeaponUpgrade::ChargeBlaze => weapon_upgrade::CHARGE_BLAZE,
            WeaponUpgrade::RapidSentry => weapon_upgrade::RAPID_SENTRY,
        }
    }
    /// Returns the `WeaponUpgrade` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        match uber_identifier {
            weapon_upgrade::EXPLODING_SPEAR => Some(WeaponUpgrade::ExplodingSpear),
            weapon_upgrade::SHOCK_HAMMER => Some(WeaponUpgrade::HammerShockwave),
            weapon_upgrade::STATIC_SHURIKEN => Some(WeaponUpgrade::StaticShuriken),
            weapon_upgrade::CHARGE_BLAZE => Some(WeaponUpgrade::ChargeBlaze),
            weapon_upgrade::RAPID_SENTRY => Some(WeaponUpgrade::RapidSentry),
            _ => None,
        }
    }
}
