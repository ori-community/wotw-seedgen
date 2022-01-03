use num_enum::TryFromPrimitive;

use crate::{util::Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum BonusUpgrade {
    RapidHammer = 0,
    RapidSword = 1,
    BlazeEfficiency = 2,
    SpearEfficiency = 3,
    ShurikenEfficiency = 4,
    SentryEfficiency = 5,
    BowEfficiency = 6,
    RegenerationEfficiency = 7,
    FlashEfficiency = 8,
    GrenadeEfficiency = 9,
    ExplodingSpike = 45,
    ShockSmash = 46,
    StaticStar = 47,
    ChargeBlaze = 48,
    RapidSentry = 49,
}
auto_display!(BonusUpgrade);

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
