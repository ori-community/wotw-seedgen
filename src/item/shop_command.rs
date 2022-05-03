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
impl ShopCommand {
    pub fn code(&self) -> String {
        match self {
            ShopCommand::SetIcon { uber_identifier, icon } => format!("0|{}|{}", uber_identifier, icon.code()),
            ShopCommand::SetTitle { uber_identifier, title } => format!("1|{}{}", uber_identifier, title.iter().map(|title| format!("|{title}")).collect::<String>()),
            ShopCommand::SetDescription { uber_identifier, description } => format!("2|{}{}", uber_identifier, description.iter().map(|description| format!("|{description}")).collect::<String>()),
            ShopCommand::SetLocked { uber_identifier, locked } => format!("3|{}|{}", uber_identifier, locked),
            ShopCommand::SetVisible { uber_identifier, visible } => format!("4|{}|{}", uber_identifier, visible),
        }
    }
}
impl fmt::Display for ShopCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShopCommand::SetIcon { uber_identifier, icon } => write!(f, "Sets the shop icon of {uber_identifier} to the {icon}"),
            ShopCommand::SetTitle { uber_identifier, title } => write!(f, "Sets the shop title of {uber_identifier} to {}", title.clone().map_or_else(|| "the default".to_string(), |title| format!("\"{title}\""))),
            ShopCommand::SetDescription { uber_identifier, description } => write!(f, "Sets the shop description of {uber_identifier} to {}", description.clone().map_or_else(|| "the default".to_string(), |description| format!("\"{description}\""))),
            ShopCommand::SetLocked { uber_identifier, locked } => write!(f, "{}ocks the shop item at {uber_identifier}", if *locked { "L" } else { "Unl" }),
            ShopCommand::SetVisible { uber_identifier, visible } => write!(f, "Turns the shop item at {uber_identifier} {}visible", if *visible { "" } else { "in" }),
        }
    }
}
