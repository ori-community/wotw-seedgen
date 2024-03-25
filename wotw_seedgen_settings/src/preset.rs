use crate::{Difficulty, Spawn, Trick};
use rustc_hash::{FxHashMap, FxHashSet};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A collection of settings that can be applied to existing settings
///
/// Use [`UniverseSettings::apply_preset`](crate::settings::UniverseSettings::apply_preset) to merge a [`UniversePreset`] into existing [`UniverseSettings`](crate::settings::UniverseSettings)
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_settings::UniversePreset;
/// use wotw_seedgen_settings::WorldPreset;
/// use wotw_seedgen_settings::UniverseSettings;
/// use wotw_seedgen_settings::Spawn;
/// use wotw_seedgen_settings::NoPresetAccess;
///
/// let mut universe_settings = UniverseSettings::new("seed".to_string());
///
/// let preset = UniversePreset {
///     world_settings: Some(vec![
///         WorldPreset {
///             spawn: Some(Spawn::Random),
///             ..Default::default()
///         }
///     ]),
///     ..Default::default()
/// };
///
/// universe_settings.apply_preset(preset, &NoPresetAccess);
/// assert_eq!(universe_settings.world_settings[0].spawn, Spawn::Random);
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct UniversePreset {
    /// User-targetted information about the preset
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub info: Option<PresetInfo>,
    /// Names of further [`UniversePreset`]s to use
    ///
    /// When applying the parent preset, these presets will be searched as .json files in the current and /presets child directory
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub includes: Option<FxHashSet<String>>,
    /// The seed that determines all randomness
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<String>,
    /// The individual settings for each world of the seed
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub world_settings: Option<Vec<WorldPreset>>,
}

/// A collection of settings that can be applied to one world of the existing settings
///
/// Use [`WorldSettings::apply_world_preset`](crate::settings::WorldSettings::apply_world_preset) to merge a [`WorldPreset`] into existing [`WorldSettings`](crate::settings::WorldSettings)
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_settings::WorldPreset;
/// use wotw_seedgen_settings::WorldSettings;
/// use wotw_seedgen_settings::Spawn;
/// use wotw_seedgen_settings::NoPresetAccess;
///
/// let mut world_settings = WorldSettings::default();
///
/// let world_preset = WorldPreset {
///     spawn: Some(Spawn::Random),
///     ..Default::default()
/// };
///
/// world_settings.apply_world_preset(world_preset, &NoPresetAccess);
/// assert_eq!(world_settings.spawn, Spawn::Random);
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
// TODO replace hashsets with vecs?
pub struct WorldPreset {
    /// User-targetted information about the preset
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub info: Option<PresetInfo>,
    /// Names of further [`WorldPreset`]s to use
    ///
    /// When applying the parent preset, these presets will be searched as .json files in the current and /presets child directory
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub includes: Option<FxHashSet<String>>,
    /// Spawn destination
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub spawn: Option<Spawn>,
    /// Logically expected difficulty
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub difficulty: Option<Difficulty>,
    /// Logically expected tricks
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tricks: Option<FxHashSet<Trick>>,
    /// Logically assume hard in-game difficulty
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub hard: Option<bool>,
    /// Names of snippets to use
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub snippets: Option<Vec<String>>,
    /// Configuration to pass to snippets
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub snippet_config: Option<FxHashMap<String, FxHashMap<String, String>>>,
}

/// Information for the user about a [`UniversePreset`] or [`WorldPreset`]
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct PresetInfo {
    /// Display name
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub name: Option<String>,
    /// Extended description
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub description: Option<String>,
    /// Where to present the preset
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub group: Option<PresetGroup>,
}

/// Special groups to display a preset in
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PresetGroup {
    /// Generally, only one base preset will be used at a time.
    ///
    /// The most common form of base presets are the difficulty presets, such as "Moki" and "Gorlek"
    Base,
}
