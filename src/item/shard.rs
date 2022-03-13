use num_enum::TryFromPrimitive;

use crate::util::Icon;

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Shard {
    Overcharge = 1,
    TripleJump = 2,
    Wingclip = 3,
    Bounty = 4,
    Swap = 5,
    Magnet = 8,
    Splinter = 9,
    Reckless = 13,
    Quickshot = 14,
    Resilience = 18,
    SpiritLightHarvest = 19,
    Vitality = 22,
    LifeHarvest = 23,
    EnergyHarvest = 25,
    Energy = 26,
    LifePact = 27,
    LastStand = 28,
    Sense = 30,
    UltraBash = 32,
    UltraGrapple = 33,
    Overflow = 34,
    Thorn = 35,
    Catalyst = 36,
    Turmoil = 38,
    Sticky = 39,
    Finesse = 40,
    SpiritSurge = 41,
    Lifeforce = 43,
    Deflector = 44,
    Fracture = 46,
    Arcing = 47,
}
impl Shard {
    pub fn icon(self) -> Option<Icon> {
        Some(Icon::Shard(self as u16))
    }
}
