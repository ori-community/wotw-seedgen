use num_enum::TryFromPrimitive;
use seedgen_derive::FromStr;

use crate::util::{Icon, icon::OpherIcon};

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum BonusUpgrade {
    RapidHammer = 0,
    RapidSword = 1,
    BlazeEfficiency = 2,
    SpearEfficiency = 3,
    ShurikenEfficiency = 4,
    SentryEfficiency = 5,
    BowEfficiency = 6,
    RegenerateEfficiency = 7,
    FlashEfficiency = 8,
    GrenadeEfficiency = 9,
    ExplodingSpike = 45,
    ShockSmash = 46,
    StaticStar = 47,
    ChargeBlaze = 48,
    RapidSentry = 49,
}

impl BonusUpgrade {
    pub fn description(self) -> Option<String> {
        match self {
            BonusUpgrade::RapidHammer => Some("Spirit Smash attacks are faster".to_string()),
            BonusUpgrade::RapidSword => Some("Sword attacks are faster".to_string()),
            BonusUpgrade::BlazeEfficiency => Some("Reduce Blaze Cost by 50%".to_string()),
            BonusUpgrade::SpearEfficiency => Some("Reduce Spear Cost by 50%".to_string()),
            BonusUpgrade::ShurikenEfficiency => Some("Reduce Shuriken Cost by 50%".to_string()),
            BonusUpgrade::SentryEfficiency => Some("Reduce Sentry Cost by 50%".to_string()),
            BonusUpgrade::BowEfficiency => Some("Reduce Bow Cost by 50%".to_string()),
            BonusUpgrade::RegenerateEfficiency => Some("Reduce Regenerate Cost by 50%".to_string()),
            BonusUpgrade::FlashEfficiency => Some("Reduce Flash Cost by 50%".to_string()),
            BonusUpgrade::GrenadeEfficiency => Some("Reduce Grenade Cost by 50%".to_string()),
            BonusUpgrade::ExplodingSpike | BonusUpgrade::ShockSmash | BonusUpgrade::StaticStar | BonusUpgrade::ChargeBlaze | BonusUpgrade::RapidSentry => None,
        }
    }
    pub fn icon(self) -> Option<Icon> {
        let icon = match self {
            BonusUpgrade::RapidHammer => Icon::File(String::from("assets/icons/bonus/rapidsmash.png")),
            BonusUpgrade::RapidSword => Icon::File(String::from("assets/icons/bonus/rapidsword.png")),
            BonusUpgrade::BlazeEfficiency => Icon::File(String::from("assets/icons/bonus/blazeefficiency.png")),
            BonusUpgrade::SpearEfficiency => Icon::File(String::from("assets/icons/bonus/spearefficiency.png")),
            BonusUpgrade::ShurikenEfficiency => Icon::File(String::from("assets/icons/bonus/shurikenefficiency.png")),
            BonusUpgrade::SentryEfficiency => Icon::File(String::from("assets/icons/bonus/sentryefficiency.png")),
            BonusUpgrade::BowEfficiency => Icon::File(String::from("assets/icons/bonus/bowefficiency.png")),
            BonusUpgrade::RegenerateEfficiency => Icon::File(String::from("assets/icons/bonus/regenerateefficiency.png")),
            BonusUpgrade::FlashEfficiency => Icon::File(String::from("assets/icons/bonus/flashefficiency.png")),
            BonusUpgrade::GrenadeEfficiency => Icon::File(String::from("assets/icons/bonus/grenadeefficiency.png")),
            BonusUpgrade::ExplodingSpike => Icon::Opher(OpherIcon::SpearUpgrade),
            BonusUpgrade::ShockSmash => Icon::Opher(OpherIcon::HammerUpgrade),
            BonusUpgrade::StaticStar => Icon::Opher(OpherIcon::ShurikenUpgrade),
            BonusUpgrade::ChargeBlaze => Icon::Opher(OpherIcon::BlazeUpgrade),
            BonusUpgrade::RapidSentry => Icon::Opher(OpherIcon::SentryUpgrade),
        };
        Some(icon)
    }
}
