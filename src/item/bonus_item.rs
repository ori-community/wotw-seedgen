use std::fmt;

use crate::util::{Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BonusItem {
    HealthRegen,
    EnergyRegen,
    ExtraDoubleJump,
    ExtraAirDash,
    Relic,
}
impl fmt::Display for BonusItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BonusItem::HealthRegen => write!(f, "Health Regeneration"),
            BonusItem::EnergyRegen => write!(f, "Energy Regeneration"),
            _ => write!(f, "{}", auto_display(self)),
        }
    }
}
impl BonusItem {
    pub fn from_id(id: u8) -> Option<BonusItem> {
        match id {
            20 => Some(BonusItem::Relic),
            30 => Some(BonusItem::HealthRegen),
            31 => Some(BonusItem::EnergyRegen),
            35 => Some(BonusItem::ExtraDoubleJump),
            36 => Some(BonusItem::ExtraAirDash),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            BonusItem::Relic => 20,
            BonusItem::HealthRegen => 30,
            BonusItem::EnergyRegen => 31,
            BonusItem::ExtraDoubleJump => 35,
            BonusItem::ExtraAirDash => 36,
        }
    }

    pub fn icon(self) -> Option<Icon> {
        match self {
            BonusItem::Relic => Some(Icon::File(String::from("icons/relic.png"))),
            BonusItem::HealthRegen => Some(Icon::File(String::from("icons/healthregeneration.png"))),
            BonusItem::EnergyRegen => Some(Icon::File(String::from("icons/energyregeneration.png"))),
            BonusItem::ExtraDoubleJump => Some(Icon::File(String::from("icons/extradoublejump.png"))),
            BonusItem::ExtraAirDash => Some(Icon::File(String::from("icons/extraairdash.png"))),
        }
    }
}
