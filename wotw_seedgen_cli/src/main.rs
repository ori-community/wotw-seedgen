mod log_init;
use log_init::initialize_log;

use std::{
    fs,
    str::FromStr,
    path::PathBuf,
    convert::TryFrom,
    io::{self, Read},
    time::Instant,
    env, error::Error, process::ExitCode,
    fmt::{self, Display, Debug},
};

use rustc_hash::FxHashMap;
use structopt::StructOpt;
use bugsalot::debugger;

use log::LevelFilter;

use wotw_seedgen::{self, item, world, settings::{Spawn, Difficulty, Trick, Goal, HeaderConfig, InlineHeader}, util, header, preset::WorldPreset, Preset, Settings, logic, Header, generator::Seed};

use item::{Item, Resource, Skill, Shard, Teleporter};
use world::World;
use util::UberState;

/// For CLI flags that contain a mixture of world specifiers and flag values
struct WorldOpt<T> {
    source: String,
    inner: WorldOptInner<T>,
}
impl<T: FromStr> FromStr for WorldOpt<T> {
    type Err = WorldOptError<T::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = if let Some(world) = s.strip_prefix(':') {
            let index = world.parse().map_err(|_| WorldOptError::IndexError(world.to_string()))?;
            WorldOptInner::World(index)
        } else {
            WorldOptInner::Opt(T::from_str(s).map_err( WorldOptError::ValueError)?)
        };
        let source = s.to_string();
        Ok(WorldOpt { source, inner })
    }
}
#[derive(Debug)]
enum WorldOptError<Err> {
    IndexError(String),
    ValueError(Err),
}
impl<Err: Display> Display for WorldOptError<Err> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WorldOptError::IndexError(index) => write!(f, "Invalid world index :{index}"),
            WorldOptError::ValueError(err) => write!(f, "{err}"),
        }
    }
}
impl<Err: Display + Debug> Error for WorldOptError<Err> {}

enum WorldOptInner<T> {
    World(usize),
    Opt(T),
}

fn resolve_world_opts<T: Clone>(world_opts: Vec<WorldOpt<T>>, world_count: usize) -> Result<Vec<Vec<T>>, String> {
    let mut world_values: Vec<Vec<T>> = vec![vec![]; world_count];
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(index) => current_world = Some(index),
            WorldOptInner::Opt(value) => {
                if let Some(index) = current_world {
                    world_values.get_mut(index).ok_or(format!("World index {index} greater than number of worlds"))?.push(value);
                } else {
                    for world in &mut world_values {
                        world.push(value.clone());
                    }
                }
            },
        }
    }

    Ok(world_values)
}

fn assign_nonduplicate<T>(assign: T, current_world_entry: &mut Option<(T, String)>, source: String) -> Result<(), String> {
    match current_world_entry {
        Some((_, prior_source)) => Err(format!("Provided multiple values for the same world: {source} and {prior_source}")),
        None => {
            *current_world_entry = Some((assign, source));
            Ok(())
        },
    }
}
fn resolve_nonduplicate_world_opts<T: Clone>(world_opts: Vec<WorldOpt<T>>, world_count: usize) -> Result<Vec<Option<T>>, String> {
    let mut world_values: Vec<Option<(T, String)>> = vec![None; world_count];
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(index) => current_world = Some(index),
            WorldOptInner::Opt(value) => {
                if let Some(index) = current_world {
                    let current_world_entry = world_values.get_mut(index).ok_or(format!("World index {index} greater than number of worlds"))?;
                    assign_nonduplicate(value, current_world_entry, world_opt.source)?;
                } else {
                    for current_world_entry in &mut world_values {
                        assign_nonduplicate(value.clone(), current_world_entry, world_opt.source.clone())?;
                    }
                }
            },
        }
    }

    let world_values = world_values.into_iter().map(|current_world_value| current_world_value.map(|t| t.0)).collect();
    Ok(world_values)
}

type CannotError = String;

