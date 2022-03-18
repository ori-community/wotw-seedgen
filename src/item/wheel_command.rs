use std::fmt;

use super::Item;
use crate::util::Icon;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum WheelCommand {
    SetName { wheel: u16, position: u8, name: String },
    SetDescription { wheel: u16, position: u8, description: String },
    SetIcon { wheel: u16, position: u8, icon: Icon },
    SetColor { wheel: u16, position: u8, r: u8, g: u8, b: u8, a: u8 },
    SetItem { wheel: u16, position: u8, bind: WheelBind, item: Box<Item> },
    SetSticky { wheel: u16, sticky: bool },
    SwitchWheel { wheel: u16 },
    RemoveItem { wheel: u16, position: u8 },
    ClearAll,
}
impl fmt::Display for WheelCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WheelCommand::SetName { wheel, position, name } => write!(f, "0|{}|{}|{}", wheel, position, name),
            WheelCommand::SetDescription { wheel, position, description } => write!(f, "1|{}|{}|{}", wheel, position, description),
            WheelCommand::SetIcon { wheel, position, icon } => write!(f, "2|{}|{}|{}", wheel, position, icon),
            WheelCommand::SetColor { wheel, position, r, g, b, a } => write!(f, "3|{}|{}|{}|{}|{}|{}", wheel, position, r, g, b, a),
            WheelCommand::SetItem { wheel, position, bind, item } => write!(f, "4|{}|{}|{}|{}", wheel, position, bind, item),
            WheelCommand::SetSticky { wheel, sticky } => write!(f, "5|{}|{}", wheel, sticky),
            WheelCommand::SwitchWheel { wheel } => write!(f, "6|{}", wheel),
            WheelCommand::RemoveItem { wheel, position } => write!(f, "7|{}|{}", wheel, position),
            WheelCommand::ClearAll => write!(f, "8"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum WheelBind {
    All,
    Ability1,
    Ability2,
    Ability3,
}
impl fmt::Display for WheelBind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WheelBind::All => write!(f, "0"),
            WheelBind::Ability1 => write!(f, "1"),
            WheelBind::Ability2 => write!(f, "2"),
            WheelBind::Ability3 => write!(f, "3"),
        }
    }
}
