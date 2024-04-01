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
//! - `uber_sate_data`: Parsing for `uber_state_dump.json`

#[cfg(feature = "wotw_seedgen_data")]
pub use wotw_seedgen_data as data;

#[cfg(feature = "loc_data")]
mod loc_data;
use std::path::Path;

#[cfg(feature = "loc_data")]
pub use loc_data::{LocData, LocDataEntry};
#[cfg(feature = "presets")]
mod presets;
#[cfg(feature = "state_data")]
mod state_data;
#[cfg(feature = "state_data")]
pub use state_data::{StateData, StateDataEntry};
#[cfg(feature = "uber_state_data")]
mod uber_state_data;
#[cfg(feature = "uber_state_data")]
pub use uber_state_data::{UberStateAlias, UberStateData, UberStateDataEntry, UberStateValue};

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

// TODO maybe this should be in seed_language?
/// Resolves include commands in the seed language
pub trait SnippetAccess {
    /// Resolve `!include(<identifier>)`
    fn read_snippet(&self, identifier: &str) -> Result<Source, String>;
    /// Resolve binary incldues such as `!include_icon(<path>)`
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String>;
}
