pub use crate::seed_language::ast::ClientEvent;

use super::{CommandBoolean, CommandVoid};
use crate::UberIdentifier;
use serde::{Deserialize, Serialize};

/// The main event (:badumtsss:)
// TODO improve documentation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    /// The Trigger defines when to give the Action
    pub trigger: Trigger,
    /// The Command defines what to do when the Trigger happens
    pub command: CommandVoid,
}

impl Event {
    pub(crate) fn on_spawn(command: CommandVoid) -> Self {
        Self {
            trigger: Trigger::ClientEvent(ClientEvent::Spawn),
            command,
        }
    }

    pub(crate) fn on_reload(command: CommandVoid) -> Self {
        Self {
            trigger: Trigger::ClientEvent(ClientEvent::Reload),
            command,
        }
    }
}

/// Trigger for an [`Event`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// Specific client events
    ClientEvent(ClientEvent),
    /// Trigger on every change to an UberIdentifier
    Binding(UberIdentifier),
    /// Trigger when the condition changes from `false` to `true`
    Condition(CommandBoolean),
}

impl Trigger {
    pub fn loc_data_trigger(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        Self::Condition(CommandBoolean::loc_data_condition(uber_identifier, value))
    }
}
