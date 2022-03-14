use std::{
    fs,
    str::FromStr,
    path::PathBuf,
    convert::TryFrom,
    io::{self, Read},
    time::Instant,
    collections::HashMap,
    process, env, error::Error,
};

use structopt::StructOpt;
use bugsalot::debugger;

use log::LevelFilter;

use seedgen::{self, item, world, settings::{Spawn, Difficulty, Trick, Goal}, util, languages::{headers::{self, parser::HeaderContext}, self}, preset::WorldPreset, Preset, Settings};

use item::{Item, Resource, Skill, Shard, Teleporter};
use world::{
    World,
    graph::Graph,
};
use util::UberState;

/// For CLI flags that contain a mixture of world specifiers and flag values
struct WorldOpt<T> {
    source: String,
    inner: WorldOptInner<T>,
}
impl<T: FromStr> FromStr for WorldOpt<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, T::Err> {
        let inner = if let Some(world) = s.strip_prefix(':') {
            WorldOptInner::World(WorldIdentifier::from(world))
        } else {
            WorldOptInner::Opt(T::from_str(s)?)
        };
        let source = s.to_string();
        Ok(WorldOpt { source, inner })
    }
}

enum WorldOptInner<T> {
    World(WorldIdentifier),
    Opt(T),
}

enum WorldIdentifier {
    Index(usize),
    Name(String),
}
impl From<&str> for WorldIdentifier {
    fn from(s: &str) -> Self {
        s.parse::<usize>().map_or_else(
            |_| WorldIdentifier::Name(s.to_string()),
            WorldIdentifier::Index
        )
    }
}

fn resolve_world_identifier(identifier: WorldIdentifier, world_names: &[String]) -> Result<usize, String> {
    match identifier {
        WorldIdentifier::Index(index) => Ok(index),
        WorldIdentifier::Name(name) => world_names.iter().enumerate()
            .find(|(_, world_name)| &&name == world_name)
            .map(|t| t.0)
            .ok_or(format!("Unknown world name {name}")),
    }
}

