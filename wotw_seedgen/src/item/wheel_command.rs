use std::fmt::{self, Display};

use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::{Display, FromStr, VVariant};

use super::{Item, VItem};
use crate::header::{vdisplay, CodeDisplay, VString};
use crate::util::Icon;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub enum WheelCommand {
    SetName {
        wheel: u32,
        position: WheelItemPosition,
        #[VType]
        name: String,
    },
    SetDescription {
        wheel: u32,
        position: WheelItemPosition,
        #[VType]
        description: String,
    },
    SetIcon {
        wheel: u32,
        position: WheelItemPosition,
        icon: Icon,
    },
    SetColor {
        wheel: u32,
        position: WheelItemPosition,
        #[VWrap]
        r: u8,
        #[VWrap]
        g: u8,
        #[VWrap]
        b: u8,
        #[VWrap]
        a: u8,
    },
    SetItem {
        wheel: u32,
        position: WheelItemPosition,
        bind: WheelBind,
        #[VType]
        item: Box<Item>,
    },
    SetSticky {
        wheel: u32,
        #[VWrap]
        sticky: bool,
    },
    SwitchWheel {
        wheel: u32,
    },
    RemoveItem {
        wheel: u32,
        position: WheelItemPosition,
    },
    ClearAll,
}
impl WheelCommand {
    pub fn code(&self) -> CodeDisplay<WheelCommand> {
        CodeDisplay::new(self, |s, f| match s {
            WheelCommand::SetName {
                wheel,
                position,
                name,
            } => write!(f, "0|{}|{}|{}", wheel, *position as u8, name),
            WheelCommand::SetDescription {
                wheel,
                position,
                description,
            } => write!(f, "1|{}|{}|{}", wheel, *position as u8, description),
            WheelCommand::SetIcon {
                wheel,
                position,
                icon,
            } => write!(f, "2|{}|{}|{}", wheel, *position as u8, icon.code()),
            WheelCommand::SetColor {
                wheel,
                position,
                r,
                g,
                b,
                a,
            } => write!(f, "3|{}|{}|{}|{}|{}|{}", wheel, *position as u8, r, g, b, a),
            WheelCommand::SetItem {
                wheel,
                position,
                bind,
                item,
            } => write!(
                f,
                "4|{}|{}|{}|{}",
                wheel,
                *position as u8,
                *bind as u8,
                item.code()
            ),
            WheelCommand::SetSticky { wheel, sticky } => write!(f, "5|{}|{}", wheel, sticky),
            WheelCommand::SwitchWheel { wheel } => write!(f, "6|{}", wheel),
            WheelCommand::RemoveItem { wheel, position } => {
                write!(f, "7|{}|{}", wheel, *position as u8)
            }
            WheelCommand::ClearAll => "8".fmt(f),
        })
    }
}
vdisplay! {
    VWheelCommand,
    impl Display for WheelCommand {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::SetName { wheel, position, name } => write!(f, "Set the name of the {position} item in wheel {wheel} to \"{name}\""),
                Self::SetDescription { wheel, position, description } => write!(f, "Set the description of the {position} item in wheel {wheel} to \"{description}\""),
                Self::SetIcon { wheel, position, icon } => write!(f, "Set the icon of the {position} item in wheel {wheel} to the {icon}"),
                Self::SetColor { wheel, position, r, g, b, a } => write!(f, "Set the icon color of the {position} item in wheel {wheel} to (rgba) {r}, {g}, {b}, {a}"),
                Self::SetItem { wheel, position, bind, item } => write!(f, "Set the action bound to {bind} of the {position} item in wheel {wheel} to this item: {item}"),
                Self::SetSticky { wheel, sticky } => write!(f, "Set the sticky value of wheel {wheel} to {sticky}"),
                Self::SwitchWheel { wheel } => write!(f, "Switch to wheel {wheel}"),
                Self::RemoveItem { wheel, position } => write!(f, "Remove the {position} item in wheel {wheel}"),
                Self::ClearAll => write!(f, "Clear all wheels"),
            }
        }
    }
}

#[derive(
    Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr,
)]
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
#[derive(
    Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr,
)]
#[repr(u8)]
pub enum WheelBind {
    AllBinds = 0,
    Ability1 = 1,
    Ability2 = 2,
    Ability3 = 3,
}
