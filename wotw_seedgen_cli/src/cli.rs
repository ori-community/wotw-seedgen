use std::error::Error;
use std::fmt;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

use rustc_hash::FxHashSet;
use structopt::StructOpt;

use wotw_seedgen::item::{Shard, Skill, Teleporter};
use wotw_seedgen::preset::{PresetGroup, PresetInfo, UniversePreset, WorldPreset};
use wotw_seedgen::settings::{Difficulty, Goal, HeaderConfig, InlineHeader, Spawn, Trick};
use wotw_seedgen::util::Zone;

#[derive(StructOpt)]
/// Generate seeds for the Ori 2 randomizer.
///
/// Type seedgen.exe seed --help for further instructions
pub struct SeedGen {
    /// wait for a debugger to attach before running
    #[structopt(short = "d", long = "debug")]
    pub wait_on_debugger: bool,
    #[structopt(subcommand)]
    pub command: SeedGenCommand,
}

#[derive(StructOpt)]
pub enum SeedGenCommand {
    /// Generate a seed
    Seed {
        #[structopt(flatten)]
        args: SeedArgs,
    },
    /// Play the most recent generated seed
    Play,
    /// Create a universe preset of the given settings
    ///
    /// A universe preset defines the settings for the entire game and can contain different settings on a per world basis
    UniversePreset {
        #[structopt(flatten)]
        args: UniversePresetArgs,
    },
    /// Create a world preset of the given settings
    ///
    /// A world preset defines the settings for one world and will be applied to all worlds the same way when generating a multiworld seed
    WorldPreset {
        #[structopt(flatten)]
        args: WorldPresetArgs,
    },
    /// Generate seed statistics
    Stats {
        #[structopt(flatten)]
        args: StatsArgs,
    },
    /// Deletes all cached seeds used to generate statistics
    CleanStatsCache,
    /// Check which locations are in logic
    ReachCheck {
        #[structopt(flatten)]
        args: ReachCheckArgs,
    },
    /// Inspect the available headers
    Headers {
        /// headers to look at in detail
        headers: Vec<String>,
        #[structopt(subcommand)]
        subcommand: Option<HeaderCommand>,
    },
}

#[derive(StructOpt)]
pub struct SeedArgs {
    /// the seed's name and name of the file it will be written to. The name also seeds the rng if no seed is given.
    #[structopt()]
    pub filename: Option<String>,
    /// which folder to write the seed into
    #[structopt(parse(from_os_str), default_value = "seeds", long = "seeddir")]
    pub seed_folder: PathBuf,
    /// the input file representing the logic
    #[structopt(parse(from_os_str), default_value = "areas.wotw", long)]
    pub areas: PathBuf,
    /// the input file representing pickup locations
    #[structopt(parse(from_os_str), default_value = "loc_data.csv", long)]
    pub locations: PathBuf,
    /// the input file representing state namings
    #[structopt(parse(from_os_str), default_value = "state_data.csv", long)]
    pub uber_states: PathBuf,
    /// create a generator.log with verbose output about the generation process
    #[structopt(short, long)]
    pub verbose: bool,
    /// skip validating the input files for a slight performance gain
    #[structopt(long)]
    pub trust: bool,
    /// write the seed to stdout instead of a file
    #[structopt(long)]
    pub tostdout: bool,
    /// write stderr logs in json format
    #[structopt(long)]
    pub json_stderr: bool,
    /// use json output where possible
    ///
    /// If --tostdout is enabled, a json object with all output data is written to stdout.
    /// If --tostdout is disabled, only spoilers will be written as json files.
    #[structopt(long)]
    pub json: bool,
    /// launch the seed after generating
    #[structopt(short, long)]
    pub launch: bool,

    #[structopt(flatten)]
    pub settings: SeedSettings,
}

#[derive(StructOpt)]
pub struct UniversePresetArgs {
    /// name of the preset
    ///
    /// later you can run seed -P <preset-name> to use this preset
    pub filename: String,
    #[structopt(flatten)]
    pub info: PresetInfoArgs,
    #[structopt(flatten)]
    pub settings: SeedSettings,
}

