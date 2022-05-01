use num_enum::TryFromPrimitive;
use seedgen_derive::FromStr;

use crate::util::Icon;

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum BonusItem {
    Relic = 20,
    HealthRegeneration = 30,
    EnergyRegeneration = 31,
    ExtraDoubleJump = 35,
    ExtraAirDash = 36,
}

impl BonusItem {
    pub fn description(self) -> Option<String> {
        match self {
            BonusItem::Relic => None,
            BonusItem::HealthRegeneration => Some("Replenish Health over time".to_string()),
            BonusItem::EnergyRegeneration => Some("Replenish Energy over time".to_string()),
            BonusItem::ExtraDoubleJump => Some("Jump an additional time mid-air before landing".to_string()),
            BonusItem::ExtraAirDash => Some("Dash an additional time mid-air before landing".to_string()),
        }
    }

    pub fn icon(self) -> Option<Icon> {
        match self {
            BonusItem::Relic => Some(Icon::File(String::from("assets/icons/game/relic.png"))),
            BonusItem::HealthRegeneration => Some(Icon::File(String::from("assets/icons/bonus/healthregeneration.png"))),
            BonusItem::EnergyRegeneration => Some(Icon::File(String::from("assets/icons/bonus/energyregeneration.png"))),
            BonusItem::ExtraDoubleJump => Some(Icon::File(String::from("assets/icons/bonus/extradoublejump.png"))),
            BonusItem::ExtraAirDash => Some(Icon::File(String::from("assets/icons/bonus/extraairdash.png"))),
        }
    }
}
