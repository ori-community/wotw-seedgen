use ordered_float::OrderedFloat;
use tokio::sync::RwLockReadGuard;
use wotw_seedgen::{
    World,
    data::{
        UberIdentifier,
        assets::UberStateValue,
        seed_language::{output::CommandsOutput, simulate::Simulation},
    },
    seed::SeedgenInfo,
};

use crate::{
    assets::Cache,
    error::{Error, Result},
};

pub fn reachable(
    cache: &RwLockReadGuard<Cache>,
    current_uber_states: Vec<(UberIdentifier, OrderedFloat<f32>)>,
    seedgen_info: SeedgenInfo,
) -> Result<Vec<usize>> {
    let spawn = cache
        .graph
        .find_node(&seedgen_info.spawn_identifier)
        .map_err(Error::Custom)?;

    let settings = seedgen_info
        .universe_settings
        .world_settings
        .get(seedgen_info.world_index)
        .ok_or_else(|| "world_index in seedgen_info out of bounds".to_string())
        .map_err(Error::Custom)?;

    let uber_states = cache.uber_states.clone();

    let mut world = World::new(&cache.graph, spawn, settings, uber_states, &mut [], None);

    for (uber_identifier, value) in current_uber_states {
        let data = cache
            .base
            .uber_state_data
            .id_lookup
            .get(&uber_identifier)
            .ok_or_else(|| Error::Custom(format!("Unknown UberIdentifier {uber_identifier}")))?;

        match &data.default_value {
            UberStateValue::Boolean(_) => {
                world.store_boolean(uber_identifier, *value > 0.5, &CommandsOutput::NONE);
            }
            UberStateValue::Integer(_) => {
                world.store_integer(uber_identifier, (*value) as i32, &CommandsOutput::NONE);
            }
            UberStateValue::Float(_) => {
                world.store_float(uber_identifier, *value, &CommandsOutput::NONE);
            }
        }
    }

    world.traverse_spawn(&CommandsOutput::NONE);

    let mut reached = world
        .reached_indices()
        .filter_map(|index| cache.node_index_to_map_icon_index.get(&index))
        .copied()
        .collect::<Vec<_>>();

    reached.push(cache.grom_shop_map_icon_index);

    Ok(reached)
}
