use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::iter;
use wotw_seedgen_settings::{
    Difficulty, GreaterOneU8, Spawn, Trick, UniverseSettings, WorldSettings,
};

/// The current version number for the assets directory.
/// Presets targetting older versions may throw an error if they're affected by breaking changes.
/// Check `assets/preset compability.md` for details
pub const CURRENT_ASSETS_VERSION: u8 = 1;

/// Information for the user about a [`UniversePreset`] or [`WorldPreset`]
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetInfo {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Extended description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Where to present the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<PresetGroup>,
}

/// Special groups to display a preset in
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PresetGroup {
    /// Generally, only one base preset will be used at a time.
    ///
    /// The most common form of base presets are the difficulty presets, such as "Moki" and "Gorlek"
    Base,
}

/// A collection of settings that can be applied to existing settings
///
/// Use [`UniversePreset::apply`] to merge a `UniversePreset` into existing [`UniverseSettings`]
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_assets::UniversePreset;
/// use wotw_seedgen_assets::{NoPresetAccess, UniversePresetSettings, WorldPresetSettings};
/// use wotw_seedgen_assets::settings::{Spawn, UniverseSettings};
///
/// let mut universe_settings = UniverseSettings::new("seed".to_string());
///
/// let preset = UniversePreset {
///     assets_version: 1,
///     info: None,
///     settings: UniversePresetSettings {
///         world_settings: Some(vec![
///             WorldPresetSettings {
///                 spawn: Some(Spawn::Random),
///                 ..Default::default()
///             }
///         ]),
///         ..Default::default()
///     }
/// };
///
/// preset.apply(&mut universe_settings, &NoPresetAccess);
/// assert_eq!(universe_settings.world_settings[0].spawn, Spawn::Random);
/// ```
///
/// [`UniverseSettings`]: wotw_seedgen_settings::UniverseSettings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniversePreset {
    /// Assets version this preset is compatible with
    #[serde(default)]
    pub assets_version: u8,
    /// User-targetted information about the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<PresetInfo>,
    /// Settings to apply
    #[serde(flatten)]
    pub settings: UniversePresetSettings,
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
        self.check_compability()?
            ._apply(settings, already_applied, preset_access)
    }

    fn check_compability(mut self) -> Result<UniversePresetSettings, String> {
        const CONVERSIONS: [fn(&mut UniversePresetSettings) -> Result<(), String>; 1] =
            [UniversePresetSettings::check_compability_0_to_1];

        for conversion in CONVERSIONS
            .get(self.assets_version as usize..)
            .into_iter()
            .flatten()
        {
            conversion(&mut self.settings)?;
        }

        Ok(self.settings)
    }
}

/// Settings to apply to [`UniverseSettings`]
///
/// Mostly used inside a [`UniversePreset`] which offers compability features
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UniversePresetSettings {
    /// Names of further [`UniversePreset`]s to use
    ///
    /// A [`PresetAccess::universe_preset`] implementation may be used to resolve the identifiers
    ///
    /// [`PresetAccess::universe_preset`]: crate::PresetAccess::universe_preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<FxHashSet<String>>,
    /// The seed that determines all randomness
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    /// The individual settings for each world of the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_settings: Option<Vec<WorldPresetSettings>>,
}

impl UniversePresetSettings {
    /// Apply these `UniversePresetSettings` to some [`UniverseSettings`]
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

