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
use crate::item::{Item, Resource};
use crate::settings::{WorldSettings, Goal};
use crate::util::constants::WISP_STATES;
use crate::uber_state::{UberStateTrigger, UberIdentifier};

#[derive(Debug, Clone)]
pub struct World<'graph, 'settings> {
    pub graph: &'graph Graph,
    pub player: Player<'settings>,
    pub pool: Pool,
    pub preplacements: FxHashMap<UberStateTrigger, Vec<Item>>,
    uber_states: FxHashMap<UberIdentifier, f32>,
    pub sets: Vec<usize>,
    pub custom_items: FxHashMap<Item, ItemDetails>,
    pub goals: Vec<Goal>,
}
impl World<'_, '_> {
    /// Creates a new world with the given [`Graph`] and [`WorldSettings`]
    /// 
    /// Note that the player will start with an empty inventory, use [`new_spawn`] if you want the player to start with the vanilla inventory of 3 energy and 30 health.
    pub fn new<'a, 'b>(graph: &'a Graph, settings: &'b WorldSettings) -> World<'a, 'b> {
        World {
            graph,
            player: Player::new(settings),
            pool: Pool::default(),
            preplacements: FxHashMap::default(),
            uber_states: FxHashMap::default(),
            sets: Vec::default(),
            custom_items: FxHashMap::default(),
            goals: Vec::default(),
        }
    }
    /// Creates a new world with the given [`Graph`] and [`WorldSettings`]
    /// 
    /// Note that the player will start with the vanilla inventory of 3 energy and 30 health, use [`new`] if you want the player to start with an empty inventory.
    pub fn new_spawn<'a, 'b>(graph: &'a Graph, settings: &'b WorldSettings) -> World<'a, 'b> {
        World {
            player: Player::spawn(settings),
            ..World::new(graph, settings)
        }
    }

    pub fn grant_player(&mut self, item: Item, amount: u32) -> Result<(), String> {
        match item {
            Item::UberState(command) => {
                for _ in 0..amount {
                    let new = command.do_the_math(&self.uber_states);
                    let old = self.uber_states.insert(command.identifier, new);
                    if !command.skip {
                        self.collect_preplacements(command.identifier, old.unwrap_or_default());
                    }
                }
            },
            Item::SpiritLight(stacked_amount) => {
                log::trace!("Granting player {} Spirit Light", amount * stacked_amount);

                self.player.inventory.grant(Item::SpiritLight(1), amount * stacked_amount);
            }
            item => {
                let triggered_state = item.attached_state();
                if item.is_progression(self.player.settings.difficulty) {
                    log::trace!("Granting player {}{}", if amount == 1 { String::new() } else { format!("{} ", amount) }, item);

                    self.player.inventory.grant(item, amount);
                }
                if let Some(identifier) = triggered_state {
                    self.set_uber_state(identifier, 1.);
                }
            },
        }

        Ok(())
    }

    pub(crate) fn preplace(&mut self, uber_state: UberStateTrigger, item: Item) {
        self.preplacements.entry(uber_state).or_default().push(item);
    }
    fn collect_preplacements(&mut self, identifier: UberIdentifier, old: f32) -> bool {
        let new = self.uber_states.get(&identifier).copied().unwrap_or_default();
        if new == old { return false }
        if WISP_STATES.contains(&identifier) {
            log::trace!("Granting player Wisp");
            self.player.inventory.grant(Item::Resource(Resource::Health), 2);
            self.player.inventory.grant(Item::Resource(Resource::Energy), 2);
        }

        let mut preplaced = false;
        let collected = self.preplacements.iter().filter_map(|(trigger, items)|
            if trigger.check(identifier, new) && (trigger.condition.is_none() || !trigger.check_value(old)) {
                preplaced = true;
                Some(items)
            } else { None }
        ).flatten().cloned().collect::<Vec<_>>();
        for item in collected {
            self.grant_player(item, 1).unwrap_or_else(|err| log::error!("{}", err));
        }

        preplaced
    }

    /// Sets the value at an [`UberIdentifier`] and collects any items preplaced on it
    /// 
    /// Returns `true` if any preplaced items were collected
    pub fn set_uber_state(&mut self, identifier: UberIdentifier, value: f32) -> bool {
        let old = self.uber_states.insert(identifier, value).unwrap_or_default();
        self.collect_preplacements(identifier, old)
    }
    /// Returns the value at an [`UberIdentifier`]
    pub fn get_uber_state(&self, identifier: UberIdentifier) -> f32 {
        self.uber_states.get(&identifier).copied().unwrap_or_default()
    }
    /// Returns the entire uber state map of this world
    pub fn uber_states(&self) -> &FxHashMap<UberIdentifier, f32> { &self.uber_states }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::super::*;
    use world::pool::Pool;
    use item::*;
    use util::*;
    use rustc_hash::FxHashSet;
    use languages::logic;
    use settings::*;

    #[test]
    fn reach_check() {
        let mut universe_settings = UniverseSettings::default();
        universe_settings.world_settings[0].difficulty = Difficulty::Gorlek;

        let areas = files::read_file("areas", "wotw", "logic").unwrap();
        let locations = files::read_file("loc_data", "csv", "logic").unwrap();
        let states = files::read_file("state_data", "csv", "logic").unwrap();
        let graph = logic::parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();
        let mut world = World::new(&graph, &universe_settings.world_settings[0]);
        world.player.inventory = Pool::preset().inventory;
        world.player.inventory.grant(Item::SpiritLight(1), 10000);

        let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
        let reached = world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).unwrap();
        let reached: FxHashSet<_> = reached.iter()
            .filter_map(|node| {
                if node.node_kind() == NodeKind::State { None }
                else { node.trigger() }
            })
            .cloned().collect();

        let input = files::read_file("loc_data", "csv", "logic").unwrap();
        let all_locations = logic::parse_locations(&input).unwrap();
        let all_locations: FxHashSet<_> = all_locations.iter().map(|location| &location.trigger).cloned().collect();

        if !(reached == all_locations) {
            let diff: Vec<_> = all_locations.difference(&reached).collect();
            eprintln!("difference ({} / {} items): {:?}", reached.len(), all_locations.len(), diff);
        }

        assert_eq!(reached, all_locations);

        let mut universe_settings = UniverseSettings::default();
        universe_settings.world_settings[0].difficulty = Difficulty::Gorlek;

        let graph = logic::parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();
        let mut world = World::new_spawn(&graph, &universe_settings.world_settings[0]);

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
