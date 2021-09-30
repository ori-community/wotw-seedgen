use std::fmt;

use crate::util::Icon;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Resource {
    Health,
    Energy,
    Ore,
    Keystone,
    ShardSlot,
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
    pub fn from_id(id: u8) -> Option<Resource> {
        match id {
            0 => Some(Resource::Health),
            1 => Some(Resource::Energy),
            2 => Some(Resource::Ore),
            3 => Some(Resource::Keystone),
            4 => Some(Resource::ShardSlot),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            Resource::Health => 0,
            Resource::Energy => 1,
            Resource::Ore => 2,
            Resource::Keystone => 3,
            Resource::ShardSlot => 4,
        }
    }

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
