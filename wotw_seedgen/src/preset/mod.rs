// TODO maybe this could be derived from the settings?

use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{settings::{Trick, Difficulty, Goal, Spawn, CreateGame, HeaderConfig, InlineHeader}, util};

/// A collection of settings that can be applied to existing settings
/// 
/// # Examples
/// 
/// [`Preset`]s can be serialized and deserialized
/// 
/// ```
/// # use wotw_seedgen::Preset;
/// use wotw_seedgen::preset::WorldPreset;
/// use wotw_seedgen::settings::Difficulty;
/// 
/// let mut preset = Preset::default();
/// let mut world_settings = WorldPreset::default();
/// world_settings.difficulty = Some(Difficulty::Gorlek);
/// preset.world_settings = Some(vec![world_settings]);
/// 
/// let json = "{\"worldSettings\":[{\"difficulty\":\"Gorlek\"}]}".to_string();
/// 
/// assert_eq!(preset.to_json(), json);
/// assert_eq!(preset, Preset::parse(&json).unwrap());
/// ```
/// 
/// Use [`Settings::apply_preset`](crate::Settings::apply_preset) to merge a [`Preset`] into existing [`Settings`](crate::Settings)
/// 
/// ```
/// # use wotw_seedgen::Preset;
/// use wotw_seedgen::Settings;
/// use wotw_seedgen::settings::Spawn;
/// 
/// let mut settings = Settings::default();
/// 
/// let preset = Preset::parse("{\"worldSettings\":[{\"spawn\":\"Random\"}]}").unwrap();
/// 
/// settings.apply_preset(preset);
/// assert_eq!(settings.world_settings[0].spawn, Spawn::Random);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preset {
    /// Names of further [`Preset`]s to use
    /// 
    /// When applying the parent preset, these presets will be searched as .json files in the current and /presets child directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    /// The individual settings for each world of the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_settings: Option<Vec<WorldPreset>>,
    /// Whether the in-logic map filter should be offered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_logic_filter: Option<bool>,
    /// Require an online connection to play the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// The seed's seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    /// Automatically create an online game when generating the seed
    /// 
    /// This exists for future compability, but does not have any effect currently
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_game: Option<CreateGame>,
}

impl Preset {
    /// Parse a [`Preset`] from json
    pub fn parse(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }
    /// Serialize the [`Preset`] into json format
    pub fn to_json(&self) -> String {
        // This is safe because the Preset struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }
    /// Serialize the [`Preset`] into pretty-printed json format
    pub fn to_json_pretty(&self) -> String {
        // This is safe because the Preset struct is known to serialize successfully
        serde_json::to_string_pretty(&self).unwrap()
    }

    /// Find and read a [`Preset`] with the given name
    /// 
    /// The [`Preset`] will be searched as .json file in the current and /presets child directory
    pub fn read_file(mut name: String) -> Result<Self, Box<dyn Error>> {
        name.push_str(".json");
        let input = util::read_file(name, "presets")?;
        Ok(Self::parse(&input)?)
    }
}

/// A collection of settings that can be applied to one world of the existing settings
/// 
/// # Examples
/// 
/// [`WorldPreset`]s can be serialized and deserialized
/// 
/// ```
/// # use wotw_seedgen::preset::WorldPreset;
/// use wotw_seedgen::settings::Difficulty;
/// 
/// let mut world_preset = WorldPreset::default();
/// world_preset.difficulty = Some(Difficulty::Gorlek);
/// 
/// let json = "{\"difficulty\":\"Gorlek\"}".to_string();
/// 
/// assert_eq!(world_preset.to_json(), json);
/// assert_eq!(world_preset, WorldPreset::parse(&json).unwrap());
/// ```
/// 
/// Use [`WorldSettings::apply_world_preset`](crate::settings::WorldSettings::apply_world_preset) to merge a [`WorldPreset`] into existing [`WorldSettings`](crate::settings::WorldSettings)
/// 
/// ```
/// # use wotw_seedgen::preset::WorldPreset;
/// use wotw_seedgen::settings::WorldSettings;
/// use wotw_seedgen::settings::Spawn;
/// 
/// let mut world_settings = WorldSettings::default();
/// 
/// let world_preset = WorldPreset::parse("{\"spawn\":\"Random\"}").unwrap();
/// 
/// world_settings.apply_world_preset(world_preset);
/// assert_eq!(world_settings.spawn, Spawn::Random);
/// ```
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldPreset {
    /// Names of further [`WorldPreset`]s to use
    /// 
    /// When applying the parent preset, these presets will be searched as .json files in the current and /presets child directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    /// Spawn destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn: Option<Spawn>,
    /// Logically expected difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,
    /// Logically expected tricks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tricks: Option<Vec<Trick>>,
    /// Logically assume hard in-game difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard: Option<bool>,
    /// Goal Requirements before finishing the game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goals: Option<Vec<Goal>>,
    /// Names of headers to use
    /// 
    /// When generating a seed with these settings, the headers will be searched as .wotwrh files in the current and /headers child directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<String>>,
    /// Configuration parameters to pass to headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_config: Option<Vec<HeaderConfig>>,
    /// Inline header syntax
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_headers: Option<Vec<InlineHeader>>,
}

impl WorldPreset {
    /// Parse a [`WorldPreset`] from json
    pub fn parse(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }
    /// Serialize the [`WorldPreset`] into json format
    pub fn to_json(&self) -> String {
        // This is safe because the WorldPreset struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }
    /// Serialize the [`WorldPreset`] into json format
    pub fn to_json_pretty(&self) -> String {
        // This is safe because the WorldPreset struct is known to serialize successfully
        serde_json::to_string_pretty(&self).unwrap()
    }

    /// Find and read a [`WorldPreset`] with the given name
    /// 
    /// The [`WorldPreset`] will be searched as .json file in the current and /presets child directory
    pub fn read_file(mut name: String) -> Result<Self, Box<dyn Error>> {
        name.push_str(".json");
        let input = util::read_file(name, "presets")?;
        Ok(Self::parse(&input)?)
    }
}
