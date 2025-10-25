// TODO why is this in a directory?

use log::warn;
use ordered_float::OrderedFloat;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cmp::Ordering, mem, ops::Index};
use strum::Display;
use wotw_seedgen_assets::UberStateData;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_seed_language::output::{ContainedReads, Trigger};

#[derive(Debug, Clone)]
pub struct UberStates {
    states: FxHashMap<UberIdentifier, UberStateEntry>,
    registered_triggers: usize,
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
                    let value = match data.default_value {
                        wotw_seedgen_assets::UberStateValue::Boolean(value) => {
                            UberStateValue::Boolean(value)
                        }
                        wotw_seedgen_assets::UberStateValue::Integer(value) => {
                            UberStateValue::Integer(value)
                        }
                        wotw_seedgen_assets::UberStateValue::Float(value) => {
                            UberStateValue::Float(value.into())
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
            snapshot: None,
        }
    }

    pub fn snapshot(&mut self) {
        self.snapshot = Some(FxHashMap::default());
    }

    pub fn restore_snapshot(&mut self) {
        for (uber_identifier, value) in mem::take(&mut self.snapshot).unwrap() {
            self.states.get_mut(&uber_identifier).unwrap().value = value;
        }
    }
}

#[derive(Debug, Clone)]
struct UberStateEntry {
    value: UberStateValue,
    triggers: FxHashSet<usize>,
}

// TODO bad display implementation
// TODO redundant with assets::UberStateValue?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum UberStateValue {
    Boolean(bool),
    Integer(i32),
    Float(OrderedFloat<f32>),
}

impl UberStateValue {
    pub fn as_boolean(self) -> bool {
        match self {
            UberStateValue::Boolean(value) => value,
            _ => {
                warn!("Attempted to access {self} UberState as Boolean");
                Default::default()
            }
        }
    }

    pub fn expect_boolean(self) -> bool {
        match self {
            UberStateValue::Boolean(value) => value,
            _ => panic!("Attempted to access {self} UberState as Boolean"),
        }
    }

    pub fn as_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => {
                warn!("Attempted to access {self} UberState as Integer");
                Default::default()
            }
        }
    }

    pub fn expect_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => panic!("Attempted to access {self} UberState as Integer"),
        }
    }

    pub fn as_float(self) -> OrderedFloat<f32> {
        match self {
            UberStateValue::Float(value) => value,
            _ => {
                warn!("Attempted to access {self} UberState as Float");
                Default::default()
            }
        }
    }

    pub fn expect_float(self) -> OrderedFloat<f32> {
        match self {
            UberStateValue::Float(value) => value,
            _ => panic!("Attempted to access {self} UberState as Float"),
        }
    }
}

impl PartialEq<bool> for UberStateValue {
    fn eq(&self, other: &bool) -> bool {
        self.as_boolean() == *other
    }
}

impl PartialOrd<bool> for UberStateValue {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        self.as_boolean().partial_cmp(other)
    }
}

impl PartialEq<i32> for UberStateValue {
    fn eq(&self, other: &i32) -> bool {
        self.as_integer() == *other
    }
}

impl PartialOrd<i32> for UberStateValue {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.as_integer().partial_cmp(other)
    }
}

impl PartialEq<OrderedFloat<f32>> for UberStateValue {
    fn eq(&self, other: &OrderedFloat<f32>) -> bool {
        self.as_float() == *other
    }
}

impl PartialOrd<OrderedFloat<f32>> for UberStateValue {
    fn partial_cmp(&self, other: &OrderedFloat<f32>) -> Option<Ordering> {
        self.as_float().partial_cmp(other)
    }
}

impl UberStates {
    // TODO unclear api, is it possible to prevent getting UberStates without registering all triggers?
    pub fn register_trigger(&mut self, trigger: &Trigger) {
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

    pub fn set(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) {
        let _ = self.set_and_return_triggers(uber_identifier, value);
    }

    pub fn set_and_return_triggers(
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
    pub fn get(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        match self.states.get(&uber_identifier) {
            None => {
                warn!("Attempted to read from unknown UberState {uber_identifier}");
                self.fallback.value
            }
            Some(entry) => entry.value,
        }
    }
}

impl Index<UberIdentifier> for UberStates {
    type Output = UberStateValue;

    fn index(&self, index: UberIdentifier) -> &Self::Output {
        &self.states[&index].value
    }
}
