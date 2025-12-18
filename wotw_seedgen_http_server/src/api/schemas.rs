use std::borrow::Cow;

use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ArrayBuilder, RefOr, Schema, schema::ArrayItems},
};

pub struct UberIdentifierSchema;

impl PartialSchema for UberIdentifierSchema {
    fn schema() -> RefOr<Schema> {
        ArrayBuilder::new()
            .items(ArrayItems::False)
            .prefix_items([utoipa::schema!(i32), utoipa::schema!(i32)])
            .into()
    }
}

impl ToSchema for UberIdentifierSchema {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("UberIdentifier")
    }
}
