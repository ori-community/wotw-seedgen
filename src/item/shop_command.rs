use std::fmt;

use crate::util::{UberState, Icon};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ShopCommand {
    SetIcon { uber_state: UberState, icon: Icon },
    SetTitle { uber_state: UberState, title: String },
    SetDescription { uber_state: UberState, description: String },
    SetLocked { uber_state: UberState, locked: bool },
    SetVisible { uber_state: UberState, visible: bool },
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_state, icon } => write!(f, "0|{}|{}", uber_state, icon),
            ShopCommand::SetTitle { uber_state, title } => write!(f, "1|{}|{}", uber_state, title),
            ShopCommand::SetDescription { uber_state, description } => write!(f, "2|{}|{}", uber_state, description),
            ShopCommand::SetLocked { uber_state, locked } => write!(f, "3|{}|{}", uber_state, locked),
            ShopCommand::SetVisible { uber_state, visible } => write!(f, "4|{}|{}", uber_state, visible),
        }
    }
}
