use std::fmt;

use crate::util::{UberState, Icon};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ShopCommand {
    SetIcon { uber_state: UberState, icon: Icon }
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_state, icon } => write!(f, "0|{}|{}", uber_state, icon),
        }
    }
}