/// For CLI flags that contain a mixture of world specifiers and flag values
pub struct WorldOpt<T> {
    pub source: String,
    pub inner: WorldOptInner<T>,
}
impl<T: FromStr> FromStr for WorldOpt<T> {
    type Err = WorldOptError<T::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = if let Some(world) = s.strip_prefix(':') {
            let index = world
                .parse()
                .map_err(|_| WorldOptError::IndexError(world.to_string()))?;
            WorldOptInner::World(index)
        } else {
            WorldOptInner::Opt(T::from_str(s).map_err(WorldOptError::ValueError)?)
        };
        let source = s.to_string();
        Ok(WorldOpt { source, inner })
    }
}
#[derive(Debug)]
pub enum WorldOptError<Err> {
    IndexError(String),
    ValueError(Err),
}
impl<Err: fmt::Display> fmt::Display for WorldOptError<Err> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorldOptError::IndexError(index) => write!(f, "Invalid world index :{index}"),
            WorldOptError::ValueError(err) => write!(f, "{err}"),
        }
    }
}
impl<Err: fmt::Display + fmt::Debug> Error for WorldOptError<Err> {}

pub enum WorldOptInner<T> {
    World(usize),
    Opt(T),
}

fn resolve_world_opts<T: Clone>(
    world_opts: Vec<WorldOpt<T>>,
    worlds: usize,
) -> Result<Vec<Vec<T>>, String> {
    let mut world_values: Vec<Vec<T>> = vec![vec![]; worlds];
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(index) => current_world = Some(index),
            WorldOptInner::Opt(value) => {
                if let Some(index) = current_world {
                    world_values
                        .get_mut(index)
                        .ok_or(format!("World index {index} greater than number of worlds"))?
                        .push(value);
                } else {
                    for world in &mut world_values {
                        world.push(value.clone());
                    }
                }
            }
        }
    }

    Ok(world_values)
}
fn resolve_nonduplicate_world_opts<T: Clone>(
    world_opts: Vec<WorldOpt<T>>,
    worlds: usize,
) -> Result<Vec<Option<T>>, String> {
    let mut world_values: Vec<Option<(T, String)>> = vec![None; worlds];
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(index) => current_world = Some(index),
            WorldOptInner::Opt(value) => {
                if let Some(index) = current_world {
                    let current_world_entry = world_values
                        .get_mut(index)
                        .ok_or(format!("World index {index} greater than number of worlds"))?;
                    *current_world_entry = Some((value, world_opt.source));
                } else {
                    for current_world_entry in &mut world_values {
                        *current_world_entry = Some((value.clone(), world_opt.source.clone()));
                    }
                }
            }
        }
    }

    let world_values = world_values
        .into_iter()
        .map(|current_world_value| current_world_value.map(|t| t.0))
        .collect();
    Ok(world_values)
}
fn resolve_flag_world_opts(
    world_opts: Option<Vec<WorldOpt<bool>>>,
    worlds: usize,
) -> Result<Vec<Option<bool>>, String> {
    match world_opts {
        Some(opts) => {
            if opts.is_empty() {
                Ok(vec![Some(true); worlds])
            } else {
                resolve_nonduplicate_world_opts(opts, worlds)
            }
        }
        None => Ok(vec![None; worlds]),
    }
}

pub type CannotError = String;

