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

/// Arithmetic Operations performed on numbers
// TODO why does this have a duplicate in the ast module?
#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ArithmeticOperator {
    /// `+`
    Add = 0,
    /// `-`
    Subtract = 1,
    /// `*`
    Multiply = 2,
    /// `/`
    Divide = 3,
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

/// Logic Operations performed on booleans
#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LogicOperator {
    /// `&&`
    And = 0,
    /// `||`
    Or = 1,
}

impl Display for LogicOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicOperator::And => write!(f, "&&"),
            LogicOperator::Or => write!(f, "||"),
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

impl Display for EqualityComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EqualityComparator::Equal => write!(f, "=="),
            EqualityComparator::NotEqual => write!(f, "!="),
        }
    }
}

/// Comparison Operations performed on numbers
#[derive(Debug, Serialize_repr, Deserialize_repr, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Comparator {
    /// `==`
    Equal = 0,
    /// `!=`
    NotEqual = 1,
    /// `<`
    Less = 2,
    /// `<=`
    LessOrEqual = 3,
    /// `>`
    Greater = 4,
    /// `>=`
    GreaterOrEqual = 5,
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
