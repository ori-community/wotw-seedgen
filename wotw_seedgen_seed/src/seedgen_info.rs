use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use wotw_seedgen_data::UniverseSettings;
use wotw_seedgen_git_info::GitInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedgenInfo {
    pub universe_settings: UniverseSettings,
    pub world_index: usize,
    pub spawn_identifier: String,
    pub logical_state_sets: FxHashSet<String>,
    pub git_info: Option<GitInfo>,
}

impl SeedgenInfo {
    pub fn new(
        universe_settings: UniverseSettings,
        world_index: usize,
        spawn_identifier: String,
        logical_state_sets: FxHashSet<String>,
    ) -> Self {
        Self {
            universe_settings,
            world_index,
            spawn_identifier,
            logical_state_sets,
            git_info: Some(GitInfo::new()),
        }
    }
}
