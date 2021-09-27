use std::fmt;

use crate::util::Zone;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ZoneHintType {
    Skills,
    Warps,
    All,
}
impl ZoneHintType {
    pub fn from_id(id: u8) -> Option<ZoneHintType> {
        match id {
            1 => Some(ZoneHintType::Skills),
            2 => Some(ZoneHintType::Warps),
            10 => Some(ZoneHintType::All),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            ZoneHintType::Skills => 1,
            ZoneHintType::Warps => 2,
            ZoneHintType::All => 10,
        }
    }
}
impl fmt::Display for ZoneHintType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZoneHintType::All => write!(f, ""),
            _ => write!(f, "{:?}", self),
        }
    }
}
impl Default for ZoneHintType {
    fn default() -> ZoneHintType {
        ZoneHintType::Skills
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Hint {
    pub zone: Zone,
    pub hint_type: ZoneHintType,
}
impl fmt::Display for Hint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {} hint", self.zone, self.hint_type)
    }
}
