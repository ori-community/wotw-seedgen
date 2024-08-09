// TODO why is this in a directory?

use crate::log::warning;
use ordered_float::OrderedFloat;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Ordering;
use strum::Display;
use wotw_seedgen_assets::UberStateData;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_seed_language::output::{
    CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid, CommandZone,
    Operation, Trigger,
};

#[derive(Debug, Clone)]
pub struct UberStates {
    states: FxHashMap<UberIdentifier, UberStateEntry>,
    registered_triggers: usize,
    fallback: UberStateEntry,
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct UberStateEntry {
    value: UberStateValue,
    triggers: FxHashSet<usize>,
}
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
                warning!("Attempted to access {self} UberState as Boolean");
                Default::default()
            }
        }
    }
    pub fn as_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => {
                warning!("Attempted to access {self} UberState as Integer");
                Default::default()
            }
        }
    }
    pub fn as_float(self) -> OrderedFloat<f32> {
        match self {
            UberStateValue::Float(value) => value,
            _ => {
                warning!("Attempted to access {self} UberState as Float");
                Default::default()
            }
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
        for uber_identifier in contained_uber_identifiers(trigger) {
            match self.states.get_mut(&uber_identifier) {
                None => warning!("Trigger contained unknown UberState {uber_identifier}"),
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
                warning!("Attempted to write to unknown UberState {uber_identifier}");
                self.fallback.triggers.iter().copied()
            }
            Some(entry) => {
                if entry.value != value {
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
                warning!("Attempted to read from unknown UberState {uber_identifier}");
                self.fallback.value
            }
            Some(entry) => entry.value,
        }
    }
}

fn contained_uber_identifiers<T: ContainedUberIdentifiers>(t: &T) -> Vec<UberIdentifier> {
    let mut output = vec![];
    t.contained_uber_identifiers(&mut output);
    output
}
trait ContainedUberIdentifiers {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>);
}
impl<T: ContainedUberIdentifiers> ContainedUberIdentifiers for Vec<T> {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        for t in self {
            t.contained_uber_identifiers(output)
        }
    }
}
impl ContainedUberIdentifiers for Trigger {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        match self {
            Trigger::ClientEvent(_) => {}
            Trigger::Binding(uber_identifier) => output.push(*uber_identifier),
            Trigger::Condition(condition) => condition.contained_uber_identifiers(output),
        }
    }
}
impl<Item: ContainedUberIdentifiers, Operator> ContainedUberIdentifiers
    for Operation<Item, Operator>
{
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        self.left.contained_uber_identifiers(output);
        self.right.contained_uber_identifiers(output);
    }
}
impl ContainedUberIdentifiers for CommandBoolean {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        match self {
            CommandBoolean::Multi { commands, last } => {
                commands.contained_uber_identifiers(output);
                last.contained_uber_identifiers(output);
            }
            CommandBoolean::CompareBoolean { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::CompareInteger { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::CompareFloat { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::CompareString { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::CompareZone { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::LogicOperation { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandBoolean::FetchBoolean { uber_identifier } => output.push(*uber_identifier),
            CommandBoolean::Constant { .. }
            | CommandBoolean::GetBoolean { .. }
            | CommandBoolean::IsInHitbox { .. } => {}
        }
    }
}
impl ContainedUberIdentifiers for CommandInteger {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        match self {
            CommandInteger::Multi { commands, last } => {
                commands.contained_uber_identifiers(output);
                last.contained_uber_identifiers(output);
            }
            CommandInteger::Arithmetic { operation } => {
                operation.contained_uber_identifiers(output)
            }
            CommandInteger::FetchInteger { uber_identifier } => output.push(*uber_identifier),
            CommandInteger::FromFloat { float } => float.contained_uber_identifiers(output),
            CommandInteger::Constant { .. } | CommandInteger::GetInteger { .. } => {}
        }
    }
}
impl ContainedUberIdentifiers for CommandFloat {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        match self {
            CommandFloat::Multi { commands, last } => {
                commands.contained_uber_identifiers(output);
                last.contained_uber_identifiers(output);
            }
            CommandFloat::Arithmetic { operation } => operation.contained_uber_identifiers(output),
            CommandFloat::FetchFloat { uber_identifier } => output.push(*uber_identifier),
            CommandFloat::FromInteger { integer } => integer.contained_uber_identifiers(output),
            CommandFloat::Constant { .. } | CommandFloat::GetFloat { .. } => {}
        }
    }
}
impl ContainedUberIdentifiers for CommandString {
    fn contained_uber_identifiers(&self, output: &mut Vec<UberIdentifier>) {
        match self {
            CommandString::Multi { commands, last } => {
                commands.contained_uber_identifiers(output);
                last.contained_uber_identifiers(output);
            }
            CommandString::Concatenate { left, right } => {
                left.contained_uber_identifiers(output);
                right.contained_uber_identifiers(output);
            }
            CommandString::FromBoolean { boolean } => boolean.contained_uber_identifiers(output),
            CommandString::FromInteger { integer } => integer.contained_uber_identifiers(output),
            CommandString::FromFloat { float } => float.contained_uber_identifiers(output),
            CommandString::Constant { .. }
            | CommandString::GetString { .. }
            | CommandString::WorldName { .. } => {}
        }
    }
}
impl ContainedUberIdentifiers for CommandZone {
    fn contained_uber_identifiers(&self, _output: &mut Vec<UberIdentifier>) {
        // None of the variants contain any UberIdentifiers
    }
}
impl ContainedUberIdentifiers for CommandVoid {
    fn contained_uber_identifiers(&self, _output: &mut Vec<UberIdentifier>) {
        // If it doesn't return anything, you can't build a condition out of it
    }
}
