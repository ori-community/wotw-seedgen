use super::StringOrPlaceholder;
use crate::{
    assets::UberStateAlias,
    seed_language::{ast, types::Type},
};
use ordered_float::OrderedFloat;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableValue {
    Literal(Literal),
    Reference(Reference),
}

impl VariableValue {
    pub fn ty(&self) -> Type {
        match self {
            VariableValue::Literal(literal) => literal.ty(),
            VariableValue::Reference(reference) => reference.ty(),
        }
    }
}

macro_rules! impl_variable_value_from {
    ($from:ty, $tag:ident) => {
        impl From<$from> for VariableValue {
            fn from(value: $from) -> Self {
                Self::$tag(value)
            }
        }
    };
}

impl_variable_value_from!(Literal, Literal);
impl_variable_value_from!(Reference, Reference);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    UberIdentifier(UberStateAlias),
    Boolean(bool),
    Integer(i32),
    Float(OrderedFloat<f32>),
    String(StringOrPlaceholder),
    Constant(Constant),
    IconAsset(String),
    CustomIcon(String),
}

macro_rules! impl_literal_from {
    ($from:ty, $tag:ident) => {
        impl From<$from> for Literal {
            fn from(value: $from) -> Self {
                Self::$tag(value)
            }
        }

        impl From<$from> for VariableValue {
            fn from(value: $from) -> Self {
                Self::Literal(Literal::from(value))
            }
        }
    };
}

impl_literal_from!(UberStateAlias, UberIdentifier);
impl_literal_from!(bool, Boolean);
impl_literal_from!(i32, Integer);
impl_literal_from!(OrderedFloat<f32>, Float);
impl_literal_from!(Constant, Constant);

pub use ast::Constant;

impl Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::ClientEvent(value) => value.fmt(f),
            Constant::Skill(value) => value.fmt(f),
            Constant::Shard(value) => value.fmt(f),
            Constant::Teleporter(value) => value.fmt(f),
            Constant::WeaponUpgrade(value) => value.fmt(f),
            Constant::Equipment(value) => value.fmt(f),
            Constant::Zone(value) => value.fmt(f),
            Constant::GenericIcon(value) => value.fmt(f),
            Constant::OpherIcon(value) => value.fmt(f),
            Constant::LupoIcon(value) => value.fmt(f),
            Constant::GromIcon(value) => value.fmt(f),
            Constant::TuleyIcon(value) => value.fmt(f),
            Constant::MapIcon(value) => value.fmt(f),
            Constant::EquipSlot(value) => value.fmt(f),
            Constant::WheelItemPosition(value) => value.fmt(f),
            Constant::WheelBind(value) => value.fmt(f),
            Constant::Alignment(value) => value.fmt(f),
            Constant::HorizontalAnchor(value) => value.fmt(f),
            Constant::VerticalAnchor(value) => value.fmt(f),
            Constant::Corner(value) => value.fmt(f),
            Constant::CoordinateSystem(value) => value.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    BooleanStack(usize),
    IntegerStack(usize),
    FloatStack(usize),
    StringStack(usize),
}
