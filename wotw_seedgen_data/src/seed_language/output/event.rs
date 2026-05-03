pub use crate::seed_language::ast::ClientEvent;

use super::{CommandBoolean, CommandVoid};
use crate::{
    seed_language::simulate::{Simulate, Simulation},
    EqIgnore, UberIdentifier,
};
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
    Condition(TriggerCondition),
}

impl Trigger {
    pub fn loc_data_trigger(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        Self::Condition(TriggerCondition::new(CommandBoolean::loc_data_condition(
            uber_identifier,
            value,
        )))
    }

    pub const fn multiworld(id: i32) -> Self {
        Self::Binding(UberIdentifier::multiworld(id))
    }

    pub const fn as_multiworld(&self) -> Option<i32> {
        match self {
            Self::Binding(uber_identifier) => uber_identifier.as_multiworld(),
            _ => None,
        }
    }

    pub const fn as_condition(&self) -> Option<&CommandBoolean> {
        match self {
            Trigger::Condition(TriggerCondition { id: _, condition }) => Some(condition),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub id: EqIgnore<Option<usize>>,
    pub condition: CommandBoolean,
}

impl TriggerCondition {
    pub const fn new(condition: CommandBoolean) -> Self {
        Self {
            id: EqIgnore(None),
            condition,
        }
    }

    pub(crate) fn register<S: Simulation>(&mut self, simulation: &mut S) {
        let initial_value = self.condition.simulate(simulation, &[]);
        let id = simulation.condition_values().register(initial_value);
        self.id = EqIgnore(Some(id));
    }
}
