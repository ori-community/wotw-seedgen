use crate::util::Zone;

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy)]
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
