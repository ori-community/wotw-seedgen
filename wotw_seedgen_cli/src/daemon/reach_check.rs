use log::trace;
use ordered_float::OrderedFloat;
use serde::Deserialize;
use wotw_seedgen::{
    data::{Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade},
    logic_language::{
        ast::Areas,
        output::{Graph, Node},
    },
    settings::{UniverseSettings, WorldSettingsHelpers},
    UberStates, World,
};
use wotw_seedgen_assets::{UberStateData, UberStateValue};

use crate::{seed::LogicFiles, Error};

pub fn relevant_uber_states(logic_files: &LogicFiles) -> Result<(), Error> {
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

    let loc_data = logic_files
        .loc_data
        .entries
        .iter()
        .map(|entry| entry.uber_identifier);

    let state_data = logic_files
        .state_data
        .entries
        .iter()
        .map(|entry| entry.uber_identifier);

    let doors = (1..=32).map(|id| UberIdentifier::new(27, id));

    let relevant_uber_states = loc_data
        .chain(state_data)
        .chain(doors)
        .chain(INVENTORY)
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&relevant_uber_states)?);

    Ok(())
}

pub fn new_world<'cache>(
    spawn_identifier: &str,
    world_index: usize,
    uber_states: &UberStates,
    graph_cache: &'cache GraphCache,
) -> Result<World<'cache, 'cache>, Error> {
    let value = graph_cache.value().unwrap();

    let spawn = value.graph.find_node(spawn_identifier)?;
    let settings = value
        .settings
        .world_settings
        .get(world_index)
        .expect("world_index in seedgen_info out of bounds");

    let mut world = World::new(&value.graph, spawn, settings, uber_states.clone());

    world.traverse_spawn(&[]);

    Ok(world)
}

pub fn reach_check(
    message: ReachCheckMessage,
    uber_state_data: &UberStateData,
    world: &mut Option<World>,
) -> Result<(), Error> {
    let world = world
        .as_mut()
        .ok_or("Received ReachCheck before SetReachCheckInfo")?;

    for (uber_identifier, value) in message.uber_states {
        let data = uber_state_data
            .id_lookup
            .get(&uber_identifier)
            .ok_or_else(|| format!("Unknown UberIdentifier {uber_identifier}"))?;
        match &data.default_value {
            UberStateValue::Boolean(_) => world.set_boolean(uber_identifier, *value > 0.5, &[]),
            UberStateValue::Integer(_) => world.set_integer(uber_identifier, (*value) as i32, &[]),
            UberStateValue::Float(_) => world.set_float(uber_identifier, value, &[]),
        }
    }

    trace!("Checking reached with {}", world.inventory_display());

    let reached = world
        .reached_nodes()
        .filter_map(|node| match node {
            Node::State(_) | Node::LogicalState(_) => None,
            _ => Some(node.identifier()),
        })
        .collect::<Vec<_>>();

    println!("{}", serde_json::to_string(&reached)?);

    Ok(())
}

#[derive(Deserialize)]
pub struct ReachCheckMessage {
    uber_states: Vec<(UberIdentifier, OrderedFloat<f32>)>,
}

pub struct GraphCache<'source, 'areas, 'logic_files> {
    areas: &'areas Areas<'source>,
    logic_files: &'logic_files LogicFiles,
    value: Option<GraphCacheValue>,
}
pub struct GraphCacheValue {
    pub settings: UniverseSettings,
    pub graph: Graph,
}

impl<'source, 'areas, 'logic_files> GraphCache<'source, 'areas, 'logic_files> {
    pub fn new(areas: &'areas Areas<'source>, logic_files: &'logic_files LogicFiles) -> Self {
        Self {
            areas,
            logic_files,
            value: None,
        }
    }

    pub fn set_settings(&mut self, universe_settings: UniverseSettings) -> Result<(), Error> {
        match &mut self.value {
            None => {
                self.value = Some(GraphCacheValue::new(
                    self.areas,
                    universe_settings,
                    self.logic_files,
                )?)
            }
            Some(value) => {
                value.update_settings(self.areas, universe_settings, self.logic_files)?
            }
        }

        Ok(())
    }

    pub fn value(&self) -> Option<&GraphCacheValue> {
        self.value.as_ref()
    }
}

impl GraphCacheValue {
    pub fn new(
        areas: &Areas,
        settings: UniverseSettings,
        logic_files: &LogicFiles,
    ) -> Result<Self, Error> {
        let graph = Graph::compile(
            areas.clone(),
            logic_files.loc_data.clone(),
            logic_files.state_data.clone(),
            &settings.world_settings,
        )
        .into_result()?;

        Ok(Self { settings, graph })
    }

    pub fn update_settings(
        &mut self,
        areas: &Areas,
        settings: UniverseSettings,
        logic_files: &LogicFiles,
    ) -> Result<(), Error> {
        if self.requires_update(&settings) {
            *self = Self::new(areas, settings, logic_files)?;
        }

        Ok(())
    }

    fn requires_update(&self, settings: &UniverseSettings) -> bool {
        settings.lowest_difficulty() < self.settings.lowest_difficulty()
            || settings.highest_difficulty() > self.settings.highest_difficulty()
            || settings
                .world_settings
                .iter()
                .flat_map(|world_settings| &world_settings.tricks)
                .any(|trick| self.settings.none_contain_trick(*trick))
    }
}
