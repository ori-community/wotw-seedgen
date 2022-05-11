use std::fmt;

use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::FromStr;

use crate::util::Icon;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum Resource {
    Health = 0,
    Energy = 1,
    Ore = 2,
    Keystone = 3,
    ShardSlot = 4,
}
impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Resource::Health => write!(f, "Health Fragment"),
            Resource::Energy => write!(f, "Energy Fragment"),
            Resource::Ore => write!(f, "Gorlek Ore"),
            Resource::Keystone => write!(f, "Keystone"),
            Resource::ShardSlot => write!(f, "Shard Slot"),
        }
    }
}
impl Resource {
    pub fn icon(self) -> Option<Icon> {
        Some(match self {
            Resource::Health => Icon::File(String::from("assets/icons/game/healthfragment.png")),
            Resource::Energy => Icon::File(String::from("assets/icons/game/energyfragment.png")),
            Resource::Ore => Icon::File(String::from("assets/icons/game/gorlekore.png")),
            Resource::Keystone => Icon::File(String::from("assets/icons/game/keystone.png")),
            Resource::ShardSlot => Icon::File(String::from("assets/icons/game/shardslot.png")),
        })
    }
}
