use std::{
    fs,
    str::FromStr,
    path::PathBuf,
    convert::TryFrom,
    io::{self, Read},
    time::Instant,
    collections::HashMap,
    process,
};

use structopt::StructOpt;
use bugsalot::debugger;

use log::LevelFilter;

use seedgen::{self, lexer, item, world, headers, settings, util};

use item::{Item, Resource, Skill, Shard, Teleporter};
use world::{
    World,
    graph::Graph,
};
use headers::parser::HeaderContext;
use settings::{Settings, Spawn};
use util::{Difficulty, Glitch, GoalMode, UberState};

#[derive(StructOpt, Debug)]
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

#[derive(StructOpt, Debug)]
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

#[derive(StructOpt, Debug)]
struct SeedArgs {
    /// the seed's name and name of the file it will be written to. The name also seeds the rng.
    #[structopt()]
    filename: Option<String>,
    /// which folder to write the seed into
    #[structopt(parse(from_os_str), default_value = "seeds", long = "seeddir")]
    seed_folder: PathBuf,
    /// seed the rng; without this flag it will be seeded from the filename instead
    #[structopt(long)]
    seed: Option<String>,
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
    #[structopt(short, long)]
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
    #[structopt(flatten)]
    settings: SeedSettings,
    /// inline headers
    #[structopt(short, long = "inline")]
    inline_headers: Vec<String>
}

#[derive(StructOpt, Debug)]
struct PresetArgs {
    /// name of the preset
    ///
    /// later you can run seed -p <preset-name> to use this preset
    #[structopt(parse(from_os_str))]
    name: PathBuf,
    #[structopt(flatten)]
    settings: SeedSettings,
}

#[derive(StructOpt, Debug)]
struct SeedSettings {
    /// derive the settings from one or more presets
    ///
    /// presets later in the list override earlier ones, and flags from the command override any preset
    #[structopt(parse(from_os_str), short, long)]
    preset: Vec<PathBuf>,
    /// How many (multi)worlds to generate
    #[structopt(short, long, default_value = "1")]
    worlds: usize,
    /// Player names in multiworld
    #[structopt(short, long)]
    names: Vec<String>,
    /// difficulty of execution you may be required to perform
    ///
    /// one of moki, gorlek, unsafe
    #[structopt(short, long, default_value = "moki")]
    difficulty: String,
    /// glitches you may be required to use
    ///
    /// glitches are shurikenbreak, sentrybreak, hammerbreak, spearbreak, swordsjump, hammersjump, sentryburn, removekillplane, launchswap, sentryswap, flashswap, blazeswap, wavedash, grenadejump, hammerjump, swordjump, grenaderedirect, sentryredirect, pausehover
    #[structopt(short = "G", long)]
    glitches: Vec<String>,
    /// which goal modes to use
    ///
    /// goal modes are trees, wisps, quests, relics. Relics can further configure the chance per area to have a relic, default is relics:60%
    #[structopt(short, long)]
    goals: Vec<String>,
    /// where to spawn the player
    ///
    /// Use an anchor name from the areas file, "r" / "random" for a random teleporter or "f" / "fullyrandom" for any location
    #[structopt(short, long, default_value = "MarshSpawn.Main")]
    spawn: String,
    /// hides spoilers
    #[structopt(short, long)]
    race: bool,
    /// prevent using the in-game logic map
    #[structopt(short = "L", long)]
    disable_logic_filter: bool,
    /// required for coop and bingo
    #[structopt(short, long)]
    multiplayer: bool,
    /// play this seed on hard (in-game) difficulty
    #[structopt(long)]
    hard: bool,
    /// paths to headers stored in files which will be added to the seed
    #[structopt(parse(from_os_str), short, long = "headers")]
    header_paths: Vec<PathBuf>,
    /// configuration parameters for headers
    ///
    /// format for one parameter: <headername>.<parametername>=<value>
    #[structopt(short = "a", long = "args")]
    header_args: Vec<String>,
}

#[derive(StructOpt, Debug)]
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

#[derive(StructOpt, Debug)]
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

fn read_header() -> String {
    // If we do not have input, skip.
    if atty::is(atty::Stream::Stdin) {
        return String::new();
    }

    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional
    let mut output = String::new();

    // Could also `match` on the `Result` if you wanted to handle `Err` 
    loop {
        let result = stdin.read_to_string(&mut output).expect("failed to read standard input");
        if result == 0 {
            break;
        }

        output += "\n";
    }

    output
}

fn parse_difficulty(difficulty: &str) -> Result<Difficulty, String> {
    match &difficulty.to_lowercase()[..] {
        "moki" => Ok(Difficulty::Moki),
        "gorlek" => Ok(Difficulty::Gorlek),
        "kii" => Ok(Difficulty::Kii),
        "unsafe" => Ok(Difficulty::Unsafe),
        _ => Err(format!("Unknown difficulty {}", difficulty)),
    }
}

