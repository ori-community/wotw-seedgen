use crate::{
    seed_language::{
        ast,
        output::{CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandZone},
    },
    Zone,
};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::{self, Display};

/// An Operation performed on two values
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Operation<Item, Operator> {
    pub left: Item,
    pub operator: Operator,
    pub right: Item,
}

pub trait ExecuteOperator<T> {
    type Output;

    fn execute(self, left: T, right: T) -> Self::Output;
}

pub use ast::ArithmeticOperator;

impl ExecuteOperator<i32> for ArithmeticOperator {
    type Output = i32;

    fn execute(self, left: i32, right: i32) -> Self::Output {
        match self {
            Self::Add => left + right,
            Self::Subtract => left - right,
            Self::Multiply => left * right,
            Self::Divide => left / right,
        }
    }
}

impl ExecuteOperator<OrderedFloat<f32>> for ArithmeticOperator {
    type Output = OrderedFloat<f32>;

    fn execute(self, left: OrderedFloat<f32>, right: OrderedFloat<f32>) -> Self::Output {
        match self {
            Self::Add => left + right,
            Self::Subtract => left - right,
            Self::Multiply => left * right,
            Self::Divide => left / right,
        }
    }
}

impl From<Operation<CommandInteger, ArithmeticOperator>> for CommandInteger {
    fn from(value: Operation<CommandInteger, ArithmeticOperator>) -> Self {
        Self::Arithmetic {
            operation: Box::new(value),
        }
    }
}

impl From<Operation<CommandFloat, ArithmeticOperator>> for CommandFloat {
    fn from(value: Operation<CommandFloat, ArithmeticOperator>) -> Self {
        Self::Arithmetic {
            operation: Box::new(value),
        }
    }
}

impl Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithmeticOperator::Add => write!(f, "+"),
            ArithmeticOperator::Subtract => write!(f, "-"),
            ArithmeticOperator::Multiply => write!(f, "*"),
            ArithmeticOperator::Divide => write!(f, "/"),
        }
    }
}

/// Concatenation performed on strings
#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Concatenator {
    /// `+`
    Concat = 0,
}

impl TryFrom<ArithmeticOperator> for Concatenator {
    type Error = ();

    fn try_from(value: ArithmeticOperator) -> Result<Self, Self::Error> {
        match value {
            ArithmeticOperator::Add => Ok(Self::Concat),
            _ => Err(()),
        }
    }
}

impl ExecuteOperator<String> for Concatenator {
    type Output = String;

    fn execute(self, left: String, right: String) -> Self::Output {
        match self {
            Concatenator::Concat => left + &right,
        }
    }
}

impl Display for Concatenator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Concatenator::Concat => write!(f, "+"),
        }
    }
}

impl From<Operation<CommandString, Concatenator>> for CommandString {
    fn from(value: Operation<CommandString, Concatenator>) -> Self {
        Self::Concatenate {
            operation: Box::new(value),
        }
    }
}

pub use ast::LogicOperator;

impl ExecuteOperator<bool> for LogicOperator {
    type Output = bool;

    fn execute(self, left: bool, right: bool) -> Self::Output {
        match self {
            Self::And => left && right,
            Self::Or => left || right,
        }
    }
}

impl From<Operation<CommandBoolean, LogicOperator>> for CommandBoolean {
    fn from(value: Operation<CommandBoolean, LogicOperator>) -> Self {
        Self::LogicOperation {
            operation: Box::new(value),
        }
    }
}

impl Display for LogicOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicOperator::And => write!(f, "&&"),
            LogicOperator::Or => write!(f, "||"),
        }
    }
}

pub use ast::Comparator;

impl ExecuteOperator<i32> for Comparator {
    type Output = bool;

    fn execute(self, left: i32, right: i32) -> Self::Output {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::Less => left < right,
            Self::LessOrEqual => left <= right,
            Self::Greater => left > right,
            Self::GreaterOrEqual => left >= right,
        }
    }
}

impl ExecuteOperator<OrderedFloat<f32>> for Comparator {
    type Output = bool;

    fn execute(self, left: OrderedFloat<f32>, right: OrderedFloat<f32>) -> Self::Output {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::Less => left < right,
            Self::LessOrEqual => left <= right,
            Self::Greater => left > right,
            Self::GreaterOrEqual => left >= right,
        }
    }
}

impl From<Operation<CommandInteger, Comparator>> for CommandBoolean {
    fn from(value: Operation<CommandInteger, Comparator>) -> Self {
        Self::CompareInteger {
            operation: Box::new(value),
        }
    }
}

impl From<Operation<CommandFloat, Comparator>> for CommandBoolean {
    fn from(value: Operation<CommandFloat, Comparator>) -> Self {
        Self::CompareFloat {
            operation: Box::new(value),
        }
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Comparator::Equal => write!(f, "=="),
            Comparator::NotEqual => write!(f, "!="),
            Comparator::Less => write!(f, "<"),
            Comparator::LessOrEqual => write!(f, "<="),
            Comparator::Greater => write!(f, ">"),
            Comparator::GreaterOrEqual => write!(f, ">="),
        }
    }
}

/// Comparison Operations performed on strings or booleans
#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EqualityComparator {
    /// `==`
    Equal = 0,
    /// `!=`
    NotEqual = 1,
}

impl TryFrom<Comparator> for EqualityComparator {
    type Error = ();

    fn try_from(value: Comparator) -> Result<Self, Self::Error> {
        match value {
            Comparator::Equal => Ok(Self::Equal),
            Comparator::NotEqual => Ok(Self::NotEqual),
            _ => Err(()),
        }
    }
}

impl ExecuteOperator<bool> for EqualityComparator {
    type Output = bool;

    fn execute(self, left: bool, right: bool) -> Self::Output {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
        }
    }
}

// TODO use reference for all string operation impls
impl ExecuteOperator<String> for EqualityComparator {
    type Output = bool;

    fn execute(self, left: String, right: String) -> Self::Output {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
        }
    }
}

impl ExecuteOperator<Zone> for EqualityComparator {
    type Output = bool;

    fn execute(self, left: Zone, right: Zone) -> Self::Output {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
        }
    }
}

impl From<Operation<CommandBoolean, EqualityComparator>> for CommandBoolean {
    fn from(value: Operation<CommandBoolean, EqualityComparator>) -> Self {
        Self::CompareBoolean {
            operation: Box::new(value),
        }
    }
}

impl From<Operation<CommandString, EqualityComparator>> for CommandBoolean {
    fn from(value: Operation<CommandString, EqualityComparator>) -> Self {
        Self::CompareString {
            operation: Box::new(value),
        }
    }
}

impl From<Operation<CommandZone, EqualityComparator>> for CommandBoolean {
    fn from(value: Operation<CommandZone, EqualityComparator>) -> Self {
        Self::CompareZone {
            operation: Box::new(value),
        }
    }
}

impl Display for EqualityComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EqualityComparator::Equal => write!(f, "=="),
            EqualityComparator::NotEqual => write!(f, "!="),
        }
    }
}
