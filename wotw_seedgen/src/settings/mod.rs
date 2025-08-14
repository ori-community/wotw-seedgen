//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information

mod slugstrings;

use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::{collections::hash_map::DefaultHasher, error::Error, fmt, hash::Hasher, iter};

use rand::distributions::{Distribution, Uniform};
use rustc_hash::FxHashSet;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::{smallvec, SmallVec};
use wotw_seedgen_derive::{Display, FromStr};

use crate::item::Skill;
use crate::{
    files::FileAccess,
    preset::{UniversePreset, WorldPreset},
    util::constants::DEFAULT_SPAWN,
};

use slugstrings::SLUGSTRINGS;

/// A representation of all the relevant settings when generating a seed
///
/// Using the same settings will result in generating the same seed (unless the used header files change)
///
/// # Examples
///
/// ```
/// # use wotw_seedgen::settings::UniverseSettings;
/// use wotw_seedgen::settings::WorldSettings;
///
/// let universe_settings = UniverseSettings::default();
///
/// assert_eq!(universe_settings.world_count(), 1);
/// assert_eq!(universe_settings.world_settings[0], WorldSettings::default());
/// assert!(!universe_settings.seed.is_empty());
/// ```
///
/// The seed on default settings will be randomly generated
///
/// UniverseSettings can be serialized and deserialized
///
/// ```
/// # use wotw_seedgen::settings::UniverseSettings;
/// #
/// let universe_settings = UniverseSettings::default();
/// let json = universe_settings.to_json();
/// ```
///
/// Settings can be read from a generated seed
///
/// ```
/// # use wotw_seedgen::settings::UniverseSettings;
/// #
/// let seed = "
/// // [...pickup data and stuff...]
///
/// // Config: {\"seed\":\"3027801186584776\",\"worldSettings\":[{\"spawn\":\"MarshSpawn.Main\",\"difficulty\":\"Moki\",\"tricks\":[],\"hard\":false,\"randomizeDoors\":false,\"goals\":[],\"headers\":[],\"headerConfig\":[],\"inlineHeaders\":[]}],\"disableLogicFilter\":false,\"online\":false,\"createGame\":\"None\"}
/// ";
///
/// let universe_settings = UniverseSettings::from_seed(seed);
/// assert!(universe_settings.is_some());
/// assert!(universe_settings.unwrap().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UniverseSettings {
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

