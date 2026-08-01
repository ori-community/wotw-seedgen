use serde::{Deserialize, Serialize};
use wotw_seedgen_data::UniverseSettings;
use wotw_seedgen_git_info::GitInfo;

// TODO direct state sets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedgenInfo {
    pub universe_settings: UniverseSettings,
    pub world_index: usize,
    pub spawn_identifier: String,
    pub git_info: Option<GitInfo>,
}

impl SeedgenInfo {
    pub fn new(
        universe_settings: UniverseSettings,
        world_index: usize,
        spawn_identifier: String,
    ) -> Self {
        Self {
            universe_settings,
            world_index,
            spawn_identifier,
            git_info: Some(GitInfo::new()),
        }
    }
}