/// Newtype to parse spawn flag
#[derive(Clone)]
pub struct SpawnOpt(Spawn);
impl SpawnOpt {
    fn into_inner(self) -> Spawn {
        self.0
    }
}
impl FromStr for SpawnOpt {
    type Err = CannotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spawn = match &s.to_lowercase()[..] {
            "r" | "random" => Spawn::Random,
            "f" | "fullyrandom" => Spawn::FullyRandom,
            _ => Spawn::Set(s.to_string()),
        };
        Ok(SpawnOpt(spawn))
    }
}
/// Newtype to parse goals flag
#[derive(Clone)]
pub struct GoalsOpt(Goal);
impl GoalsOpt {
    fn into_inner(self) -> Goal {
        self.0
    }
}
impl FromStr for GoalsOpt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (identifier, details) = s.split_once(':').unwrap_or((s, ""));

        let goal = match identifier {
            "t" | "trees" => Goal::Trees,
            "w" | "wisps" => Goal::Wisps,
            "q" | "quests" => Goal::Quests,
            "r" | "relics" => {
                if !details.is_empty() {
                    if let Some(chance) = details.strip_suffix('%') {
                        let chance = chance.parse::<f64>().map_err(|_| {
                            format!("Invalid chance in details string for goal {s}")
                        })?;
                        if !(0.0..=100.0).contains(&chance) {
                            return Err(format!("Invalid chance in details string for goal {s}"));
                        }
                        Goal::RelicChance(chance / 100.0)
                    } else {
                        let amount = details.parse().map_err(|_| {
                            format!(
                                "expected amount or % expression in details string for goal {s}"
                            )
                        })?;
                        if !(0..=11).contains(&amount) {
                            return Err(format!("Invalid amount in details string for goal {s}"));
                        }
                        Goal::Relics(amount)
                    }
                } else {
                    Goal::RelicChance(0.6)
                }
            }
            other => return Err(format!("Unknown goal {other}")),
        };

        Ok(GoalsOpt(goal))
    }
}
/// Newtype to parse header config
#[derive(Clone)]
pub struct HeaderConfigOpt(HeaderConfig);
impl HeaderConfigOpt {
    fn into_inner(self) -> HeaderConfig {
        self.0
    }
}
impl FromStr for HeaderConfigOpt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (identifier, config_value) = s.split_once('=').unwrap_or((s, "true"));
        let (header_name, config_name) = identifier.split_once('.').ok_or_else(|| {
            format!("Expected <header>.<parameter> in header configuration parameter {s}")
        })?;

        let header_config = HeaderConfig {
            header_name: header_name.to_string(),
            config_name: config_name.to_string(),
            config_value: config_value.to_string(),
        };

        Ok(HeaderConfigOpt(header_config))
    }
}
/// Newtype to parse inline headers
#[derive(Clone)]
pub struct InlineHeaderOpt(InlineHeader);
impl InlineHeaderOpt {
    fn into_inner(self) -> InlineHeader {
        self.0
    }
}
impl FromStr for InlineHeaderOpt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inline_header = InlineHeader {
            name: None,
            content: s.to_string(),
        };
        Ok(InlineHeaderOpt(inline_header))
    }
}

#[derive(StructOpt)]
pub struct SeedSettings {
    /// Derive the settings from one or more presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "P", long)]
    pub universe_presets: Option<Vec<String>>,
    /// Derive the settings for individual worlds from one or more presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "p", long)]
    pub world_presets: Vec<WorldOpt<String>>,
    /// How many worlds to generate
    ///
    /// Seeds with more than one world are called multiworld seeds
    #[structopt(short, long, default_value = "1")]
    pub worlds: usize,
    /// Spawn destination
    ///
    /// Use an anchor name from the areas file, "r" / "random" for a random teleporter or "f" / "fullyrandom" for any location
    #[structopt(short, long)]
    pub spawn: Vec<WorldOpt<SpawnOpt>>,
    /// Logically expected difficulty of execution you may be required to perform
    ///
    /// Available difficulties are "moki", "gorlek", "unsafe"
    #[structopt(short, long)]
    pub difficulty: Vec<WorldOpt<Difficulty>>,
    /// Logically expected tricks you may have to use
    ///
    /// Available tricks are "swordsentryjump", "hammersentryjump", "shurikenbreak", "sentrybreak", "hammerbreak", "spearbreak", "sentryburn", "removekillplane", "launchswap", "sentryswap", "flashswap", "blazeswap", "wavedash", "grenadejump", "hammerjump", "swordjump", "grenaderedirect", "sentryredirect", "pausehover", "glidejump", "glidehammerjump", "spearjump"
    #[structopt(short, long)]
    pub tricks: Vec<WorldOpt<Trick>>,
    /// Logically assume hard in-game difficulty
    #[structopt(long)]
    pub hard: Option<Vec<WorldOpt<bool>>>,
    /// Goal Requirements before finishing the game
    ///
    /// Available goals are trees, wisps, quests, relics. Relics can further configure the chance per area to have a relic, default is relics:60%
    #[structopt(short, long)]
    pub goals: Vec<WorldOpt<GoalsOpt>>,
    /// Names of headers that will be used when generating the seed
    ///
    /// The headers will be searched as .wotwrh files in the current and /headers child directory
    #[structopt(short, long)]
    pub headers: Vec<WorldOpt<String>>,
    /// Configuration parameters to pass to headers
    ///
    /// Format for one parameter: <headername>.<parametername>=<value>
    #[structopt(short = "c", long = "config")]
    pub header_config: Vec<WorldOpt<HeaderConfigOpt>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    pub inline_headers: Vec<WorldOpt<InlineHeaderOpt>>,
    /// Disallow the use of the In-Logic filter while playing the seed
    #[structopt(short = "L", long)]
    pub disable_logic_filter: bool,
    /// Require an online connection to play the seed
    ///
    /// This is needed for Co-op, Multiworld and Bingo
    #[structopt(short, long)]
    pub online: bool,
    /// Seed the random number generator
    ///
    /// Without this flag, the rng seed will be randomly generated
    #[structopt(long)]
    pub seed: Option<String>,
}

