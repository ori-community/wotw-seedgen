// TODO maybe this could be derived from the settings?

use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{settings::{Trick, Difficulty, Goal, Spawn, CreateGame}, util};

/// A collection of settings that can be applied to existing settings
/// 
/// # Examples
/// 
/// Presets can be serialized and deserialized
/// 
/// ```
/// # use seedgen::Preset;
/// use seedgen::preset::PresetWorldSettings;
/// use seedgen::settings::Difficulty;
/// 
/// let mut preset = Preset::default();
/// let mut world_settings = PresetWorldSettings::default();
/// world_settings.difficulty = Some(Difficulty::Gorlek);
/// preset.world_settings = Some(vec![world_settings]);
/// 
/// let json = "{\"worldSettings\":[{\"difficulty\":\"Gorlek\"}]}".to_string();
/// 
/// assert_eq!(preset.to_json(), json);
/// assert_eq!(preset, Preset::parse(&json).unwrap());
/// ```
/// 
/// Use `Settings::apply_preset` to merge a Preset into existing Settings
/// 
/// ```
/// # use seedgen::Preset;
/// use seedgen::Settings;
/// use seedgen::settings::Spawn;
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
    /// Names of further presets to use
    /// 
    /// When applying the parent preset, these presets will be searched as .json files in the current and /presets child directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    /// The individual settings for each world of the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_settings: Option<Vec<PresetWorldSettings>>,
    /// Don't write spoiler comments into the seed
    /// 
    /// This will create a separate copy of the seed with spoilers included
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_spoilers: Option<bool>,
    /// Disallow the use of the In-Logic filter while playing the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_logic_filter: Option<bool>,
    /// Require an online connection to play the seed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// Automatically create an online game when generating the seed
    /// 
    /// This exists for future compability, but does not have any effect currently
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_game: Option<CreateGame>,
}

impl Preset {
    /// Parse a preset from json
    pub fn parse(input: &str) -> Result<Preset, serde_json::Error> {
        serde_json::from_str(input)
    }
    /// Serialize the preset into json format
    pub fn to_json(&self) -> String {
        // This is safe because the preset struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }

    /// Find and read a preset with the given name
    /// 
    /// The preset will be searched as .json file in the current and /presets child directory
    pub fn read_file(mut name: String) -> Result<Preset, Box<dyn Error>> {
        name.push_str(".json");
        let input = util::read_file(name, "presets")?;
        Ok(Self::parse(&input)?)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetWorldSettings {
    /// The name of this world (usually the name of the player or co-op team)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_name: Option<String>,
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
    ///
    /// format for one parameter: <headername>.<parametername>=<value>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_config: Option<Vec<String>>,
    /// Inline header syntax
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_header: Option<String>,
}
