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
pub use snippet_access::{ChainedSnippetAccess, InlineSnippets, SnippetAccess};
mod presets;
pub use presets::{
    PresetAccess, PresetGroup, PresetInfo, Tricks, UniversePreset, UniversePresetSettings,
    WorldPreset, WorldPresetSettings, CURRENT_ASSETS_VERSION,
};
mod no_access;
pub use no_access::NoAccess;
mod file_access;
pub use file_access::{
    canonicalize, create_dir_all, file_create, file_err, file_open, metadata, read_to_string,
    write, AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues,
    DefaultFileAccess, PlandoFileAccess, PresetFileAccess, SnippetFileAccess, Watcher,
    WatcherError, EXECUTABLE_DIR, LOG_DATA_DIR, RANDOMIZER_USER_DATA_DIR, SEEDGEN_USER_DATA_DIR,
};
#[cfg(any(test, feature = "test_helpers"))]
mod test_access;
#[cfg(any(test, feature = "test_helpers"))]
pub use test_access::{TestAccess, TEST_ASSETS};
