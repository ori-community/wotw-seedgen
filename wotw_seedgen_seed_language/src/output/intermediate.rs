use super::StringOrPlaceholder;
use crate::ast;
use ordered_float::OrderedFloat;
use std::fmt::{self, Display};
use wotw_seedgen_assets::UberStateAlias;

// TODO is this still used for anything other than variables?
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
            Constant::ScreenPosition(value) => value.fmt(f),
            Constant::CoordinateSystem(value) => value.fmt(f),
        }
    }
}