fn slice_in_option<T, S: Deref<Target = [T]>>(slice: S) -> Option<S> {
    if slice.is_empty() {
        None
    } else {
        Some(slice)
    }
}

impl SeedSettings {
    pub fn into_universe_preset(self) -> Result<UniversePreset, String> {
        let Self {
            universe_presets,
            world_presets,
            worlds,
            spawn,
            difficulty,
            tricks,
            hard,
            goals,
            headers,
            header_config,
            inline_headers,
            disable_logic_filter,
            online,
            seed,
        } = self;

        let world_presets = resolve_world_opts(world_presets, worlds)?;
        let world_spawns = resolve_nonduplicate_world_opts(spawn, worlds)?;
        let world_difficulties = resolve_nonduplicate_world_opts(difficulty, worlds)?;
        let world_tricks = resolve_world_opts(tricks, worlds)?;
        let world_hard_flags = resolve_flag_world_opts(hard, worlds)?;
        let world_goals = resolve_world_opts(goals, worlds)?;
        let world_headers = resolve_world_opts(headers, worlds)?;
        let world_header_configs = resolve_world_opts(header_config, worlds)?;
        let world_inline_headers = resolve_world_opts(inline_headers, worlds)?;

        let disable_logic_filter = if disable_logic_filter {
            Some(true)
        } else {
            None
        };
        let online = if online { Some(true) } else { None };

        let yes_fun = world_presets
            .into_iter()
            .zip(world_spawns)
            .zip(world_difficulties)
            .zip(world_tricks)
            .zip(world_hard_flags)
            .zip(world_goals)
            .zip(world_headers)
            .zip(world_header_configs)
            .zip(world_inline_headers)
            .map(
                |(
                    (
                        ((((((world_presets, spawn), difficulty), tricks), hard), goals), headers),
                        header_config,
                    ),
                    inline_headers,
                )| {
                    WorldPreset {
                        info: None,
                        includes: slice_in_option(world_presets).map(FxHashSet::from_iter),
                        spawn: spawn.map(SpawnOpt::into_inner),
                        difficulty,
                        tricks: slice_in_option(tricks).map(FxHashSet::from_iter),
                        goals: slice_in_option(
                            goals.into_iter().map(GoalsOpt::into_inner).collect(),
                        ),
                        hard,
                        headers: slice_in_option(headers).map(FxHashSet::from_iter),
                        header_config: slice_in_option(
                            header_config
                                .into_iter()
                                .map(HeaderConfigOpt::into_inner)
                                .collect(),
                        ),
                        inline_headers: slice_in_option(
                            inline_headers
                                .into_iter()
                                .map(InlineHeaderOpt::into_inner)
                                .collect(),
                        ),
                    }
                },
            )
            .collect::<Vec<_>>();

        Ok(UniversePreset {
            info: None,
            includes: universe_presets.map(FxHashSet::from_iter),
            world_settings: Some(yes_fun),
            disable_logic_filter,
            seed,
            online,
            create_game: None,
        })
    }
}

#[derive(StructOpt)]
pub struct WorldPresetArgs {
    /// Name of the preset
    ///
    /// Later you can run seed -p <preset-name> to use this preset
    pub filename: String,
    #[structopt(flatten)]
    pub settings: WorldPresetSettings,
}

