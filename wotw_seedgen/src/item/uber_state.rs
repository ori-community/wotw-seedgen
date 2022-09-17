use std::{fmt::{self, Display}, str::FromStr, ops::Deref};

use decorum::R32;
use rustc_hash::FxHashMap;
use wotw_seedgen_derive::VVariant;

use crate::{util::{UberIdentifier, UberType}, header::{vdisplay, CodeDisplay}, Item};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
pub struct UberStateItem {
    pub identifier: UberIdentifier,
    pub uber_type: UberType,
    pub signed: bool,
    pub sign: bool,
    #[VType]
    pub operator: UberStateOperator,
    pub skip: bool,
}
impl UberStateItem {
    pub fn code(&self) -> CodeDisplay<UberStateItem> {
        CodeDisplay::new(self, |s, f| {
            write!(f, "{}|{}|", s.identifier.code(), s.uber_type.code())?;
            if s.signed { if s.sign { write!(f, "+")? } else { write!(f, "-")? } }
            write!(f, "{}", s.operator.code())?;
            if s.skip { write!(f, "|skip=1") } else { Ok(()) }
        })
    }

    /// Returns the result of this [`UberStateItem`]'s operation when applied to the given uber_states
    pub(crate) fn do_the_math(&self, uber_states: &FxHashMap<UberIdentifier, f32>) -> f32 {
        let mut new = match &self.operator {
            UberStateOperator::Value(value) => value.into_inner(),
            UberStateOperator::Pointer(uber_identifier) => uber_states.get(uber_identifier).copied().unwrap_or_default(),
            UberStateOperator::Range(range) => match &range.start {  // Use the lower boundary for consistent results
                UberStateRangeBoundary::Value(value) => value.into_inner(),
                UberStateRangeBoundary::Pointer(uber_identifier) => uber_states.get(uber_identifier).copied().unwrap_or_default(),
            },
        };
        if self.signed {
            if !self.sign { new = -new }
            uber_states.get(&self.identifier).copied().unwrap_or_default() + new
        } else { new }
    }

    /// Helper to construct a simple [`UberStateItem`]
    pub(crate) fn simple_setter<I: Into<R32>>(uber_identifier: UberIdentifier, uber_type: UberType, value: I) -> Item {
        Item::UberState(UberStateItem {
            identifier: uber_identifier,
            uber_type,
            signed: false,
            sign: false,
            operator: UberStateOperator::Value(value.into().into()),
            skip: false,
        })
    }
}
vdisplay! {
    VUberStateItem,
    impl Display for UberStateItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let identifier = &self.identifier;
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
    Value(#[VWrap] UberStateValue),
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
    Value(#[VWrap] UberStateValue),
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct UberStateValue(R32);
impl From<R32> for UberStateValue {
    fn from(value: R32) -> Self {
        Self(value)
    }
}
impl FromStr for UberStateValue {
    type Err = <R32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "false" => Ok(Self(0_f32.into())),
            "true" => Ok(Self(1_f32.into())),
            _ => s.parse().map(Self),
        }
    }
}
impl Deref for UberStateValue {
    type Target = R32;
    fn deref(&self) -> &R32 {
        &self.0
    }
}
impl Display for UberStateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
