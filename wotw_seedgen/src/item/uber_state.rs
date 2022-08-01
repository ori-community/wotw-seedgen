use std::fmt::{self, Display};

use wotw_seedgen_derive::VVariant;

use crate::{util::{UberIdentifier, UberType}, header::{vdisplay, CodeDisplay}};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
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
    pub fn code(&self) -> CodeDisplay<UberStateItem> {
        CodeDisplay::new(self, |s, f| write!(f, "{}|{}|{}{}",
            s.uber_identifier.code(),
            s.uber_type.code(),
            if s.signed { if s.sign { "+" } else { "-" } } else { "" },
            s.operator.code()))
    }
}
vdisplay! {
    VUberStateItem,
    impl Display for UberStateItem {
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub enum UberStateOperator {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
    Range(#[VType] UberStateRange)
}
impl UberStateOperator {
    pub fn code(&self) -> CodeDisplay<UberStateOperator> {
        CodeDisplay::new(self, |s, f| {
            match s {
                Self::Value(value) => value.fmt(f),
                Self::Pointer(uber_identifier) => write!(f, "$({})", uber_identifier.code()),
                Self::Range(range) => range.code().fmt(f),
            }
        })
    }
}
vdisplay! {
    VUberStateOperator,
    impl Display for UberStateOperator {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Value(value) => value.fmt(f),
                Self::Pointer(identifier) => write!(f, "the value of {identifier}"),
                Self::Range(range) => range.fmt(f),
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub struct UberStateRange {
    #[VType]
    pub start: UberStateRangeBoundary,
    #[VType]
    pub end: UberStateRangeBoundary,
}
impl UberStateRange {
    pub fn code(&self) -> CodeDisplay<UberStateRange> {
        CodeDisplay::new(self, |s, f| write!(f, "[{},{}]", s.start.code(), s.end.code()))
    }
}
vdisplay! {
    VUberStateRange,
    impl Display for UberStateRange {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a random value between {} and {}", self.start, self.end)
        }
    }
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub enum UberStateRangeBoundary {
    Value(#[VWrap] String),
    Pointer(UberIdentifier),
}
impl UberStateRangeBoundary {
    pub fn code(&self) -> CodeDisplay<UberStateRangeBoundary> {
        CodeDisplay::new(self, |s, f| {
            match s {
                Self::Value(value) => value.fmt(f),
                Self::Pointer(identifier) => write!(f, "$({})", identifier.code()),
            }
        })
    }
}
vdisplay! {
    VUberStateRangeBoundary,
    impl Display for UberStateRangeBoundary {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::Value(value) => write!(f, "{value}"),
                Self::Pointer(identifier) => write!(f, "the value of {identifier}"),
            }
        }
    }
}
