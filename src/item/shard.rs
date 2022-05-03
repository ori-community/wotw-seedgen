use num_enum::TryFromPrimitive;
use seedgen_derive::FromStr;

use crate::util::Icon;

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum Shard {
    Overcharge = 1,
    TripleJump = 2,
    Wingclip = 3,
    Bounty = 4,
    Swap = 5,
//  CrescentShotDeprecated = 6,
//  Pierce = 7,
    Magnet = 8,
    Splinter = 9,
//  BlazeDeprecated = 10,
//  FrostDeprecated = 11,
//  LifeLeechDeprecated = 12,
    Reckless = 13,
    Quickshot = 14,
//  ExplosiveDeprecated = 15,
//  Ricochet = 16,
//  ClimbDeprecated = 17,
    Resilience = 18,
    SpiritLightHarvest = 19,
//  CompassDeprecated = 20,
//  WaterbreathingDeprecated = 21,
    Vitality = 22,
    LifeHarvest = 23,
//  SpiritWellShieldDeprecated = 24,
    EnergyHarvest = 25,
    Energy = 26,
    LifePact = 27,
    LastStand = 28,
//  HarvestOfLightDeprecated = 29,
    Sense = 30,
//  UnderwaterEfficiencyDeprecated = 31,
    UltraBash = 32,
    UltraGrapple = 33,
    Overflow = 34,
    Thorn = 35,
    Catalyst = 36,
//  Supressor = 37,
    Turmoil = 38,
    Sticky = 39,
    Finesse = 40,
    SpiritSurge = 41,
//  OverchargeDeprecated = 42,
    Lifeforce = 43,
    Deflector = 44,
//  Stinger = 45,
    Fracture = 46,
    Arcing = 47,
}
impl Shard {
    pub fn icon(self) -> Option<Icon> {
        Some(Icon::Shard(self))
    }
}
