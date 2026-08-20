//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information

mod slug;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Write},
    iter,
    num::NonZeroU8,
    ops::Deref,
    slice,
    str::FromStr,
};

use heck::ToTitleCase;
use itertools::Itertools;
use rand::{distributions::Open01, seq::SliceRandom, Rng};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumMessage, EnumString, VariantArray, VariantNames};
use utoipa::{
    openapi::{ObjectBuilder, RefOr, Schema, Type},
    PartialSchema, ToSchema,
};

use crate::{
    assets::{InlineSnippets, SnippetAccess},
    parse::Source,
    seed_language::metadata::{ConfigValue, Metadata},
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
    fn is_empty(&self) -> bool;

    fn lowest_difficulty(&self) -> Difficulty;

    fn highest_difficulty(&self) -> Difficulty;

    fn iter_tricks(&self) -> impl Iterator<Item = &FxHashSet<Trick>>;

    fn iter_hard(&self) -> impl Iterator<Item = bool>;

    fn all_play_hard(&self) -> bool {
        !self.iter_hard().contains(&false)
    }

    fn any_play_hard(&self) -> bool {
        self.iter_hard().contains(&true)
    }

    fn none_play_hard(&self) -> bool {
        !self.any_play_hard()
    }

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
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

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

    fn iter_hard(&self) -> impl Iterator<Item = bool> {
        self.iter().map(|settings| settings.hard)
    }
}

impl WorldSettingsHelpers for UniverseSettings {
    fn is_empty(&self) -> bool {
        self.world_settings.is_empty()
    }

    fn lowest_difficulty(&self) -> Difficulty {
        self.world_settings.lowest_difficulty()
    }

    fn highest_difficulty(&self) -> Difficulty {
        self.world_settings.highest_difficulty()
    }

    fn iter_tricks(&self) -> impl Iterator<Item = &FxHashSet<Trick>> {
        self.world_settings.iter_tricks()
    }

    fn iter_hard(&self) -> impl Iterator<Item = bool> {
        self.world_settings.iter_hard()
    }
}

/// Seed settings bound to a specific world of a seed
///
/// See the [Multiplayer wiki page](https://wiki.orirando.com/features/multiplayer) for an explanation of worlds
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(default = WorldSettings::default)]
pub struct WorldSettings {
    /// Spawn destination
    pub spawn: Spawn,
    /// Logically expected difficulty
    pub difficulty: Difficulty,
    /// Logically expected tricks
    pub tricks: FxHashSet<Trick>,
    /// Logically assume hard in-game difficulty
    pub hard: bool,
    /// Randomize entrance connections with the given max loop size
    pub randomize_entrances: Option<GreaterOneU8>,
    /// Names of snippets to use
    pub snippets: Vec<String>,
    /// Additional inline snippets that don't exist on the filesystem
    #[schema(value_type = FxHashMap<String, Source>)]
    pub inline_snippets: InlineSnippets,
    /// Configuration to pass to snippets
    pub snippet_config: FxHashMap<String, FxHashMap<String, String>>,
}