    fn check_compability_0_to_1(&mut self) -> Result<(), String> {
        for world_settings in self.world_settings.iter_mut().flatten() {
            world_settings.check_compability_0_to_1()?;
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

/// A collection of settings that can be applied to one world of existing settings
///
/// Use [`WorldPreset::apply`] to merge a `WorldPreset` into existing [`WorldSettings`]
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_assets::WorldPreset;
/// use wotw_seedgen_assets::{NoPresetAccess, WorldPresetSettings};
/// use wotw_seedgen_assets::settings::{Spawn, WorldSettings};
///
/// let mut world_settings = WorldSettings::default();
///
/// let world_preset = WorldPreset {
///     assets_version: 1,
///     info: None,
///     settings: WorldPresetSettings {
///         spawn: Some(Spawn::Random),
///         ..Default::default()
///     }
/// };
///
/// world_preset.apply(&mut world_settings, &NoPresetAccess);
/// assert_eq!(world_settings.spawn, Spawn::Random);
/// ```
///
/// [`WorldSettings`]: wotw_seedgen_settings::WorldSettings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldPreset {
    /// Assets version this preset is compatible with
    #[serde(default)]
    pub assets_version: u8,
    /// User-targetted information about the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<PresetInfo>,
    /// Settings to apply
    #[serde(flatten)]
    pub settings: WorldPresetSettings,
}

impl WorldPreset {
    /// Apply this `WorldPreset` to some [`WorldSettings`]
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
        self.check_compability()?
            ._apply(settings, already_applied, preset_access)
    }

    fn check_compability(mut self) -> Result<WorldPresetSettings, String> {
        const CONVERSIONS: [fn(&mut WorldPresetSettings) -> Result<(), String>; 1] =
            [WorldPresetSettings::check_compability_0_to_1];

        for conversion in CONVERSIONS
            .get(self.assets_version as usize..)
            .into_iter()
            .flatten()
        {
            conversion(&mut self.settings)?;
        }

        Ok(self.settings)
    }
}

/// Settings to apply to [`WorldSettings`]
///
/// Mostly used inside a [`WorldPreset`] which offers compability features
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
// TODO replace hashsets with vecs?
pub struct WorldPresetSettings {
    /// Names of further [`WorldPreset`]s to use
    ///
    /// A [`PresetAccess::world_preset`] implementation may be used to resolve the identifiers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<FxHashSet<String>>,
    /// Spawn location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn: Option<Spawn>,
    /// Logically expected difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,
    /// Logically expected tricks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tricks: Option<FxHashSet<Trick>>,
    /// Logically assume hard in-game difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard: Option<bool>,
    /// Randomize door connections with the given max loop size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub randomize_doors: Option<GreaterOneU8>,
    /// Names of snippets to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippets: Option<Vec<String>>,
    /// Configuration to pass to snippets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_config: Option<FxHashMap<String, FxHashMap<String, String>>>,
}

impl WorldPresetSettings {
    /// Apply these `WorldPresetSettings` to some [`WorldSettings`]
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
            includes,
            difficulty,
            tricks,
            spawn,
            hard,
            randomize_doors,
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
        if let Some(randomize_doors) = randomize_doors {
            settings.randomize_doors = Some(randomize_doors);
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

    fn check_compability_0_to_1(&mut self) -> Result<(), String> {
        for snippet in self.snippets.iter_mut().flatten() {
            match &snippet[..] {
                "util_twillen" => return err_removed("util_twillen"),
                "vanilla_opher_upgrades" => return err_removed("vanilla_opher_upgrades"),
                "bonus_opher_upgrades" => return err_removed("bonus_opher_upgrades"),
                "open_mode" => return err_removed("open_mode"),
                "autoplants" => *snippet = "no_cutscenes".to_string(),
                "better_stomp" | "fragment_overflow" | "tp_refill" | "shriek_escape_health_bar" => {
                    *snippet = "better_mechanics".to_string()
                }
                _ => {}
            }
        }

        Ok(())
    }
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

fn err_removed(identifier: &str) -> Result<(), String> {
    Err(format!(
        "{identifier} was removed from the settings available by default"
    ))
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
        panic!("Attempted to read world preset \"{identifier}\" while explicitely using NoPresetAccess");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_incompatible() {
        let preset = WorldPreset {
            assets_version: 0,
            info: None,
            settings: WorldPresetSettings {
                snippets: Some(vec!["util_twillen".to_string()]),
                ..Default::default()
            },
        };
        assert_eq!(
            preset.check_compability().map(|_| ()),
            err_removed("util_twillen")
        );
    }

    #[test]
    fn upgrade() {
        let preset = WorldPreset {
            assets_version: 0,
            info: None,
            settings: WorldPresetSettings {
                snippets: Some(vec![
                    "bonus_items".to_string(),
                    "autoplants".to_string(),
                    "better_stomp".to_string(),
                ]),
                ..Default::default()
            },
        };
        assert_eq!(
            preset.check_compability(),
            Ok(WorldPresetSettings {
                snippets: Some(vec![
                    "bonus_items".to_string(),
                    "no_cutscenes".to_string(),
                    "better_mechanics".to_string()
                ]),
                ..Default::default()
            })
        );
    }
}
