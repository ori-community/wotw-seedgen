//! Utilities to deal with some of the simpler asset files used by the seed generator.

mod loc_data;
pub use loc_data::{LocData, LocDataEntry};
mod state_data;
pub use state_data::{StateData, StateDataEntry};
mod uber_state_data;
pub use uber_state_data::{
    UberStateAlias, UberStateData, UberStateDataEntry, UberStateDump, UberStateDumpGroup,
    UberStateDumpMember, UberStateDumpValueType, UberStateValue,
};
mod snippet_access;
pub use snippet_access::{ChainedSnippetAccess, InlineSnippets, NoSnippetAccess, SnippetAccess};
mod presets;
pub use presets::{
    NoPresetAccess, PresetAccess, PresetGroup, PresetInfo, UniversePreset, UniversePresetSettings,
    WorldPreset, WorldPresetSettings, CURRENT_ASSETS_VERSION,
};
mod file_access;
pub use file_access::{
    canonicalize, create_dir_all, file_create, file_err, file_open, write, AssetCache,
    AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues, DefaultFileAccess,
    PlandoFileAccess, PresetFileAccess, SnippetFileAccess, Watcher, WatcherError, EXECUTABLE_DIR,
    LOG_DATA_DIR, RANDOMIZER_USER_DATA_DIR, SEEDGEN_USER_DATA_DIR,
};
mod test_access;
pub use test_access::{TestAccess, TEST_ASSETS};
