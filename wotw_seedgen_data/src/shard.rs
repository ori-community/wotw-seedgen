use crate::UberIdentifier;
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, FromRepr, VariantArray};
use wotw_seedgen_derive::FromStr;

/// Spirit Shards
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
pub enum Shard {
    Overcharge = 1,
    TripleJump = 2,
    Wingclip = 3,
    Bounty = 4,
    Swap = 5,
    CrescentShotDeprecated = 6,
    Pierce = 7,
    Magnet = 8,
    Splinter = 9,
    BlazeDeprecated = 10,
    FrostDeprecated = 11,
    LifeLeechDeprecated = 12,
    Reckless = 13,
    Quickshot = 14,
    ExplosiveDeprecated = 15,
    Ricochet = 16,
    ClimbDeprecated = 17,
    Resilience = 18,
    SpiritLightHarvest = 19,
    CompassDeprecated = 20,
    WaterbreathingDeprecated = 21,
    Vitality = 22,
    LifeHarvest = 23,
    SpiritWellShieldDeprecated = 24,
    EnergyHarvest = 25,
    Energy = 26,
    LifePact = 27,
    LastStand = 28,
    HarvestOfLightDeprecated = 29,
    Sense = 30,
    UnderwaterEfficiencyDeprecated = 31,
    UltraBash = 32,
    UltraGrapple = 33,
    Overflow = 34,
    Thorn = 35,
    Catalyst = 36,
    Supressor = 37,
    Turmoil = 38,
    Sticky = 39,
    Finesse = 40,
    SpiritSurge = 41,
    OverchargeDeprecated = 42,
    Lifeforce = 43,
    Deflector = 44,
    Stinger = 45,
    Fracture = 46,
    Arcing = 47,
}

impl Shard {
    pub const OVERCHARGE_ID: UberIdentifier = Self::Overcharge.uber_identifier();
    pub const TRIPLE_JUMP_ID: UberIdentifier = Self::TripleJump.uber_identifier();
    pub const WINGCLIP_ID: UberIdentifier = Self::Wingclip.uber_identifier();
    pub const BOUNTY_ID: UberIdentifier = Self::Bounty.uber_identifier();
    pub const SWAP_ID: UberIdentifier = Self::Swap.uber_identifier();
    // pub const CRESCENT_SHOT_DEPRECATED_ID: UberIdentifier = Self::CrescentShotDeprecated.uber_identifier();
    // pub const PIERCE_ID: UberIdentifier = Self::Pierce.uber_identifier();
    pub const MAGNET_ID: UberIdentifier = Self::Magnet.uber_identifier();
    pub const SPLINTER_ID: UberIdentifier = Self::Splinter.uber_identifier();
    // pub const BLAZE_DEPRECATED_ID: UberIdentifier = Self::BlazeDeprecated.uber_identifier();
    // pub const FROST_DEPRECATED_ID: UberIdentifier = Self::FrostDeprecated.uber_identifier();
    // pub const LIFE_LEECH_DEPRECATED_ID: UberIdentifier = Self::LifeLeechDeprecated.uber_identifier();
    pub const RECKLESS_ID: UberIdentifier = Self::Reckless.uber_identifier();
    pub const QUICKSHOT_ID: UberIdentifier = Self::Quickshot.uber_identifier();
    // pub const EXPLOSIVE_DEPRECATED_ID: UberIdentifier = Self::ExplosiveDeprecated.uber_identifier();
    // pub const RICOCHET_ID: UberIdentifier = Self::Ricochet.uber_identifier();
    // pub const CLIMB_DEPRECATED_ID: UberIdentifier = Self::ClimbDeprecated.uber_identifier();
    pub const RESILIENCE_ID: UberIdentifier = Self::Resilience.uber_identifier();
    pub const SPIRIT_LIGHT_HARVEST_ID: UberIdentifier = Self::SpiritLightHarvest.uber_identifier();
    // pub const COMPASS_DEPRECATED_ID: UberIdentifier = Self::CompassDeprecated.uber_identifier();
    // pub const WATERBREATHING_DEPRECATED_ID: UberIdentifier = Self::WaterbreathingDeprecated.uber_identifier();
    pub const VITALITY_ID: UberIdentifier = Self::Vitality.uber_identifier();
    pub const LIFE_HARVEST_ID: UberIdentifier = Self::LifeHarvest.uber_identifier();
    // pub const SPIRIT_WELL_SHIELD_DEPRECATED_ID: UberIdentifier = Self::SpiritWellShieldDeprecated.uber_identifier();
    pub const ENERGY_HARVEST_ID: UberIdentifier = Self::EnergyHarvest.uber_identifier();
    pub const ENERGY_ID: UberIdentifier = Self::Energy.uber_identifier();
    pub const LIFE_PACT_ID: UberIdentifier = Self::LifePact.uber_identifier();
    pub const LAST_STAND_ID: UberIdentifier = Self::LastStand.uber_identifier();
    // pub const HARVEST_OF_LIGHT_DEPRECATED_ID: UberIdentifier = Self::HarvestOfLightDeprecated.uber_identifier();
    pub const SENSE_ID: UberIdentifier = Self::Sense.uber_identifier();
    // pub const UNDERWATER_EFFICIENCY_DEPRECATED_ID: UberIdentifier = Self::UnderwaterEfficiencyDeprecated.uber_identifier();
    pub const ULTRA_BASH_ID: UberIdentifier = Self::UltraBash.uber_identifier();
    pub const ULTRA_GRAPPLE_ID: UberIdentifier = Self::UltraGrapple.uber_identifier();
    pub const OVERFLOW_ID: UberIdentifier = Self::Overflow.uber_identifier();
    pub const THORN_ID: UberIdentifier = Self::Thorn.uber_identifier();
    pub const CATALYST_ID: UberIdentifier = Self::Catalyst.uber_identifier();
    // pub const SUPRESSOR_ID: UberIdentifier = Self::Supressor.uber_identifier();
    pub const TURMOIL_ID: UberIdentifier = Self::Turmoil.uber_identifier();
    pub const STICKY_ID: UberIdentifier = Self::Sticky.uber_identifier();
    pub const FINESSE_ID: UberIdentifier = Self::Finesse.uber_identifier();
    pub const SPIRIT_SURGE_ID: UberIdentifier = Self::SpiritSurge.uber_identifier();
    // pub const OVERCHARGE_DEPRECATED_ID: UberIdentifier = Self::OverchargeDeprecated.uber_identifier();
    pub const LIFEFORCE_ID: UberIdentifier = Self::Lifeforce.uber_identifier();
    pub const DEFLECTOR_ID: UberIdentifier = Self::Deflector.uber_identifier();
    // pub const STINGER_ID: UberIdentifier = Self::Stinger.uber_identifier();
    pub const FRACTURE_ID: UberIdentifier = Self::Fracture.uber_identifier();
    pub const ARCING_ID: UberIdentifier = Self::Arcing.uber_identifier();

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
