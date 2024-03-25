//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information

mod access;
mod preset;
mod settings;

pub use access::{NoPresetAccess, PresetAccess};
pub use preset::{PresetGroup, PresetInfo, UniversePreset, WorldPreset};
pub use settings::{Difficulty, Spawn, Trick, UniverseSettings, WorldSettings, DEFAULT_SPAWN};
