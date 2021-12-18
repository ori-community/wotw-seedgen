use std::fmt;

use num_enum::TryFromPrimitive;

use crate::util::Zone;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum ZoneHintType {
    Skills = 1,
    Warps,
    All = 10,
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
