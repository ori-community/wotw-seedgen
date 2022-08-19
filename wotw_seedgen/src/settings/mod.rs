//! Data structures to represent the settings when generating a seed
//! 
//! See the [`Settings`] struct for more information

mod slugstrings;

use std::{error::Error, fmt, iter, collections::hash_map::DefaultHasher, hash::Hasher};

use rand::distributions::{Distribution, Uniform};
use wotw_seedgen_derive::FromStr;
use serde::{Serialize, Deserialize};

use crate::{preset::{GamePreset, WorldPreset}, util::constants::DEFAULT_SPAWN, files::FileAccess};

use slugstrings::SLUGSTRINGS;

/// A representation of all the relevant settings when generating a seed
/// 
/// Using the same settings will result in generating the same seed (unless the used header files change)
/// 
/// # Examples
/// 
/// ```
/// # use wotw_seedgen::Settings;
/// use wotw_seedgen::settings::WorldSettings;
/// 
/// let settings = Settings::default();
/// 
/// assert_eq!(settings.world_count(), 1);
/// assert_eq!(settings.world_settings[0], WorldSettings::default());
/// assert!(!settings.seed.is_empty());
/// ```
/// 
/// The seed on default settings will be randomly generated
/// 
/// Settings can be serialized and deserialized
/// 
/// ```
/// # use wotw_seedgen::Settings;
/// #
/// let settings = Settings::default();
/// let json = settings.to_json();
/// ```
/// 
/// Settings can be read from a generated seed
/// 
/// ```
/// # use wotw_seedgen::Settings;
/// #
/// let seed = "
/// // [...pickup data and stuff...]
/// 
/// // Config: {\"seed\":\"3027801186584776\",\"worldSettings\":[{\"spawn\":{\"Set\":\"MarshSpawn.Main\"},\"difficulty\":\"Moki\",\"tricks\":[],\"hard\":false,\"goals\":[],\"headers\":[],\"headerConfig\":[],\"inlineHeaders\":[]}],\"disableLogicFilter\":false,\"online\":false,\"createGame\":\"None\"}
/// ";
/// 
/// let settings = Settings::from_seed(seed);
/// assert!(settings.is_some());
/// assert!(settings.unwrap().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    /// The seed that determines all randomness
    pub seed: String,
    /// The individual settings for each world of the seed
    /// 
    /// This is assumed never to be empty
    pub world_settings: Vec<WorldSettings>,
    /// Whether the in-logic map filter should be offered
    pub disable_logic_filter: bool,
    /// Require an online connection to play the seed
    /// 
    /// This is needed for Co-op, Multiworld and Bingo
    pub online: bool,
    /// Automatically create an online game when generating the seed
    /// 
    /// This exists for future compability, but does not have any effect currently
    pub create_game: CreateGame,
}