fn parse_glitches(names: &[String]) -> Vec<Glitch> {
    let mut glitches = Vec::default();

    for glitch in names {
        match &glitch.to_lowercase()[..] {
            "shurikenbreak" => glitches.push(Glitch::ShurikenBreak),
            "sentrybreak" => glitches.push(Glitch::SentryBreak),
            "hammerbreak" => glitches.push(Glitch::HammerBreak),
            "spearbreak" => glitches.push(Glitch::SpearBreak),
            "swordsjump" | "swordsentryjump" => glitches.push(Glitch::SwordSentryJump),
            "hammersjump" | "hammersentryjump" => glitches.push(Glitch::HammerSentryJump),
            "sentryburn" => glitches.push(Glitch::SentryBurn),
            "removekillplane" => glitches.push(Glitch::RemoveKillPlane),
            "launchswap" => glitches.push(Glitch::LaunchSwap),
            "sentryswap" => glitches.push(Glitch::SentrySwap),
            "flashswap" => glitches.push(Glitch::FlashSwap),
            "blazeswap" => glitches.push(Glitch::BlazeSwap),
            "wavedash" => glitches.push(Glitch::WaveDash),
            "grenadejump" => glitches.push(Glitch::GrenadeJump),
            "hammerjump" => glitches.push(Glitch::HammerJump),
            "swordjump" => glitches.push(Glitch::SwordJump),
            "grenaderedirect" => glitches.push(Glitch::GrenadeRedirect),
            "sentryredirect" => glitches.push(Glitch::SentryRedirect),
            "pausehover" => glitches.push(Glitch::PauseHover),
            "glidejump" => glitches.push(Glitch::GlideJump),
            "glidehammerjump" => glitches.push(Glitch::GlideHammerJump),
            "spearjump" => glitches.push(Glitch::SpearJump),
            other => log::warn!("Unknown glitch {}", other),
        }
    }

    glitches
}
fn parse_goalmodes(names: &[String]) -> Result<Vec<GoalMode>, String> {
    let mut goalmodes = Vec::new();

    for goalmode in names {
        let mut parts = goalmode.split(':');
        let identifier = parts.next().unwrap();

        match identifier {
            "t" | "trees" => { goalmodes.push(GoalMode::Trees); },
            "w" | "wisps" => { goalmodes.push(GoalMode::Wisps); },
            "q" | "quests" => { goalmodes.push(GoalMode::Quests); },
            "r" | "relics" => {
                let goal = if let Some(details) = parts.next() {
                    if let Some(chance) = details.strip_suffix('%') {
                        let chance = chance.parse::<f64>().map_err(|_| format!("Invalid chance in details string for goal mode {}", goalmode))?;
                        if !(0.0..=100.0).contains(&chance) { return Err(format!("Invalid chance in details string for goal mode {}", goalmode)); }
                        GoalMode::RelicChance(chance / 100.0)
                    } else {
                        let amount = details.parse().map_err(|_| format!("expected amount or % expression in details string for goal mode {}", goalmode))?;
                        if !(0..=11).contains(&amount) { return Err(format!("Invalid amount in details string for goal mode {}", goalmode)); }
                        GoalMode::Relics(amount)
                    }
                } else { GoalMode::RelicChance(0.6) };

                goalmodes.push(goal);
            },
            other => log::warn!("Unknown goal mode {}", other),
        }

        if parts.next().is_some() {
            return Err(format!("Unexpected details string in goal mode {}", goalmode));
        }
    }

    if !goalmodes.is_empty() {
        goalmodes.sort_unstable_by_key(GoalMode::to_string);

        let mut key = goalmodes[0].to_string();
        for index in 1..goalmodes.len() {
            let a = key;
            key = goalmodes[index].to_string();

            if a == key {
                return Err(format!("Duplicate goalmode {}", key));
            }
        }
    }

    Ok(goalmodes)
}
fn parse_spawn(spawn: String) -> Spawn {
    match &spawn.to_lowercase()[..] {
        "r" | "random" => Spawn::Random,
        "f" | "fullyrandom" => Spawn::FullyRandom,
        _ => Spawn::Set(spawn),
    }
}
fn parse_settings(settings: SeedSettings) -> Result<Settings, String> {
    let SeedSettings {
        preset,
        worlds,
        names,
        difficulty,
        glitches,
        race,
        disable_logic_filter,
        mut multiplayer,
        hard,
        spawn,
        goals,
        header_paths,
        header_args,
    } = settings;

    let difficulty = parse_difficulty(&difficulty)?;
    let glitches = parse_glitches(&glitches);
    let goalmodes = parse_goalmodes(&goals)?;
    let spawn = parse_spawn(spawn);

    if worlds == 0 {
        return Err(String::from("Tried to create a seed with zero worlds"));
    } else if worlds > 1 {
        multiplayer = true;
    }

    Ok(Settings {
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        presets: preset,
        worlds,
        players: names,
        difficulty,
        glitches,
        race,
        disable_logic_filter,
        goalmodes,
        web_conn: multiplayer,
        spawn_loc: spawn,
        hard,
        header_list: header_paths,
        header_args,
    })
}

