use crate::{
    assets::{UberStateData, UberStateValue},
    seed_language::{
        output::{ContainedReads, Trigger},
        simulate::Snapshot,
    },
    UberIdentifier,
};
use log::{trace, warn};
use rustc_hash::FxHashMap;

pub const UBER_STATES_TARGET_PREFIX: &str = "uber_states ";

#[derive(Debug, Clone)]
pub struct UberStates {
    states: FxHashMap<UberIdentifier, UberStateEntry>,
    fallback: UberStateEntry,
    snapshot: Option<FxHashMap<UberIdentifier, UberStateValue>>,
}

impl UberStates {
    pub fn new(uber_state_data: &UberStateData) -> Self {
        Self {
            states: uber_state_data
                .id_lookup
                .iter()
                .map(|(uber_identifier, data)| {
                    (*uber_identifier, UberStateEntry::new(data.default_value))
                })
                .collect(),
            fallback: UberStateEntry::new(UberStateValue::Boolean(false)),
            snapshot: None,
        }
    }

    /// Register `trigger` to be checked in simulation.
    pub fn register_trigger(&mut self, trigger: &Trigger, event_index: usize) {
        for uber_identifier in trigger.contained_reads() {
            match self.states.get_mut(&uber_identifier) {
                None => warn!("Trigger contained unknown UberState {uber_identifier}"),
                Some(entry) => {
                    entry.triggers.push(event_index);
                }
            }
        }
    }

    pub(crate) fn store(
        &mut self,
        uber_identifier: UberIdentifier,
        value: UberStateValue,
    ) -> &[usize] {
        match self.states.get_mut(&uber_identifier) {
            None => {
                warn!("Attempted to write to unknown UberState {uber_identifier}");

                &self.fallback.triggers
            }
            Some(entry) => {
                if entry.value == value {
                    &self.fallback.triggers
                } else {
                    if let Some(snapshot) = &mut self.snapshot {
                        snapshot.entry(uber_identifier).or_insert(entry.value);
                    }

                    trace!(
                        target: format!("{UBER_STATES_TARGET_PREFIX}{uber_identifier}").as_str(),
                        "{uber_identifier} set {old} -> {new}",
                        old = entry.value,
                        new = value,
                    );

                    // TODO type check maybe?
                    entry.value = value;
                    &entry.triggers
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

#[derive(Debug, Clone)]
struct UberStateEntry {
    value: UberStateValue,
    triggers: Vec<usize>,
}

impl UberStateEntry {
    fn new(value: UberStateValue) -> Self {
        Self {
            value,
            triggers: vec![],
        }
    }
}
