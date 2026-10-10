use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use utoipa::ToSchema;
use wotw_seedgen_data::seed_language::metadata::Metadata;

/// Attributes specified by a plando source
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlandoAttributes {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl PlandoAttributes {
    pub fn from_source(source: &str) -> Self {
        Metadata::from_source(source).into()
    }
}

impl From<Metadata> for PlandoAttributes {
    fn from(value: Metadata) -> Self {
        Self {
            name: value.name,
            description: value.description,
        }
    }
}