/// Newtype to parse spawn flag
#[derive(Clone)]
struct SpawnOpt(Spawn);
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
struct GoalsOpt(Goal);
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
                        let chance = chance.parse::<f64>().map_err(|_| format!("Invalid chance in details string for goal {s}"))?;
                        if !(0.0..=100.0).contains(&chance) { return Err(format!("Invalid chance in details string for goal {s}")); }
                        Goal::RelicChance(chance / 100.0)
                    } else {
                        let amount = details.parse().map_err(|_| format!("expected amount or % expression in details string for goal {s}"))?;
                        if !(0..=11).contains(&amount) { return Err(format!("Invalid amount in details string for goal {s}")); }
                        Goal::Relics(amount)
                    }
                } else { Goal::RelicChance(0.6) }
            },
            other => return Err(format!("Unknown goal {other}")),
        };

        Ok(GoalsOpt(goal))
    }
}
/// Newtype to parse header config
#[derive(Clone)]
struct HeaderConfigOpt(HeaderConfig);
impl HeaderConfigOpt {
    fn into_inner(self) -> HeaderConfig {
        self.0
    }
}
impl FromStr for HeaderConfigOpt {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (identifier, config_value) = s.split_once('=').unwrap_or((s, "true"));
        let (header_name, config_name) = identifier.split_once('.').ok_or_else(|| format!("Expected <header>.<parameter> in header configuration parameter {s}"))?;

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
struct InlineHeaderOpt(InlineHeader);
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
/// Generate seeds for the Ori 2 randomizer.
///
/// Type seedgen.exe seed --help for further instructions
struct SeedGen {
    /// wait for a debugger to attach before running
    #[structopt(short = "d", long = "debug")]
    wait_on_debugger: bool,
    #[structopt(subcommand)]
    command: SeedGenCommand,
}

#[derive(StructOpt)]
enum SeedGenCommand {
    /// Generate a seed
    Seed {
        #[structopt(flatten)]
        args: SeedArgs,
    },
    /// Play the most recent generated seed
    Play,
    /// Create a preset of the given settings
    Preset {
        #[structopt(flatten)]
        args: PresetArgs,
    },
    /// Create a world preset of the given settings
    WorldPreset {
        #[structopt(flatten)]
        args: WorldPresetArgs,
    },
    /// Check which locations are in logic
    ReachCheck {
        #[structopt(flatten)]
        args: ReachCheckArgs,
    },
    /// Inspect the available headers
    Headers {
        /// headers to look at in detail
        #[structopt(parse(from_os_str))]
        headers: Vec<PathBuf>,
        #[structopt(subcommand)]
        subcommand: Option<HeaderCommand>,
    },
}

#[derive(StructOpt)]
struct SeedArgs {
    /// the seed's name and name of the file it will be written to. The name also seeds the rng if no seed is given.
    #[structopt()]
    filename: Option<String>,
    /// which folder to write the seed into
    #[structopt(parse(from_os_str), default_value = "seeds", long = "seeddir")]
    seed_folder: PathBuf,
    /// the input file representing the logic
    #[structopt(parse(from_os_str), default_value = "areas.wotw", long)]
    areas: PathBuf,
    /// the input file representing pickup locations
    #[structopt(parse(from_os_str), default_value = "loc_data.csv", long)]
    locations: PathBuf,
    /// the input file representing state namings
    #[structopt(parse(from_os_str), default_value = "state_data.csv", long)]
    uber_states: PathBuf,
    /// create a generator.log with verbose output about the generation process
    #[structopt(short, long)]
    verbose: bool,
    /// skip validating the input files for a slight performance gain
    #[structopt(long)]
    trust: bool,
    /// write the seed to stdout instead of a file
    #[structopt(long)]
    tostdout: bool,
    /// write stderr logs in json format
    #[structopt(long)]
    json_stderr: bool,
    /// write spoiler in json format
    #[structopt(long)]
    json_spoiler: bool,
    /// launch the seed after generating
    #[structopt(short, long)]
    launch: bool,
    /// Seed the random number generator
    /// 
    /// Without this flag, the rng seed will be randomly generated
    #[structopt(long)]
    seed: Option<String>,
    #[structopt(flatten)]
    settings: SeedSettings,
}

#[derive(StructOpt)]
struct SeedSettings {
    /// Derive the settings from one or more presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "P", long)]
    presets: Option<Vec<String>>,
    /// Derive the settings for individual worlds from one or more presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "p", long)]
    world_presets: Vec<WorldOpt<String>>,
    /// How many worlds to generate
    /// 
    /// Seeds with more than one world are called multiworld seeds
    #[structopt(short, long, default_value = "1")]
    world_count: usize,
    /// Spawn destination
    ///
    /// Use an anchor name from the areas file, "r" / "random" for a random teleporter or "f" / "fullyrandom" for any location
    #[structopt(short, long)]
    spawn: Vec<WorldOpt<SpawnOpt>>,
    /// Logically expected difficulty of execution you may be required to perform
    ///
    /// Available difficulties are "moki", "gorlek", "unsafe"
    #[structopt(short, long)]
    difficulty: Vec<WorldOpt<Difficulty>>,
    /// Logically expected tricks you may have to use
    ///
    /// Available tricks are "swordsentryjump", "hammersentryjump", "shurikenbreak", "sentrybreak", "hammerbreak", "spearbreak", "sentryburn", "removekillplane", "launchswap", "sentryswap", "flashswap", "blazeswap", "wavedash", "grenadejump", "hammerjump", "swordjump", "grenaderedirect", "sentryredirect", "pausehover", "glidejump", "glidehammerjump", "spearjump"
    #[structopt(short, long)]
    tricks: Vec<WorldOpt<Trick>>,
    /// Logically assume hard in-game difficulty
    #[structopt(long)]
    hard: Vec<WorldOpt<bool>>,
    /// Goal Requirements before finishing the game
    ///
    /// Available goals are trees, wisps, quests, relics. Relics can further configure the chance per area to have a relic, default is relics:60%
    #[structopt(short, long)]
    goals: Vec<WorldOpt<GoalsOpt>>,
    /// Names of headers that will be used when generating the seed
    /// 
    /// The headers will be searched as .wotwrh files in the current and /headers child directory
    #[structopt(short, long)]
    headers: Vec<WorldOpt<String>>,
    /// Configuration parameters to pass to headers
    ///
    /// Format for one parameter: <headername>.<parametername>=<value>
    #[structopt(short = "c", long = "config")]
    header_config: Vec<WorldOpt<HeaderConfigOpt>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    inline_headers: Vec<WorldOpt<InlineHeaderOpt>>,
    /// Disallow the use of the In-Logic filter while playing the seed
    #[structopt(short = "L", long)]
    disable_logic_filter: bool,
    /// Require an online connection to play the seed
    /// 
    /// This is needed for Co-op, Multiworld and Bingo
    #[structopt(short, long)]
    online: bool,
}

fn vec_in_option<T>(vector: Vec<T>) -> Option<Vec<T>> {
    if vector.is_empty() { None } else { Some(vector) }
}

impl SeedSettings {
    fn into_preset(self) -> Result<Preset, String> {
        let Self {
            presets,
            world_presets,
            world_count,
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
        } = self;

        let world_presets = resolve_world_opts(world_presets, world_count)?;
        let world_spawns = resolve_nonduplicate_world_opts(spawn, world_count)?;
        let world_difficulties = resolve_nonduplicate_world_opts(difficulty, world_count)?;
        let world_tricks = resolve_world_opts(tricks, world_count)?;
        let world_hard_flags = resolve_nonduplicate_world_opts(hard, world_count)?;
        let world_goals = resolve_world_opts(goals, world_count)?;
        let world_headers = resolve_world_opts(headers, world_count)?;
        let world_header_configs = resolve_world_opts(header_config, world_count)?;
        let world_inline_headers = resolve_world_opts(inline_headers, world_count)?;

        let disable_logic_filter = if disable_logic_filter { Some(true) } else { None };
        let online = if online { Some(true) } else { None };

        let yes_fun = world_presets.into_iter()
            .zip(world_spawns)
            .zip(world_difficulties)
            .zip(world_tricks)
            .zip(world_hard_flags)
            .zip(world_goals)
            .zip(world_headers)
            .zip(world_header_configs)
            .zip(world_inline_headers)
            .map(|((((((((world_presets, spawn), difficulty), tricks), hard), goals), headers), header_config), inline_headers)| {
                WorldPreset {
                    includes: vec_in_option(world_presets),
                    spawn: spawn.map(SpawnOpt::into_inner),
                    difficulty,
                    tricks: vec_in_option(tricks),
                    goals: vec_in_option(goals.into_iter().map(GoalsOpt::into_inner).collect()),
                    hard,
                    headers: vec_in_option(headers),
                    header_config: vec_in_option(header_config.into_iter().map(HeaderConfigOpt::into_inner).collect()),
                    inline_headers: vec_in_option(inline_headers.into_iter().map(InlineHeaderOpt::into_inner).collect()),
                }
            }).collect::<Vec<_>>();

        Ok(Preset {
            includes: presets,
            world_settings: Some(yes_fun),
            disable_logic_filter,
            online,
            create_game: None,
        })
    }
}

#[derive(StructOpt)]
struct PresetArgs {
    /// name of the preset
    ///
    /// later you can run seed -p <preset-name> to use this preset
    #[structopt(parse(from_os_str))]
    name: PathBuf,
    #[structopt(flatten)]
    settings: SeedSettings,
}

#[derive(StructOpt)]
struct WorldPresetArgs {
    /// Name of the preset
    ///
    /// Later you can run seed -p <preset-name> to use this preset
    #[structopt(parse(from_os_str))]
    name: PathBuf,
    #[structopt(flatten)]
    settings: WorldPresetSettings,
}

#[derive(StructOpt)]
struct WorldPresetSettings {
    /// Include further world presets
    ///
    /// Presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(short = "p", long)]
    includes: Option<Vec<String>>,
    /// Spawn destination
    ///
    /// Use an anchor name from the areas file, "r" / "random" for a random teleporter or "f" / "fullyrandom" for any location
    #[structopt(short, long)]
    spawn: Option<SpawnOpt>,
    /// Logically expected difficulty of execution you may be required to perform
    ///
    /// Available difficulties are "moki", "gorlek", "unsafe"
    #[structopt(short, long)]
    difficulty: Option<Difficulty>,
    /// Logically expected tricks you may have to use
    ///
    /// Available tricks are "swordsentryjump", "hammersentryjump", "shurikenbreak", "sentrybreak", "hammerbreak", "spearbreak", "sentryburn", "removekillplane", "launchswap", "sentryswap", "flashswap", "blazeswap", "wavedash", "grenadejump", "hammerjump", "swordjump", "grenaderedirect", "sentryredirect", "pausehover", "glidejump", "glidehammerjump", "spearjump"
    #[structopt(short, long)]
    tricks: Option<Vec<Trick>>,
    /// Logically assume hard in-game difficulty
    #[structopt(long)]
    hard: bool,
    /// Goal Requirements before finishing the game
    ///
    /// Available goals are trees, wisps, quests, relics. Relics can further configure the chance per area to have a relic, default is relics:60%
    #[structopt(short, long)]
    goals: Option<Vec<GoalsOpt>>,
    /// Names of headers that will be used when generating the seed
    /// 
    /// The headers will be searched as .wotwrh files in the current and /headers child directory
    #[structopt(short, long)]
    headers: Option<Vec<String>>,
    /// Configuration parameters to pass to headers
    ///
    /// Format for one parameter: <headername>.<parametername>=<value>
    #[structopt(short = "c", long = "config")]
    header_config: Option<Vec<HeaderConfigOpt>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    inline_headers: Option<Vec<InlineHeaderOpt>>,
}

impl WorldPresetSettings {
    fn into_world_preset(self) -> WorldPreset {
        let Self {
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
            includes,
            spawn: spawn.map(SpawnOpt::into_inner),
            difficulty,
            tricks,
            hard: if hard { Some(true) } else { None },
            goals: goals.map(|goals| goals.into_iter().map(GoalsOpt::into_inner).collect()),
            headers,
            header_config: header_config.map(|header_config| header_config.into_iter().map(HeaderConfigOpt::into_inner).collect()),
            inline_headers: inline_headers.map(|inline_headers| inline_headers.into_iter().map(InlineHeaderOpt::into_inner).collect()),
        }
    }
}

#[derive(StructOpt)]
struct ReachCheckArgs {
    /// the seed file for which logical reach should be checked
    #[structopt(parse(from_os_str))]
    seed_file: PathBuf,
    /// the input file representing the logic
    #[structopt(parse(from_os_str), default_value = "areas.wotw", short, long)]
    areas: PathBuf,
    /// the input file representing pickup locations
    #[structopt(parse(from_os_str), default_value = "loc_data.csv", short, long)]
    locations: PathBuf,
    /// the input file representing state namings
    #[structopt(parse(from_os_str), default_value = "state_data.csv", short, long)]
    uber_states: PathBuf,
    /// player health (one orb is 10 health)
    health: u32,
    /// player energy (one orb is 1 energy)
    energy: f32,
    /// player keystones
    keystones: u32,
    /// player ore
    ore: u32,
    /// player spirit light
    spirit_light: u32,
    /// any additional player items in the format s:<skill id>, t:<teleporter id>, sh:<shard id>, w:<world event id> or u:<ubergroup>,<uberid>
    items: Vec<String>,
}

#[derive(StructOpt)]
enum HeaderCommand {
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
    }
}

