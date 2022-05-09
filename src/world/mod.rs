pub mod graph;
pub mod pool;
pub mod player;
pub mod requirements;

pub use graph::Graph;
pub use pool::Pool;
pub use player::Player;
pub use requirements::Requirement;

use rustc_hash::FxHashMap;

use crate::header::ItemDetails;
use crate::item::{Item, Resource, UberStateOperator, UberStateRangeBoundary};
use crate::settings::WorldSettings;
use crate::util::{UberState, UberIdentifier, UberType, constants::WISP_STATES};

#[derive(Debug, Clone)]
pub struct World<'a> {
    pub graph: &'a Graph,
    pub player: Player,
    pub pool: Pool,
    pub preplacements: FxHashMap<UberState, Vec<Item>>,
    pub uber_states: FxHashMap<UberIdentifier, String>,
    pub sets: Vec<usize>,
    pub custom_items: FxHashMap<Item, ItemDetails>,
}
impl<'a> World<'a> {
    pub fn new(graph: &Graph, settings: WorldSettings) -> World {
        World {
            graph,
            player: Player::spawn(settings),
            pool: Pool::default(),
            preplacements: FxHashMap::default(),
            uber_states: FxHashMap::default(),
            sets: Vec::default(),
            custom_items: FxHashMap::default(),
        }
    }

    pub fn grant_player(&mut self, item: Item, amount: u32) -> Result<(), String> {
        match item {
            Item::UberState(command) => {
                for _ in 0..amount {
                    let default = String::from("0");
                    let uber_value = match &command.operator {
                        UberStateOperator::Value(value) => value,
                        UberStateOperator::Pointer(uber_identifier) => self.uber_states.get(uber_identifier).unwrap_or(&default),
                        UberStateOperator::Range(range) => match &range.start {
                            UberStateRangeBoundary::Value(value) => value,
                            UberStateRangeBoundary::Pointer(uber_identifier) => self.uber_states.get(uber_identifier).unwrap_or(&default),
                        },
                    }.to_owned();

                    let entry = self.uber_states.entry(command.uber_identifier.to_owned()).or_insert_with(|| String::from("0"));
                    let uber_value = match command.uber_type {
                        UberType::Bool | UberType::Teleporter => uber_value.to_string(),
                        UberType::Byte | UberType::Int => {
                            if command.signed {
                                let uber_value = uber_value.parse::<i32>().unwrap();
                                let mut prior = entry.parse::<i32>().map_err(|_| format!("Failed to apply uberState command {} because the current state ({}) doesn't match the specified type", command.code(), entry))?;

                                if command.sign {
                                    prior += uber_value;
                                } else {
                                    prior -= uber_value;
                                }
                                prior.to_string()
                            } else {
                                uber_value.to_string()
                            }
                        },
                        UberType::Float => {
                            if command.signed {
                                let uber_value = uber_value.parse::<f32>().unwrap();
                                let mut prior = entry.parse::<f32>().map_err(|_| format!("Failed to apply uberState command {} because the current state ({}) doesn't match the specified type", command.code(), entry))?;

                                if command.sign {
                                    prior += uber_value;
                                } else {
                                    prior -= uber_value;
                                }
                                prior.to_string()
                            } else {
                                uber_value.to_string()
                            }
                        },
                    };
                    if uber_value == "false" || uber_value == "0" || uber_value == *entry { return Ok(()); }

                    *entry = uber_value;

                    let uber_state = UberState {
                        identifier: command.uber_identifier.to_owned(),
                        value: entry.clone(),
                    };

                    if command.skip {
                        log::trace!("Skipped granting UberState {}", uber_state);
                        return Ok(());
                    }

                    log::trace!("Granting player UberState {}", uber_state);
                    self.collect_preplacements(&uber_state);
                    let without_value = UberState {
                        value: String::new(),
                        ..uber_state
                    };
                    self.collect_preplacements(&without_value);
                }
            },
            Item::SpiritLight(stacked_amount) => {
                log::trace!("Granting player {} Spirit Light", amount * stacked_amount);

                self.player.inventory.grant(Item::SpiritLight(1), amount * stacked_amount);
            }
            item => {
                if item.is_progression(self.player.settings.difficulty) {
                    log::trace!("Granting player {}{}", if amount == 1 { String::new() } else { format!("{} ", amount) }, item);

                    self.player.inventory.grant(item, amount);
                }
            },
        }

        Ok(())
    }

