use std::fmt;

use crate::util::Zone;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum SysMessage {
    RelicList,
    MapRelicList(Zone),
    PickupCount,
    GoalProgress,
}
impl SysMessage {
    pub fn from_id(id: u8) -> Option<SysMessage> {
        match id {
            0 => Some(SysMessage::RelicList),
            1 => Some(SysMessage::MapRelicList(Zone::Void)),
            2 => Some(SysMessage::PickupCount),
            3 => Some(SysMessage::GoalProgress),
            _ => None,
        }
    }
    pub fn to_id(self) -> u8 {
        match self {
            SysMessage::RelicList => 0,
            SysMessage::MapRelicList(_) => 1,
            SysMessage::PickupCount => 2,
            SysMessage::GoalProgress => 3,
        }
    }
}
impl fmt::Display for SysMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SysMessage::RelicList => write!(f, "Relic List"),
            SysMessage::MapRelicList(zone) => write!(f, "{zone} Map Relic List"),
            SysMessage::PickupCount => write!(f, "Pickup Count"),
            SysMessage::GoalProgress => write!(f, "Goal Progress"),
        }
    }
}
