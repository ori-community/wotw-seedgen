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

    pub(crate) fn preplace(&mut self, trigger: UberStateTrigger, item: Item) {
        self.preplacements.entry(trigger).or_default().push(item);
    }
    /// Some UberStates are chained to other UberStates that should get set as consequence
    /// 
    /// This should mirror the SpecialHandlers in [https://github.com/ori-rando/wotw-client/blob/dev/projects/RandoMainDLL/UberStateController.cs]
    // With the official logic file and headers, these are all redundant, but they may be relevant for custom logic or header files
    fn set_chained_states(&mut self, identifier: UberIdentifier, value: f32) {
        match identifier {
            UberIdentifier { uber_group: 5377, uber_id: 53480 } if value as u8 == 4 => {  // Water Dash Fight Arena
                self.set_uber_state(UberIdentifier::new(5377, 1373), 4.); },  // Waterdashless Arena
            UberIdentifier { uber_group: 42178, uber_id: 2654 } if value > 0.5 && value < 2.5 => {  // Diamond in the Rough
                self.set_uber_state(identifier, 3.);
                self.set_uber_state(UberIdentifier::new(23987, 14832), 1.); },  // Waterdashless Arena
            UberIdentifier { uber_group: 37858, uber_id: 12379 } if value > 0.5 => {  // Mill complete
                self.set_uber_state(UberIdentifier::new(937, 34641), 3.); },  // Mill Quest
            // Voice Hackfix has no meaning to seedgen
            UberIdentifier { uber_group: 937, uber_id: 34641 } if value > 2.5 => {  // Mill Quest
                self.set_uber_state(UberIdentifier::new(6, 300), 1.); },  // Tuley
            UberIdentifier { uber_group: 58674, uber_id: 32810 } if value as u8 == 7 => {  // Cat and Mouse
                self.set_uber_state(identifier, 8.); },  // Cat and Mouse complete
            UberIdentifier { uber_group: 16155, uber_id: 28478 } if value > 0.5 => {  // Willow Stone Vine
                self.set_uber_state(UberIdentifier::new(16155, 12971), 4.); },  // Willow Stone Boss
            // 5377, 63173 Any sane kind of logic file should already account for this
            UberIdentifier { uber_group: 0, uber_id: 100 } if value > 0.5 => {  // Sword Tree
                self.set_uber_state(UberIdentifier::new(6, 401), 1.); },  // Rain Lifted
            _ => {},
        };
    }
    fn collect_preplacements(&mut self, identifier: UberIdentifier, old: f32) -> bool {
        let new = self.get_uber_state(identifier);
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
        self.set_chained_states(identifier, value);
        self.collect_preplacements(identifier, old)
    }
    /// Sets the value at an [`UberIdentifier`] and collects any items preplaced on it, but only if the current value is lower than the target value
    /// 
    /// This is used to set vanilla quest and world state UberStates, since they need to behave strictly incrementally
    /// 
    /// Returns `true` if any preplaced items were collected or the current value was already equal or higher than the target value
    pub(crate) fn set_incremental_uber_state(&mut self, identifier: UberIdentifier, value: f32) -> bool {
        if self.get_uber_state(identifier) < value {
            self.set_uber_state(identifier, value)
        } else { true }  // If a quest uberState was already manually set to a higher value, it should block placements on all the skipped quest steps
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
