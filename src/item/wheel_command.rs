use std::fmt;

use num_enum::TryFromPrimitive;
use seedgen_derive::{VVariant, FromStr};

use super::{Item, VItem};
use crate::util::Icon;
use crate::header::VString;

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum WheelCommand {
    SetName { wheel: u32, position: WheelItemPosition, #[VType] name: String },
    SetDescription { wheel: u32, position: WheelItemPosition, #[VType] description: String },
    SetIcon { wheel: u32, position: WheelItemPosition, icon: Icon },
    SetColor { wheel: u32, position: WheelItemPosition, #[VWrap] r: u8, #[VWrap] g: u8, #[VWrap] b: u8, #[VWrap] a: u8 },
    SetItem { wheel: u32, position: WheelItemPosition, bind: WheelBind, #[VType] item: Box<Item> },
    SetSticky { wheel: u32, #[VWrap] sticky: bool },
    SwitchWheel { wheel: u32 },
    RemoveItem { wheel: u32, position: WheelItemPosition },
    ClearAll,
}
impl fmt::Display for WheelCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WheelCommand::SetName { wheel, position, name } => write!(f, "0|{}|{}|{}", wheel, *position as u8, name),
            WheelCommand::SetDescription { wheel, position, description } => write!(f, "1|{}|{}|{}", wheel, *position as u8, description),
            WheelCommand::SetIcon { wheel, position, icon } => write!(f, "2|{}|{}|{}", wheel, *position as u8, icon.code()),
            WheelCommand::SetColor { wheel, position, r, g, b, a } => write!(f, "3|{}|{}|{}|{}|{}|{}", wheel, *position as u8, r, g, b, a),
            WheelCommand::SetItem { wheel, position, bind, item } => write!(f, "4|{}|{}|{}|{}", wheel, *position as u8, *bind as u8, item),
            WheelCommand::SetSticky { wheel, sticky } => write!(f, "5|{}|{}", wheel, sticky),
            WheelCommand::SwitchWheel { wheel } => write!(f, "6|{}", wheel),
            WheelCommand::RemoveItem { wheel, position } => write!(f, "7|{}|{}", wheel, *position as u8),
            WheelCommand::ClearAll => write!(f, "8"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum WheelItemPosition {
    Top = 0,
    TopRight = 1,
    RightTop = 2,
    Right = 3,
    RightBottom = 4,
    BottomRight = 5,
    Bottom = 6,
    BottomLeft = 7,
    LeftBottom = 8,
    Left = 9,
    LeftTop = 10,
    TopLeft = 11,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum WheelBind {
    All = 0,
    Ability1 = 1,
    Ability2 = 2,
    Ability3 = 3,
}
