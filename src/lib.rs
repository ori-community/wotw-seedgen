mod languages;
pub mod world;
pub mod inventory;
pub mod item;
pub mod preset;
pub mod settings;
pub mod generator;
pub mod util;

pub use languages::{logic, header::{self, Header}, seed};
pub use world::World;
pub use inventory::Inventory;
pub use item::{Item, VItem};
pub use preset::Preset;
pub use settings::Settings;

use rustc_hash::{FxHashMap, FxHashSet};

use rand_seeder::Seeder;
use rand::{
    Rng,
    rngs::StdRng,
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
    graph::{Graph, Node, Pickup},
    pool::Pool
};
use generator::Placement;
use header::{HeaderBuild, ItemDetails};
use settings::{Spawn, Difficulty, Goal, WorldSettings, HeaderConfig};
use util::{
    Position, Zone, UberState,
    constants::{DEFAULT_SPAWN, MOKI_SPAWNS, GORLEK_SPAWNS, SPAWN_GRANTS, RETRIES},
};

use crate::item::UberStateOperator;

fn pick_spawn<'a, R>(graph: &'a Graph, world_settings: &WorldSettings, rng: &mut R) -> Result<&'a Node, String>
where
    R: Rng
{
    let mut valid = graph.nodes.iter().filter(|node| node.can_spawn());
    let spawn = match &world_settings.spawn {
        Spawn::Random => valid
            .filter(|&node| {
                let identifier = node.identifier();
                if world_settings.difficulty >= Difficulty::Gorlek {
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

pub fn write_flags(world_settings: &WorldSettings, mut flags: Vec<String>) -> String {
    let mut settings_flags = Vec::new();

    for flag in world_settings.goals.iter().map(Goal::flag_name) {
        settings_flags.push(flag.to_string());
    }

    if world_settings.is_random_spawn() { settings_flags.push("RandomSpawn".to_string()); }

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

fn build_config_map(header_config: &[HeaderConfig]) -> Result<FxHashMap::<String, FxHashMap<String, String>>, String> {
    let mut config_map = FxHashMap::<String, FxHashMap<_, _>>::default();

    for config in header_config {
        if let Some(prior) = config_map.entry(config.header_name.clone())
            .or_default()
            .insert(config.config_name.clone(), config.config_value.clone())
        {
            return Err(format!("provided multiple values for configuration parameter {} for header {} ({} and {})", config.config_name, config.header_name, prior, config.config_value));
        }
    }

    Ok(config_map)
}

fn parse_header(
    header_name: &String,
    headers: &mut Vec<(String, HeaderBuild)>,
    includes: &mut FxHashSet<String>,
    config_map: &mut FxHashMap::<String, FxHashMap<String, String>>,
    rng: &mut impl Rng
) -> Result<(), String> {
    log::trace!("Parsing header {header_name}");

    let header_config = config_map.remove(header_name).unwrap_or_default();

    let header = util::read_file(format!("{header_name}.wotwrh"), "headers")?;
    let header = Header::parse(header, rng)
        .map_err(|err| format!("{err} in header {header_name}"))?
        .build(header_config)?;

    for include in &header.includes {
        if includes.insert(include.clone()) {
            parse_header(include, headers, includes, config_map, rng)?;
        }
    }

    headers.push((header_name.clone(), header));

    Ok(())
}

fn block_spawn_sets(preplacement: &seed::Pickup, world: &mut World) {
    if let Item::UberState(uber_state_item) = &preplacement.item {
        if preplacement.trigger.identifier == UberState::spawn().identifier {
            if let UberStateOperator::Value(value) = &uber_state_item.operator {
                let target = UberState {
                    identifier: uber_state_item.uber_identifier.clone(),
                    value: if value == "true" { String::new() } else { value.clone() },
                };

                if world.graph.nodes.iter().any(|node| node.can_place() && node.uber_state().map_or(false, |uber_state| uber_state == &target)) {
                    log::trace!("adding an empty pickup at {uber_state_item} to prevent placements");
                    let null_item = Item::Message("6|f=0|quiet|noclear".to_string());
                    world.preplace(target, null_item);
                }
            }
        }
    }
}

fn parse_headers<R>(world: &mut World, rng: &mut R) -> Result<String, String>
where R: Rng
{
    let mut config_map = build_config_map(&world.player.settings.header_config)?;

    let mut headers = vec![];
    let mut includes = FxHashSet::default();
    includes.extend(world.player.settings.headers.iter().cloned());

    for header_name in &world.player.settings.headers {
        parse_header(header_name, &mut headers, &mut includes, &mut config_map, rng)?;
    }

    if !world.player.settings.inline_header.is_empty() {
        log::trace!("Parsing inline header");
        let inline_header = Header::parse(world.player.settings.inline_header.clone(), rng)
            .map_err(|err| format!("{err} in inline header"))?
            .build(FxHashMap::default())?;
        headers.push(("##INLINE_HEADER##".to_string(), inline_header));
    }

    let mut excludes = FxHashMap::default();
    let mut seed_contents = String::new();
    let mut flags = vec![];
    let mut state_sets = vec![];

    let header_names = headers.into_iter().map(|(header_name, mut header)| {
        for exclude in header.excludes {
            excludes.insert(exclude, header_name.clone());
        }

        seed_contents.push_str(&header.seed_content);
        seed_contents.push('\n');

        flags.append(&mut header.flags);

        for preplacement in header.preplacements {
            block_spawn_sets(&preplacement, world);
            header.item_pool_changes.entry(preplacement.item.clone()).and_modify(|prior| *prior -= 1).or_insert(-1);
            world.preplace(preplacement.trigger, preplacement.item);
        }

        for (item, amount) in header.item_pool_changes {
            if amount > 0 {
                world.pool.grant(item, amount as u32);
            } else if amount < 0 {
                world.pool.remove(&item, (-amount) as u32);
            }
        }

        for (item, details) in header.item_details {
            let display = item.to_string();
            if world.custom_items.insert(item, details).is_some() {
                return Err(format!("multiple headers tried to customize the item {display}"));
            }
        }

        state_sets.append(&mut header.state_sets);

        Ok(header_name)
    }).collect::<Result<Vec<_>, _>>()?;

    for header_name in &header_names {
        if let Some(other) = excludes.get(header_name) {
            return Err(format!("headers {other} and {header_name} are incompatible"));
        }
    }
    for header_with_parameters in config_map.keys() {
        if !header_names.iter().any(|header| header == header_with_parameters) {
            log::warn!("The header {header_with_parameters} referenced in a header argument isn't active");
        }
    }

    for flag in world.player.settings.goals.iter().map(Goal::flag_name) {
        flags.push(flag.to_string());
    }
    if world.player.settings.is_random_spawn() { flags.push("RandomSpawn".to_string()); }

    let flags = if flags.is_empty() { String::new() } else { format!("Flags: {}\n", flags.join(", ")) };
    let state_sets = if state_sets.is_empty() { String::new() } else { format!("// Sets: {}\n", state_sets.join(", ")) };
    let header_block = format!("{flags}{seed_contents}{state_sets}");

    Ok(header_block)
}

fn generate_placements<'a, R>(
    graph: &'a Graph,
    worlds: &[World<'a>],
    spawn_pickup_node: &'a Node,
    rng: &mut R
) -> Result<(Vec<Vec<Placement<'a>>>, Vec<&'a Node>), String>
where R: Rng
{
    let mut index = 0;
    loop {
        let spawn_locs = worlds.iter()
            .map(|world| pick_spawn(graph, &world.player.settings, rng))
            .collect::<Result<Vec<_>, String>>()?;
        let identifiers = spawn_locs.iter().map(|spawn_loc| spawn_loc.identifier()).collect::<Vec<_>>();
        log::trace!("Spawning on {}", identifiers.join(", "));

        match generator::generate_placements(worlds.to_vec(), &spawn_locs, spawn_pickup_node, rng) {
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
fn format_placements(world_placements: Vec<Placement>, custom_items: &FxHashMap<Item, ItemDetails>) -> String {
    let mut placement_block = String::with_capacity(world_placements.len() * 20);

    for placement in world_placements {
        let mut placement_line = format!("{}", placement);

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
        let item = custom_items.get(&placement.item)
            .and_then(|details| details.name.clone())
            .unwrap_or_else(|| placement.item.to_string());
        let item = util::with_leading_spaces(&item, 36);

        placement_line += &format!("  // {} from {}\n", item, location);
        placement_block.push_str(&placement_line);
    }

    placement_block.push('\n');

    placement_block
}

pub fn generate_seed(graph: &Graph, settings: Settings) -> Result<Vec<String>, String> {
    let slug = settings.slugify();

    let config = settings.to_json();
    log::trace!("Generating with Settings: {}", config);

    let mut rng: StdRng = Seeder::from(&settings.seed).make_rng();
    log::trace!("Seeded RNG with {}", settings.seed);

    let (worlds, world_data): (Vec<_>, Vec<_>) = settings.world_settings.into_iter().enumerate().map(|(index, world_settings)| {
        let mut world = World::new(graph, world_settings);
        world.pool = Pool::preset();

        let mut header_block = parse_headers(&mut world, &mut rng)?;
        let world_line = format!("\n// This World: {index}\n");
        header_block.push_str(&world_line);

        Ok((world, header_block))
    }).collect::<Result<Vec<_>, String>>()?.into_iter().unzip();

    let spawn_pickup_node = Node::Pickup(Pickup {
        identifier: String::from("Spawn"),
        zone: Zone::Spawn,
        index: usize::MAX,
        uber_state: UberState::spawn(),
        position: Position::default(),
    });

    let (placements, spawn_locs) = generate_placements(graph, &worlds, &spawn_pickup_node, &mut rng)?;

    let spawn_lines = spawn_locs.into_iter().map(|spawn_loc| {
        let identifier = spawn_loc.identifier();

        if identifier != DEFAULT_SPAWN {
            let mut spawn_item = String::new();
            if let Some(spawn_grant) = SPAWN_GRANTS.iter().find_map(|(spawn, item)| if *spawn == identifier { Some(item) } else { None }) {
                spawn_item = format!("{}|{}|mute\n", UberState::spawn(), spawn_grant.code());
            }

            let position = spawn_loc.position().ok_or_else(|| format!("Tried to spawn on {} which has no specified coordinates", identifier))?;
            return Ok(format!("Spawn: {position}  // {identifier}\n{spawn_item}\n\n"));
        }
        Ok(String::new())
    }).collect::<Result<Vec<_>, String>>()?;

    let target_line = "// Target: ^2.0\n";
    let version_line = format!("// Generator Version: {}\n", env!("CARGO_PKG_VERSION"));
    let slug_line = format!("// Slug: {slug}\n");
    let config_line = format!("// Config: {config}\n");

    let mut seeds = placements.into_iter()
        .zip(worlds.iter().map(|world| &world.custom_items))
        .map(|(world_placements, custom_items)| format_placements(world_placements, custom_items))
        .zip(world_data).zip(spawn_lines)
        .map(|((placement_block, header_block), spawn_line)| {
            format!("{spawn_line}{placement_block}{header_block}{target_line}{version_line}{slug_line}{config_line}")
        }).collect::<Vec<_>>();

    header::parser::postprocess(&mut seeds, graph, &worlds.iter().map(|world| &world.player.settings).collect::<Vec<_>>())?;

    Ok(seeds)
}

#[cfg(test)]
mod tests {
    use crate::preset::WorldPreset;

    use super::*;

    #[test]
    fn some_seeds() {
        // initialize_log(Some("generator.log"), LevelFilter::Trace, false).unwrap();

        let mut settings = Settings::default();
        let mut graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();

        generate_seed(&graph, settings.clone()).unwrap();

        settings.world_settings[0].difficulty = Difficulty::Unsafe;
        graph = languages::parse_logic("areas.wotw", "loc_data.csv", "state_data.csv", &settings, false).unwrap();
        generate_seed(&graph, settings.clone()).unwrap();

        settings.world_settings[0].headers = vec![
            "bingo".to_string(),
            "bonus+".to_string(),
            "glades_done".to_string(),
            "launch_fragments".to_string(),
            "launch_from_bingo".to_string(),
            "no_combat".to_string(),
            "no_ks_doors".to_string(),
            "no_quests".to_string(),
            "no_willow_hearts".to_string(),
            "open_mode".to_string(),
            "spawn_with_sword".to_string(),
            "util_twillen".to_string(),
            "vanilla_opher_upgrades".to_string(),
            "bonus_opher_upgrades".to_string(),
        ];

        for preset in ["gorlek", "rspawn"] {
            let preset = WorldPreset::read_file(preset.to_string()).unwrap();
            settings.world_settings[0].apply_world_preset(preset).unwrap();
        }

        let preset = Preset {
            world_settings: Some(vec![WorldPreset::default(); 2]),
            ..Preset::default()
        };
        settings.apply_preset(preset).unwrap();

        generate_seed(&graph, settings.clone()).unwrap();
    }
}
