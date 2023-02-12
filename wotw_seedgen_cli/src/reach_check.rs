use super::cli;
use super::log_init;

use std::fs;
use std::env;

use log::LevelFilter;

use wotw_seedgen::{item, world::{self, graph::Node}, util, logic};
use wotw_seedgen::settings::UniverseSettings;

use item::{Item, Resource};
use world::World;

// TODO some of this logic probably belongs in the library
pub fn reach_check(mut args: cli::ReachCheckArgs) -> Result<(), String> {
    log_init::initialize_log(Some("reach.log"), LevelFilter::Off, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    let command = env::args().collect::<Vec<_>>().join(" ");
    log::trace!("{command}");

    args.seed_file.set_extension("wotwr");
    let contents = fs::read_to_string(&args.seed_file).map_err(|err| format!("Error reading seed: {err}"))?;

    let universe_settings = UniverseSettings::from_seed(&contents).unwrap_or_else(|| {
        log::trace!("No settings found in seed, using default settings");
        Ok(UniverseSettings::default())
    }).map_err(|err| format!("Error reading settings: {err}"))?;

    let world_index = contents.lines().find_map(|line| line.strip_prefix("// This World: ").map(str::parse)).unwrap_or_else(|| {
        log::trace!("No current world information found in seed, using first world");
        Ok(0)
    }).map_err(|err| format!("Error reading current world: {err}"))?;

    let areas = fs::read_to_string(&args.areas).map_err(|err| format!("Failed to read {}: {}", args.areas.display(), err))?;
    let locations = fs::read_to_string(&args.locations).map_err(|err| format!("Failed to read {}: {}", args.locations.display(), err))?;
    let states = fs::read_to_string(&args.uber_states).map_err(|err| format!("Failed to read {}: {}", args.uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &universe_settings, false)?;
    let world_settings = universe_settings.world_settings.into_iter().nth(world_index).ok_or_else(|| "Current world index out of bounds".to_string())?;
    let mut world = World::new(&graph, &world_settings);

    world.player.inventory.grant(Item::Resource(Resource::Health), args.health / 5);
    #[allow(clippy::cast_possible_truncation)]
    world.player.inventory.grant(Item::Resource(Resource::Energy), (args.energy * 2.0) as u32);
    world.player.inventory.grant(Item::Resource(Resource::Keystone), args.keystones);
    world.player.inventory.grant(Item::Resource(Resource::Ore), args.ore);
    world.player.inventory.grant(Item::SpiritLight(1), args.spirit_light);

    let mut set_node = |identifier: &str| -> Result<(), String> {
        let node = world.graph.nodes.iter().find(|&node| node.identifier() == identifier);

        if let Some(found_node) = node {
            log::trace!("Setting state {}", identifier);
            world.sets.push(found_node.index());
        } else {
            log::warn!("State {} not found", identifier);
        }

        Ok(())
    };

    for item in args.items {
        match item {
            cli::ReachData::Skill(skill) => world.player.inventory.grant(Item::Skill(skill), 1),
            cli::ReachData::Teleporter(teleporter) => world.player.inventory.grant(Item::Teleporter(teleporter), 1),
            cli::ReachData::Shard(shard) => world.player.inventory.grant(Item::Shard(shard), 1),
            cli::ReachData::Water => world.player.inventory.grant(Item::Water, 1),
            cli::ReachData::Node(identifier) => set_node(&identifier)?,
        }
    }

    for line in contents.lines() {
        if let Some(sets) = line.strip_prefix("// Sets: ") {
            if !sets.is_empty() {
                sets.split(',').map(str::trim).try_for_each(set_node)?;
            }

            break;
        }
    }

    let spawn_name = util::spawn_from_seed(&contents)?;
    let spawn = world.graph.find_spawn(&spawn_name)?;

    let mut reached = world.graph.reached_locations(&world.player, spawn, world.uber_states(), &world.sets).expect("Invalid Reach Check");
    reached.retain(|&node| node.can_place());

    let identifiers = reached.into_iter()
        .map(Node::identifier)
        .collect::<Vec<_>>()
        .join(", ");
    log::info!("reachable locations: {}", identifiers);

    println!("{identifiers}");
    Ok(())
}
