// TODO I don't think the trigger implementation is finished

use crate::output::{ContainedReads, Trigger};
use log::warn;
use rustc_hash::{FxHashMap, FxHashSet};
use wotw_seedgen_assets::{UberStateData, UberStateValue};
use wotw_seedgen_data::UberIdentifier;

#[derive(Debug, Clone)]
pub struct UberStates {
    states: FxHashMap<UberIdentifier, UberStateEntry>,
    registered_triggers: usize,
    fallback: UberStateEntry,
    snapshot: FxHashMap<u8, FxHashMap<UberIdentifier, UberStateValue>>,
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
                    let value = match data.default_value {
                        wotw_seedgen_assets::UberStateValue::Boolean(value) => {
                            UberStateValue::Boolean(value)
                        }
                        wotw_seedgen_assets::UberStateValue::Integer(value) => {
                            UberStateValue::Integer(value)
                        }
                        wotw_seedgen_assets::UberStateValue::Float(value) => {
                            UberStateValue::Float(value)
                        }
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
            snapshot: FxHashMap::default(),
        }
    }

    pub(crate) fn snapshot(&mut self, id: u8) {
        self.snapshot.insert(id, FxHashMap::default());
    }

    pub(crate) fn take_snapshot(&mut self, id: u8) -> FxHashMap<UberIdentifier, UberStateValue> {
        self.snapshot.remove(&id).unwrap()
    }

    pub(crate) fn restore_snapshot(&mut self, id: u8) {
        for (uber_identifier, value) in self.take_snapshot(id) {
            self.states.get_mut(&uber_identifier).unwrap().value = value;
        }
    }

    // mirrors https://github.com/ori-community/wotw-rando-client/blob/v5/projects/Randomizer/uber_states/uber_state_intercepts.cpp
    pub(crate) fn prevent_change(
        &self,
        uber_identifier: UberIdentifier,
        value: UberStateValue,
    ) -> bool {
        const WELLSPRING_QUEST: UberIdentifier = UberIdentifier::new(937, 34641);
        const KU_QUEST: UberIdentifier = UberIdentifier::new(14019, 34504);

        match uber_identifier {
            WELLSPRING_QUEST => self.fetch(WELLSPRING_QUEST) >= value.as_integer(),
            KU_QUEST => value <= 4,
            _ => false,
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

    pub(crate) fn set(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) {
        let _ = self.set_and_return_triggers(uber_identifier, value);
    }

    pub(crate) fn set_and_return_triggers(
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
                    for snapshot in self.snapshot.values_mut() {
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