impl WorldSettings {
    pub fn difficulty_default(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            ..Default::default()
        }
    }

    pub fn random<R: Rng, A: SnippetAccess>(rng: &mut R, snippet_access: &A) -> Self {
        let snippets = snippet_access.available_snippets_metadata();
        Self::random_with_metadata(rng, &snippets)
    }

    pub fn random_with_metadata<R: Rng>(rng: &mut R, snippets: &[(String, Metadata)]) -> Self {
        fn filter_default<T>(identifier: &str, value: T, default: T) -> Option<(String, String)>
        where
            T: PartialEq + ToString,
        {
            (value != default).then(|| (identifier.to_string(), value.to_string()))
        }

        fn gen_config_deviation<R: Rng>(rng: &mut R) -> f32 {
            // Irwin-Hall distribution
            rng.sample_iter::<f32, _>(Open01).take(12).sum::<f32>() - 6.
        }

        let difficulty = *<Difficulty as VariantArray>::VARIANTS.choose(rng).unwrap();

        let tricks = <Trick as VariantArray>::VARIANTS
            .iter()
            .filter(|trick| difficulty >= trick.min_difficulty() && rng.gen())
            .copied()
            .collect();

        let randomize_entrances = rng.gen::<bool>().then(|| {
            let loop_size = if rng.gen() { 2 } else { rng.gen_range(3..=32) };
            GreaterOneU8::new(loop_size).unwrap()
        });

        let mut snippet_config = FxHashMap::default();

        let snippets = snippets
            .iter()
            .filter_map(|(identifier, metadata)| {
                if metadata.hidden || rng.gen() {
                    return None;
                }

                snippet_config.insert(
                    identifier.clone(),
                    metadata
                        .config
                        .iter()
                        .filter_map(|(identifier, arg)| match arg.value {
                            ConfigValue::Boolean { default } => rng
                                .gen::<bool>()
                                .then(|| (identifier.clone(), (!default).to_string())),
                            ConfigValue::Integer { default } => {
                                let value = i32::max(default + gen_config_deviation(rng) as i32, 0);
                                filter_default(identifier, value, default)
                            }
                            ConfigValue::IntegerRange { default, min, max } => {
                                let value = (min <= max).then(|| rng.gen_range(min..=max))?;
                                filter_default(identifier, value, default)
                            }
                            ConfigValue::Float { default } => {
                                let value = default + gen_config_deviation(rng);
                                filter_default(identifier, value, default)
                            }
                            ConfigValue::FloatRange { default, min, max } => {
                                let value = (min <= max).then(|| rng.gen_range(*min..=*max))?;
                                filter_default(identifier, value, *default)
                            }
                        })
                        .collect(),
                );

                Some(identifier.clone())
            })
            .collect();

        Self {
            spawn: Spawn::FullyRandom,
            difficulty,
            tricks,
            hard: rng.gen_bool(0.25),
            randomize_entrances,
            snippets,
            inline_snippets: InlineSnippets::default(),
            snippet_config,
        }
    }

    /// Checks whether these settings feature a random spawn location
    pub fn is_random_spawn(&self) -> bool {
        matches!(self.spawn, Spawn::Random | Spawn::FullyRandom)
    }

    pub fn write_tags(&self, tags: &mut Vec<String>) {
        // Debug variant for the uppercase formatting
        tags.push(format!("{:?}", self.difficulty));

        if !self.tricks.is_empty() {
            let available = self.difficulty.available_tricks();

            // Especially with random settings some tricks may be logically irrelevant
            let mut enabled = available.iter().filter(|trick| self.tricks.contains(trick));

            if let Some(first) = enabled.next() {
                let available_len = available.len();
                let enabled_len = 1 + enabled.count();

                let tag = if enabled_len == available_len {
                    "All Tricks".to_string()
                } else if enabled_len == 1 {
                    first.to_string().to_title_case()
                } else {
                    // TODO allow checking details somewhere?
                    format!("Tricks ({enabled_len}/{available_len} enabled)")
                };

                tags.push(tag);
            }
        }

        if let Some(loop_size) = self.randomize_entrances {
            let mut random_entrances = "Random Entrances".to_string();

            if loop_size.get() > 2 {
                let _ = write!(&mut random_entrances, " (Loop Size {loop_size})");
            }

            tags.push(random_entrances);
        }
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

impl Spawn {
    /// Reproduces [`std::intrinsics::discriminant_value`].
    const fn discriminant_value(&self) -> isize {
        match self {
            Spawn::Set(_) => 0,
            Spawn::Random => 1,
            Spawn::FullyRandom => 2,
        }
    }
}

pub const DEFAULT_SPAWN: &str = "MarshSpawn.Main";
impl Default for Spawn {
    fn default() -> Spawn {
        Spawn::Set(DEFAULT_SPAWN.to_string())
    }
}

impl PartialOrd for Spawn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.discriminant_value().cmp(&other.discriminant_value()) {
            Ordering::Equal => match (self, other) {
                (Self::Set(a), Self::Set(b)) => (a == b).then_some(Ordering::Equal),
                _ => Some(Ordering::Equal),
            },
            non_equal => Some(non_equal),
        }
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

impl Difficulty {
    pub const fn available_tricks(self) -> &'static [Trick] {
        const MOKI: &[Trick] = &[];
        const GORLEK: &[Trick] = &Difficulty::Gorlek
            .available_tricks_array::<{ Difficulty::Gorlek.available_tricks_len() }>();
        const KII: &[Trick] =
            &Difficulty::Kii.available_tricks_array::<{ Difficulty::Kii.available_tricks_len() }>();
        const UNSAFE: &[Trick] = <Trick as VariantArray>::VARIANTS;

        match self {
            Difficulty::Moki => MOKI,
            Difficulty::Gorlek => GORLEK,
            Difficulty::Kii => KII,
            Difficulty::Unsafe => UNSAFE,
        }
    }

    pub fn available_tricks_iter(self) -> iter::Copied<slice::Iter<'static, Trick>> {
        self.available_tricks().iter().copied()
    }

    const fn available_tricks_array<const N: usize>(self) -> [Trick; N] {
        let mut tricks = [Trick::SwordSentryJump; N];
        let mut len = 0;

        let mut index = 0;
        while index < <Trick as VariantArray>::VARIANTS.len() {
            let trick = <Trick as VariantArray>::VARIANTS[index];

            if (self as u8) >= trick.min_difficulty() as u8 {
                tricks[len] = trick;
                len += 1;
            }

            index += 1;
        }

        assert!(len == N);

        tricks
    }

    const fn available_tricks_len(self) -> usize {
        let mut len = 0;

        let mut index = 0;
        while index < <Trick as VariantArray>::VARIANTS.len() {
            let trick = <Trick as VariantArray>::VARIANTS[index];

            if (self as u8) >= trick.min_difficulty() as u8 {
                len += 1;
            }

            index += 1;
        }

        len
    }
}

