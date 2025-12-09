use std::borrow::Cow;

use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema, Type, schema::SchemaType},
};
use wotw_seedgen::data::{MapIcon, VariantArray};

#[derive(ToSchema)]
#[schema(value_type = (i32, i32))]
pub struct UberIdentifierSchema(());

pub struct MapIconSchema;

impl PartialSchema for MapIconSchema {
    fn schema() -> RefOr<Schema> {
        RefOr::T(
            ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::String))
                .enum_values(Some(MapIcon::VARIANTS.iter().map(MapIcon::to_string)))
                .into(),
        )
    }
}

impl ToSchema for MapIconSchema {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("MapIconKind")
    }
}

pub struct PositionSchema;

impl PartialSchema for PositionSchema {
    fn schema() -> RefOr<Schema> {
        RefOr::T(
            ObjectBuilder::new()
                .property("x", utoipa::schema!(f32))
                .required("x")
                .property("y", utoipa::schema!(f32))
                .required("y")
                .into(),
        )
    }
}

impl ToSchema for PositionSchema {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("Position")
    }
}

pub struct ComparatorSchema;

impl PartialSchema for ComparatorSchema {
    fn schema() -> RefOr<Schema> {
        RefOr::T(
            ObjectBuilder::new()
                .schema_type(SchemaType::new(Type::Number))
                .enum_values(Some(0..=5))
                .into(),
        )
    }
}

impl ToSchema for ComparatorSchema {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("Comparator")
    }
}
