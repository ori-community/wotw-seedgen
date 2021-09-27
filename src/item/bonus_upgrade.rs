use std::fmt;

use crate::util::auto_display;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BonusUpgrade {
    RapidHammer,
    RapidSword,
    BlazeEfficiency,
    SpearEfficiency,
    ShurikenEfficiency,
    SentryEfficiency,
    BowEfficiency,
    RegenerationEfficiency,
    FlashEfficiency,
    GrenadeEfficiency,
    ExplodingSpike,
    ShockSmash,
    StaticStar,
    ChargeBlaze,
    RapidSentry,
}
impl fmt::Display for BonusUpgrade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", auto_display(self))
    }
}
impl BonusUpgrade {
    pub fn from_id(id: u8) -> Option<BonusUpgrade> {
        match id {
            0 => Some(BonusUpgrade::RapidHammer),
            1 => Some(BonusUpgrade::RapidSword),
            2 => Some(BonusUpgrade::BlazeEfficiency),
            3 => Some(BonusUpgrade::SpearEfficiency),
            4 => Some(BonusUpgrade::ShurikenEfficiency),
            5 => Some(BonusUpgrade::SentryEfficiency),
            6 => Some(BonusUpgrade::BowEfficiency),
            7 => Some(BonusUpgrade::RegenerationEfficiency),
            8 => Some(BonusUpgrade::FlashEfficiency),
            9 => Some(BonusUpgrade::GrenadeEfficiency),
            45 => Some(BonusUpgrade::ExplodingSpike),
            46 => Some(BonusUpgrade::ShockSmash),
            47 => Some(BonusUpgrade::StaticStar),
            48 => Some(BonusUpgrade::ChargeBlaze),
            49 => Some(BonusUpgrade::RapidSentry),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            BonusUpgrade::RapidHammer => 0,
            BonusUpgrade::RapidSword => 1,
            BonusUpgrade::BlazeEfficiency => 2,
            BonusUpgrade::SpearEfficiency => 3,
            BonusUpgrade::ShurikenEfficiency => 4,
            BonusUpgrade::SentryEfficiency => 5,
            BonusUpgrade::BowEfficiency => 6,
            BonusUpgrade::RegenerationEfficiency => 7,
            BonusUpgrade::FlashEfficiency => 8,
            BonusUpgrade::GrenadeEfficiency => 9,
            BonusUpgrade::ExplodingSpike => 45,
            BonusUpgrade::ShockSmash => 46,
            BonusUpgrade::StaticStar => 47,
            BonusUpgrade::ChargeBlaze => 48,
            BonusUpgrade::RapidSentry => 49,
        }
    }
}