impl Settings {
    /// Parse settings from json
    pub fn parse(input: &str) -> Result<Settings, serde_json::Error> {
        serde_json::from_str(input)
    }
    /// Serialize the settings into json format
    pub fn to_json(&self) -> String {
        // This is safe because the settings struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }

    /// Read the settings from a generated seed
    /// 
    /// Returns [`None`] if the seed contains no information about the settings used to generate it
    /// Returns an [`Error`] if the settings format could not be read
    pub fn from_seed(input: &str) -> Option<Result<Settings, serde_json::Error>> {
        input.lines().find_map(|line| line.strip_prefix("// Config: ").map(serde_json::from_str))
    }

    /// Apply the settings from a [`GamePreset`]
    /// 
    /// This follows various rules to retain all unrelated parts of the existing Settings:
    /// - Any [`None`] values of the preset will be ignored
    /// - [`Vec`]s will be appended to the current contents
    /// - Other values will be overwritten
    /// - If the number of worlds matches, the preset will be applied to each world per index
    /// - If only one world is in the preset, but multiple in the existing settings, the preset is applied to all worlds
    /// - If multiple worlds are in the preset, but only one in the existing settings, the existing settings will be copied for all worlds, then the preset will be applied per index
    /// - If multiple worlds are in both and their number does not match, returns an [`Error`]
    /// - Nested presets will be applied before the parent preset
    pub fn apply_preset(&mut self, preset: GamePreset, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        self.apply_preset_guarded(preset, &mut vec![], file_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_preset_guarded(&mut self, preset: GamePreset, already_applied: &mut Vec<String>, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        let GamePreset {
            info: _,
            includes,
            world_settings,
            disable_logic_filter,
            online,
            seed,
            create_game,
        } = preset;

        if let Some(includes) = includes {
            for nested_preset in includes {
                self.apply_nested_preset(nested_preset, already_applied, file_access)?;
            }
        }

        let setting_worlds = self.world_count();

        if let Some(preset_world_settings) = world_settings {
            let preset_worlds = preset_world_settings.len();

            if preset_worlds == 0 {
                // do nothing
            } else if setting_worlds == preset_worlds {
                for (world_settings, preset_world_settings) in self.world_settings.iter_mut().zip(preset_world_settings) {
                    world_settings.apply_world_preset(preset_world_settings, file_access)?;
                }
            } else if preset_worlds == 1 {
                for world_settings in &mut self.world_settings {
                    world_settings.apply_world_preset(preset_world_settings[0].clone(), file_access)?;
                }
            } else if setting_worlds == 1 {
                let diff = preset_worlds - setting_worlds;
                self.world_settings.extend(iter::repeat(self.world_settings[0].clone()).take(diff));
                for (world_settings, preset_world_settings) in self.world_settings.iter_mut().zip(preset_world_settings) {
                    world_settings.apply_world_preset(preset_world_settings, file_access)?;
                }
            } else {
                let message = format!("Cannot apply preset with {preset_worlds} worlds to settings with {setting_worlds} worlds");
                return Err(Box::new(ApplyPresetError { message }));
            }
        }

        if let Some(disable_logic_filter) = disable_logic_filter {
            self.disable_logic_filter = disable_logic_filter;
        }
        if let Some(online) = online {
            self.online = online;
        }
        if let Some(seed) = seed {
            self.seed = seed;
        }
        if let Some(create_game) = create_game {
            self.create_game = create_game;
        }

        Ok(())
    }

    /// Find and apply nested presets
    fn apply_nested_preset(&mut self, preset: String, already_applied: &mut Vec<String>, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        // Prevent cyclic patterns
        if already_applied.contains(&preset) {
            return Ok(());
        }
        already_applied.push(preset.clone());
        let preset = GamePreset::read_file(&preset, file_access)?;
        self.apply_preset_guarded(preset, already_applied, file_access)
    }

    /// Returns the number of worlds
    pub fn world_count(&self) -> usize {
        self.world_settings.len()
    }

    /// Returns a slug unique to these settings
    pub fn slugify(&self) -> String {
        let serialized = self.to_json();

        let mut hasher = DefaultHasher::new();
        hasher.write(serialized.as_bytes());
        let hash = hasher.finish();

        SLUGSTRINGS.iter().enumerate().map(|(index, slug_strings)| {
            let length = slug_strings.len();

            let mut shift = 1;
            loop {
                if length < 2_usize.pow(shift) {
                    shift -= 1;
                    break;
                }
                shift += 1;
            };

            #[allow(clippy::cast_possible_truncation)]
            let word_index = (hash >> (index as u32 * shift)) as usize & (2_usize.pow(shift) - 1);
            slug_strings[word_index]
        }).collect()
    }

    /// Returns a random seed that can be used for the Settings
    fn random_seed() -> String {
        let numeric = Uniform::from('0'..='9');
        let mut rng = rand::thread_rng();
        iter::repeat_with(|| numeric.sample(&mut rng)).take(16).collect()
    }

    /// Returns the highest [`Difficulty`] present among all [`WorldSettings`]s
    /// 
    /// # Panics
    /// 
    /// Panics if the [`Settings`] contain no worlds
    pub fn highest_difficulty(&self) -> Difficulty {
        // world_settings is assumed not to be empty
        self.world_settings.iter().map(|world| world.difficulty).max().unwrap()
    }
    /// Returns the lowest [`Difficulty`] present among all [`WorldSettings`]s
    /// 
    /// # Panics
    /// 
    /// Panics if the [`Settings`] contain no worlds
    pub fn lowest_difficulty(&self) -> Difficulty {
        // world_settings is assumed not to be empty
        self.world_settings.iter().map(|world| world.difficulty).min().unwrap()
    }
    /// Checks if any of the [`WorldSettings`]s have the [`Difficulty`]
    pub fn any_have_difficulty(&self, difficulty: Difficulty) -> bool {
        self.world_settings.iter().any(|world| world.difficulty == difficulty)
    }

    /// Checks if any of the [`WorldSettings`]s contain the [`Trick`]
    pub fn any_contain_trick(&self, trick: Trick) -> bool {
        self.world_settings.iter().any(|world| world.tricks.contains(&trick))
    }
    /// Checks if all of the [`WorldSettings`]s contain the [`Trick`]
    pub fn all_contain_trick(&self, trick: Trick) -> bool {
        self.world_settings.iter().all(|world| world.tricks.contains(&trick))
    }

    /// Checks if any of the [`WorldSettings`]s play on hard in-game difficulty
    pub fn any_play_hard(&self) -> bool {
        self.world_settings.iter().any(|world| world.hard)
    }
    /// Checks if all of the [`WorldSettings`]s play on hard in-game difficulty
    pub fn all_play_hard(&self) -> bool {
        self.world_settings.iter().all(|world| world.hard)
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            seed: Self::random_seed(),
            world_settings: vec![WorldSettings::default()],
            disable_logic_filter: false,
            online: false,
            create_game: CreateGame::default(),
        }
    }
}

#[derive(Debug)]
struct ApplyPresetError { message: String }
impl fmt::Display for ApplyPresetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = &self.message;
        write!(f, "{message}")
    }
}
impl Error for ApplyPresetError {}