fn write_seeds_to_files(seeds: &[String], spoilers: &[String], mut filename: String, mut folder: PathBuf, players: &[String], race: bool) -> Result<(), String> {
    let seed_count = seeds.len();
    let multiworld = seed_count > 1;

    if multiworld {
        let mut multi_folder = folder.clone();
        multi_folder.push(filename.clone());
        folder = util::create_folder(&multi_folder).map_err(|err| format!("Error creating seed folder: {}", err))?;
    }

    let mut first = true;
    for index in 0..seed_count {
        let seed = &seeds[index];
        let player = players.get(index).cloned().unwrap_or_else(|| format!("Player {}", index + 1));

        if multiworld {
            filename = player.clone();
        }
        let mut path = folder.clone();
        path.push(filename.clone());
        path.set_extension("wotwr");

        let file = util::create_file(&path, seed, "", true)?;
        log::info!("Wrote seed for {} to {}", player, file.display());

        if race {
            let spoiler = &spoilers[index];

            let spoiler_filename = format!("{}_spoiler", file.with_extension("").file_name().unwrap().to_string_lossy());
            path.set_file_name(spoiler_filename);
            path.set_extension("wotwr");

            let file = util::create_file(&path, spoiler, "", true)?;
            log::info!("Wrote spoiler for {} to {}", player, file.display());
        }

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

fn generate_seeds(mut args: SeedArgs) -> Result<(), String> {
    let now = Instant::now();

    let seed = args.seed.as_ref().map_or_else(
        || args.filename.as_ref(),
        |seed| Some(seed),
    ).cloned();

    let settings = parse_settings(args.settings)?.apply_presets()?;

    let graph = lexer::parse_logic(&args.areas, &args.locations, &args.uber_states, &settings, !args.trust)?;
    log::info!("Parsed logic in {:?}", now.elapsed());

    let header = read_header();
    if !header.is_empty() {
        args.inline_headers.push(header)
    }

    let worlds = settings.worlds;
    let race = settings.race;
    let players = settings.players.clone();
    let (seeds, spoilers) = seedgen::generate_seed(&graph, settings, &args.inline_headers, seed).map_err(|err| format!("Error generating seed: {}", err))?;
    if worlds == 1 {
        log::info!("Generated seed in {:?}", now.elapsed());
    } else {
        log::info!("Generated {} worlds in {:?}", worlds, now.elapsed());
    }

    if args.tostdout {
        write_seeds_to_stdout(seeds);
        if race {
            println!("\n======= SPOILERS =======\n");
            write_seeds_to_stdout(spoilers);
        }
    } else {
        let filename = args.filename.unwrap_or_else(|| String::from("seed"));

        write_seeds_to_files(&seeds, &spoilers, filename, args.seed_folder, &players, race).unwrap_or_else(|err| log::error!("{}", err));
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

fn create_preset(mut args: PresetArgs) -> Result<(), String> {
    let settings = parse_settings(args.settings)?;
    let settings = settings.write()?;

    args.name.set_extension("json");

    let path = util::create_file(&args.name, &settings, "presets", false)?;
    log::info!("Created preset {}", path.display());

    Ok(())
}

fn reach_check(mut args: ReachCheckArgs) -> Result<String, String> {
    let command = format!("reach-check {} --areas {} --locations {} --uber-states {} {} {} {} {} {} {}",
        args.seed_file.display(),
        args.areas.display(),
        args.locations.display(),
        args.uber_states.display(),
        args.health,
        args.energy,
        args.keystones,
        args.ore,
        args.spirit_light,
        args.items.join(" "),
    );
    log::trace!("{}", command);

    args.seed_file.set_extension("wotwr");
    let contents = util::read_file(&args.seed_file, "seeds")?;

    let settings = Settings::from_seed(&contents)?;
    let graph = &lexer::parse_logic(&args.areas, &args.locations, &args.uber_states, &settings, false)?;
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

    let spawn = settings::read_spawn(&contents)?;
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

    let header_block = headers::parser::parse_header(&path, &header, &mut world, &mut context, &HashMap::default(), &mut rng)?;
    let flag_line = seedgen::write_flags(&settings, context.flags);

    let compiled = format!("{}{}", flag_line, header_block);

    path.set_extension("wotwr");
    let path = util::create_file(&PathBuf::from(path.file_name().unwrap()), &compiled, "target", false)?;
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
