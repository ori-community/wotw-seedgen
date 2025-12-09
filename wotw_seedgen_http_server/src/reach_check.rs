use std::hash::BuildHasher;

use ordered_float::OrderedFloat;
use rustc_hash::FxBuildHasher;
use tokio::sync::RwLockReadGuard;
use wotw_seedgen::{
    World,
    assets::{LocData, StateData, UberStateValue},
    data::{Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade},
    seed::SeedgenInfo,
    seed_language::simulate::Simulation,
};

use crate::{
    api::reach_check::{MapIcon, MapIcons, RelevantUberStates},
    assets::Cache,
    error::{Error, Result},
};

impl MapIcons {
    pub fn new(loc_data: &LocData) -> Self {
        let map_icons = loc_data
            .entries
            .iter()
            .filter_map(|entry| {
                entry.map_position.as_ref().map(|map_position| MapIcon {
                    identifier: entry.identifier.clone(),
                    kind: entry.map_icon,
                    x: map_position.x,
                    y: map_position.y,
                })
            })
            .collect();

        let hash = FxBuildHasher.hash_one(&map_icons);

        Self { map_icons, hash }
    }
}

impl RelevantUberStates {
    pub fn new(loc_data: &LocData, state_data: &StateData) -> Self {
        const INVENTORY: [UberIdentifier; 81] = [
            UberIdentifier::SPIRIT_LIGHT,
            UberIdentifier::GORLEK_ORE,
            UberIdentifier::KEYSTONES,
            UberIdentifier::SHARD_SLOTS,
            UberIdentifier::CLEAN_WATER,
            UberIdentifier::MAX_HEALTH,
            UberIdentifier::MAX_ENERGY,
            Skill::BASH_ID,
            Skill::DOUBLE_JUMP_ID,
            Skill::LAUNCH_ID,
            Skill::GLIDE_ID,
            Skill::WATER_BREATH_ID,
            Skill::GRENADE_ID,
            Skill::GRAPPLE_ID,
            Skill::FLASH_ID,
            Skill::SPEAR_ID,
            Skill::REGENERATE_ID,
            Skill::BOW_ID,
            Skill::HAMMER_ID,
            Skill::SWORD_ID,
            Skill::BURROW_ID,
            Skill::DASH_ID,
            Skill::WATER_DASH_ID,
            Skill::SHURIKEN_ID,
            Skill::BLAZE_ID,
            Skill::SENTRY_ID,
            Skill::FLAP_ID,
            Skill::GLADES_ANCESTRAL_LIGHT_ID,
            Skill::MARSH_ANCESTRAL_LIGHT_ID,
            Shard::OVERCHARGE_ID,
            Shard::TRIPLE_JUMP_ID,
            Shard::WINGCLIP_ID,
            Shard::BOUNTY_ID,
            Shard::SWAP_ID,
            Shard::MAGNET_ID,
            Shard::SPLINTER_ID,
            Shard::RECKLESS_ID,
            Shard::QUICKSHOT_ID,
            Shard::RESILIENCE_ID,
            Shard::VITALITY_ID,
            Shard::LIFE_HARVEST_ID,
            Shard::ENERGY_HARVEST_ID,
            Shard::ENERGY_ID,
            Shard::LIFE_PACT_ID,
            Shard::LAST_STAND_ID,
            Shard::ULTRA_BASH_ID,
            Shard::ULTRA_GRAPPLE_ID,
            Shard::OVERFLOW_ID,
            Shard::THORN_ID,
            Shard::CATALYST_ID,
            Shard::TURMOIL_ID,
            Shard::STICKY_ID,
            Shard::FINESSE_ID,
            Shard::SPIRIT_SURGE_ID,
            Shard::LIFEFORCE_ID,
            Shard::DEFLECTOR_ID,
            Shard::FRACTURE_ID,
            Shard::ARCING_ID,
            Teleporter::MARSH_ID,
            Teleporter::DEN_ID,
            Teleporter::HOLLOW_ID,
            Teleporter::GLADES_ID,
            Teleporter::WELLSPRING_ID,
            Teleporter::BURROWS_ID,
            Teleporter::WOODS_ENTRANCE_ID,
            Teleporter::WOODS_EXIT_ID,
            Teleporter::REACH_ID,
            Teleporter::DEPTHS_ID,
            Teleporter::CENTRAL_POOLS_ID,
            Teleporter::POOLS_BOSS_ID,
            Teleporter::FEEDING_GROUNDS_ID,
            Teleporter::CENTRAL_WASTES_ID,
            Teleporter::OUTER_RUINS_ID,
            Teleporter::INNER_RUINS_ID,
            Teleporter::WILLOW_ID,
            Teleporter::SHRIEK_ID,
            WeaponUpgrade::EXPLODING_SPEAR_ID,
            WeaponUpgrade::SHOCK_HAMMER_ID,
            WeaponUpgrade::STATIC_SHURIKEN_ID,
            WeaponUpgrade::CHARGE_BLAZE_ID,
            WeaponUpgrade::RAPID_SENTRY_ID,
        ];

        let loc_data = loc_data.entries.iter().map(|entry| entry.uber_identifier);
        let state_data = state_data.entries.iter().map(|entry| entry.uber_identifier);

        let doors = (1..=32).map(|id| UberIdentifier::new(27, id));

        let identifiers = loc_data
            .chain(state_data)
            .chain(doors)
            .chain(INVENTORY)
            .collect();

        let hash = FxBuildHasher.hash_one(&identifiers);

        Self { identifiers, hash }
    }
}

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

    let mut world = World::new(&cache.graph, spawn, settings, uber_states);

    for (uber_identifier, value) in current_uber_states {
        let data = cache
            .base
            .uber_state_data
            .id_lookup
            .get(&uber_identifier)
            .ok_or_else(|| Error::Custom(format!("Unknown UberIdentifier {uber_identifier}")))?;
        match &data.default_value {
            UberStateValue::Boolean(_) => world.store_boolean(uber_identifier, *value > 0.5, &[]),
            UberStateValue::Integer(_) => {
                world.store_integer(uber_identifier, (*value) as i32, &[])
            }
            UberStateValue::Float(_) => world.store_float(uber_identifier, *value, &[]),
        }
    }

    world.traverse_spawn(&[]);

    let reached = world
        .reached_indices()
        .filter_map(|index| cache.node_index_to_map_icon_index.get(&index))
        .copied()
        .collect();

    Ok(reached)
}