// TODO compability aliases?
// PauseHover -> PauseFloat
// HammerJump -> AerialHammerJump
// ExtendedHammer -> HammerExtension
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
    PartialOrd,
    Ord,
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
    HammerExtension,
    /// Redirecting projectiles with Grenade
    GrenadeRedirect,
    /// Redirecting projectiles with Sentry
    SentryRedirect,
    /// Cancelling falling momentum through the pause menu
    PauseFloat,
    /// Storing a grounded jump into the air with Spear
    SpearJump,
    /// Chaining Bash on the same target by cancelling the momentum with Glide
    GlideBashChain,
    /// Chaining Bash on the same target by cancelling the momentum with Double Jump
    DoubleJumpBashChain,
    /// Chaining Bash on the same target by cancelling the momentum with Dash
    DashBashChain,
    /// Chaining Bash on the same target by cancelling the momentum with Launch
    LaunchBashChain,
    /// Any specific trick that is unpopular for any reason
    Unpopular,
}

impl Trick {
    // TODO verify usage in logic?
    pub const fn min_difficulty(self) -> Difficulty {
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
            | Self::GlideHammerJump
            | Self::CoyoteHammerJump
            | Self::WallHammerJump
            | Self::GroundedHammerJump
            | Self::HammerExtension
            | Self::GrenadeRedirect
            | Self::SentryRedirect
            | Self::PauseFloat
            | Self::SpearJump
            | Self::GlideBashChain
            | Self::DoubleJumpBashChain
            | Self::DashBashChain
            | Self::LaunchBashChain
            | Self::Unpopular => Difficulty::Unsafe,
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
