use std::fmt;

use seedgen_derive::VVariant;

use crate::util::{UberIdentifier, Icon};
use crate::header::{V, VResolve, VString};

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum ShopCommand {
    SetIcon { uber_state: UberIdentifier, icon: Icon },
    SetTitle { uber_state: UberIdentifier, #[VType] title: Option<String> },
    SetDescription { uber_state: UberIdentifier, #[VType] description: Option<String> },
    SetLocked { uber_state: UberIdentifier, #[VWrap] locked: bool },
    SetVisible { uber_state: UberIdentifier, #[VWrap] visible: bool },
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_state, icon } => write!(f, "0|{}|{}", uber_state, icon),
            ShopCommand::SetTitle { uber_state, title } => {
                match title {
                    None => write!(f, "1|{}", uber_state),
                    Some(title) => write!(f, "1|{}|{}", uber_state, title)
                }
            }
            ShopCommand::SetDescription { uber_state, description } => {
                match description {
                    None => write!(f, "2|{}", uber_state),
                    Some(description) => write!(f, "2|{}|{}", uber_state, description)
                }
            }
            ShopCommand::SetLocked { uber_state, locked } => write!(f, "3|{}|{}", uber_state, locked),
            ShopCommand::SetVisible { uber_state, visible } => write!(f, "4|{}|{}", uber_state, visible),
        }
    }
}
