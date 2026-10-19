use std::iter;

use crate::{
    seed_language::output::{
        CommandArea, CommandBoolean, CommandFloat, CommandInteger, CommandString, Operation,
        Trigger, TriggerCondition,
    },
    UberIdentifier,
};

fn none<'a, T: 'a>() -> Box<dyn Iterator<Item = T> + 'a> {
    Box::new(iter::empty())
}

fn one<'a, T: 'a>(t: T) -> Box<dyn Iterator<Item = T> + 'a> {
    Box::new(iter::once(t))
}

impl Trigger {
    pub fn contained_reads(&self) -> impl Iterator<Item = UberIdentifier> + '_ {
        match self {
            Self::ClientEvent(_) => none(),
            Self::Binding(uber_identifier) => one(*uber_identifier),
            Self::Condition(condition) => condition.contained_reads(),
        }
    }
}

trait ContainedReads {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_>;
}

impl<Item: ContainedReads, Operator> ContainedReads for Operation<Item, Operator> {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        Box::new(
            self.left
                .contained_reads()
                .chain(self.right.contained_reads()),
        )
    }
}

impl ContainedReads for TriggerCondition {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        self.condition.contained_reads()
    }
}

impl ContainedReads for CommandBoolean {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        debug_assert!(!matches!(self, Self::Multi { .. }));

        match self {
            Self::CompareBoolean { operation } => operation.contained_reads(),
            Self::CompareInteger { operation } => operation.contained_reads(),
            Self::CompareFloat { operation } => operation.contained_reads(),
            Self::CompareString { operation } => operation.contained_reads(),
            Self::CompareArea { operation } => operation.contained_reads(),
            Self::LogicOperation { operation } => operation.contained_reads(),
            Self::FetchBoolean { uber_identifier } => one(*uber_identifier),
            Self::Constant { .. }
            | Self::Multi { .. }
            | Self::FunctionArgument { .. }
            | Self::GetBoolean { .. }
            | Self::IsInCircle { .. }
            | Self::IsInPositionTrigger { .. }
            | Self::IsInRectangle { .. }
            | Self::StringContains { .. } => none(),
        }
    }
}

impl ContainedReads for CommandInteger {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        debug_assert!(!matches!(self, Self::Multi { .. }));

        match self {
            Self::Arithmetic { operation } => operation.contained_reads(),
            Self::FetchInteger { uber_identifier } => one(*uber_identifier),
            Self::FromFloat { float } => float.contained_reads(),
            Self::StringLength { string } => string.contained_reads(),
            Self::Constant { .. }
            | Self::Multi { .. }
            | Self::FunctionArgument { .. }
            | Self::GetInteger { .. } => none(),
        }
    }
}

impl ContainedReads for CommandFloat {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        debug_assert!(!matches!(self, Self::Multi { .. }));

        match self {
            Self::Arithmetic { operation } => operation.contained_reads(),
            Self::FetchFloat { uber_identifier } => one(*uber_identifier),
            Self::FromInteger { integer } => integer.contained_reads(),
            Self::Constant { .. }
            | Self::Multi { .. }
            | Self::FunctionArgument { .. }
            | Self::GetFloat { .. } => none(),
        }
    }
}

impl ContainedReads for CommandString {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        debug_assert!(!matches!(self, Self::Multi { .. }));

        match self {
            Self::Concatenate { operation } => operation.contained_reads(),
            Self::FromBoolean { boolean } => boolean.contained_reads(),
            Self::FromInteger { integer } => integer.contained_reads(),
            Self::FromFloat { float } => float.contained_reads(),
            Self::Constant { .. }
            | Self::Multi { .. }
            | Self::FunctionArgument { .. }
            | Self::GetString { .. }
            | Self::WorldName { .. } => none(),
        }
    }
}

impl ContainedReads for CommandArea {
    fn contained_reads(&self) -> Box<dyn Iterator<Item = UberIdentifier> + '_> {
        debug_assert!(!matches!(self, Self::Multi { .. }));

        match self {
            Self::PlayerArea {} => one(UberIdentifier::PLAYER_AREA),
            Self::MapFocusedArea {} => one(UberIdentifier::MAP_FOCUSED_AREA),
            Self::Constant { .. } | Self::Multi { .. } => none(),
        }
    }
}
