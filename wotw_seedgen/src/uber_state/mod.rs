mod game_data;
mod rando_data;

use std::{fmt::{self, Display}, str::FromStr};

use rustc_hash::FxHashMap;
use wotw_seedgen_derive::VVariant;

use crate::{header::{CodeDisplay, parser, VResolve}};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum UberType {
    Bool,
    Teleporter,
    Byte,
    Int,
    Float,
}
impl UberType {
    pub fn code(&self) -> CodeDisplay<UberType> {
        CodeDisplay::new(self, |s, f| write!(f, "{}", match s {
            UberType::Bool => "bool",
            UberType::Teleporter => "teleporter",
            UberType::Byte => "byte",
            UberType::Int => "int",
            UberType::Float => "float",
        }))
    }
}
impl std::str::FromStr for UberType {
    type Err = String;

    fn from_str(uber_type: &str) -> Result<UberType, String> {
        match uber_type {
            "bool" | "teleporter" => Ok(UberType::Bool),
            "byte" => Ok(UberType::Byte),
            "int" => Ok(UberType::Int),
            "float" => Ok(UberType::Float),
            _ => Err(format!("Invalid uberState type {}", uber_type)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct UberIdentifier {
    pub uber_group: u16,
    pub uber_id: u16,
}
impl UberIdentifier {
    pub const fn new(uber_group: u16, uber_id: u16) -> Self {
        Self { uber_group, uber_id }
    }

    pub fn code(&self) -> CodeDisplay<UberIdentifier> {
        CodeDisplay::new(self, |s, f| { write!(f, "{}|{}", s.uber_group, s.uber_id)})
    }

    pub fn is_shop(&self) -> bool {
        matches!(self.uber_group, 1 | 2) ||
        self.uber_group == 48248 && matches!(self.uber_id, 19396 | 57987 | 41666)
    }
    pub fn is_purchasable(&self) -> bool {
        matches!(self.uber_group, 1 | 2) ||
        self.uber_group == 48248 && matches!(self.uber_id, 18767 | 45538 | 3638 | 1590 | 1557 | 29604 | 48423 | 61146 | 4045 | 19396 | 57987 | 41666)
    }

    #[inline]
    pub fn spawn() -> UberIdentifier {
        UberIdentifier::new(3, 0)
    }
    #[inline]
    pub fn load() -> UberIdentifier {
        UberIdentifier::new(3, 1)
    }
}
impl Display for UberIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        rando_data::NAMED_UBER_STATES.iter()
            .find(|(_, identifier)| self == identifier)
            .map_or_else(
                ||
                game_data::UBER_STATES.iter()
                    .find(|(_, identifier)| self == identifier)
                    .map_or_else(|| self.code().to_string(), |(name, _)| (*name).to_string()),
                |(name, _)| (*name).to_string())
            .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, VVariant)]
pub struct UberStateTrigger {
    pub identifier: UberIdentifier,
    #[VType] pub condition: Option<UberStateCondition>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, VVariant)]
pub struct UberStateCondition {
    pub comparator: UberStateComparator,
    #[VWrap] pub value: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UberStateComparator {
    Equals,
    Greater,
    GreaterOrEquals,
    Less,
    LessOrEquals,
}

impl UberStateCondition {
    // consider <3 and <=2 to be equal
    pub fn functionally_eq(&self, other: &Self) -> bool {
        match (&self.comparator, &other.comparator) {
            (UberStateComparator::Greater, UberStateComparator::GreaterOrEquals) | (UberStateComparator::LessOrEquals, UberStateComparator::Less)
            => other.value.saturating_sub(self.value) == 1,
            (UberStateComparator::GreaterOrEquals, UberStateComparator::Greater) | (UberStateComparator::Less, UberStateComparator::LessOrEquals)
            => self.value.saturating_sub(other.value) == 1,
            _ => self.comparator == other.comparator && self.value == other.value,
        }
    }
}
impl FromStr for UberStateTrigger {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = parser::new(input);
        let item = VUberStateTrigger::parse(&mut parser).map_err(|err| err.verbose_display())?
            .resolve(&FxHashMap::default())?;
        parser.expect_end().map_err(|err| err.verbose_display())?;
        Ok(item)
    }
}
impl UberStateTrigger {
    pub fn code(&self) -> CodeDisplay<UberStateTrigger> {
        CodeDisplay::new(self, |s, f| {
            s.identifier.code().fmt(f)?;
            if let Some(condition) = &s.condition {
                condition.fmt(f)?;
            }

            Ok(())
        })
    }

    pub(crate) fn check(&self, identifier: UberIdentifier, value: f32) -> bool {
        self.identifier == identifier && self.check_value(value)
    }
    pub(crate) fn check_value(&self, value: f32) -> bool {
        self.condition.as_ref().map_or(true, |condition| condition.met_by(value))
    }

    // consider <3 and <=2 to be equal
    pub fn functionally_eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier && match &self.condition {
            Some(condition) => other.condition.as_ref().map_or(false, |other| condition.functionally_eq(other)),
            None => other.condition.is_none(),
        }
    }

    #[inline]
    pub fn spawn() -> UberStateTrigger {
        UberStateTrigger {
            identifier: UberIdentifier::spawn(),
            condition: None,
        }
    }
    #[inline]
    pub fn load() -> UberStateTrigger {
        UberStateTrigger {
            identifier: UberIdentifier::load(),
            condition: None,
        }
    }

    /// For convenience, [`UberStateTrigger`]s are used to represent what UberStates [`Node`]s correspond to.
    /// 
    /// These have additional guarantees since they always use the `>=` [`UberStateOperator`].
    /// 
    /// I am lazy and don't want to express those additional guarantees through the type system so this is a private helper function to get the associated value when working with those nodes.
    /// 
    /// [`Node`]: crate::world::graph::Node
    /// [`UberStateOperator`]: crate::item::UberStateOperator
    pub(crate) fn set_value(&self) -> u32 {
        self.condition.as_ref().map_or(1, |condition| condition.value)
    }
}

impl Display for UberStateCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.comparator, self.value)
    }
}

impl UberStateCondition {
    pub fn met_by(&self, value: f32) -> bool {
        match self.comparator {
            UberStateComparator::Equals => value == self.value as f32,
            UberStateComparator::Greater => value > self.value as f32,
            UberStateComparator::GreaterOrEquals => value >= self.value as f32,
            UberStateComparator::Less => value < self.value as f32,
            UberStateComparator::LessOrEquals => value <= self.value as f32,
        }
    }
}

impl Display for UberStateComparator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => "=",
            Self::Greater => ">",
            Self::GreaterOrEquals => ">=",
            Self::Less => "<",
            Self::LessOrEquals => "<=",
        }.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_eq() {
        let trigger = |s| UberStateTrigger::from_str(s).unwrap();

        assert!(trigger("9|0>1").functionally_eq(&trigger("9|0>=2")));
        assert!(trigger("9|0>=2").functionally_eq(&trigger("9|0>1")));
        assert!(!trigger("9|0=2").functionally_eq(&trigger("9|0>=2")));
        assert!(!trigger("9|0=2").functionally_eq(&trigger("9|0")));
        assert!(trigger("9|0<2").functionally_eq(&trigger("9|0<=1")));
        assert!(trigger("9|0<=1").functionally_eq(&trigger("9|0<2")));
    }
}
