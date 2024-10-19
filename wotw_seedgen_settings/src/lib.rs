//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information
//!
//! ## Features
//!
//! - `serde`: Enables [`Deserialize`] and [`Serialize`] implementations on all types
//! - `strum`: Enables [`Display`], [`FromStr`] and [`VariantNames`] implementations on [`Difficulty`] and [`Trick`]
//!
//! [`Deserialize`]: serde::Deserialize
//! [`Serialize`]: serde::Serialize
//! [`Display`]: std::fmt::Display
//! [`FromStr`]: std::str::FromStr
//! [`VariantNames`]: strum::VariantNames

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, VariantArray, VariantNames};

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

    /// Returns the number of worlds
    pub fn world_count(&self) -> usize {
        self.world_settings.len()
    }
}

/// Seed settings bound to a specific world of a seed
///
/// See the [Multiplayer wiki page](https://wiki.orirando.com/features/multiplayer) for an explanation of worlds
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
}

/// The Spawn location, which may either be fixed or randomly decided during seed generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    VariantNames,
    VariantArray,
)]
#[strum(serialize_all = "lowercase")]
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
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    VariantNames,
    VariantArray,
)]
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