fn parse_settings(seed: Option<String>, args: SeedSettings, settings: &mut Settings) -> Result<(), Box<dyn Error>> {
    let preset = args.into_preset()?;
    settings.apply_preset(preset)?;

    if let Some(seed) = seed {
        settings.seed = seed;
    }

    Ok(())
}

fn read_stdin() -> Result<String, String> {
    // If we do not have input, skip.
    if atty::is(atty::Stream::Stdin) {
        return Ok(String::new());
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional
    let mut output = String::new();

    loop {
        let result = stdin.read_to_string(&mut output).map_err(|err| format!("failed to read standard input: {err}"))?;
        if result == 0 {
            break;
        }

        output.push('\n');
    }

    Ok(output)
}

fn write_seeds_to_files(seed: &Seed, filename: &str, mut folder: PathBuf, json_spoiler: bool) -> Result<(), String> {
    let seeds = seed.seed_files()?;
    let multiworld = seeds.len() > 1;

    if multiworld {
        let mut multi_folder = folder.clone();
        multi_folder.push(filename.clone());
        folder = util::create_folder(&multi_folder).map_err(|err| format!("Error creating seed folder: {}", err))?;
    }

    let mut first = true;
    for (index, seed) in seeds.iter().enumerate() {
        let mut path = folder.clone();
        if multiworld {
            path.push(format!("world_{}", index.to_string()));
        } else {
            path.push(filename);
        }
        path.set_extension("wotwr");

        let file = util::create_file(&path, seed, "", true)?;
        log::info!("Wrote seed for World {} to {}", index, file.display());

        if first {
            first = false;
            if let Some(path) = file.to_str() {
                fs::write(".currentseedpath", path.to_string()).unwrap_or_else(|err| log::warn!("Unable to write .currentseedpath: {}", err));
            } else {
                log::warn!("Unable to write .currentseedpath: path is not valid unicode");
            }
        }
    }

    let mut path = folder;
    path.push(format!("{filename}_spoiler"));

    let contents = match json_spoiler {
        true => {
            path.set_extension("json");
            seed.spoiler.to_json()
        },
        false => {
            path.set_extension("txt");
            seed.spoiler.to_string()
        },
    };

    let file = util::create_file(path, &contents, "", true).map_err(|err| format!("Error writing spoiler: {err}"))?;
    log::info!("Wrote spoiler to {}", file.display());

    Ok(())
}

fn write_seeds_to_stdout(seed: &Seed, json_spoiler: bool) -> Result<(), String> {
    let seeds =  seed.seed_files()?.join("\n======= END SEED =======\n");
    let spoiler =  match json_spoiler {
        true => seed.spoiler.to_json(),
        false => seed.spoiler.to_string(),
    };
    println!("{seeds}\n======= SPOILER =======\n{spoiler}");
    Ok(())
}

fn generate_seeds(args: SeedArgs) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();

    let mut settings = Settings::default();

    let stdin = read_stdin()?;
    if !stdin.is_empty() {
        let preset = serde_json::from_str(&stdin)?;
        settings.apply_preset(preset)?;
    }

    parse_settings(args.seed, args.settings, &mut settings)?;

    let areas = fs::read_to_string(&args.areas).map_err(|err| format!("Failed to read {}: {}", args.areas.display(), err))?;
    let locations = fs::read_to_string(&args.locations).map_err(|err| format!("Failed to read {}: {}", args.locations.display(), err))?;
    let states = fs::read_to_string(&args.uber_states).map_err(|err| format!("Failed to read {}: {}", args.uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &settings, !args.trust)?;
    log::info!("Parsed logic in {:?}", now.elapsed());

    let worlds = settings.world_count();
    let seed = wotw_seedgen::generate_seed(&graph, settings).map_err(|err| format!("Error generating seed: {}", err))?;
    if worlds == 1 {
        log::info!("Generated seed in {:?}", now.elapsed());
    } else {
        log::info!("Generated {} worlds in {:?}", worlds, now.elapsed());
    }

    if args.tostdout {
        write_seeds_to_stdout(&seed, args.json_spoiler)?;
    } else {
        let filename = args.filename.unwrap_or_else(|| String::from("seed"));

        write_seeds_to_files(&seed, &filename, args.seed_folder, args.json_spoiler)?;
    }

    if args.launch {
        if args.tostdout {
            log::warn!("Can't launch a seed that has been written to stdout");
        } else {
            play_last_seed()?;
        }
    }

    Ok(())
}

fn play_last_seed() -> Result<(), String> {
    let last_seed = fs::read_to_string(".currentseedpath").map_err(|err| format!("Failed to read last generated seed from .currentseedpath: {}", err))?;
    log::info!("Launching seed {}", last_seed);
    open::that(last_seed).map_err(|err| format!("Failed to launch seed: {}", err))?;
    Ok(())
}

fn create_preset(mut args: PresetArgs) -> Result<(), Box<dyn Error>> {
    let preset = args.settings.into_preset()?;
    let preset = preset.to_json();
    args.name.set_extension("json");

    let path = util::create_file(&args.name, &preset, "presets", false)?;
    log::info!("Created preset {}", path.display());

    Ok(())
}

fn create_world_preset(mut args: WorldPresetArgs) -> Result<(), Box<dyn Error>> {
    let preset = args.settings.into_world_preset();
    let preset = preset.to_json();
    args.name.set_extension("json");

    let path = util::create_file(&args.name, &preset, "presets", false)?;
    log::info!("Created preset {}", path.display());

    Ok(())
}

// TODO some of this logic probably belongs in the library
fn reach_check(mut args: ReachCheckArgs) -> Result<(), String> {
    let command = env::args().collect::<Vec<_>>().join(" ");
    log::trace!("{command}");

    args.seed_file.set_extension("wotwr");
    let contents = util::read_file(&args.seed_file, "seeds")?;

    let settings = Settings::from_seed(&contents).unwrap_or_else(|| {
        log::trace!("No settings found in seed, using default settings");
        Ok(Settings::default())
    }).map_err(|err| format!("Error reading settings: {err}"))?;

    let world_index = contents.lines().find_map(|line| line.strip_prefix("// This World: ").map(str::parse)).unwrap_or_else(|| {
        log::trace!("No current world information found in seed, using first world");
        Ok(0)
    }).map_err(|err| format!("Error reading current world: {err}"))?;

    let areas = fs::read_to_string(&args.areas).map_err(|err| format!("Failed to read {}: {}", args.areas.display(), err))?;
    let locations = fs::read_to_string(&args.locations).map_err(|err| format!("Failed to read {}: {}", args.locations.display(), err))?;
    let states = fs::read_to_string(&args.uber_states).map_err(|err| format!("Failed to read {}: {}", args.uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &settings, false)?;
    let world_settings = settings.world_settings.into_iter().nth(world_index).ok_or_else(|| "Current world index out of bounds".to_string())?;
    let mut world = World::new(&graph, &world_settings);

    world.player.inventory.grant(Item::Resource(Resource::Health), args.health / 5);
    #[allow(clippy::cast_possible_truncation)]
    world.player.inventory.grant(Item::Resource(Resource::Energy), (args.energy * 2.0) as u32);
    world.player.inventory.grant(Item::Resource(Resource::Keystone), args.keystones);
    world.player.inventory.grant(Item::Resource(Resource::Ore), args.ore);
    world.player.inventory.grant(Item::SpiritLight(1), args.spirit_light);

    for item in args.items {
        if let Some(skill) = item.strip_prefix("s:") {
            let id: u8 = skill.parse().map_err(|_| format!("expected numeric skill id in {}", item))?;
            world.player.inventory.grant(Item::Skill(Skill::try_from(id).map_err(|_| format!("{} is not a valid skill id", id))?), 1);
        }
        else if let Some(teleporter) = item.strip_prefix("t:") {
            let id: u8 = teleporter.parse().map_err(|_| format!("expected numeric teleporter id in {}", item))?;
            world.player.inventory.grant(Item::Teleporter(Teleporter::try_from(id).map_err(|_| format!("{} is not a valid teleporter id", id))?), 1);
        }
        else if let Some(shard) = item.strip_prefix("sh:") {
            let id: u8 = shard.parse().map_err(|_| format!("expected numeric shard id in {}", item))?;
            world.player.inventory.grant(Item::Shard(Shard::try_from(id).map_err(|_| format!("{} is not a valid shard id", id))?), 1);
        }
        else if let Some(world_event) = item.strip_prefix("w:") {
            let id: u8 = world_event.parse().map_err(|_| format!("expected numeric world event id in {}", item))?;
            if id != 0 { return Err(format!("{} is not a valid world event id (only 0 is)", id)); } 
            world.player.inventory.grant(Item::Water, 1);
        }
        else if let Some(uber_state) = item.strip_prefix("u:") {
            let uber_state = UberState::from_str(uber_state).map_err(|err| format!("failed to parse uber state in {}: {}", item, err))?;

            world.uber_states.insert(uber_state.identifier, uber_state.value);
        }
        else {
            return Err(format!("items have to start with s:, t:, sh:, w: or u: (for skill, teleporter, shard, world event or uber state), except found {}", item));
        }
    }

    for line in contents.lines() {
        if let Some(sets) = line.strip_prefix("// Sets: ") {
            if !sets.is_empty() {
                for identifier in sets.split(',').map(str::trim) {
                    let node = world.graph.nodes.iter().find(|&node| node.identifier() == identifier).ok_or_else(|| format!("target {} not found", identifier))?;
                    log::trace!("Setting state {}", identifier);
                    world.sets.push(node.index());
                }
            }

            break;
        }
    }

    let spawn = util::spawn_from_seed(&contents).ok_or_else(|| "Failed to read spawn location from seed".to_string())?;
    let spawn = world.graph.find_spawn(&spawn)?;

    let mut reached = world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).expect("Invalid Reach Check");
    reached.retain(|&node| node.can_place());

    let identifiers = reached.iter()
        .map(|&node| node.identifier())
        .collect::<Vec<_>>()
        .join(", ");
    log::info!("reachable locations: {}", identifiers);

    let reached = reached.into_iter()
        .filter_map(|node| node.uber_state())
        .map(|uber_state| uber_state.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    println!("{reached}");
    Ok(())
}

fn compile_seed(mut path: PathBuf) -> Result<(), String> {
    if path.extension().is_none() {
        path.set_extension("wotwrh");
    }

    let header = fs::read_to_string(path.clone()).map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;

    let mut rng = rand::thread_rng();

    let header = Header::parse(header, &mut rng)
        .map_err(|errors| (**errors).into_iter().map(|err| err.verbose_display()).collect::<Vec<_>>().join("\n"))?
        .build(FxHashMap::default())?;

    path.set_extension("wotwr");
    let path = util::create_file(path.file_name().unwrap(), &header.seed_content, "target", false)?;
    log::info!("Compiled to {}", path.display());

    Ok(())
}

fn main() -> ExitCode {
    let args = SeedGen::from_args();

    if args.wait_on_debugger {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).expect("state() not implemented on this platform");
    }

    match match args.command {
        SeedGenCommand::Seed { args } => {
            let use_file = if args.verbose { Some("generator.log") } else { None };
            initialize_log(use_file, LevelFilter::Info, args.json_stderr).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            generate_seeds(args).map_err(|err| err.to_string())
        },
        SeedGenCommand::Play => {
            initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            play_last_seed()
        },
        SeedGenCommand::Preset { args } => {
            initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            create_preset(args).map_err(|err| err.to_string())
        },
        SeedGenCommand::WorldPreset { args } => {
            initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            create_world_preset(args).map_err(|err| err.to_string())
        },
        SeedGenCommand::Headers { headers, subcommand } => {
            initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            match subcommand {
                Some(HeaderCommand::Validate { path }) => {
                   header::validate(path).map(|_| ())
                },
                Some(HeaderCommand::Parse { path }) => {
                    compile_seed(path)
                },
                None => {
                    if headers.is_empty() {
                        header::list()
                    } else {
                        header::inspect(headers)
                    }
                },
            }
        },
        SeedGenCommand::ReachCheck { args } => {
            initialize_log(Some("reach.log"), LevelFilter::Off, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            reach_check(args)
        },
    } {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        },
    }
}
