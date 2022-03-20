use seedgen_derive::VVariant;

use crate::{
    Item, VItem,
    header::VResolve,
    util::{UberIdentifier, UberState, VUberState}
};

pub mod parser;

#[derive(Debug, Clone)]
pub struct TimerDefinition {
    pub toggle: UberIdentifier,
    pub timer: UberIdentifier,
}
impl TimerDefinition {
    pub fn code(&self) -> String {
        format!("{}|{}", self.toggle, self.timer)
    }
}

/// An item placed at a location trigger
#[derive(Debug, Clone, VVariant)]
pub struct Pickup {
    /// UberState trigger that should grant the [`Item`]
    #[VType]
    pub trigger: UberState,
    /// [`Item`] to be granted
    #[VType]
    pub item: Item,
    /// Whether this pickup should be ignored for any logic the seed generator applies based on header
    pub ignore: bool,
    /// Whether this pickup should be ignored during header validation
    pub skip_validation: bool,
}

impl Pickup {
    pub fn code(&self) -> String {
        format!("{}|{}", self.trigger, self.item)
    }
}