/// Seed settings bound to a specific world of a seed
/// 
/// See the [Multiplayer wiki page](https://wiki.orirando.com/features/multiplayer) for an explanation of worlds
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldSettings {
    /// Spawn destination
    pub spawn: Spawn,
    /// Logically expected difficulty
    pub difficulty: Difficulty,
    /// Logically expected tricks
    pub tricks: Vec<Trick>,
    /// Logically assume hard in-game difficulty
    pub hard: bool,
    /// Goal Requirements before finishing the game
    pub goals: Vec<Goal>,
    /// Names of headers to use
    /// 
    /// When generating a seed with these settings, the headers will be searched as .wotwrh files in the current and /headers child directory
    pub headers: Vec<String>,
    /// Configuration parameters to pass to headers
    /// 
    /// Format for one parameter: <headername>.<parametername>=<value>
    pub header_config: Vec<HeaderConfig>,
    /// Fully qualified header syntax
    pub inline_headers: Vec<InlineHeader>,
}

impl WorldSettings {
    /// Parse [`WorldSettings`] from json
    pub fn parse(input: &str) -> Result<WorldSettings, serde_json::Error> {
        serde_json::from_str(input)
    }
    /// Serialize the [`WorldSettings`] into json format
    pub fn to_json(&self) -> String {
        // This is safe because the settings struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }

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
    pub fn apply_world_preset(&mut self, preset: WorldPreset, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        self.apply_world_preset_guarded(preset, &mut vec![], file_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_world_preset_guarded(&mut self, preset: WorldPreset, already_applied: &mut Vec<String>, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        let WorldPreset {
            info: _,
            includes,
            difficulty,
            tricks,
            goals,
            spawn,
            hard,
            headers,
            header_config,
            inline_headers,
        } = preset;

        if let Some(includes) = includes {
            for nested_preset in includes {
                self.apply_nested_preset(nested_preset, already_applied, file_access)?;
            }
        }

        if let Some(difficulty) = difficulty {
            self.difficulty = difficulty;
        }
        if let Some(mut tricks) = tricks {
            self.tricks.append(&mut tricks);
        }
        if let Some(mut goals) = goals {
            self.goals.append(&mut goals);
        }
        if let Some(spawn) = spawn {
            self.spawn = spawn;
        }
        if let Some(hard) = hard {
            self.hard = hard;
        }
        if let Some(mut headers) = headers {
            self.headers.append(&mut headers);
        }
        if let Some(mut header_config) = header_config {
            self.header_config.append(&mut header_config);
        }
        if let Some(mut inline_headers) = inline_headers {
            self.inline_headers.append(&mut inline_headers);
        }

        Ok(())
    }

    /// Find and apply nested presets
    fn apply_nested_preset(&mut self, preset: String, already_applied: &mut Vec<String>, file_access: &impl FileAccess) -> Result<(), Box<dyn Error>> {
        // Prevent cyclic patterns
        if already_applied.contains(&preset) {
            return Ok(());
        }
        already_applied.push(preset.clone());
        let preset = WorldPreset::read_file(&preset, file_access)?;
        self.apply_world_preset_guarded(preset, already_applied, file_access)
    }
}

/// The Spawn destination, determining the starting location of the seed
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Spawn {
    /// Spawn in a specific location, described by the anchor name from the logic file
    Set(String),
    /// Spawn in a random location out of a curated set, depending on the seed's difficulty
    Random,
    /// Spawn on any valid anchor from the logic file
    FullyRandom,
}

impl Default for Spawn {
    fn default() -> Spawn {
        Spawn::Set(DEFAULT_SPAWN.to_string())
    }
}

/// The logical difficulty to expect in a seed
/// 
/// This represents how demanding the required core movement should be
/// Difficulties don't include glitches by default, these can be toggled through the Trick settings
/// 
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, FromStr)]
#[ParseFromIdentifier]
pub enum Difficulty {
    Moki,
    Gorlek,
    Kii,
    Unsafe,
}
impl Default for Difficulty {
    fn default() -> Difficulty { Difficulty::Moki }
}

/// A Trick that can be logically required
/// 
/// This includes mostly Glitches but also other techniques that can be toggled for logic, such as damage boosting
/// 
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, FromStr)]
#[ParseFromIdentifier]
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

