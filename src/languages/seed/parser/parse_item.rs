use rustc_hash::FxHashMap;

use crate::{
    Item, VItem,
    header::VResolve,
};

impl Item {
    /// Parse item syntax
    pub fn parse(item: &str) -> Result<Item, String> {
        let v_item = VItem::parse(item)?;
        v_item.resolve(&FxHashMap::default()).map_err(|_| "$PARAM() syntax is unique to headers and cannot be used in seed files".to_string())
    }
}
