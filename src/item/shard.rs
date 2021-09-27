use std::fmt;

use crate::util::{Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Shard {
    Overcharge,
    TripleJump,
    Wingclip,
    Bounty,
    Swap,
    Magnet,
    Splinter,
    Reckless,
    Quickshot,
    Resilience,
    SpiritLightHarvest,
    Vitality,
    LifeHarvest,
    EnergyHarvest,
    Energy,
    LifePact,
    LastStand,
    Sense,
    UltraBash,
    UltraGrapple,
    Overflow,
    Thorn,
    Catalyst,
    Turmoil,
    Sticky,
    Finesse,
    SpiritSurge,
    Lifeforce,
    Deflector,
    Fracture,
    Arcing,
}
impl fmt::Display for Shard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", auto_display(self))
    }
}
impl Shard {
    pub fn from_id(id: u8) -> Option<Shard> {
        match id {
            1 => Some(Shard::Overcharge),
            2 => Some(Shard::TripleJump),
            3 => Some(Shard::Wingclip),
            4 => Some(Shard::Bounty),
            5 => Some(Shard::Swap),
            8 => Some(Shard::Magnet),
            9 => Some(Shard::Splinter),
            13 => Some(Shard::Reckless),
            14 => Some(Shard::Quickshot),
            18 => Some(Shard::Resilience),
            19 => Some(Shard::SpiritLightHarvest),
            22 => Some(Shard::Vitality),
            23 => Some(Shard::LifeHarvest),
            25 => Some(Shard::EnergyHarvest),
            26 => Some(Shard::Energy),
            27 => Some(Shard::LifePact),
            28 => Some(Shard::LastStand),
            30 => Some(Shard::Sense),
            32 => Some(Shard::UltraBash),
            33 => Some(Shard::UltraGrapple),
            34 => Some(Shard::Overflow),
            35 => Some(Shard::Thorn),
            36 => Some(Shard::Catalyst),
            38 => Some(Shard::Turmoil),
            39 => Some(Shard::Sticky),
            40 => Some(Shard::Finesse),
            41 => Some(Shard::SpiritSurge),
            43 => Some(Shard::Lifeforce),
            44 => Some(Shard::Deflector),
            46 => Some(Shard::Fracture),
            47 => Some(Shard::Arcing),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            Shard::Overcharge => 1,
            Shard::TripleJump => 2,
            Shard::Wingclip => 3,
            Shard::Bounty => 4,
            Shard::Swap => 5,
            Shard::Magnet => 8,
            Shard::Splinter => 9,
            Shard::Reckless => 13,
            Shard::Quickshot => 14,
            Shard::Resilience => 18,
            Shard::SpiritLightHarvest => 19,
            Shard::Vitality => 22,
            Shard::LifeHarvest => 23,
            Shard::EnergyHarvest => 25,
            Shard::Energy => 26,
            Shard::LifePact => 27,
            Shard::LastStand => 28,
            Shard::Sense => 30,
            Shard::UltraBash => 32,
            Shard::UltraGrapple => 33,
            Shard::Overflow => 34,
            Shard::Thorn => 35,
            Shard::Catalyst => 36,
            Shard::Turmoil => 38,
            Shard::Sticky => 39,
            Shard::Finesse => 40,
            Shard::SpiritSurge => 41,
            Shard::Lifeforce => 43,
            Shard::Deflector => 44,
            Shard::Fracture => 46,
            Shard::Arcing => 47,
        }
    }

    pub fn icon(self) -> Option<Icon> {
        Some(Icon::Shard(self.to_id()))
    }
}
