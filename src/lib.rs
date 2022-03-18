pub mod languages;
pub mod world;
pub mod inventory;
pub mod item;
pub mod settings;
pub mod generator;
pub mod util;

use std::collections::HashMap;

use rand_seeder::Seeder;
use rand::{
    Rng,
    rngs::StdRng,
    distributions::{Distribution, Uniform},
    seq::IteratorRandom
};

use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    encode::{
        Encode,
        pattern::PatternEncoder,
        json::JsonEncoder,
    },
    config::{Appender, Config, Root},
    filter::threshold::ThresholdFilter,
};

use world::{
    World,
    graph::{Graph, Node, Pickup},
    pool::Pool
};
use generator::Placement;
use languages::headers::parser::HeaderContext;
use settings::{Settings, Spawn};
use util::{
    Difficulty, Position, Zone, UberState, Icon,
    constants::{DEFAULT_SPAWN, MOKI_SPAWNS, GORLEK_SPAWNS, SPAWN_GRANTS, RETRIES},
};

use crate::languages::headers;

fn pick_spawn<'a, R>(graph: &'a Graph, settings: &Settings, rng: &mut R) -> Result<&'a Node, String>
where
    R: Rng
{
    let mut valid = graph.nodes.iter().filter(|node| node.can_spawn());
    let spawn = match &settings.spawn_loc {
        Spawn::Random => valid
            .filter(|&node| {
                let identifier = node.identifier();
                if settings.difficulty >= Difficulty::Gorlek {
                    GORLEK_SPAWNS.contains(&identifier)
                } else {
                    MOKI_SPAWNS.contains(&identifier)
                }
            })
            .choose(rng)
            .ok_or_else(|| String::from("No valid spawn locations available"))?,
        Spawn::FullyRandom => valid
            .choose(rng)
            .ok_or_else(|| String::from("No valid spawn locations available"))?,
        Spawn::Set(spawn_loc) => valid
            .find(|&node| node.identifier() == spawn_loc)
            .ok_or_else(|| format!("Spawn {} not found", spawn_loc))?
    };
    Ok(spawn)
}

pub fn write_flags(settings: &Settings, mut flags: Vec<String>) -> String {
    let mut settings_flags = Vec::new();

    for flag in settings.goalmodes.iter().map(|goal| format!("{}", goal)) {
        settings_flags.push(flag);
    }

    if matches!(settings.spawn_loc, Spawn::Random | Spawn::FullyRandom) { settings_flags.push(String::from("RandomSpawn")); }

    settings_flags.append(&mut flags);

    if settings_flags.is_empty() {
        String::default()
    } else {
        format!("Flags: {}\n", settings_flags.join(", "))
    }
}

pub fn initialize_log(use_file: Option<&str>, stderr_log_level: LevelFilter, json: bool) -> Result<(), String> {
    let encoder: Box<dyn Encode> = if json {
        Box::new(JsonEncoder::new())
    } else {
        Box::new(PatternEncoder::new("{h({l}):5}  {m}{n}"))
    };

    let stderr = ConsoleAppender::builder()
        .target(Target::Stderr)
        .encoder(encoder)
        .build();

    let log_config = if let Some(path) = use_file {
        let log_file = FileAppender::builder()
        .append(false)
        .encoder(Box::new(PatternEncoder::new("{l:5}  {m}{n}")))
        .build(path)
        .map_err(|err| format!("Failed to create log file: {}", err))?;

        Config::builder()
            .appender(Appender::builder().build("log_file", Box::new(log_file)))
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(stderr_log_level)))
                    .build("stderr", Box::new(stderr))
            )
            .build(
                Root::builder()
                    .appender("stderr")
                    .appender("log_file")
                    .build(LevelFilter::Trace)
            )
            .map_err(|err| format!("Failed to configure logger: {}", err))?
    } else {
        Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(stderr_log_level))
            .map_err(|err| format!("Failed to configure logger: {}", err))?
    };

    log4rs::init_config(log_config).map_err(|err| format!("Failed to initialize logger: {}", err))?;
    #[cfg(target_os = "windows")]
    ansi_term::enable_ansi_support().unwrap_or_else(|err| log::warn!("Failed to enable ansi support: {}", err));

    Ok(())
}

#[derive(Debug, Default)]
pub struct ItemDetails {
    name: Option<String>,
    display: Option<String>,
    price: Option<u16>,
    icon: Option<Icon>,
}

