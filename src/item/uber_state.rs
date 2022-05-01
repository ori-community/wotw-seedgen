use std::fmt;

use seedgen_derive::VVariant;

use crate::util::{UberIdentifier, UberType};

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub struct UberStateItem {
    pub uber_identifier: UberIdentifier,
    pub uber_type: UberType,
    pub signed: bool,
    pub sign: bool,
    #[VType]
    pub operator: UberStateOperator,
    pub skip: bool,
}
impl fmt::Display for UberStateItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}|{}{}",
            self.uber_identifier,
            self.uber_type,
            if self.signed { if self.sign { "+" } else { "-" } } else { "" },
            self.operator
        )
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum UberStateOperator {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
    Range(#[VType] UberStateRange)
}
impl fmt::Display for UberStateOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UberStateOperator::Value(value) => write!(f, "{}", value),
            UberStateOperator::Pointer(uber_identifier) => write!(f, "$({})", uber_identifier),
            UberStateOperator::Range(range) => write!(f, "{}", range),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub struct UberStateRange {
    #[VType]
    pub start: UberStateRangeBoundary,
    #[VType]
    pub end: UberStateRangeBoundary,
}
impl fmt::Display for UberStateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}]", self.start, self.end)
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum UberStateRangeBoundary {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
}
impl fmt::Display for UberStateRangeBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UberStateRangeBoundary::Value(value) => write!(f, "{}", value),
            UberStateRangeBoundary::Pointer(uber_identifier) => write!(f, "$({})", uber_identifier),
        }
    }
}