#[derive(StructOpt)]
pub struct WorldPresetSettings {
    #[structopt(flatten)]
    pub info: PresetInfoArgs,
    /// Include further world presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "p", long)]
    pub includes: Option<Vec<String>>,
    /// Spawn destination
    ///
    /// Use an anchor name from the areas file, "r" / "random" for a random teleporter or "f" / "fullyrandom" for any location
    #[structopt(short, long)]
    pub spawn: Option<SpawnOpt>,
    /// Logically expected difficulty of execution you may be required to perform
    ///
    /// Available difficulties are "moki", "gorlek", "unsafe"
    #[structopt(short, long)]
    pub difficulty: Option<Difficulty>,
    /// Logically expected tricks you may have to use
    ///
    /// Available tricks are "swordsentryjump", "hammersentryjump", "shurikenbreak", "sentrybreak", "hammerbreak", "spearbreak", "sentryburn", "removekillplane", "launchswap", "sentryswap", "flashswap", "blazeswap", "wavedash", "grenadejump", "hammerjump", "swordjump", "grenaderedirect", "sentryredirect", "pausehover", "glidejump", "glidehammerjump", "spearjump"
    #[structopt(short, long)]
    pub tricks: Option<Vec<Trick>>,
    /// Logically assume hard in-game difficulty
    #[structopt(long)]
    pub hard: bool,
    /// Goal Requirements before finishing the game
    ///
    /// Available goals are trees, wisps, quests, relics. Relics can further configure the chance per area to have a relic, default is relics:60%
    #[structopt(short, long)]
    pub goals: Option<Vec<GoalsOpt>>,
    /// Names of headers that will be used when generating the seed
    ///
    /// The headers will be searched as .wotwrh files in the current and /headers child directory
    #[structopt(short, long)]
    pub headers: Option<Vec<String>>,
    /// Configuration parameters to pass to headers
    ///
    /// Format for one parameter: <headername>.<parametername>=<value>
    #[structopt(short = "c", long = "config")]
    pub header_config: Option<Vec<HeaderConfigOpt>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    pub inline_headers: Option<Vec<InlineHeaderOpt>>,
}

impl WorldPresetSettings {
    pub fn into_world_preset(self) -> WorldPreset {
        let Self {
            info,
            includes,
            spawn,
            difficulty,
            tricks,
            hard,
            goals,
            headers,
            header_config,
            inline_headers,
        } = self;

        WorldPreset {
            info: info.into_preset_info(),
            includes: includes.map(FxHashSet::from_iter),
            spawn: spawn.map(SpawnOpt::into_inner),
            difficulty,
            tricks: tricks.map(FxHashSet::from_iter),
            hard: if hard { Some(true) } else { None },
            goals: goals.map(|goals| goals.into_iter().map(GoalsOpt::into_inner).collect()),
            headers: headers.map(FxHashSet::from_iter),
            header_config: header_config.map(|header_config| {
                header_config
                    .into_iter()
                    .map(HeaderConfigOpt::into_inner)
                    .collect()
            }),
            inline_headers: inline_headers.map(|inline_headers| {
                inline_headers
                    .into_iter()
                    .map(InlineHeaderOpt::into_inner)
                    .collect()
            }),
        }
    }
}

#[derive(StructOpt)]
pub struct StatsArgs {
    /// the input file representing the logic
    #[structopt(parse(from_os_str), default_value = "areas.wotw", long)]
    pub areas: PathBuf,
    /// the input file representing pickup locations
    #[structopt(parse(from_os_str), default_value = "loc_data.csv", long)]
    pub locations: PathBuf,
    /// the input file representing state namings
    #[structopt(parse(from_os_str), default_value = "state_data.csv", long)]
    pub uber_states: PathBuf,
    /// How many samples (seeds) to use
    #[structopt(short = "z", long, default_value = "10000")]
    pub sample_size: usize,
    /// Any amount of analyzers that will provide statistics
    ///
    /// Multiple analyzers separated with spaces will create separate stats files analyzing their individual criteria
    ///
    /// However, you can also chain analyzers together by separating them with plus (no spaces).
    /// E.g. --analyzers spawn-locations+zone-unlocks will generate stats that analyze the zone unlocks for each individual spawn location
    #[structopt(short, long, required = true)]
    pub analyzers: Vec<ChainedAnalyzers>,
    /// The generated stats will be written to "stats/<folder_name>/<stats go here>"
    ///
    /// This will default to a summary of the provided settings
    #[structopt(short, long)]
    pub folder_name: Option<String>,
    /// How many errors during seed generation should be tolerated before aborting
    ///
    /// If not provided, this will default to a value based on `sample_size`
    #[structopt(long)]
    pub tolerated_errors: Option<usize>,
    /// How many error messages should be displayed after aborting due to `tolerated_errors` being exceeded
    #[structopt(long, default_value = "10")]
    pub error_message_limit: usize,
    /// cleans the cache for the provided settings and generates new seeds from scratch
    #[structopt(long)]
    pub overwrite_cache: bool,
    #[structopt(flatten)]
    pub settings: SeedSettings,
}

