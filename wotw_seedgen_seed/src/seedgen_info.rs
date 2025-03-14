use serde::{Deserialize, Serialize};
use wotw_seedgen_seed_language::assets::settings::UniverseSettings;

// TODO direct state sets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedgenInfo {
    pub universe_settings: UniverseSettings,
    pub world_index: usize,
    pub spawn_identifier: String,
}
