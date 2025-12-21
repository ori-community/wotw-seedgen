//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information

use std::{
    fmt::{self, Display},
    num::NonZeroU8,
    ops::Deref,
    str::FromStr,
};

use rand::{distributions::Bernoulli, seq::SliceRandom, Rng};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumMessage, EnumString, VariantArray, VariantNames};
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, Type},
    PartialSchema, ToSchema,
};

use crate::{
    assets::{InlineSnippets, SnippetAccess},
    seed_language::metadata::ConfigDefault,
};

/// A representation of all the relevant settings when generating a seed
///
/// Using the same settings will result in generating the same seed (as long as the same seedgen version and snippets are used)
///
/// # Examples
///
/// ```
/// # use wotw_seedgen_data::UniverseSettings;
/// use wotw_seedgen_data::WorldSettings;
///
/// let universe_settings = UniverseSettings::new("seed".to_string());
///
/// assert_eq!(universe_settings.world_count(), 1);
/// assert_eq!(universe_settings.world_settings[0], WorldSettings::default());
/// assert_eq!(universe_settings.seed, "seed");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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

pub trait WorldSettingsHelpers {
    fn lowest_difficulty(&self) -> Difficulty;

    fn highest_difficulty(&self) -> Difficulty;

    fn iter_tricks(&self) -> impl Iterator<Item = &FxHashSet<Trick>>;

    fn all_contain_trick(&self, trick: Trick) -> bool {
        self.iter_tricks().all(|tricks| tricks.contains(&trick))
    }

    fn any_contain_trick(&self, trick: Trick) -> bool {
        self.iter_tricks().any(|tricks| tricks.contains(&trick))
    }

    fn none_contain_trick(&self, trick: Trick) -> bool {
        !self.any_contain_trick(trick)
    }
}

impl WorldSettingsHelpers for [WorldSettings] {
    fn lowest_difficulty(&self) -> Difficulty {
        self.iter()
            .map(|settings| settings.difficulty)
            .min()
            .unwrap_or(Difficulty::Moki)
    }

    fn highest_difficulty(&self) -> Difficulty {
        self.iter()
            .map(|settings| settings.difficulty)
            .max()
            .unwrap_or(Difficulty::Unsafe)
    }

    fn iter_tricks(&self) -> impl Iterator<Item = &FxHashSet<Trick>> {
        self.iter().map(|settings| &settings.tricks)
    }
}

impl WorldSettingsHelpers for UniverseSettings {
    fn lowest_difficulty(&self) -> Difficulty {
        self.world_settings.lowest_difficulty()
    }

    fn highest_difficulty(&self) -> Difficulty {
        self.world_settings.highest_difficulty()
    }

    fn iter_tricks(&self) -> impl Iterator<Item = &FxHashSet<Trick>> {
        self.world_settings.iter_tricks()
    }
}

/// Seed settings bound to a specific world of a seed
///
/// See the [Multiplayer wiki page](https://wiki.orirando.com/features/multiplayer) for an explanation of worlds
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
    /// Randomize door connections with the given max loop size
    pub randomize_doors: Option<GreaterOneU8>,
    /// Names of snippets to use
    pub snippets: Vec<String>,
    /// Additional inline snippets that don't exist on the filesystem
    pub inline_snippets: InlineSnippets,
    /// Configuration to pass to snippets
    pub snippet_config: FxHashMap<String, FxHashMap<String, String>>,
}

impl WorldSettings {
    pub fn random<R: Rng, A: SnippetAccess>(rng: &mut R, snippet_access: &A) -> Self {
        let difficulty = *<Difficulty as VariantArray>::VARIANTS.choose(rng).unwrap();

        let distr_50 = Bernoulli::new(0.5).unwrap();

        let tricks = <Trick as VariantArray>::VARIANTS
            .iter()
            .filter(|trick| difficulty >= trick.min_difficulty() && rng.sample(distr_50))
            .copied()
            .collect();

        let mut snippets = snippet_access.available_snippets_metadata();
        snippets.retain(|(_, metadata)| !metadata.hidden && rng.sample(distr_50));

        let mut snippet_config = FxHashMap::default();

        let snippets = snippets
            .into_iter()
            .map(|(identifier, metadata)| {
                snippet_config.insert(
                    identifier.clone(),
                    metadata
                        .config
                        .into_iter()
                        .filter_map(|(identifier, value)| match value.default {
                            ConfigDefault::Boolean(value) if rng.sample(distr_50) => {
                                Some((identifier, (!value).to_string()))
                            }
                            _ => None,
                        })
                        .collect(),
                );

                identifier
            })
            .collect();

        Self {
            spawn: Spawn::FullyRandom,
            difficulty,
            tricks,
            hard: rng.gen_bool(0.25),
            randomize_doors: rng
                .sample(distr_50)
                .then_some(GreaterOneU8::new(2).unwrap()),
            snippets,
            inline_snippets: InlineSnippets::default(),
            snippet_config,
        }
    }

