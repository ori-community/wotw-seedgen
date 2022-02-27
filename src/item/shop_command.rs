use std::fmt;

use crate::util::{UberState, Icon};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ShopCommand {
    SetIcon { uber_state: UberState, icon: Icon },
    SetTitle { uber_state: UberState, title: String },
    SetDescription { uber_state: UberState, description: String },
    SetLockedAndVisible { uber_state: UberState, locked: bool, visible: bool },
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_state, icon } => write!(f, "0|{}|{}", uber_state, icon),
            ShopCommand::SetTitle { uber_state, title } => write!(f, "1|{}|{}", uber_state, title),
            ShopCommand::SetDescription { uber_state, description } => write!(f, "2|{}|{}", uber_state, description),
            ShopCommand::SetLockedAndVisible { uber_state, locked, visible } => write!(f, "3|{}|{}|{}", uber_state, locked, visible),
        }
    }
}
