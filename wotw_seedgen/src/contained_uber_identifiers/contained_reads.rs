use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_seed_language::output::{
    CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid, CommandZone,
    Operation, Trigger,
};

use super::{none, some};

fn nested<'a, T>(nested: &'a T) -> Box<dyn Iterator<Item = UberIdentifier> + 'a>
where
    T: ContainedReads,
{
    Box::new(nested.contained_reads())
}

fn chain<'a, T1, T2>(left: &'a T1, right: &'a T2) -> Box<dyn Iterator<Item = UberIdentifier> + 'a>
where
    T1: ContainedReads,
    T2: ContainedReads,
{
    Box::new(left.contained_reads().chain(right.contained_reads()))
}

pub trait ContainedReads {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier>;
}

impl<T: ContainedReads> ContainedReads for Box<T> {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        (**self).contained_reads()
    }
}

impl<T: ContainedReads> ContainedReads for Vec<T> {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        self.iter().flat_map(T::contained_reads)
    }
}

impl<Item: ContainedReads, Operator> ContainedReads for Operation<Item, Operator> {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        chain(&self.left, &self.right)
    }
}

impl ContainedReads for Trigger {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        match self {
            Trigger::ClientEvent(_) => none(),
            Trigger::Binding(uber_identifier) => some(*uber_identifier),
            Trigger::Condition(condition) => nested(condition),
        }
    }
}

impl ContainedReads for CommandBoolean {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        match self {
            CommandBoolean::Multi { commands, last } => chain(commands, last),
            CommandBoolean::CompareBoolean { operation } => nested(operation),
            CommandBoolean::CompareInteger { operation } => nested(operation),
            CommandBoolean::CompareFloat { operation } => nested(operation),
            CommandBoolean::CompareString { operation } => nested(operation),
            CommandBoolean::CompareZone { operation } => nested(operation),
            CommandBoolean::LogicOperation { operation } => nested(operation),
            CommandBoolean::FetchBoolean { uber_identifier } => some(*uber_identifier),
            CommandBoolean::Constant { .. }
            | CommandBoolean::GetBoolean { .. }
            | CommandBoolean::IsInHitbox { .. } => none(),
        }
    }
}

impl ContainedReads for CommandInteger {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        match self {
            CommandInteger::Multi { commands, last } => chain(commands, last),
            CommandInteger::Arithmetic { operation } => nested(operation),
            CommandInteger::FetchInteger { uber_identifier } => some(*uber_identifier),
            CommandInteger::FromFloat { float } => nested(float),
            CommandInteger::Constant { .. } | CommandInteger::GetInteger { .. } => none(),
        }
    }
}

impl ContainedReads for CommandFloat {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        match self {
            CommandFloat::Multi { commands, last } => chain(commands, last),
            CommandFloat::Arithmetic { operation } => nested(operation),
            CommandFloat::FetchFloat { uber_identifier } => some(*uber_identifier),
            CommandFloat::FromInteger { integer } => nested(integer),
            CommandFloat::Constant { .. } | CommandFloat::GetFloat { .. } => none(),
        }
    }
}

impl ContainedReads for CommandString {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        match self {
            CommandString::Multi { commands, last } => chain(commands, last),
            CommandString::Concatenate { left, right } => chain(left, right),
            CommandString::FromBoolean { boolean } => nested(boolean),
            CommandString::FromInteger { integer } => nested(integer),
            CommandString::FromFloat { float } => nested(float),
            CommandString::Constant { .. }
            | CommandString::GetString { .. }
            | CommandString::WorldName { .. } => none(),
        }
    }
}

impl ContainedReads for CommandZone {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        // None of the variants contain any reads
        none()
    }
}

impl ContainedReads for CommandVoid {
    fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> {
        // If it doesn't return anything, you can't build a condition out of it
        // TODO set commands might be relevant?
        none()
    }
}
