// TODO I don't think the trigger implementation is finished

use crate::{
    assets::{self, UberStateData, UberStateValue},
    seed_language::{
        output::{ContainedReads, Trigger},
        simulate::Snapshot,
    },
    UberIdentifier,
};
use log::warn;
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone)]
pub struct UberStates {
    states: FxHashMap<UberIdentifier, UberStateEntry>,
    registered_triggers: usize,
    fallback: UberStateEntry,
    snapshot: Option<FxHashMap<UberIdentifier, UberStateValue>>,
}

#[derive(Debug, Clone)]
struct UberStateEntry {
    value: UberStateValue,
    triggers: FxHashSet<usize>,
}

impl UberStates {
    pub fn new(uber_state_data: &UberStateData) -> Self {
        Self {
            states: uber_state_data
                .id_lookup
                .iter()
                .map(|(uber_identifier, data)| {
                    // TODO these are equivalent types?
                    let value = match data.default_value {
                        assets::UberStateValue::Boolean(value) => UberStateValue::Boolean(value),
                        assets::UberStateValue::Integer(value) => UberStateValue::Integer(value),
                        assets::UberStateValue::Float(value) => UberStateValue::Float(value),
                    };

                    (
                        *uber_identifier,
                        UberStateEntry {
                            value,
                            triggers: Default::default(),
                        },
                    )
                })
                .collect(),
            registered_triggers: 0,
            fallback: UberStateEntry {
                value: UberStateValue::Boolean(false),
                triggers: Default::default(),
            },
            snapshot: None,
        }
    }

    // TODO unclear api, is it possible to prevent getting UberStates without registering all triggers?
    pub(crate) fn register_trigger(&mut self, trigger: &Trigger) {
        for uber_identifier in trigger.contained_reads() {
            match self.states.get_mut(&uber_identifier) {
                None => warn!("Trigger contained unknown UberState {uber_identifier}"),
                Some(entry) => {
                    entry.triggers.insert(self.registered_triggers);
                }
            }
        }

        self.registered_triggers += 1;
    }

    pub(crate) fn store(
        &mut self,
        uber_identifier: UberIdentifier,
        value: UberStateValue,
    ) -> impl Iterator<Item = usize> + '_ {
        match self.states.get_mut(&uber_identifier) {
            None => {
                warn!("Attempted to write to unknown UberState {uber_identifier}");

                self.fallback.triggers.iter().copied()
            }
            Some(entry) => {
                if entry.value != value {
                    if let Some(snapshot) = &mut self.snapshot {
                        snapshot.entry(uber_identifier).or_insert(entry.value);
                    }

                    // TODO type check maybe?
                    entry.value = value;
                    entry.triggers.iter().copied()
                } else {
                    self.fallback.triggers.iter().copied()
                }
            }
        }
    }

    pub(crate) fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        match self.states.get(&uber_identifier) {
            None => {
                warn!("Attempted to read from unknown UberState {uber_identifier}");
                self.fallback.value
            }
            Some(entry) => entry.value,
        }
    }
}

impl Snapshot for UberStates {
    fn snapshot(&mut self) {
        self.snapshot = Some(FxHashMap::default());
    }

    fn restore_snapshot(&mut self) {
        for (uber_identifier, value) in self.snapshot.take().unwrap() {
            self.states.get_mut(&uber_identifier).unwrap().value = value;
        }
    }
}
