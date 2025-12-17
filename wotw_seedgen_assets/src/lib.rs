//! Utilities to deal with some of the simpler asset files used by the seed generator. Support for the individual assets is feature gated.
//!
//! See also the `wotw_seedgen_static_assets` crate which parses assets at compile time, allowing you to include them statically.
//!
//! The more complex assets have their own crates: `wotw_seedgen_logic_language` for `areas.wotw` and `wotw_seedgen_seed_language` for `.wotws` files.
//!
//! ## Features
//!
//! - `loc_data`: Parsing for `loc_data.csv`
//! - `state_data`: Parsing for `state_data.csv`
//! - `uber_state_data`: Parsing for `uber_state_dump.json`
//! - `snippet_access`: Trait and implementations to access snippets
//! - `presets`: Parsing and access trait for preset files
//! - `file_access`: Filesystem-based implementation to access asset files

pub use wotw_seedgen_data as data;
pub use wotw_seedgen_settings as settings;

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
pub use snippet_access::{NoSnippetAccess, SnippetAccess};
mod presets;
pub use presets::{
    NoPresetAccess, PresetAccess, PresetGroup, PresetInfo, UniversePreset, UniversePresetSettings,
    WorldPreset, WorldPresetSettings, CURRENT_ASSETS_VERSION,
};
mod file_access;
pub use file_access::{
    file_err, AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets,
    DefaultAssetCacheValues, DefaultFileAccess, PlandoFileAccess, PresetFileAccess,
    SnippetFileAccess, Watcher, WatcherError, CONFIG_DIR, EXECUTABLE_DIR,
};

/// Representation of a source file with the necessary information to display useful error messages.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Source {
    /// An identifier to be used in error messages that should allow the reader to determine which file the error originated from.
    ///
    /// This might be the file path relative to the workspace root, or just the filename.
    pub id: String,
    /// The contents of the file, which will be referenced for better error messages.
    ///
    /// This should be the same contents you were parsing, otherwise error messages will reference arbitrary spans in your source and possibly panic.
    pub content: String, // TODO maybe use &str?
}

impl Source {
    /// Creates a new `Source` from its parts
    pub fn new(id: String, content: String) -> Self {
        Self { id, content }
    }
}
