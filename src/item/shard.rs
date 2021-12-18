use std::fmt;

use num_enum::TryFromPrimitive;

use crate::util::{Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Shard {
    Overcharge = 1,
    TripleJump,
    Wingclip,
    Bounty,
    Swap,
    Magnet = 8,
    Splinter,
    Reckless = 13,
    Quickshot,
    Resilience = 18,
    SpiritLightHarvest,
    Vitality = 22,
    LifeHarvest,
    EnergyHarvest = 25,
    Energy,
    LifePact,
    LastStand,
    Sense = 30,
    UltraBash = 32,
    UltraGrapple,
    Overflow,
    Thorn,
    Catalyst,
    Turmoil = 38,
    Sticky,
    Finesse,
    SpiritSurge,
    Lifeforce = 43,
    Deflector,
    Fracture = 46,
    Arcing,
}
impl fmt::Display for Shard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", auto_display(self))
    }
}
impl Shard {
    pub fn icon(self) -> Option<Icon> {
        Some(Icon::Shard(self as u16))
    }
}