impl UniverseSettings {
    /// Parse settings from json
    pub fn parse(input: &str) -> Result<UniverseSettings, serde_json::Error> {
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
    pub fn from_seed(input: &str) -> Option<Result<UniverseSettings, String>> {
        input.lines().find_map(|line| {
            line.strip_prefix("// Config: ")
                .map(|config| serde_json::from_str(config).map_err(|err| err.to_string()))
        })
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
    /// - If multiple worlds are in both and their number does not match, returns an [`Error`]
    /// - Nested presets will be applied before the parent preset
    pub fn apply_preset(
        &mut self,
        preset: UniversePreset,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
        self.apply_preset_guarded(preset, &mut vec![], file_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_preset_guarded(
        &mut self,
        preset: UniversePreset,
        already_applied: &mut Vec<String>,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
        let UniversePreset {
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
                for (world_settings, preset_world_settings) in
                    self.world_settings.iter_mut().zip(preset_world_settings)
                {
                    world_settings.apply_world_preset(preset_world_settings, file_access)?;
                }
            } else if preset_worlds == 1 {
                for world_settings in &mut self.world_settings {
                    world_settings
                        .apply_world_preset(preset_world_settings[0].clone(), file_access)?;
                }
            } else if setting_worlds == 1 {
                let diff = preset_worlds - setting_worlds;
                self.world_settings
                    .extend(iter::repeat(self.world_settings[0].clone()).take(diff));
                for (world_settings, preset_world_settings) in
                    self.world_settings.iter_mut().zip(preset_world_settings)
                {
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
    fn apply_nested_preset(
        &mut self,
        preset: String,
        already_applied: &mut Vec<String>,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
        // Prevent cyclic patterns
        if already_applied.contains(&preset) {
            return Ok(());
        }
        already_applied.push(preset.clone());
        let preset = UniversePreset::read_file(&preset, file_access)?;
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

        SLUGSTRINGS
            .iter()
            .enumerate()
            .map(|(index, slug_strings)| {
                let length = slug_strings.len();

                let mut shift = 1;
                loop {
                    if length < 2_usize.pow(shift) {
                        shift -= 1;
                        break;
                    }
                    shift += 1;
                }

                #[allow(clippy::cast_possible_truncation)]
                let word_index =
                    (hash >> (index as u32 * shift)) as usize & (2_usize.pow(shift) - 1);
                slug_strings[word_index]
            })
            .collect()
    }

    /// Returns a random seed that can be used for the Settings
    fn random_seed() -> String {
        let numeric = Uniform::from('0'..='9');
        let mut rng = rand::thread_rng();
        iter::repeat_with(|| numeric.sample(&mut rng))
            .take(16)
            .collect()
    }

    /// Returns the highest [`Difficulty`] present among all [`WorldSettings`]s
    ///
    /// # Panics
    ///
    /// Panics if the [`UniverseSettings`] contain no worlds
    pub fn highest_difficulty(&self) -> Difficulty {
        // world_settings is assumed not to be empty
        self.world_settings
            .iter()
            .map(|world| world.difficulty)
            .max()
            .unwrap()
    }
    /// Returns the lowest [`Difficulty`] present among all [`WorldSettings`]s
    ///
    /// # Panics
    ///
    /// Panics if the [`UniverseSettings`] contain no worlds
    pub fn lowest_difficulty(&self) -> Difficulty {
        // world_settings is assumed not to be empty
        self.world_settings
            .iter()
            .map(|world| world.difficulty)
            .min()
            .unwrap()
    }
    /// Checks if any of the [`WorldSettings`]s have the [`Difficulty`]
    pub fn any_have_difficulty(&self, difficulty: Difficulty) -> bool {
        self.world_settings
            .iter()
            .any(|world| world.difficulty == difficulty)
    }

    /// Checks if any of the [`WorldSettings`]s contain the [`Trick`]
    pub fn any_contain_trick(&self, trick: Trick) -> bool {
        self.world_settings
            .iter()
            .any(|world| world.tricks.contains(&trick))
    }
    /// Checks if all of the [`WorldSettings`]s contain the [`Trick`]
    pub fn all_contain_trick(&self, trick: Trick) -> bool {
        self.world_settings
            .iter()
            .all(|world| world.tricks.contains(&trick))
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

impl Default for UniverseSettings {
    fn default() -> UniverseSettings {
        UniverseSettings {
            seed: Self::random_seed(),
            world_settings: vec![WorldSettings::default()],
            disable_logic_filter: false,
            online: false,
            create_game: CreateGame::default(),
        }
    }
}

#[derive(Debug)]
struct ApplyPresetError {
    message: String,
}
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
    pub tricks: FxHashSet<Trick>,
    /// Logically assume hard in-game difficulty
    pub hard: bool,
    /// Randomize door connections with a max loop size of n
    pub randomize_doors: bool,
    /// Goal Requirements before finishing the game
    pub goals: GoalModes,
    /// Names of headers to use
    ///
    /// When generating a seed with these settings, the headers will be searched as .wotwrh files in the current and /headers child directory
    pub headers: FxHashSet<String>,
    /// Configuration parameters to pass to headers
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

    /// Read the settings from a generated seed
    ///
    /// You can obtain the [`UniverseSettings`] using [`UniverseSettings::from_seed`]
    ///
    /// Returns [`None`] if the seed is multiworld and contains no information about which world it belongs to
    /// Returns an [`Error`] if the world index format could not be read
    pub fn from_seed(
        input: &str,
        universe_settings: UniverseSettings,
    ) -> Option<Result<WorldSettings, String>> {
        if universe_settings.world_settings.len() == 1 {
            return Some(Ok(universe_settings
                .world_settings
                .into_iter()
                .next()
                .unwrap()));
        }

        world_index_from_seed(input).map(|parse_result| {
            parse_result.and_then(|world_index| {
                universe_settings
                    .world_settings
                    .into_iter()
                    .nth(world_index)
                    .ok_or_else(|| "Current world index out of bounds".to_string())
            })
        })
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
    pub fn apply_world_preset(
        &mut self,
        preset: WorldPreset,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
        self.apply_world_preset_guarded(preset, &mut vec![], file_access)
    }

    /// Inner method to memorize nested presets to prevent cyclic patterns
    fn apply_world_preset_guarded(
        &mut self,
        preset: WorldPreset,
        already_applied: &mut Vec<String>,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
        let WorldPreset {
            info: _,
            includes,
            difficulty,
            tricks,
            goals,
            spawn,
            hard,
            randomize_doors,
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
        if let Some(tricks) = tricks {
            self.tricks.extend(tricks);
        }
        if let Some(goals) = goals {
            for goal in goals {
                self.goals.add(goal)?;
            }
        }
        if let Some(spawn) = spawn {
            self.spawn = spawn;
        }
        if let Some(hard) = hard {
            self.hard = hard;
        }
        if let Some(randomize_doors) = randomize_doors {
            self.randomize_doors = randomize_doors;
        }
        if let Some(headers) = headers {
            self.headers.extend(headers);
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
    fn apply_nested_preset(
        &mut self,
        preset: String,
        already_applied: &mut Vec<String>,
        file_access: &impl FileAccess,
    ) -> Result<(), Box<dyn Error>> {
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
#[derive(Debug, Clone, PartialEq)]
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

impl Serialize for Spawn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            Spawn::Random => serializer.serialize_str("Random"),
            Spawn::FullyRandom => serializer.serialize_str("FullyRandom"),
            Spawn::Set(spawn_loc) => serializer.serialize_str(spawn_loc),
        }
    }
}

struct SpawnStringVisitor;

impl<'de> Visitor<'de> for SpawnStringVisitor {
    type Value = Spawn;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a string representing an anchor, 'Random' or 'FullyRandom'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "Random" => Ok(Spawn::Random),
            "FullyRandom" => Ok(Spawn::FullyRandom),
            set => Ok(Spawn::Set(set.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Spawn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SpawnStringVisitor)
    }
}

/// The logical difficulty to expect in a seed
///
/// This represents how demanding the required core movement should be
/// Difficulties don't include glitches by default, these can be toggled through the Trick settings
///
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, FromStr, Display,
)]
#[ParseFromIdentifier]
#[derive(Default)]
pub enum Difficulty {
    #[default]
    Moki,
    Gorlek,
    Kii,
    Unsafe,
}
impl Difficulty {
    /// Allowed spawns on this difficulty when using the random spawn setting
    pub fn spawn_locations(self) -> &'static [&'static str] {
        pub const MOKI_SPAWNS: &[&str] = &[
            "MarshSpawn.Main",
            "HowlsDen.Teleporter",
            "GladesTown.Teleporter",
            "InnerWellspring.Teleporter",
            "MidnightBurrows.Teleporter",
        ];
        pub const GORLEK_SPAWNS: &[&str] = &[
            "MarshSpawn.Main",
            "HowlsDen.Teleporter",
            "EastHollow.Teleporter",
            "GladesTown.Teleporter",
            "InnerWellspring.Teleporter",
            "MidnightBurrows.Teleporter",
            "WoodsEntry.Teleporter",
            "WoodsMain.Teleporter",
            "LowerReach.Teleporter",
            "UpperDepths.Teleporter",
            "EastPools.Teleporter",
            "LowerWastes.WestTP",
            "LowerWastes.EastTP",
        ];
        match self {
            Difficulty::Moki => MOKI_SPAWNS,
            _ => GORLEK_SPAWNS,
        }
    }

    // TODO would it be worth to precompile the resulting slices for all variants?
    /// Allowed weapons on this difficulty
    pub fn weapons<const TARGET_IS_WALL: bool>(self) -> SmallVec<[Skill; 9]> {
        let mut weapons = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Spear,
        ];
        if !TARGET_IS_WALL {
            weapons.push(Skill::Flash);
        }
        if self >= Difficulty::Unsafe {
            weapons.push(Skill::Sentry);
        }
        weapons
    }
    /// Allowed ranged weapons on this difficulty
    pub fn ranged_weapons(self) -> SmallVec<[Skill; 6]> {
        let mut weapons = smallvec![Skill::Bow, Skill::Spear,];
        if self >= Difficulty::Gorlek {
            weapons.push(Skill::Grenade);
            weapons.push(Skill::Shuriken);
            if self >= Difficulty::Unsafe {
                weapons.push(Skill::Flash);
                weapons.push(Skill::Blaze);
            }
        }
        weapons
    }
    /// Allowed shield weapons on this difficulty
    pub fn shield_weapons(self) -> SmallVec<[Skill; 4]> {
        smallvec![Skill::Hammer, Skill::Launch, Skill::Grenade, Skill::Spear,]
    }
}

/// [`Difficulty`] requirements to use certain items that the seed generator may require as part of energy, damage etc. requirements
pub mod logical_difficulty {
    use super::Difficulty;

    pub const TRIPLE_JUMP: Difficulty = Difficulty::Gorlek;
    pub const RESILIENCE: Difficulty = Difficulty::Gorlek;
    pub const VITALITY: Difficulty = Difficulty::Gorlek;
    pub const ENERGY_SHARD: Difficulty = Difficulty::Gorlek;
    pub const DAMAGE_BUFFS: Difficulty = Difficulty::Unsafe;
    pub const OVERCHARGE: Difficulty = Difficulty::Unsafe;
    pub const LIFE_PACT: Difficulty = Difficulty::Unsafe;
    pub const ULTRA_BASH: Difficulty = Difficulty::Unsafe;
    pub const OVERFLOW: Difficulty = Difficulty::Unsafe;
    pub const THORN: Difficulty = Difficulty::Unsafe;
    pub const CATALYST: Difficulty = Difficulty::Unsafe;
}

/// A Trick that can be logically required
///
/// This includes mostly Glitches but also other techniques that can be toggled for logic, such as damage boosting
///
/// See the [Paths wiki page](https://wiki.orirando.com/seedgen/paths) for more information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, FromStr)]
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
    /// Preserving Double Jump momentum with Sword
    SwordJump,
    /// Redirecting projectiles with Grenade
    GrenadeRedirect,
    /// Redirecting projectiles with Sentry
    SentryRedirect,
    /// Cancelling falling momentum through the pause menu
    PauseFloat,
    /// Storing a grounded jump into the air with Glide
    GlideJump,
    /// Preserving Hammer momentum in a lot of ways
    HammerJump,
    /// Hammer Jump with Double Jump
    AerialHammerJump,
    /// Preserving Glide Jump momentum with Hammer
    GlideHammerJump,
    /// Hammer Jump with a coyote jump
    CoyoteHammerJump,
    /// Hammer Jump with a wall jump
    WallHammerJump,
    /// Hammer Jump from a standard jump
    GroundedHammerJump,
    /// Extending momentum with Hammer
    HammerExtension,
    /// Storing a grounded jump into the air with Spear
    SpearJump,
    /// Cancelling Bash momentum by using Glide so you can Bash again the same object
    GlideBashChain,
    /// Cancelling Bash momentum by using Double Jump so you can Bash again the same object
    DoubleJumpBashChain,
    /// Cancelling Bash momentum by using Dash in the opposite way so you can Bash again the same object
    DashBashChain,
    /// Cancelling Bash momentum by using Launch in the opposite way so you can Bash again the same object
    LaunchBashChain,
    /// Any specific trick that is unpopular for any reason
    Unpopular,
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
impl Display for Goal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Goal::Wisps => "Wisps".fmt(f),
            Goal::Trees => "Trees".fmt(f),
            Goal::Quests => "Quests".fmt(f),
            Goal::Relics(count) => write!(f, "{} Relics", count),
            Goal::RelicChance(chance) => write!(f, "{}% Relic chance", chance * 100.),
        }
    }
}
impl Goal {
    /// Returns the flag name representing this goal
    ///
    /// The flag name communicates to the randomizer client which restrictions to apply before allowing to finish the game
    pub fn flag_name(&self) -> &'static str {
        match self {
            Goal::Wisps => "All Wisps",
            Goal::Trees => "All Trees",
            Goal::Quests => "All Quests",
            Goal::Relics(_) | Goal::RelicChance(_) => "Relics",
        }
    }

    fn is_relic_goal(&self) -> bool {
        matches!(self, Goal::Relics(_) | Goal::RelicChance(_))
    }
}

/// A collection of non-redundant [`Goal`]s
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GoalModes {
    goals: Vec<Goal>,
}
impl GoalModes {
    /// Adds a [`Goal`], ensuring that it isn't redundant with existing [`Goal`]s
    pub fn add(&mut self, goal: Goal) -> Result<(), String> {
        if self.goals.contains(&goal) {
            return Ok(());
        }

        if goal.is_relic_goal() {
            if let Some(other) = self.goals.iter().find(|goal| goal.is_relic_goal()) {
                return Err(format!("Contradicting goal modes {} and {}", other, goal));
            }
        }

        self.goals.push(goal);
        Ok(())
    }
}
impl FromIterator<Goal> for GoalModes {
    fn from_iter<T: IntoIterator<Item = Goal>>(iter: T) -> Self {
        Self {
            goals: Vec::<Goal>::from_iter(iter),
        }
    }
}
impl IntoIterator for GoalModes {
    type Item = Goal;
    type IntoIter = <Vec<Goal> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.goals.into_iter()
    }
}
impl Deref for GoalModes {
    type Target = [Goal];
    fn deref(&self) -> &[Goal] {
        self.goals.deref()
    }
}
impl DerefMut for GoalModes {
    fn deref_mut(&mut self) -> &mut [Goal] {
        self.goals.deref_mut()
    }
}

/// Different types of online games that can be automatically created when generating the seed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromStr)]
#[ParseFromIdentifier]
#[derive(Default)]
pub enum CreateGame {
    /// Don't create an online game
    #[default]
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

pub fn world_index_from_seed(seed: &str) -> Option<Result<usize, String>> {
    seed.lines()
        .find_map(|line| line.strip_prefix("// This World: "))
        .map(|line| {
            line.parse()
                .map_err(|err| format!("Error reading current world: {err}"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::Rng;
    use rustc_hash::FxHashSet;

    #[test]
    fn slugification() {
        let mut rng = rand::thread_rng();
        let mut slugs = FxHashSet::default();

        for _ in 0..1000 {
            let mut universe_settings = UniverseSettings::default();

            let goals = vec![
                Goal::Wisps,
                Goal::Trees,
                Goal::Quests,
                Goal::RelicChance(0.8),
            ];
            for goal in goals {
                if rng.gen_bool(0.25) {
                    universe_settings.world_settings[0].goals.add(goal).unwrap();
                }
            }

            let slug = universe_settings.slugify();

            if slugs.contains(&slug) {
                panic!(
                    "After {} settings, two had the same slug: {}",
                    slugs.len(),
                    slug
                );
            } else {
                slugs.insert(slug);
            }
        }
    }
}