fn resolve_world_opts<T: Clone>(world_opts: Vec<WorldOpt<T>>, world_names: &[String]) -> Result<Vec<Vec<T>>, String> {
    let mut world_values: Vec<Vec<T>> = world_names.iter().map(|_| vec![]).collect();
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(identifier) => current_world = Some(resolve_world_identifier(identifier, world_names)?),
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
fn resolve_nonduplicate_world_opts<T: Clone>(world_opts: Vec<WorldOpt<T>>, world_names: &[String]) -> Result<Vec<Option<T>>, String> {
    let mut world_values: Vec<Option<(T, String)>> = world_names.iter().map(|_| None).collect();
    let mut current_world = None;

    for world_opt in world_opts {
        match world_opt.inner {
            WorldOptInner::World(identifier) => current_world = Some(resolve_world_identifier(identifier, world_names)?),
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
    /// World names in multiworld
    /// 
    /// Usually the names of the players or teams playing in a world
    /// This also determines how many worlds to generate the seed with
    /// Without this flag, one world with a default name will be generated
    #[structopt(short, long)]
    world_names: Option<Vec<String>>,
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
    // TODO parse these into a struct
    #[structopt(short = "c", long)]
    header_config: Vec<WorldOpt<String>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    inline_headers: Vec<WorldOpt<String>>,
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
            world_names,
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

        let has_world_names = world_names.is_some();
        let internal_world_names = world_names.unwrap_or_else(|| vec!["World".to_string()]);

        let world_presets = resolve_world_opts(world_presets, &internal_world_names)?;
        let world_spawns = resolve_nonduplicate_world_opts(spawn, &internal_world_names)?;
        let world_difficulties = resolve_nonduplicate_world_opts(difficulty, &internal_world_names)?;
        let world_tricks = resolve_world_opts(tricks, &internal_world_names)?;
        let world_hard_flags = resolve_nonduplicate_world_opts(hard, &internal_world_names)?;
        let world_goals = resolve_world_opts(goals, &internal_world_names)?;
        let world_headers = resolve_world_opts(headers, &internal_world_names)?;
        let world_header_configs = resolve_world_opts(header_config, &internal_world_names)?;
        let world_inline_headers = resolve_world_opts(inline_headers, &internal_world_names)?;

        let disable_logic_filter = if disable_logic_filter { Some(true) } else { None };
        let online = if online { Some(true) } else { None };

        let yes_fun = internal_world_names.into_iter()
            .zip(world_presets)
            .zip(world_spawns)
            .zip(world_difficulties)
            .zip(world_tricks)
            .zip(world_hard_flags)
            .zip(world_goals)
            .zip(world_headers)
            .zip(world_header_configs)
            .zip(world_inline_headers)
            .map(|(((((((((world_name, world_presets), spawn), difficulty), tricks), hard), goals), headers), header_config), inline_headers)| {
                let world_name = if has_world_names { Some(world_name) } else { None };
                let inline_header = if inline_headers.is_empty() { None } else { Some(inline_headers.join("\n")) };

                WorldPreset {
                    includes: vec_in_option(world_presets),
                    world_name,
                    spawn: spawn.map(SpawnOpt::into_inner),
                    difficulty,
                    tricks: vec_in_option(tricks),
                    goals: vec_in_option(goals.into_iter().map(GoalsOpt::into_inner).collect()),
                    hard,
                    headers: vec_in_option(headers),
                    header_config: vec_in_option(header_config),
                    inline_header,
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
    /// World name in multiworld
    /// 
    /// Usually the name of the player or team playing in the world
    #[structopt(short, long)]
    world_name: Option<String>,
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
    #[structopt(short = "c", long)]
    header_config: Option<Vec<String>>,
    /// Inline header syntax
    #[structopt(short, long = "inline")]
    inline_headers: Option<Vec<String>>,
}

impl WorldPresetSettings {
    fn into_world_preset(self) -> WorldPreset {
        let Self {
            includes,
            world_name,
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
            world_name,
            spawn: spawn.map(SpawnOpt::into_inner),
            difficulty,
            tricks,
            hard: if hard { Some(true) } else { None },
            goals: goals.map(|goals| goals.into_iter().map(GoalsOpt::into_inner).collect()),
            headers,
            header_config,
            inline_header: inline_headers.map(|inline| inline.join("\n")),
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
    health: u16,
    /// player energy (one orb is 1 energy)
    energy: f32,
    /// player keystones
    keystones: u16,
    /// player ore
    ore: u16,
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

fn parse_settings(seed: Option<String>, settings: SeedSettings) -> Result<Settings, Box<dyn Error>> {
    let has_world_names = settings.world_names.is_some();
    let preset = settings.into_preset()?;

    let mut settings = Settings::default();
    settings.apply_preset(preset)?;

    if let Some(seed) = seed {
        settings.seed = seed;
    }
    if !has_world_names {
        settings.world_settings[0].world_name = "World".to_string();
    }

    Ok(settings)
}

fn read_header() -> Result<String, String> {
    // If we do not have input, skip.
    if atty::is(atty::Stream::Stdin) {
        return Ok(String::new());
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional
    let mut output = String::new();

    // Could also `match` on the `Result` if you wanted to handle `Err` 
    loop {
        let result = stdin.read_to_string(&mut output).map_err(|err| format!("failed to read standard input: {err}"))?;
        if result == 0 {
            break;
        }

        output.push('\n');
    }

    Ok(output)
}

fn write_seeds_to_files(seeds: &[String], mut filename: String, mut folder: PathBuf, players: &[String]) -> Result<(), String> {
    let seed_count = seeds.len();
    let multiworld = seed_count > 1;

    if multiworld {
        let mut multi_folder = folder.clone();
        multi_folder.push(filename.clone());
        folder = util::create_folder(&multi_folder).map_err(|err| format!("Error creating seed folder: {}", err))?;
    }

    let mut first = true;
    for (index, seed) in seeds.iter().enumerate() {
        let player = players.get(index).cloned().unwrap_or_else(|| format!("Player {}", index + 1));

        if multiworld {
            filename = player.clone();
        }
        let mut path = folder.clone();
        path.push(filename.clone());
        path.set_extension("wotwr");

        let file = util::create_file(&path, seed, "", true)?;
        log::info!("Wrote seed for {} to {}", player, file.display());

        if first {
            first = false;
            if let Some(path) = file.to_str() {
                fs::write(".currentseedpath", path.to_string()).unwrap_or_else(|err| log::warn!("Unable to write .currentseedpath: {}", err));
            } else {
                log::warn!("Unable to write .currentseedpath: path is not valid unicode");
            }
        }
    }

    Ok(())
}

fn write_seeds_to_stdout(seeds: Vec<String>) {
    println!("{}", seeds.join("\n======= END SEED =======\n"));
}

fn generate_seeds(mut args: SeedArgs) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();

    let header = read_header()?;
    if !header.is_empty() {
        // TODO how do these address worlds?
        args.settings.inline_headers.insert(0, WorldOpt::from_str(&header)?);
    }

    let settings = parse_settings(args.seed, args.settings)?;

    let graph = languages::parse_logic(&args.areas, &args.locations, &args.uber_states, &settings, !args.trust)?;
    log::info!("Parsed logic in {:?}", now.elapsed());

    let worlds = settings.world_count();
    let players = settings.world_settings.iter().map(|world_settings| world_settings.world_name.clone()).collect::<Vec<_>>();
    let seeds = seedgen::generate_seed(&graph, settings).map_err(|err| format!("Error generating seed: {}", err))?;
    if worlds == 1 {
        log::info!("Generated seed in {:?}", now.elapsed());
    } else {
        log::info!("Generated {} worlds in {:?}", worlds, now.elapsed());
    }

    if args.tostdout {
        write_seeds_to_stdout(seeds);
    } else {
        let filename = args.filename.unwrap_or_else(|| String::from("seed"));

        write_seeds_to_files(&seeds, filename, args.seed_folder, &players).unwrap_or_else(|err| log::error!("{}", err));
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
fn reach_check(mut args: ReachCheckArgs) -> Result<String, String> {
    let command = env::args().collect::<Vec<_>>().join(" ");
    log::trace!("{}", command);

    args.seed_file.set_extension("wotwr");
    let contents = util::read_file(&args.seed_file, "seeds")?;

    let settings = Settings::from_seed(&contents).unwrap_or_else(|| {
        log::trace!("No settings found in seed, using default settings");
        Ok(Settings::default())
    }).map_err(|err| format!("Error reading settings: {}", err))?;

    let graph = &languages::parse_logic(&args.areas, &args.locations, &args.uber_states, &settings, false)?;
    let mut world = World::new(graph);

    world.player.apply_settings(&settings);

    world.player.inventory.grant(Item::Resource(Resource::Health), args.health / 5);
    #[allow(clippy::cast_possible_truncation)]
    world.player.inventory.grant(Item::Resource(Resource::Energy), util::float_to_int(args.energy * 2.0).map_err(|_| format!("Invalid energy parameter {}", args.energy))?);
    world.player.inventory.grant(Item::Resource(Resource::Keystone), args.keystones);
    world.player.inventory.grant(Item::Resource(Resource::Ore), args.ore);
    world.player.inventory.grant(Item::SpiritLight(1), u16::try_from(args.spirit_light).unwrap_or(u16::MAX));  // Higher amounts of Spirit Light are irrelevant, just want to accept high values in case the player has that much);

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

    Ok(reached)
}

fn compile_seed(mut path: PathBuf) -> Result<(), String> {
    if path.extension().is_none() {
        path.set_extension("wotwrh");
    }

    let header = fs::read_to_string(path.clone()).map_err(|err| format!("Failed to read {}: {}", path.display(), err))?;

    let graph = Graph::default();
    let mut world = World::new(&graph);
    let settings = Settings::default();
    let mut rng = rand::thread_rng();

    let mut context = HeaderContext::default();

    let name = path.file_stem().unwrap().to_string_lossy().into_owned();
    let header_block = headers::parser::parse_header(&name, &header, &mut world, &mut context, &HashMap::default(), &mut rng)?;
    let flag_line = seedgen::write_flags(&settings, context.flags);

    let compiled = format!("{}{}", flag_line, header_block);

    path.set_extension("wotwr");
    let path = util::create_file(path.file_name().unwrap(), &compiled, "target", false)?;
    log::info!("Compiled to {}", path.display());

    Ok(())
}

fn main() {
    let args = SeedGen::from_args();

    if args.wait_on_debugger {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).expect("state() not implemented on this platform");
    }

    match args.command {
        SeedGenCommand::Seed { args } => {
            let use_file = if args.verbose { Some("generator.log") } else { None };
            seedgen::initialize_log(use_file, LevelFilter::Info, args.json_stderr).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            generate_seeds(args).unwrap_or_else(|err| {
              log::error!("{}", err);
              process::exit(2);
            });
        },
        SeedGenCommand::Play => {
            seedgen::initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            play_last_seed().unwrap_or_else(|err| log::error!("{}", err));
        },
        SeedGenCommand::Preset { args } => {
            seedgen::initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            create_preset(args).unwrap_or_else(|err| log::error!("{}", err));
        },
        SeedGenCommand::WorldPreset { args } => {
            seedgen::initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            create_world_preset(args).unwrap_or_else(|err| log::error!("{}", err));
        },
        SeedGenCommand::Headers { headers, subcommand } => {
            seedgen::initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            match subcommand {
                Some(HeaderCommand::Validate { path }) => {
                    if let Err(err) = headers::validate(path) { log::error!("{}", err) }
                },
                Some(HeaderCommand::Parse { path }) => {
                    compile_seed(path).unwrap_or_else(|err| log::error!("{}", err));
                },
                None => {
                    if headers.is_empty() {
                        headers::list().unwrap_or_else(|err| log::error!("{}", err));
                    } else {
                        headers::inspect(headers).unwrap_or_else(|err| log::error!("{}", err));
                    }
                },
            }
        },
        SeedGenCommand::ReachCheck { args } => {
            seedgen::initialize_log(Some("reach.log"), LevelFilter::Off, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

            match reach_check(args) {
                Ok(reached) => println!("{}", reached),
                Err(err) => log::error!("{}", err),
            }
        },
    }
}
