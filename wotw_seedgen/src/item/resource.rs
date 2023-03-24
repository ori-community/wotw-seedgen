use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::{Display, FromStr};

use crate::util::{Icon, MapIcon};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr, Display,
)]
#[repr(u8)]
pub enum Resource {
    HealthFragment = 0,
    EnergyFragment = 1,
    GorlekOre = 2,
    Keystone = 3,
    ShardSlot = 4,
}
impl Resource {
    pub fn icon(self) -> Option<Icon> {
        Some(match self {
            Resource::HealthFragment => {
                Icon::File(String::from("assets/icons/game/healthfragment.png"))
            }
            Resource::EnergyFragment => {
                Icon::File(String::from("assets/icons/game/energyfragment.png"))
            }
            Resource::GorlekOre => Icon::File(String::from("assets/icons/game/gorlekore.png")),
            Resource::Keystone => Icon::File(String::from("assets/icons/game/keystone.png")),
            Resource::ShardSlot => Icon::File(String::from("assets/icons/game/shardslot.png")),
        })
    }
    pub fn map_icon(&self) -> MapIcon {
        match self {
            Resource::HealthFragment => MapIcon::Health,
            Resource::EnergyFragment => MapIcon::Energy,
            Resource::GorlekOre => MapIcon::Ore,
            Resource::Keystone => MapIcon::Keystone,
            Resource::ShardSlot => MapIcon::ShardSlot,
        }
    }
}
