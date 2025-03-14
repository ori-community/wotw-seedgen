use ordered_float::OrderedFloat;
use serde::Deserialize;
use wotw_seedgen::{
    data::{uber_identifier, UberIdentifier},
    logic_language::{
        ast::Areas,
        output::{Graph, Node},
    },
    settings::{UniverseSettings, WorldSettingsHelpers},
    UberStates, World,
};
use wotw_seedgen_assets::{UberStateData, UberStateValue};
use wotw_seedgen_seed_language::output::IntermediateOutput;

use crate::{seed::LogicFiles, Error};

pub fn relevant_uber_states(logic_files: &LogicFiles) -> Result<(), Error> {
    const INVENTORY: [UberIdentifier; 81] = [
        uber_identifier::SPIRIT_LIGHT,
        uber_identifier::GORLEK_ORE,
        uber_identifier::KEYSTONES,
        uber_identifier::SHARD_SLOTS,
        uber_identifier::CLEAN_WATER,
        uber_identifier::MAX_HEALTH,
        uber_identifier::MAX_ENERGY,
        uber_identifier::skill::BASH,
        uber_identifier::skill::DOUBLE_JUMP,
        uber_identifier::skill::LAUNCH,
        uber_identifier::skill::GLIDE,
        uber_identifier::skill::WATER_BREATH,
        uber_identifier::skill::GRENADE,
        uber_identifier::skill::GRAPPLE,
        uber_identifier::skill::FLASH,
        uber_identifier::skill::SPEAR,
        uber_identifier::skill::REGENERATE,
        uber_identifier::skill::BOW,
        uber_identifier::skill::HAMMER,
        uber_identifier::skill::SWORD,
        uber_identifier::skill::BURROW,
        uber_identifier::skill::DASH,
        uber_identifier::skill::WATER_DASH,
        uber_identifier::skill::SHURIKEN,
        uber_identifier::skill::BLAZE,
        uber_identifier::skill::SENTRY,
        uber_identifier::skill::FLAP,
        uber_identifier::skill::GLADES_ANCESTRAL_LIGHT,
        uber_identifier::skill::MARSH_ANCESTRAL_LIGHT,
        uber_identifier::shard::OVERCHARGE,
        uber_identifier::shard::TRIPLE_JUMP,
        uber_identifier::shard::WINGCLIP,
        uber_identifier::shard::BOUNTY,
        uber_identifier::shard::SWAP,
        uber_identifier::shard::MAGNET,
        uber_identifier::shard::SPLINTER,
        uber_identifier::shard::RECKLESS,
        uber_identifier::shard::QUICKSHOT,
        uber_identifier::shard::RESILIENCE,
        uber_identifier::shard::VITALITY,
        uber_identifier::shard::LIFE_HARVEST,
        uber_identifier::shard::ENERGY_HARVEST,
        uber_identifier::shard::ENERGY,
        uber_identifier::shard::LIFE_PACT,
        uber_identifier::shard::LAST_STAND,
        uber_identifier::shard::ULTRA_BASH,
        uber_identifier::shard::ULTRA_GRAPPLE,
        uber_identifier::shard::OVERFLOW,
        uber_identifier::shard::THORN,
        uber_identifier::shard::CATALYST,
        uber_identifier::shard::TURMOIL,
        uber_identifier::shard::STICKY,
        uber_identifier::shard::FINESSE,
        uber_identifier::shard::SPIRIT_SURGE,
        uber_identifier::shard::LIFEFORCE,
        uber_identifier::shard::DEFLECTOR,
        uber_identifier::shard::FRACTURE,
        uber_identifier::shard::ARCING,
        uber_identifier::teleporter::MARSH,
        uber_identifier::teleporter::DEN,
        uber_identifier::teleporter::HOLLOW,
        uber_identifier::teleporter::GLADES,
        uber_identifier::teleporter::WELLSPRING,
        uber_identifier::teleporter::BURROWS,
        uber_identifier::teleporter::WOODS_ENTRANCE,
        uber_identifier::teleporter::WOODS_EXIT,
        uber_identifier::teleporter::REACH,
        uber_identifier::teleporter::DEPTHS,
        uber_identifier::teleporter::CENTRAL_POOLS,
        uber_identifier::teleporter::POOLS_BOSS,
        uber_identifier::teleporter::FEEDING_GROUNDS,
        uber_identifier::teleporter::CENTRAL_WASTES,
        uber_identifier::teleporter::OUTER_RUINS,
        uber_identifier::teleporter::INNER_RUINS,
        uber_identifier::teleporter::WILLOW,
        uber_identifier::teleporter::SHRIEK,
        uber_identifier::weapon_upgrade::EXPLODING_SPEAR,
        uber_identifier::weapon_upgrade::SHOCK_HAMMER,
        uber_identifier::weapon_upgrade::STATIC_SHURIKEN,
        uber_identifier::weapon_upgrade::CHARGE_BLAZE,
        uber_identifier::weapon_upgrade::RAPID_SENTRY,
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

    Ok(World::new_spawn(
        &value.graph,
        spawn,
        settings,
        uber_states.clone(),
    ))
}

pub fn reach_check(
    message: ReachCheckMessage,
    uber_state_data: &UberStateData,
    world: &mut Option<World>,
) -> Result<(), Error> {
    let world = world
        .as_mut()
        .ok_or("Received ReachCheck before SetReachCheckInfo")?;

    let output = IntermediateOutput::default();

    for (uber_identifier, value) in message.uber_states {
        let data = uber_state_data
            .id_lookup
            .get(&uber_identifier)
            .ok_or_else(|| format!("Unknown UberIdentifier {uber_identifier}"))?;
        match &data.default_value {
            UberStateValue::Boolean(_) => world.set_boolean(uber_identifier, *value > 0.5, &output),
            UberStateValue::Integer(_) => {
                world.set_integer(uber_identifier, (*value) as i32, &output)
            }
            UberStateValue::Float(_) => world.set_float(uber_identifier, value, &output),
        }
    }

    let reached = world
        .reached()
        .iter()
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
