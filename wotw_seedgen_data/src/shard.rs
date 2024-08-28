use crate::UberIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumString, FromRepr};

/// Spirit Shards
///
/// Currently excludes unused shards
// TODO ^ why?
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
    EnumString,
    FromRepr,
)]
#[repr(u8)]
pub enum Shard {
    Overcharge = 1,
    TripleJump = 2,
    Wingclip = 3,
    Bounty = 4,
    Swap = 5,
    // CrescentShotDeprecated = 6,
    // Pierce = 7,
    Magnet = 8,
    Splinter = 9,
    // BlazeDeprecated = 10,
    // FrostDeprecated = 11,
    // LifeLeechDeprecated = 12,
    Reckless = 13,
    Quickshot = 14,
    // ExplosiveDeprecated = 15,
    // Ricochet = 16,
    // ClimbDeprecated = 17,
    Resilience = 18,
    SpiritLightHarvest = 19,
    // CompassDeprecated = 20,
    // WaterbreathingDeprecated = 21,
    Vitality = 22,
    LifeHarvest = 23,
    // SpiritWellShieldDeprecated = 24,
    EnergyHarvest = 25,
    Energy = 26,
    LifePact = 27,
    LastStand = 28,
    // HarvestOfLightDeprecated = 29,
    Sense = 30,
    // UnderwaterEfficiencyDeprecated = 31,
    UltraBash = 32,
    UltraGrapple = 33,
    Overflow = 34,
    Thorn = 35,
    Catalyst = 36,
    // Supressor = 37,
    Turmoil = 38,
    Sticky = 39,
    Finesse = 40,
    SpiritSurge = 41,
    // OverchargeDeprecated = 42,
    Lifeforce = 43,
    Deflector = 44,
    // Stinger = 45,
    Fracture = 46,
    Arcing = 47,
}
impl Shard {
    /// Returns the [`UberIdentifier`] tracking whether the player has this `Shard`
    pub const fn uber_identifier(self) -> UberIdentifier {
        UberIdentifier::new(25, self as i32)
    }
    /// Returns the `Shard` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        const MIN: i32 = u8::MIN as i32;
        const MAX: i32 = u8::MAX as i32;
        match uber_identifier {
            UberIdentifier {
                group: 25,
                member: id @ MIN..=MAX,
            } => Self::from_repr(id as u8),
            _ => None,
        }
    }
}
