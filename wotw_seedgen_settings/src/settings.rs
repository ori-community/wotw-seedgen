use crate::{PresetAccess, UniversePreset, WorldPreset};
use rustc_hash::{FxHashMap, FxHashSet};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::iter;
#[cfg(feature = "strum")]
use strum::{Display, EnumString, EnumVariantNames};

/// A representation of all the relevant settings when generating a seed
///
/// Using the same settings will result in generating the same seed (as long as the same seedgen version and headers are used)
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_settings::UniverseSettings;
/// use wotw_seedgen_settings::WorldSettings;
///
/// let universe_settings = UniverseSettings::new("seed".to_string());
///
/// assert_eq!(universe_settings.world_count(), 1);
/// assert_eq!(universe_settings.world_settings[0], WorldSettings::default());
/// assert_eq!(universe_settings.seed, "seed");
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct UniverseSettings {
    /// The seed that determines all randomness
    pub seed: String,
    /// The individual settings for each world of the seed
    ///
    /// This should never be empty
    pub world_settings: Vec<WorldSettings>,
}

impl UniverseSettings {
    pub fn new(seed: String) -> Self {
        Self {
            seed,
            world_settings: vec![WorldSettings::default()],
        }
    }

    /// Apply the settings from a [`UniversePreset`]
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
    pub fn apply_preset<A: PresetAccess>(
        &mut self,
        preset: UniversePreset,
        preset_access: &A,
    ) -> Result<(), String> {
        self.apply_preset_guarded(preset, &mut vec![], preset_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_preset_guarded<A: PresetAccess>(
        &mut self,
        preset: UniversePreset,
        already_applied: &mut Vec<String>,
        preset_access: &A,
    ) -> Result<(), String> {
        let UniversePreset {
            info: _,
            includes,
            world_settings,
            seed,
        } = preset;

        if let Some(includes) = includes {
            for nested_preset in includes {
                self.apply_nested_preset(nested_preset, already_applied, preset_access)?;
            }
        }

        if let Some(seed) = seed {
            self.seed = seed;
        }

        let setting_worlds = self.world_count();

        if let Some(preset_world_settings) = world_settings {
            let preset_worlds = preset_world_settings.len();

            if preset_worlds == 0 {
                // do nothing
            } else if setting_worlds == preset_worlds {
                for (world_settings, preset_world_settings) in
                    self.world_settings.iter_mut().zip(preset_world_settings)
                {
                    world_settings.apply_world_preset(preset_world_settings, preset_access)?;
                }
            } else if preset_worlds == 1 {
                for world_settings in &mut self.world_settings {
                    world_settings
                        .apply_world_preset(preset_world_settings[0].clone(), preset_access)?;
                }
            } else if setting_worlds == 1 {
                let diff = preset_worlds - setting_worlds;
                self.world_settings
                    .extend(iter::repeat(self.world_settings[0].clone()).take(diff));
                for (world_settings, preset_world_settings) in
                    self.world_settings.iter_mut().zip(preset_world_settings)
                {
                    world_settings.apply_world_preset(preset_world_settings, preset_access)?;
                }
            } else {
                return Err(format!("Cannot apply preset with {preset_worlds} worlds to settings with {setting_worlds} worlds"));
            }
        }

        Ok(())
    }

    /// Find and apply nested presets
    fn apply_nested_preset<A: PresetAccess>(
        &mut self,
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
        self.apply_preset_guarded(preset, already_applied, preset_access)
    }

    /// Returns the number of worlds
    pub fn world_count(&self) -> usize {
        self.world_settings.len()
    }
}

/// Seed settings bound to a specific world of a seed
///
/// See the [Multiplayer wiki page](https://wiki.orirando.com/features/multiplayer) for an explanation of worlds
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
pub struct WorldSettings {
    /// Spawn destination
    pub spawn: Spawn,
    /// Logically expected difficulty
    pub difficulty: Difficulty,
    /// Logically expected tricks
    pub tricks: FxHashSet<Trick>,
    /// Logically assume hard in-game difficulty
    pub hard: bool,
    /// Names of snippets to use
    pub snippets: Vec<String>,
    /// Configuration to pass to snippets
    pub snippet_config: FxHashMap<String, FxHashMap<String, String>>,
}

impl WorldSettings {
    /// Checks whether these settings feature a random spawn location
    pub fn is_random_spawn(&self) -> bool {
        matches!(self.spawn, Spawn::Random | Spawn::FullyRandom)
    }

    /// Apply the settings from a [`WorldPreset`]
    ///
    /// This follows various rules to retain all unrelated parts of the existing Settings:
    /// - Any [`None`] values of the preset will be ignored
    /// - [`Vec`]s will be appended to the current contents
    /// - Other values will be overwritten
    /// - Nested presets will be applied before the parent preset
    pub fn apply_world_preset<A: PresetAccess>(
        &mut self,
        preset: WorldPreset,
        preset_access: &A,
    ) -> Result<(), String> {
        self.apply_world_preset_guarded(preset, &mut vec![], preset_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_world_preset_guarded<A: PresetAccess>(
        &mut self,
        preset: WorldPreset,
        already_applied: &mut Vec<String>,
        preset_access: &A,
    ) -> Result<(), String> {
        let WorldPreset {
            info: _,
            includes,
            difficulty,
            tricks,
            spawn,
            hard,
            snippets,
            snippet_config,
        } = preset;

        if let Some(includes) = includes {
            for nested_preset in includes {
                self.apply_nested_preset(nested_preset, already_applied, preset_access)?;
            }
        }

        // TODO surely there's a handy command for this
        if let Some(difficulty) = difficulty {
            self.difficulty = difficulty;
        }
        if let Some(tricks) = tricks {
            self.tricks.extend(tricks);
        }
        if let Some(spawn) = spawn {
            self.spawn = spawn;
        }
        if let Some(hard) = hard {
            self.hard = hard;
        }
        if let Some(snippets) = snippets {
            self.snippets.extend(snippets);
        }
        if let Some(snippet_config) = snippet_config {
            for (snippet_name, config) in snippet_config {
                let entry = self.snippet_config.entry(snippet_name).or_default();
                for (config_name, value) in config {
                    entry.insert(config_name, value);
                }
            }
        }

        Ok(())
    }

    /// Find and apply nested presets
    fn apply_nested_preset<A: PresetAccess>(
        &mut self,
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
        self.apply_world_preset_guarded(preset, already_applied, preset_access)
    }
}

/// The Spawn location, which may either be fixed or randomly decided during seed generation
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Spawn {
    /// Spawn in a specific location, described by the anchor name from the logic file
    Set(String),
    /// Spawn in a random location out of a curated set, depending on the seed's difficulty
    Random,
    /// Spawn on any valid anchor from the logic file
    FullyRandom,
}
pub const DEFAULT_SPAWN: &str = "MarshSpawn.Main";
impl Default for Spawn {
    fn default() -> Spawn {
        Spawn::Set(DEFAULT_SPAWN.to_string())
    }
}

/// The logical difficulty to expect in a seed
///
/// This represents how demanding the required core movement should be
/// Difficulties don't include glitches by default, these are handled separately with [`Trick`]s
///
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "strum",
    derive(Display, EnumString, EnumVariantNames),
    strum(serialize_all = "lowercase")
)]
pub enum Difficulty {
    #[default]
    Moki,
    Gorlek,
    Kii,
    Unsafe,
}

/// A Trick that can be logically required
///
/// This includes mostly Glitches but also other techniques that can be toggled for logic, such as damage boosting
///
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "strum", derive(Display, EnumString, EnumVariantNames))]
pub enum Trick {
    /// Grounded Sentry Jumps with Sword
    SwordSentryJump,
    /// Grounded Sentry Jump with Hammer
    HammerSentryJump,
    /// Breaking Walls from behind with Shuriken
    ShurikenBreak,
    /// Breaking Walls from behind with Sentry
    SentryBreak,
    /// Breaking Walls from behind with Hammer
    HammerBreak,
    /// Breaking Walls from behind with Spear
    SpearBreak,
    /// Melting Ice using Sentries
    SentryBurn,
    /// Removing Shriek's Killplane at Feeding Grounds
    RemoveKillPlane,
    /// Using the weapon wheel to cancel Launch
    LaunchSwap,
    /// Using the weapon wheel to cancel Sentry
    SentrySwap,
    /// Using the weapon wheel to cancel Flash
    FlashSwap,
    /// Using the weapon wheel to cancel Blaze
    BlazeSwap,
    /// Gaining speed off a wall with Regenerate and Dash
    WaveDash,
    /// Preserving jump momentum with Grenade
    GrenadeJump,
    /// Preserving Double Jump momentum with Hammer
    HammerJump,
    /// Preserving Double Jump momentum with Sword
    SwordJump,
    /// Redirecting projectiles with Grenade
    GrenadeRedirect,
    /// Redirecting projectiles with Sentry
    SentryRedirect,
    /// Cancelling falling momentum through the pause menu
    PauseHover,
    /// Storing a grounded jump into the air with Glide
    GlideJump,
    /// Preserving Glide Jump momentum with Hammer
    GlideHammerJump,
    /// Storing a grounded jump into the air with Spear
    SpearJump,
}
