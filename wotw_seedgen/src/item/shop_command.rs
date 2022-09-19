use std::fmt;

use wotw_seedgen_derive::VVariant;

use crate::util::Icon;
use crate::uber_state::UberIdentifier;
use crate::header::{VString, vdisplay, CodeDisplay};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub enum ShopCommand {
    SetIcon { uber_identifier: UberIdentifier, icon: Icon },
    SetTitle { uber_identifier: UberIdentifier, #[VType] title: Option<String> },
    SetDescription { uber_identifier: UberIdentifier, #[VType] description: Option<String> },
    SetLocked { uber_identifier: UberIdentifier, #[VWrap] locked: bool },
    SetVisible { uber_identifier: UberIdentifier, #[VWrap] visible: bool },
}
impl ShopCommand {
    pub fn code(&self) -> CodeDisplay<ShopCommand> {
        CodeDisplay::new(self, |s, f| {
            match s {
                ShopCommand::SetIcon { uber_identifier, icon } => write!(f, "0|{}|{}", uber_identifier.code(), icon.code()),
                ShopCommand::SetTitle { uber_identifier, title } => {
                    write!(f, "1|{}", uber_identifier.code())?;
                    match title {
                        Some(title) => write!(f, "|{}", title),
                        None => Ok(()),
                    }
                },
                ShopCommand::SetDescription { uber_identifier, description } => {
                    write!(f, "2|{}", uber_identifier.code())?;
                    match description {
                        Some(description) => write!(f, "|{}", description),
                        None => Ok(()),
                    }
                },
                ShopCommand::SetLocked { uber_identifier, locked } => write!(f, "3|{}|{}", uber_identifier.code(), locked),
                ShopCommand::SetVisible { uber_identifier, visible } => write!(f, "4|{}|{}", uber_identifier.code(), visible),
            }
        })
    }
}
vdisplay! {
    VShopCommand,
    impl fmt::Display for ShopCommand {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::SetIcon { uber_identifier, icon } => write!(f, "Sets the shop icon of {uber_identifier} to the {icon}"),
                Self::SetTitle { uber_identifier, title } => write!(f, "Sets the shop title of {uber_identifier} to {}", title.clone().map_or_else(|| "the default".to_string(), |title| format!("\"{title}\""))),
                Self::SetDescription { uber_identifier, description } => write!(f, "Sets the shop description of {uber_identifier} to {}", description.clone().map_or_else(|| "the default".to_string(), |description| format!("\"{description}\""))),
                Self::SetLocked { uber_identifier, locked } => write!(f, "Sets the locked state of the shop item at {uber_identifier} to be {locked}"),
                Self::SetVisible { uber_identifier, visible } => write!(f, "Sets the visible state of the shop item at {uber_identifier} to be {visible}"),
            }
        }
    }
}
