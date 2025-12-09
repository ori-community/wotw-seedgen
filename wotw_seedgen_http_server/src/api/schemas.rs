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