type Flags = Vec<String>;
type Sets = Vec<String>;
fn parse_headers<R>(world: &mut World, inline_headers: &[String], settings: &Settings, rng: &mut R) -> Result<(String, Flags, HashMap<String, ItemDetails>, Sets), String>
where R: Rng + ?Sized
{
    let mut header_block = String::new();

    let mut dependencies = settings.header_list.clone();
    dependencies.sort();
    for dependency in &mut dependencies {
        dependency.set_extension("wotwrh");
    }
    let mut context = HeaderContext {
        dependencies,
        ..HeaderContext::default()
    };

    let mut param_values = HashMap::new();

    for header_arg in &settings.header_args {
        let mut parts = header_arg.splitn(2, '=');
        let identifier = parts.next().unwrap();
        let mut identifier_parts = identifier.splitn(2, '.');
        let header = identifier_parts.next().unwrap();
        let identifier = identifier_parts.next().ok_or_else(|| format!("Expected <header>.<parameter> in header arg {}", header_arg))?;
        let value = parts.next().unwrap_or("true");

        let prior = param_values.entry(header).or_insert_with(HashMap::new);
        if let Some(lost) = prior.insert(identifier, value) {
            log::warn!("Overwriting duplicate header argument {}", lost);
        }
    }

    for &header_with_parameters in param_values.keys() {
        if !settings.header_list.iter().any(|header| header.to_str().map_or(false, |header| header == header_with_parameters)) {
            log::warn!("The header {} referenced in a header argument isn't active", header_with_parameters);
        }
    }

    for header in inline_headers {
        log::trace!("Parsing inline header");

        let header = headers::parser::parse_header("inline header".as_ref(), header, world, &mut context, &param_values, rng).map_err(|err| format!("{} in inline header", err))?;

        header_block += &header;
    }

    let mut parsed = Vec::new();
    while let Some(path) = context.dependencies.pop() {
        if parsed.contains(&path) {
            continue;
        }

        log::trace!("Parsing header {}", path.display());
        let header = util::read_file(&path, "headers")?;
        let header = headers::parser::parse_header(&path, &header, world, &mut context, &param_values, rng).map_err(|err| format!("{} in header {}", err, path.display()))?;

        parsed.push(path);
        header_block += &header;
    }

    for header in parsed {
        let name = header.file_stem().unwrap().to_string_lossy().to_string();

        if let Some(incompability) = context.excludes.get(&name) {
            return Err(format!("{} and {} are incompatible", header.display(), incompability));
        }
    }

    for (item, amount) in context.negative_inventory.items {
        world.pool.inventory.remove(&item, amount);
    }

    Ok((header_block, context.flags, context.custom_items, context.sets))
}

fn generate_placements<'a, R>(
    graph: &'a Graph,
    worlds: Vec<World<'a>>,
    settings: &Settings,
    spawn_pickup_node: &'a Node,
    custom_items: &HashMap<String, ItemDetails>,
    rng: &mut R
) -> Result<(Vec<Vec<Placement<'a>>>, Vec<&'a Node>), String>
where R: Rng
{
    let mut index = 0;
    loop {
        let spawn_locs = (0..settings.worlds)
            .map(|_| pick_spawn(graph, settings, rng))
            .collect::<Result<Vec<_>, String>>()?;
        let identifiers = spawn_locs.iter().map(|spawn_loc| spawn_loc.identifier()).collect::<Vec<_>>();
        log::trace!("Spawning on {}", identifiers.join(", "));

        match generator::generate_placements(worlds.clone(), &spawn_locs, spawn_pickup_node, custom_items, settings, rng) {
            Ok(seed) => {
                if index > 0 {
                    log::info!("Generated seed after {} tries{}", index + 1, if index < RETRIES / 2 { "" } else { " (phew)" });
                }
                return Ok((seed, spawn_locs));
            },
            Err(err) => log::error!("{}\nRetrying...", err),
        }

        index += 1;
        if index == RETRIES {
            return Err(format!("All {} attempts to generate a seed failed :(", RETRIES));
        }
    };
}

#[inline]
fn format_placements(world_placements: Vec<Placement>, custom_items: &HashMap<String, ItemDetails>, race: bool) -> String {
    let mut placement_block = String::with_capacity(world_placements.len() * 20);

    for placement in world_placements {
        let mut placement_line = format!("{}", placement);

        if !race {
            let location = placement.node.map_or_else(
                || placement.uber_state.to_string(),
                |node| {
                    let mut location = node.to_string();
                    util::add_trailing_spaces(&mut location, 33);
                    let mut position = format!("({})", node.position().unwrap());
                    util::add_trailing_spaces(&mut position, 15);
                    format!("{}  {} {}", location, position, node.zone().unwrap())
                }
            );

            util::add_trailing_spaces(&mut placement_line, 46);
            let item = custom_items.get(&placement.item.code())
                .and_then(|details| details.name.clone())
                .unwrap_or_else(|| placement.item.to_string());
            let item = util::with_leading_spaces(&item, 36);

            placement_line += &format!("  // {} from {}", item, location);
        }

        placement_line.push('\n');
        placement_block.push_str(&placement_line);
    }

    placement_block
}

