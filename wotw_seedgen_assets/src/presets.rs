use rustc_hash::{FxHashMap, FxHashSet};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::iter;
use wotw_seedgen_settings::{Difficulty, Spawn, Trick, UniverseSettings, WorldSettings};

/// A collection of settings that can be applied to existing settings
///
/// Use [`UniversePreset::apply`] to merge a `UniversePreset` into existing [`UniverseSettings`]
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_assets::UniversePreset;
/// use wotw_seedgen_assets::{NoPresetAccess, WorldPreset};
/// use wotw_seedgen_assets::settings::{Spawn, UniverseSettings};
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
/// preset.apply(&mut universe_settings, &NoPresetAccess);
/// assert_eq!(universe_settings.world_settings[0].spawn, Spawn::Random);
/// ```
///
/// [`UniverseSettings`]: wotw_seedgen_settings::UniverseSettings
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
    /// A [`PresetAccess::universe_preset`] implementation may be used to resolve the identifiers
    ///
    /// [`PresetAccess::universe_preset`]: crate::PresetAccess::universe_preset
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub includes: Option<FxHashSet<String>>,
    /// The seed that determines all randomness
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub seed: Option<String>,
    /// The individual settings for each world of the seed
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub world_settings: Option<Vec<WorldPreset>>,
}

impl UniversePreset {
    /// Apply this `UniversePreset` to some [`UniverseSettings`]
    ///
    /// This follows various rules to retain all unrelated parts of the existing Settings:
    /// - Any [`None`] values of the preset will be ignored
    /// - [`Vec`]s will be appended to the current contents
    /// - Other values will be overwritten
    /// - If the number of worlds matches, the preset will be applied to each world per index
    /// - If only one world is in the preset, but multiple in the existing settings, the preset is applied to all worlds
    /// - If multiple worlds are in the preset, but only one in the existing settings, the existing settings will be copied for all worlds, then the preset will be applied per index
    /// - If multiple worlds are in both and their number does not match, returns an error
    /// - Nested presets will be applied before the parent preset
    pub fn apply<A: PresetAccess>(
        self,
        settings: &mut UniverseSettings,
        preset_access: &A,
    ) -> Result<(), String> {
        self._apply(settings, &mut vec![], preset_access)
    }

    fn _apply<A: PresetAccess>(
        self,
        settings: &mut UniverseSettings,
        already_applied: &mut Vec<String>,
        preset_access: &A,
    ) -> Result<(), String> {
        let Self {
            info: _,
            includes,
            world_settings,
            seed,
        } = self;

        for identifier in includes.into_iter().flatten() {
            include_universe_preset(settings, identifier, already_applied, preset_access)?;
        }

        if let Some(seed) = seed {
            settings.seed = seed;
        }

        let setting_worlds = settings.world_count();

        if let Some(preset_world_settings) = world_settings {
            let preset_worlds = preset_world_settings.len();

            if preset_worlds == 0 {
                // do nothing
            } else if setting_worlds == preset_worlds {
                for (world_settings, preset_world_settings) in settings
                    .world_settings
                    .iter_mut()
                    .zip(preset_world_settings)
                {
                    preset_world_settings.apply(world_settings, preset_access)?;
                }
            } else if preset_worlds == 1 {
                for world_settings in &mut settings.world_settings {
                    preset_world_settings[0]
                        .clone()
                        .apply(world_settings, preset_access)?;
                }
            } else if setting_worlds == 1 {
                settings.world_settings.extend(
                    iter::repeat(settings.world_settings[0].clone()).take(preset_worlds - 1),
                );
                for (world_settings, preset_world_settings) in settings
                    .world_settings
                    .iter_mut()
                    .zip(preset_world_settings)
                {
                    preset_world_settings.apply(world_settings, preset_access)?;
                }
            } else {
                return Err(format!("Cannot apply preset with {preset_worlds} worlds to settings with {setting_worlds} worlds"));
            }
        }

        Ok(())
    }
}

fn include_universe_preset<A: PresetAccess>(
    settings: &mut UniverseSettings,
    identifier: String,
    already_applied: &mut Vec<String>,
    preset_access: &A,
) -> Result<(), String> {
    // Prevent cyclic patterns
    if already_applied.contains(&identifier) {
        return Ok(());
    }
    let preset = preset_access.universe_preset(&identifier)?;
    already_applied.push(identifier);
    preset._apply(settings, already_applied, preset_access)
}

fn include_world_preset<A: PresetAccess>(
    settings: &mut WorldSettings,
    identifier: String,
    already_applied: &mut Vec<String>,
    preset_access: &A,
) -> Result<(), String> {
    // Prevent cyclic patterns
    if already_applied.contains(&identifier) {
        return Ok(());
    }
    let preset = preset_access.world_preset(&identifier)?;
    already_applied.push(identifier);
    preset._apply(settings, already_applied, preset_access)
}

/// A collection of settings that can be applied to one world of the existing settings
///
/// Use [`WorldPreset::apply`] to merge a `WorldPreset` into existing [`WorldSettings`]
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_assets::WorldPreset;
/// use wotw_seedgen_assets::NoPresetAccess;
/// use wotw_seedgen_assets::settings::{Spawn, WorldSettings};
///
/// let mut world_settings = WorldSettings::default();
///
/// let world_preset = WorldPreset {
///     spawn: Some(Spawn::Random),
///     ..Default::default()
/// };
///
/// world_preset.apply(&mut world_settings, &NoPresetAccess);
/// assert_eq!(world_settings.spawn, Spawn::Random);
/// ```
///
/// [`WorldSettings`]: wotw_seedgen_settings::WorldSettings
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
    /// A [`PresetAccess::world_preset`] implementation may be used to resolve the identifiers
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub includes: Option<FxHashSet<String>>,
    /// Spawn location
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

impl WorldPreset {
    pub fn apply<A: PresetAccess>(
        self,
        settings: &mut WorldSettings,
        preset_access: &A,
    ) -> Result<(), String> {
        self._apply(settings, &mut vec![], preset_access)
    }

    fn _apply<A: PresetAccess>(
        self,
        settings: &mut WorldSettings,
        already_applied: &mut Vec<String>,
        preset_access: &A,
    ) -> Result<(), String> {
        let Self {
            info: _,
            includes,
            difficulty,
            tricks,
            spawn,
            hard,
            snippets,
            snippet_config,
        } = self;

        for identifier in includes.into_iter().flatten() {
            include_world_preset(settings, identifier, already_applied, preset_access)?;
        }

        // TODO surely there's a handy command for this
        if let Some(difficulty) = difficulty {
            settings.difficulty = difficulty;
        }
        if let Some(tricks) = tricks {
            settings.tricks.extend(tricks);
        }
        if let Some(spawn) = spawn {
            settings.spawn = spawn;
        }
        if let Some(hard) = hard {
            settings.hard = hard;
        }
        if let Some(snippets) = snippets {
            settings.snippets.extend(snippets);
        }
        if let Some(snippet_config) = snippet_config {
            for (snippet_name, config) in snippet_config {
                let entry = settings.snippet_config.entry(snippet_name).or_default();
                for (config_name, value) in config {
                    entry.insert(config_name, value);
                }
            }
        }

        Ok(())
    }
}

/// Access to presets by identifier
pub trait PresetAccess {
    /// Returns the [`UniversePreset`] with the given identifier
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String>;
    /// Returns the [`WorldPreset`] with the given identifier
    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String>;
}

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