pub struct ChainedAnalyzers(pub Vec<Analyzer>);
#[derive(StructOpt)]
pub enum Analyzer {
    SpawnLocations,
    SpawnRegion,
    ZoneUnlock { zone: Zone },
}
impl FromStr for ChainedAnalyzers {
    type Err = structopt::clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('+')
            .map(|s| {
                match s.split_once(':') {
                    None => Analyzer::from_iter_safe(["", s]), // The first arg is the "executable name", I think clap would have a more elegant solution here
                    Some((identifier, args)) => Analyzer::from_iter_safe(
                        ["", identifier].into_iter().chain(args.split(',')),
                    ),
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

#[derive(StructOpt)]
pub struct ReachCheckArgs {
    /// the seed file for which logical reach should be checked
    #[structopt(parse(from_os_str))]
    pub seed_file: PathBuf,
    /// the input file representing the logic
    #[structopt(parse(from_os_str), default_value = "areas.wotw", short, long)]
    pub areas: PathBuf,
    /// the input file representing pickup locations
    #[structopt(parse(from_os_str), default_value = "loc_data.csv", short, long)]
    pub locations: PathBuf,
    /// the input file representing state namings
    #[structopt(parse(from_os_str), default_value = "state_data.csv", short, long)]
    pub uber_states: PathBuf,
    /// player health (one orb is 10 health)
    pub health: u32,
    /// player energy (one orb is 1 energy)
    pub energy: f32,
    /// player keystones
    pub keystones: u32,
    /// player ore
    pub ore: u32,
    /// player spirit light
    pub spirit_light: u32,
    /// any additional player items in the format s:<skill id>, t:<teleporter id>, sh:<shard id>, w:<world event id> or n:<node identifier>
    pub items: Vec<ReachData>,
}

pub enum ReachData {
    Skill(Skill),
    Teleporter(Teleporter),
    Shard(Shard),
    Water,
    Node(String),
}
impl FromStr for ReachData {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, data) = s
            .split_once(':')
            .ok_or_else(|| "Expected <kind>:<data>".to_string())?;
        match kind {
            "s" => data.parse().map(Self::Skill).map_err(|err| err.to_string()),
            "t" => data
                .parse()
                .map(Self::Teleporter)
                .map_err(|err| err.to_string()),
            "sh" => data.parse().map(Self::Shard).map_err(|err| err.to_string()),
            "w" => {
                if data == "0" {
                    Ok(Self::Water)
                } else {
                    Err(format!("Unknown world event \"{data}\""))
                }
            }
            "n" => Ok(Self::Node(data.to_string())),
            _ => Err(format!(
                "Invalid arg \"{s}\", args have to start with s:, t:, sh:, w: or n:"
            )),
        }
    }
}

#[derive(StructOpt)]
pub enum HeaderCommand {
    /// Check header compability
    Validate {
        /// A file to validate, or leave empty to validate all headers in the directory
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },
    /// Parse a header or plandomizer into the seed format
    Parse {
        /// The file to parse
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

#[derive(StructOpt)]
pub struct PresetInfoArgs {
    /// Display name
    #[structopt(long)]
    pub name: Option<String>,
    /// Extended description
    #[structopt(long)]
    pub description: Option<String>,
    /// Mark this as a base preset
    ///
    /// Base presets are displayed more prominently
    #[structopt(long)]
    pub base_preset: bool,
}

impl PresetInfoArgs {
    pub fn into_preset_info(self) -> Option<PresetInfo> {
        let Self {
            name,
            description,
            base_preset,
        } = self;

        let preset_info = PresetInfo {
            name,
            description,
            group: if base_preset {
                Some(PresetGroup::Base)
            } else {
                None
            },
        };

        if preset_info == PresetInfo::default() {
            None
        } else {
            Some(preset_info)
        }
    }
}