type Seeds = Vec<String>;
type Spoilers = Vec<String>;
pub fn generate_seed(graph: &Graph, settings: Settings, inline_headers: &[String], seed: Option<String>) -> Result<(Seeds, Spoilers), String> {
    let mut settings = settings.apply_presets()?;

    let seed = seed.unwrap_or_else(|| {
        let mut generated_seed = String::new();
        let numeric = Uniform::from('0'..='9');
        let mut rng = rand::thread_rng();

        for _ in 0..16 {
            generated_seed.push(numeric.sample(&mut rng));
        }

        generated_seed
    });

    let slug = settings.slugify(&seed);

    let config = settings.write()?;
    log::trace!("Generating with Settings: {}", config);

    let mut rng: StdRng = Seeder::from(&seed).make_rng();
    log::trace!("Seeded RNG with {}", seed);

    let mut world = World::new(graph);
    world.pool = Pool::preset();
    world.player.spawn(&settings);

    let (header_block, custom_flags, custom_items, sets) = parse_headers(&mut world, inline_headers, &settings, &mut rng)?;

    let flag_line = write_flags(&settings, custom_flags);

    let mut worlds = vec![world];
    for _ in 1..settings.worlds {
        worlds.push(worlds[0].clone());
    }

    let spawn_pickup_node = Node::Pickup(Pickup {
        identifier: String::from("Spawn"),
        zone: Zone::Spawn,
        index: usize::MAX,
        uber_state: UberState::spawn(),
        position: Position::default(),
    });

    let (placements, spawn_locs) = generate_placements(graph, worlds, &settings, &spawn_pickup_node, &custom_items, &mut rng)?;

    let spawn_lines = spawn_locs.into_iter().map(|spawn_loc| {
        let identifier = spawn_loc.identifier();

        if identifier != DEFAULT_SPAWN {
            let mut spawn_item = String::new();
            if let Some(spawn_grant) = SPAWN_GRANTS.iter().find_map(|(spawn, item)| if *spawn == identifier { Some(item) } else { None }) {
                spawn_item = format!("{}|{}|mute\n", UberState::spawn(), spawn_grant.code());
            }

            let position = spawn_loc.position().ok_or_else(|| format!("Tried to spawn on {} which has no specified coordinates", identifier))?;
            return Ok(format!("Spawn: {}  // {}\n{}", position, identifier, spawn_item));
        }
        Ok(String::new())
    }).collect::<Result<Vec<_>, String>>()?;

    let spoiler_blocks = if settings.race {
        Some(placements.iter()
            .map(|world_placements| format_placements(world_placements.clone(), &custom_items, false))
            .collect::<Vec<_>>())
    } else { None };
    let placement_blocks = placements.into_iter()
        .map(|world_placements| format_placements(world_placements, &custom_items, settings.race))
        .collect::<Vec<_>>();

    let slug_line = format!("// Slug: {}", slug);
    let seed_line = format!("// Seed: {}", seed);
    let set_line = if sets.is_empty() {
        String::new()
    } else {
        format!("// Sets: {}", sets.join(", "))
    };
    let config_line = format!("// Config: {}", config);

    let mut seeds = (0..settings.worlds).map(|index| {
        format!("{}{}\n{}\n{}{}\n{}\n{}\n{}", flag_line, spawn_lines[index], placement_blocks[index], header_block, slug_line, seed_line, set_line, config_line)
    }).collect::<Vec<_>>();
    headers::parser::postprocess(&mut seeds, graph, &settings)?;

    let spoilers = spoiler_blocks.map_or_else::<Result<_, String>, _, _>(
        || Ok(Vec::new()),
        |spoiler_blocks| {
            settings.race = false;
            settings.disable_logic_filter = false;
            let spoiler_config = settings.write()?;
            let spoiler_config_line = format!("// Config: {}", spoiler_config);

            let mut spoiler_seeds = (0..settings.worlds).map(|index| {
                format!("{}{}\n{}\n{}{}\n{}\n{}\n{}", flag_line, spawn_lines[index], spoiler_blocks[index], header_block, slug_line, seed_line, set_line, spoiler_config_line)
            }).collect::<Vec<_>>();
            headers::parser::postprocess(&mut spoiler_seeds, graph, &settings)?;

            Ok(spoiler_seeds)
        })?;

    Ok((seeds, spoilers))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn some_seeds() {
        initialize_log(Some("generator.log"), LevelFilter::Off, false).unwrap();

        let mut settings = Settings::default();
        let mut graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();

        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();

        settings.hard = true;
        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();

        settings.hard = false;
        settings.difficulty = Difficulty::Unsafe;
        graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();
        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();

        settings.difficulty = Difficulty::Gorlek;
        graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();
        settings.presets = vec![PathBuf::from("gorlek")];
        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();

        settings.presets.push(PathBuf::from("rspawn"));
        settings.header_list = vec![
            PathBuf::from("bingo"),
            PathBuf::from("bonus+"),
            PathBuf::from("glades_done"),
            PathBuf::from("launch_fragments"),
            PathBuf::from("launch_from_bingo"),
            PathBuf::from("no_combat"),
            PathBuf::from("no_ks_doors"),
            PathBuf::from("no_quests"),
            PathBuf::from("no_willow_hearts"),
            PathBuf::from("open_mode"),
            PathBuf::from("spawn_with_sword"),
            PathBuf::from("util_twillen"),
            PathBuf::from("vanilla_opher_upgrades"),
            PathBuf::from("bonus_opher_upgrades"),
        ];
        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();

        settings = Settings::default();
        graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();
        settings.worlds = 5;
        generate_seed(&graph, settings.clone(), &Vec::new(), None).unwrap();
    }
}
