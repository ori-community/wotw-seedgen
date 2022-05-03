use std::fmt;

use seedgen_derive::VVariant;

use crate::util::{UberIdentifier, Icon};
use crate::header::VString;

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum ShopCommand {
    SetIcon { uber_identifier: UberIdentifier, icon: Icon },
    SetTitle { uber_identifier: UberIdentifier, #[VType] title: Option<String> },
    SetDescription { uber_identifier: UberIdentifier, #[VType] description: Option<String> },
    SetLocked { uber_identifier: UberIdentifier, #[VWrap] locked: bool },
    SetVisible { uber_identifier: UberIdentifier, #[VWrap] visible: bool },
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_identifier, icon } => write!(f, "0|{}|{}", uber_identifier, icon.code()),
            ShopCommand::SetTitle { uber_identifier, title } => {
                match title {
                    None => write!(f, "1|{}", uber_identifier),
                    Some(title) => write!(f, "1|{}|{}", uber_identifier, title)
                }
            }
            ShopCommand::SetDescription { uber_identifier, description } => {
                match description {
                    None => write!(f, "2|{}", uber_identifier),
                    Some(description) => write!(f, "2|{}|{}", uber_identifier, description)
                }
            }
            ShopCommand::SetLocked { uber_identifier, locked } => write!(f, "3|{}|{}", uber_identifier, locked),
            ShopCommand::SetVisible { uber_identifier, visible } => write!(f, "4|{}|{}", uber_identifier, visible),
        }
    }
}
