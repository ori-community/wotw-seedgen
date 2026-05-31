use std::path::Path;

use wotw_seedgen_parse::Source;

use crate::assets::{PresetAccess, SnippetAccess, UniversePreset, WorldPreset};

/// [`PresetAccess`] and [`SnippetAccess`] implementation that forbids accessing any assets
///
/// You may use this is you're using assets that don't include any further assets
pub struct NoAccess;

impl PresetAccess for NoAccess {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        panic!(
            "Attempted to read universe preset \"{identifier}\" while explicitely using NoAccess"
        );
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        panic!("Attempted to read world preset \"{identifier}\" while explicitely using NoAccess");
    }

    fn available_universe_presets(&self) -> Vec<String> {
        vec![]
    }

    fn available_world_presets(&self) -> Vec<String> {
        vec![]
    }
}

impl SnippetAccess for NoAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        panic!("Attempted to read snippet \"{identifier}\" while explicitely using NoAccess");
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        panic!(
            "Attempted to read \"{}\" while explicitely using NoAccess",
            path.display()
        );
    }

    fn available_snippets(&self) -> Vec<String> {
        vec![]
    }
}