    pub fn preplace(&mut self, uber_state: UberState, item: Item) {
        self.preplacements.entry(uber_state).or_default().push(item);
    }
    pub fn collect_preplacements(&mut self, reached: &UberState) -> bool {
        if let Some(items) = self.preplacements.get(reached) {
            log::trace!("Collecting preplacements on {}", reached);
            let items = items.clone();

            for item in items {
                self.grant_player(item, 1).unwrap_or_else(|err| log::error!("{}", err));
            }

            true
        } else if WISP_STATES.contains(&reached.identifier) {
            log::trace!("Granting player Wisp");
            self.player.inventory.grant(Item::Resource(Resource::Health), 2);
            self.player.inventory.grant(Item::Resource(Resource::Energy), 2);

            false
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::languages::logic;

    use super::*;
    use super::super::*;
    use world::pool::Pool;
    use item::*;
    use util::*;
    use rustc_hash::FxHashSet;

    #[test]
    fn reach_check() {
        let mut settings = Settings::default();
        settings.world_settings[0].difficulty = Difficulty::Gorlek;

        let areas = util::read_file("areas.wotw", "logic").unwrap();
        let locations = util::read_file("loc_data.csv", "logic").unwrap();
        let states = util::read_file("state_data.csv", "logic").unwrap();
        let graph = logic::parse_logic(&areas, &locations, &states, &settings, false).unwrap();
        let mut world = World::new(&graph, settings.world_settings[0].clone());
        world.player.inventory = Pool::preset().inventory;
        world.player.inventory.grant(Item::SpiritLight(1), 10000);

        let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
        let reached = world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).unwrap();
        let reached: FxHashSet<_> = reached.iter()
            .filter_map(|node| {
                if node.node_kind() == NodeKind::State { None }
                else { node.uber_state() }
            })
            .cloned().collect();

        let input = util::read_file("loc_data.csv", "logic").unwrap();
        let all_locations = logic::parse_locations(&input).unwrap();
        let all_locations: FxHashSet<_> = all_locations.iter().map(|location| &location.uber_state).cloned().collect();

        if !(reached == all_locations) {
            let diff: Vec<_> = all_locations.difference(&reached).collect();
            eprintln!("difference ({} / {} items): {:?}", reached.len(), all_locations.len(), diff);
        }

        assert_eq!(reached, all_locations);

        let mut settings = Settings::default();
        settings.world_settings[0].difficulty = Difficulty::Gorlek;

        let graph = logic::parse_logic(&areas, &locations, &states, &settings, false).unwrap();
        let mut world = World::new(&graph, settings.world_settings[0].clone());

        world.player.settings.difficulty = Difficulty::Unsafe;
        world.player.inventory.grant(Item::Resource(Resource::Health), 7);
        world.player.inventory.grant(Item::Resource(Resource::Energy), 6);
        world.player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        world.player.inventory.grant(Item::Shard(Shard::TripleJump), 1);

        let spawn = world.graph.find_spawn("GladesTown.Teleporter").unwrap();
        let reached = world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).unwrap();
        let reached: Vec<_> = reached.iter().map(|node| node.identifier()).collect();
        assert_eq!(reached, vec!["GladesTown.UpdraftCeilingEX", "GladesTown.AboveTpEX", "GladesTown.BountyShard", "GladesTown.BelowHoleHutEX"]);
    }
}
