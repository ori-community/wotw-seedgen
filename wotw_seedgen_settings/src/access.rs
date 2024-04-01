use crate::{UniversePreset, WorldPreset};

/// Access to presets by identifier
pub trait PresetAccess {
    /// Returns the [`UniversePreset`] with the given identifier
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String>;
    /// Returns the [`WorldPreset`] with the given identifier
    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String>;
}

// TODO put this everywhere
/// [`PresetAccess`] implementation that forbids accessing any presets
///
/// You may use this is you're using presets that don't include any other presets
pub struct NoPresetAccess;
impl PresetAccess for NoPresetAccess {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        panic!("Attempted to read universe preset \"{identifier}\" while explicitely using NoPresetAccess");
    }
    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        panic!("Attempted to read universe preset \"{identifier}\" while explicitely using NoPresetAccess");
    }
}