    /// Checks whether these settings feature a random spawn location
    pub fn is_random_spawn(&self) -> bool {
        matches!(self.spawn, Spawn::Random | Spawn::FullyRandom)
    }
}

/// The Spawn location, which may either be fixed or randomly decided during seed generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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
    EnumMessage,
    VariantNames,
    VariantArray,
    ToSchema,
)]
#[strum(serialize_all = "lowercase")]
pub enum Difficulty {
    /// The default game paths, designed for players who have finished the game at least once.
    #[default]
    Moki,
    /// Intermediate game paths for more advanced players. More precise utilisation of skills is required.
    Gorlek,
    /// Advanced game paths for players seeking a challenge. Glitched paths may be incomplete.
    Kii,
    /// Unvalidated game paths. Some paths may be very hard. Many paths are missing. Don't try at home.
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
    EnumMessage,
    VariantNames,
    VariantArray,
    ToSchema,
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
    /// Preserving Double Jump momentum with Sword
    SwordJump,
    /// Preserving momentum with Hammer
    HammerJump,
    /// Preserving Glide Jump momentum with Hammer
    AerialHammerJump,
    /// Storing a grounded jump into the air with Glide
    GlideJump,
    /// Preserving Glide Jump momentum with Hammer
    GlideHammerJump,
    /// Preserving Coyote Jump momentum with Hammer
    CoyoteHammerJump,
    /// Preserving Wall Jump momentum with Hammer
    WallHammerJump,
    /// Preserving Jump momentum with Hammer
    GroundedHammerJump,
    /// Swinging Hammer back and forth to preserve movementum longer
    ExtendedHammer,
    /// Redirecting projectiles with Grenade
    GrenadeRedirect,
    /// Redirecting projectiles with Sentry
    SentryRedirect,
    /// Cancelling falling momentum through the pause menu
    PauseHover,
    /// Storing a grounded jump into the air with Spear
    SpearJump,
}

impl Trick {
    // TODO verify usage in logic?
    pub fn min_difficulty(self) -> Difficulty {
        match self {
            Self::SwordSentryJump
            | Self::HammerSentryJump
            | Self::ShurikenBreak
            | Self::SentryBurn
            | Self::RemoveKillPlane => Difficulty::Gorlek,
            Self::LaunchSwap | Self::GrenadeJump | Self::AerialHammerJump | Self::GlideJump => {
                Difficulty::Kii
            }
            Self::SentryBreak
            | Self::HammerBreak
            | Self::SpearBreak
            | Self::SentrySwap
            | Self::FlashSwap
            | Self::BlazeSwap
            | Self::WaveDash
            | Self::SwordJump
            | Self::HammerJump
            | Self::GlideHammerJump
            | Self::CoyoteHammerJump
            | Self::WallHammerJump
            | Self::GroundedHammerJump
            | Self::ExtendedHammer
            | Self::GrenadeRedirect
            | Self::SentryRedirect
            | Self::PauseHover
            | Self::SpearJump => Difficulty::Unsafe,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GreaterOneU8(NonZeroU8);

impl PartialSchema for GreaterOneU8 {
    fn schema() -> RefOr<Schema> {
        RefOr::T(
            ObjectBuilder::new()
                .schema_type(Type::Number)
                .exclusive_minimum(Some(1))
                .into(),
        )
    }
}

impl ToSchema for GreaterOneU8 {}

impl GreaterOneU8 {
    pub fn new(n: u8) -> Option<Self> {
        if n > 1 {
            NonZeroU8::new(n).map(Self)
        } else {
            None
        }
    }
}

impl Deref for GreaterOneU8 {
    type Target = NonZeroU8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for GreaterOneU8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for GreaterOneU8 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = u8::from_str(s).map_err(|err| err.to_string())?;
        Self::new(n).ok_or("number would be zero or one for greater-one type".to_string())
    }
}
