use super::cli;
use super::log_init;

use std::fs;
use std::env;

use log::LevelFilter;

use wotw_seedgen::Inventory;
use wotw_seedgen::{item, world::{self, graph::Node}, util, logic};
use wotw_seedgen::settings::UniverseSettings;

use item::{Item, Resource};
use world::World;

pub fn reach_check(mut args: cli::ReachCheckArgs) -> Result<(), String> {
    log_init::initialize_log(Some("reach.log"), LevelFilter::Off, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    let command = env::args().collect::<Vec<_>>().join(" ");
    log::trace!("{command}");

    args.seed_file.set_extension("wotwr");
    let contents = fs::read_to_string(&args.seed_file).map_err(|err| format!("Error reading seed: {err}"))?;

    let universe_settings = UniverseSettings::from_seed(&contents)
        .unwrap_or_else(|| Err("No settings found in seed".into())).map_err(|err| format!("Error reading settings: {err}"))?;

    let areas = fs::read_to_string(&args.areas).map_err(|err| format!("Failed to read {}: {}", args.areas.display(), err))?;
    let locations = fs::read_to_string(&args.locations).map_err(|err| format!("Failed to read {}: {}", args.locations.display(), err))?;
    let states = fs::read_to_string(&args.uber_states).map_err(|err| format!("Failed to read {}: {}", args.uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &universe_settings, false)?;

    let mut inventory = Inventory::default();
    inventory.grant(Item::Resource(Resource::Health), args.health / 5);
    #[allow(clippy::cast_possible_truncation)]
    inventory.grant(Item::Resource(Resource::Energy), (args.energy * 2.0) as u32);
    inventory.grant(Item::Resource(Resource::Keystone), args.keystones);
    inventory.grant(Item::Resource(Resource::Ore), args.ore);
    inventory.grant(Item::SpiritLight(1), args.spirit_light);

    let mut nodes = vec![];
    for item in args.items {
        match item {
            cli::ReachData::Skill(skill) => inventory.grant(Item::Skill(skill), 1),
            cli::ReachData::Teleporter(teleporter) => inventory.grant(Item::Teleporter(teleporter), 1),
            cli::ReachData::Shard(shard) => inventory.grant(Item::Shard(shard), 1),
            cli::ReachData::Water => inventory.grant(Item::Water, 1),
            cli::ReachData::Node(identifier) => nodes.push(identifier),
        }
    }

    let reached = wotw_seedgen::reach_check(inventory, &graph, &contents, &nodes)?;

    let identifiers = reached.into_iter()
        .map(Node::identifier)
        .collect::<Vec<_>>()
        .join(", ");
    log::info!("reachable locations: {}", identifiers);

    println!("{identifiers}");
    Ok(())
}
