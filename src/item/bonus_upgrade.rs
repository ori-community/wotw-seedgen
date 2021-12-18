use std::fmt;

use num_enum::TryFromPrimitive;

use crate::util::{Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
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
    ExplodingSpike = 45,
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
    pub fn icon(self) -> Option<Icon> {
        match self {
            BonusUpgrade::RapidHammer => Some(Icon::File(String::from("assets/icons/bonus/rapidsmash.png"))),
            BonusUpgrade::RapidSword => Some(Icon::File(String::from("assets/icons/bonus/rapidsword.png"))),
            BonusUpgrade::BlazeEfficiency => Some(Icon::File(String::from("assets/icons/bonus/blazeefficiency.png"))),
            BonusUpgrade::SpearEfficiency => Some(Icon::File(String::from("assets/icons/bonus/spearefficiency.png"))),
            BonusUpgrade::ShurikenEfficiency => Some(Icon::File(String::from("assets/icons/bonus/shurikenefficiency.png"))),
            BonusUpgrade::SentryEfficiency => Some(Icon::File(String::from("assets/icons/bonus/sentryefficiency.png"))),
            BonusUpgrade::BowEfficiency => Some(Icon::File(String::from("assets/icons/bonus/bowefficiency.png"))),
            BonusUpgrade::RegenerationEfficiency => Some(Icon::File(String::from("assets/icons/bonus/regenerateefficiency.png"))),
            BonusUpgrade::FlashEfficiency => Some(Icon::File(String::from("assets/icons/bonus/flashefficiency.png"))),
            BonusUpgrade::GrenadeEfficiency => Some(Icon::File(String::from("assets/icons/bonus/grenadeefficiency.png"))),
            BonusUpgrade::ExplodingSpike => Some(Icon::Opher(7)),
            BonusUpgrade::ShockSmash => Some(Icon::Opher(3)),
            BonusUpgrade::StaticStar => Some(Icon::Opher(5)),
            BonusUpgrade::ChargeBlaze => Some(Icon::Opher(9)),
            BonusUpgrade::RapidSentry => Some(Icon::Opher(1)),
        }
    }
}