/// Enforced Requirement before being allowed to finish the game
/// 
/// See the [Goals wiki page](https://wiki.orirando.com/seedgen/goals) for more information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Goal {
    /// Require all five Wisps before finishing the game
    Wisps,
    /// Require all 14 Trees before finishing the game
    Trees,
    /// Require all 17 Quests before finishing the game
    Quests,
    /// Require collecting the specified amount of Relics before finishing the game
    /// 
    /// Each zone of the game will have at most one Relic
    /// There are 11 zones that allow Relics, any further Relics will not be placed
    Relics(usize),
    /// Require collecting a random amount of Relics before finishing the game
    /// 
    /// Each zone of the game will have at most one Relic
    /// There are 11 zones that allow Relics, the specified chance represents how likely each single zone will have a relic
    RelicChance(f64),
}

impl Goal {
    /// Returns the flag name representing this goal
    /// 
    /// The flag name communicates to the randomizer client which restrictions to apply before allowing to finish the game
    pub fn flag_name(&self) -> &'static str {
        match self {
            Goal::Wisps => "ForceWisps",
            Goal::Trees => "ForceTrees",
            Goal::Quests => "ForceQuests",
            Goal::Relics(_) | Goal::RelicChance(_) => "WorldTour",
        }
    }
}

/// Different types of online games that can be automatically created when generating the seed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromStr)]
#[ParseFromIdentifier]
pub enum CreateGame {
    /// Don't create an online game
    None,
    /// Create a normal online game suited for co-op and multiworld
    Normal,
    /// Create a bingo game, which can optionally be used for co-op and multiworld
    Bingo,
    /// Create a discovery bingo game with two starting squares, which can optionally be used for co-op and multiworld
    DiscoveryBingo,
    /// Create a lockout bingo game, which can optionally be used for co-op and multiworld
    LockoutBingo,
}

impl Default for CreateGame {
    fn default() -> CreateGame {
        CreateGame::None
    }
}

/// Configuration parameter for a header
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderConfig {
    /// The name of the header
    pub header_name: String,
    /// The name of the configuration parameter
    pub config_name: String,
    /// The value to use for the configuration parameter
    pub config_value: String,
}

/// Headers passed through explicit syntax
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InlineHeader {
    /// The name of the header
    pub name: Option<String>,
    /// Contained header syntax
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    use rustc_hash::FxHashSet;
    use rand::Rng;

    #[test]
    fn slugification() {
        let mut rng = rand::thread_rng();
        let mut slugs = FxHashSet::default();

        for _ in 0..1000 {
            let mut settings = Settings::default();

            let goals = vec![Goal::Wisps, Goal::Trees, Goal::Quests, Goal::RelicChance(0.8)];
            for goal in goals {
                if rng.gen_bool(0.25) {
                    settings.world_settings[0].goals.push(goal);
                }
            }

            let slug = settings.slugify();

            if slugs.contains(&slug) {
                panic!("After {} settings, two had the same slug: {}", slugs.len(), slug);
            } else {
                slugs.insert(slug);
            }
        }
    }
}
