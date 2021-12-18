use std::fmt;

use num_enum::TryFromPrimitive;

use crate::util::{Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum BonusItem {
    Relic = 20,
    HealthRegen = 30,
    EnergyRegen = 31,
    ExtraDoubleJump = 35,
    ExtraAirDash = 36,
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
    pub fn icon(self) -> Option<Icon> {
        match self {
            BonusItem::Relic => Some(Icon::File(String::from("assets/icons/game/relic.png"))),
            BonusItem::HealthRegen => Some(Icon::File(String::from("assets/icons/bonus/healthregeneration.png"))),
            BonusItem::EnergyRegen => Some(Icon::File(String::from("assets/icons/bonus/energyregeneration.png"))),
            BonusItem::ExtraDoubleJump => Some(Icon::File(String::from("assets/icons/bonus/extradoublejump.png"))),
            BonusItem::ExtraAirDash => Some(Icon::File(String::from("assets/icons/bonus/extraairdash.png"))),
        }
    }
}
