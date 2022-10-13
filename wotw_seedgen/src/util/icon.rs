use std::fmt;

use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::{FromStr, Display};

use crate::{item::Shard, header::CodeDisplay};

use super::Spell;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Icon {
    Shard(Shard),
    Spell(Spell),
    Opher(OpherIcon),
    Lupo(LupoIcon),
    Grom(GromIcon),
    Tuley(TuleyIcon),
    File(String),
}
impl Icon {
    pub fn code(&self) -> CodeDisplay<Icon> {
        CodeDisplay::new(self, |s, f| {
            match s {
                Icon::Shard(shard) => write!(f, "shard:{}", *shard as u8),
                Icon::Spell(spell) => write!(f, "spell:{}", *spell as u16),
                Icon::Opher(opher) => write!(f, "opher:{}", *opher as u8),
                Icon::Lupo(lupo) => write!(f, "lupo:{}", *lupo as u8),
                Icon::Grom(grom) => write!(f, "grom:{}", *grom as u8),
                Icon::Tuley(tuley) => write!(f, "tuley:{}", *tuley as u8),
                Icon::File(file) => write!(f, "file:{file}"),
            }
        })
    }
}
impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Icon::Shard(shard) => write!(f, "{shard} Shard Icon"),
            Icon::Spell(spell) => write!(f, "{spell} Spell Icon"),
            Icon::Opher(opher) => write!(f, "{opher} Opher Icon"),
            Icon::Lupo(lupo) => write!(f, "{lupo} Lupo Icon"),
            Icon::Grom(grom) => write!(f, "{grom} Grom Icon"),
            Icon::Tuley(tuley) => write!(f, "{tuley} Tuley Icon"),
            Icon::File(file) => write!(f, "File Icon at \"{file}\""),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum OpherIcon {
    Sentry = 0,
    SentryUpgrade = 1,
    Hammer = 2,
    HammerUpgrade = 3,
    Shuriken = 4,
    ShurikenUpgrade = 5,
    Spear = 6,
    SpearUpgrade = 7,
    Blaze = 8,
    BlazeUpgrade = 9,
    WaterBreath = 10,
    FastTravel = 11,
}
#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum LupoIcon {
    EnergyFragmentsMap = 0,
    HealthFragmentsMap = 1,
    ShardsMap = 2,
}
#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum GromIcon {
    RepairTheSpiritWell = 0,
    DwellingRepairs = 1,
    RoofsOverHeads = 2,
    OnwardsAndUpwards = 3,
    ClearTheCaveEntrance = 4,
    ThornySituation = 5,
    TheGorlekTouch = 6,
}
#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum TuleyIcon {
    SelaFlowers = 0,
    StickyGrass = 1,
    Lightcatchers = 2,
    BlueMoon = 3,
    SpringPlants = 4,
    TheLastSeed = 5,
}

pub enum MapIcon {
    Keystone,
    Health,
    Energy,
    Ore,
    ShardSlot,
    SpiritLight,
    Skill,
    Shard,
    Teleporter,
    QuestItem,
    Other,
}
