use std::fmt;

use seedgen_derive::VVariant;

use crate::{util::{UberIdentifier, UberType}, header::vdisplay};

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
impl UberStateItem {
    pub fn code(&self) -> String {
        format!("{}|{}|{}{}",
            self.uber_identifier,
            self.uber_type.code(),
            if self.signed { if self.sign { "+" } else { "-" } } else { "" },
            self.operator.code()
        )
    }
}
vdisplay! {
    VUberStateItem,
    impl fmt::Display for UberStateItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let identifier = &self.uber_identifier;
            let value = self.operator.to_string();
            let operation = if self.signed {
                if self.sign {
                    format!("Add {value} to {identifier}")
                } else {
                    format!("Remove {value} from {identifier}")
                }
            } else {
                format!("Set {identifier} to {value}")
            };
            let skip = if self.skip {
                ", skipping any triggers caused by this change"
            } else { "" };
            write!(f, "{operation}{skip}")
        }
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum UberStateOperator {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
    Range(#[VType] UberStateRange)
}
impl UberStateOperator {
    pub fn code(&self) -> String {
        match self {
            Self::Value(value) => format!("{value}"),
            Self::Pointer(uber_identifier) => format!("$({uber_identifier})"),
            Self::Range(range) => format!("{}", range.code()),
        }
    }
}
vdisplay! {
    VUberStateOperator,
    impl fmt::Display for UberStateOperator {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Value(value) => value.fmt(f),
                Self::Pointer(identifier) => write!(f, "the value of {identifier}"),
                Self::Range(range) => range.fmt(f),
            }
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
impl UberStateRange {
    pub fn code(&self) -> String {
        format!("[{},{}]", self.start.code(), self.end.code())
    }
}
vdisplay! {
    VUberStateRange,
    impl fmt::Display for UberStateRange {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a random value between {} and {}", self.start, self.end)
        }
    }
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum UberStateRangeBoundary {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
}
impl UberStateRangeBoundary {
    pub fn code(&self) -> String {
        match self {
            Self::Value(value) => format!("{value}"),
            Self::Pointer(uber_identifier) => format!("$({uber_identifier})"),
        }
    }
}
vdisplay! {
    VUberStateRangeBoundary,
    impl fmt::Display for UberStateRangeBoundary {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Value(value) => write!(f, "{value}"),
                Self::Pointer(identifier) => write!(f, "the value of {identifier}"),
            }
        }
    }
}
