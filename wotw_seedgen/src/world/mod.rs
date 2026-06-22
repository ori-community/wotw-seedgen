mod graph_ref;
mod is_met;
mod reached;
#[cfg(test)]
pub(crate) mod tests;

use arrayvec::ArrayVec;
pub(crate) use graph_ref::GraphRef;
pub(crate) use is_met::Missing;
pub(crate) use reached::{
    ConnectionIndex, ConnectionOrRefill, ConnectionRequirement, ConnectionRequirementPartial,
    ReachStateFails,
};

use std::{
    fmt::{self, Display},
    mem,
};

use crate::{
    logical_difficulty::{LogicalDifficulty, SHIELD_WEAPONS},
    orbs::{OrbVariants, Orbs},
};
use reached::Reach;
use smallvec::smallvec;
use wotw_seedgen_data::{
    assets::UberStateValue,
    logic_language::output::{Graph, RefillValue},
    seed_language::{
        output::Event,
        simulate::{
            ConditionValues, Heap, Simulation, SimulationCache, Snapshot, Stack, UberStates,
            WorldState,
        },
    },
    Difficulty, Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade, WorldSettings,
};

// TODO A stateful reach check would have some advantages, for instance currently seedgen would not correctly account for "Grant Launch on breaking this Wall"

// TODO design interfaces instead of spamming pub(crate)?
#[derive(Debug)]
pub struct World<'graph, 'settings> {
    pub(crate) graph: &'graph Graph,
    pub(crate) spawn: usize,
    pub(crate) settings: &'settings WorldSettings,
    pub(crate) reach: Reach<'graph>,
    state: SimulationCache<WorldState>,
    updating_reach: bool,
}

impl<'graph, 'settings> World<'graph, 'settings> {
    /// Creates a new world with the given [`Graph`] and [`WorldSettings`]
    ///
    /// It will not start tracking reached locations until you [`World::traverse_spawn`]
    pub fn new(
        graph: &'graph Graph,
        spawn: usize,
        settings: &'settings WorldSettings,
        uber_states: UberStates,
        events: &mut [Event],
    ) -> Self {
        Self {
            state: SimulationCache::new(WorldState::new(uber_states, events)),
            graph,
            spawn,
            settings,
            updating_reach: false,
            reach: Reach::new(graph),
        }
    }

    // TODO there are progressions where the requirements is a pure "Impossible". Are we not optimizing those away?
    // TODO it seems like we are returning progressions to nodes that are already reached. Maybe we have to filter that in post since they
    // may have been reached after initially encountering the unmet requirement? This is common for teleporters

    /// Returns the maximum health and energy
    pub fn max_orbs(&self) -> Orbs {
        Orbs {
            health: self.max_health(),
            energy: self.max_energy(),
        }
    }

    pub fn cap_health<const CHECKPOINT: bool>(&self, orbs: &mut Orbs) {
        // checkpoints don't refill health given by the Vitality shard
        let max_health = if CHECKPOINT {
            self.base_max_health()
        } else {
            self.max_health()
        };

        // TODO helpers for combined setting and inventory checks?
        if !CHECKPOINT && self.settings.difficulty.overflow() && self.shard(Shard::Overflow) {
            if orbs.health > max_health {
                let overflow_energy = (orbs.health - max_health) / 10.;
                orbs.energy = f32::min(orbs.energy + overflow_energy, self.max_energy());
                orbs.health = max_health;
            }
        } else {
            orbs.health = f32::min(orbs.health, max_health);
        }

        debug_assert!(orbs.health <= max_health);
    }

    pub fn cap_energy<const CHECKPOINT: bool>(&self, orbs: &mut Orbs) {
        // checkpoints do refill energy from the Energy shard
        let max_energy = self.max_energy();

        if !CHECKPOINT && self.settings.difficulty.overflow() && self.shard(Shard::Overflow) {
            if orbs.energy > max_energy {
                let overflow_health = (orbs.energy - max_energy) * 10.;
                orbs.health = f32::min(orbs.health + overflow_health, self.max_health());
                orbs.energy = max_energy;
            }
        } else {
            orbs.energy = f32::min(orbs.energy, max_energy);
        }

        debug_assert!(orbs.energy <= max_energy);
    }

    /// Reduces the [`Orbs`] to the maximum health and energy of this [`Player`] if they exceed it
    ///
    /// This follows the in-game rules when adding health or energy to the in-game player
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::World;
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::logic_language::output::Graph;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen::data::WorldSettings;
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let world_settings = WorldSettings::default();
    /// let world = World::new(&graph, spawn, &world_settings, uber_states, &mut events);
    ///
    /// let mut orbs = Orbs { health: 90.0, energy: 5.0 };
    /// world.cap_orbs::<false>(&mut orbs);
    /// assert_eq!(orbs, world.max_orbs());
    /// ```
    ///
    /// `CHECKPOINT` represents whether the Orbs are a result of the player respawning on a checkpoint, in which case special rules apply
    ///
    /// ```
    /// # use wotw_seedgen::World;
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::logic_language::output::Graph;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen::data::{seed_language::simulate::Simulation, Difficulty, Shard, WorldSettings};
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let mut world_settings = WorldSettings::default();
    /// world_settings.difficulty = Difficulty::Gorlek;
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states, &mut events);
    /// world.store_shard(Shard::Vitality, true, &events);
    ///
    /// let mut orbs = Orbs { health: 90.0, energy: 1.0 };
    /// world.cap_orbs::<false>(&mut orbs);
    /// assert_eq!(orbs, Orbs { health: 40.0, energy: 1.0 });
    ///
    /// world.cap_orbs::<true>(&mut orbs);
    /// assert_eq!(orbs, Orbs { health: 30.0, energy: 1.0 });
    /// ```
    // TODO this didn't end up being used much, maybe it should be used more to have the overflow check?
    pub fn cap_orbs<const CHECKPOINT: bool>(&self, orbs: &mut Orbs) {
        self.cap_health::<CHECKPOINT>(orbs);
        self.cap_energy::<CHECKPOINT>(orbs);
    }

    /// Returns the [`Orbs`] after respawning on a checkpoint
    ///
    /// This follows the in-game rules when respawning on a checkpoint
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::World;
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::logic_language::output::Graph;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen::data::{seed_language::simulate::Simulation, WorldSettings};
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let world_settings = WorldSettings::default();
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states, &mut events);
    /// assert_eq!(world.max_orbs(), Orbs { health: 30.0, energy: 3.0 });
    /// assert_eq!(world.checkpoint_orbs(), Orbs { health: 30.0, energy: 1.0 });
    ///
    /// world.add_base_max_health(110, &events);
    /// world.add_base_max_energy((12.).into(), &events);
    /// assert_eq!(world.max_orbs(), Orbs { health: 140.0, energy: 15.0 });
    /// assert_eq!(world.checkpoint_orbs(), Orbs { health: 42.0, energy: 3.0 });
    /// ```
    pub fn checkpoint_orbs(&self) -> Orbs {
        let mut orbs = Orbs {
            health: f32::max((self.max_health() * 0.3).ceil(), 40.0),
            energy: f32::max((self.max_energy() * 0.2).ceil(), 1.0),
        };

        self.cap_orbs::<true>(&mut orbs);

        orbs
    }

    /// Returns how many health orbs plants will drop
    ///
    /// This follows the in-game rules when opening a health plant
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::World;
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::logic_language::output::Graph;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen::data::{seed_language::simulate::Simulation, WorldSettings};
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let world_settings = WorldSettings::default();
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states, &mut events);
    /// assert_eq!(world.health_plant_drops(), 1.0);
    ///
    /// world.add_base_max_health(40, &events);
    /// assert_eq!(world.health_plant_drops(), 2.0);
    ///
    /// world.add_base_max_health(90, &events);
    /// assert_eq!(world.health_plant_drops(), 5.0);
    /// ```
    pub fn health_plant_drops(&self) -> f32 {
        let value = self.max_health() / 30.0;
        // the game rounds to even
        #[allow(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            clippy::float_cmp
        )]
        if value % 1. == 0.5 && (value as u8).is_multiple_of(2) {
            value.floor()
        } else {
            value.round()
        }
    }

    /// Replenish health, but don't exceed the player's maximum health
    pub fn heal(&self, orbs: &mut Orbs, amount: f32) {
        orbs.health += amount;
        self.cap_health::<false>(orbs);
    }

    /// Replenish energy, but don't exceed the player's maximum energy
    pub fn recharge(&self, orbs: &mut Orbs, amount: f32) {
        orbs.energy += amount;
        self.cap_energy::<false>(orbs);
    }

    /// Apply the refill from a [`RefillValue`] to a set of [`OrbVariants`]
    pub(crate) fn refill(&self, refill: RefillValue, orb_variants: &mut OrbVariants) {
        debug_assert!(!orb_variants.is_empty());

        match refill {
            RefillValue::Full => *orb_variants = smallvec![self.max_orbs()],
            RefillValue::Checkpoint => {
                let checkpoint_orbs = self.checkpoint_orbs();

                for orbs in orb_variants {
                    orbs.health = f32::max(orbs.health, checkpoint_orbs.health);
                    orbs.energy = f32::max(orbs.energy, checkpoint_orbs.energy);
                }
            }
            RefillValue::Health(amount) => {
                let amount = amount * self.health_plant_drops();

                for orbs in orb_variants {
                    self.heal(orbs, amount)
                }
            }
            RefillValue::Energy(amount) => {
                for orbs in orb_variants {
                    self.recharge(orbs, amount)
                }
            }
        }
    }

    pub fn damage_mod(&self, flying_target: bool, bow: bool) -> f32 {
        let mut damage_mod = 1.0;

        // These all don't account for Spirit Shard upgrades
        if self.settings.difficulty.damage_buffs() {
            if self.skill(Skill::GladesAncestralLight) {
                damage_mod += 0.25;
            }
            if self.skill(Skill::MarshAncestralLight) {
                damage_mod += 0.25;
            }

            let mut slots = self.shard_slots();
            let mut splinter = false;

            if flying_target && slots > 0 && self.shard(Shard::Wingclip) {
                damage_mod += 1.0;
                slots -= 1;
            }

            if slots > 0 && bow && self.shard(Shard::Splinter) {
                splinter = true;
                slots -= 1;
            }

            // TODO not necessarily the best pick depending on spirit light
            if slots > 0 && self.shard(Shard::SpiritSurge) {
                damage_mod += self.spirit_light().min(3000) as f32 * 0.00005;
                slots -= 1;
            }

            if slots > 0 && self.shard(Shard::LastStand) {
                damage_mod += 0.2;
                slots -= 1;
            }

            if slots > 0 && self.shard(Shard::Reckless) {
                damage_mod += 0.15;
                slots -= 1;
            }

            if slots > 0 && self.shard(Shard::Lifeforce) {
                damage_mod += 0.1;
                slots -= 1;
            }

            if slots > 0 && self.shard(Shard::Finesse) {
                damage_mod += 0.05;
            }

            if splinter {
                // Splinter stacks multiplicatively where other buffs stack additively
                damage_mod *= 1.5;
            }
        }

        damage_mod
    }

    pub fn defense_mod(&self) -> f32 {
        let mut defense_mod = 1.;

        if self.settings.difficulty.resilience() && self.shard(Shard::Resilience) {
            defense_mod *= 0.9;
        }

        if self.settings.hard {
            defense_mod *= 2.0;
        }

        defense_mod
    }

    pub fn energy_mod(&self) -> f32 {
        let mut energy_mod = 1.0;

        if self.settings.difficulty < Difficulty::Unsafe {
            energy_mod *= 2.0;
        } else if self.shard(Shard::Overcharge) {
            energy_mod *= 0.5;
        }

        energy_mod
    }

    pub fn use_cost(&self, skill: Skill) -> f32 {
        skill.energy_cost() * self.energy_mod()
    }

    pub fn destroy_cost<const TARGET_IS_WALL: bool>(
        &self,
        target_health: f32,
        flying_target: bool,
    ) -> Option<f32> {
        // TODO don't repeatedly search the weapon if nothing changed?
        let mut weapons = self.owned_weapons::<TARGET_IS_WALL>().peekable();

        if weapons.peek()?.energy_cost() == 0.0 {
            Some(0.0)
        } else {
            self.destroy_cost_with_any_of(weapons, target_health, flying_target)
        }
    }

    pub fn destroy_cost_ranged(&self, target_health: f32, flying_target: bool) -> Option<f32> {
        self.destroy_cost_with_any_of(self.owned_ranged_weapons(), target_health, flying_target)
    }

    pub fn destroy_cost_with(&self, target_health: f32, weapon: Skill, flying_target: bool) -> f32 {
        let (damage, cost) = self.weapon_stats(weapon, flying_target);
        (target_health / damage).ceil() * cost
    }

    /// Returns the energy required to destroy the target with the given combination of weapons, or `None` if `weapons` is empty
    ///
    /// We optimize based on the assumption that `weapons` has energy-less weapons in front
    fn destroy_cost_with_any_of<I: Iterator<Item = Skill>>(
        &self,
        weapons: I,
        mut target_health: f32,
        flying_target: bool,
    ) -> Option<f32> {
        let mut weapon_stats = ArrayVec::<_, 9>::new();
        let mut best_dpe = ((0., 0.), 0.);

        for weapon in weapons {
            let (damage, cost) = self.weapon_stats(weapon, flying_target);

            let dpe = damage / cost;
            if dpe > best_dpe.1 {
                best_dpe = ((damage, cost), dpe);
            }

            weapon_stats.push((damage, cost));
        }

        if weapon_stats.is_empty() {
            return None;
        }

        let ((damage, mut cost), _) = best_dpe;

        let optimal_hits = (target_health / damage).floor();
        target_health -= optimal_hits * damage;
        cost *= optimal_hits;

        // Figure out the best weapon to deal the last bit of damage
        cost += weapon_stats
            .into_iter()
            .map(|(damage, cost)| (target_health / damage).ceil() * cost)
            .min_by(f32::total_cmp)?;

        // On arbitrary energy costs and damage amounts this procedure might choose suboptimal weapons to use, but for the defaults it should be exhaustive

        Some(cost)
    }

    /// Returns the damage and cost of the weapon after all modifiers
    fn weapon_stats(&self, weapon: Skill, flying_target: bool) -> (f32, f32) {
        let damage_mod = self.damage_mod(flying_target, matches!(weapon, Skill::Bow));

        let damage = weapon.total_damage(self.settings.difficulty.charge_grenade()) * damage_mod;

        let cost = self.use_cost(weapon);

        (damage, cost)
    }

    pub fn owned_weapons<const TARGET_IS_WALL: bool>(&self) -> impl Iterator<Item = Skill> + '_ {
        self.owned_weapons_from(Difficulty::weapons_iter::<TARGET_IS_WALL>)
    }

    pub fn owned_ranged_weapons(&self) -> impl Iterator<Item = Skill> + '_ {
        self.owned_weapons_from(Difficulty::ranged_weapons_iter)
    }

    pub fn owned_shield_weapons(&self) -> impl Iterator<Item = Skill> + '_ {
        self.owned_weapons_from(|_| SHIELD_WEAPONS.into_iter())
    }

    fn owned_weapons_from<'a, F, I>(&'a self, f: F) -> impl Iterator<Item = Skill> + 'a
    where
        F: FnOnce(Difficulty) -> I,
        I: Iterator<Item = Skill> + 'a,
    {
        f(self.settings.difficulty).filter(|weapon| self.skill(*weapon))
    }

    pub fn inventory_display(&self) -> InventoryDisplay<'_, '_, '_> {
        InventoryDisplay { world: self }
    }
}

impl Simulation for World<'_, '_> {
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.state.fetch(uber_identifier)
    }

    fn store_impl(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) -> &[usize] {
        self.state.store_impl(uber_identifier, value)
    }

    fn on_change(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        self.update_reached(uber_identifier, events);
    }

    #[inline]
    fn stack(&self) -> &Stack {
        self.state.stack()
    }

    #[inline]
    fn stack_mut(&mut self) -> &mut Stack {
        self.state.stack_mut()
    }

    #[inline]
    fn heap(&self) -> &Heap {
        self.state.heap()
    }

    #[inline]
    fn heap_mut(&mut self) -> &mut Heap {
        self.state.heap_mut()
    }

    fn condition_values(&mut self) -> &mut ConditionValues {
        self.state.condition_values()
    }

    // Not sure how we could use the cache-efficient specialized stores without invalidating our reach

    fn spirit_light(&self) -> i32 {
        self.state.spirit_light()
    }

    fn gorlek_ore(&self) -> i32 {
        self.state.gorlek_ore()
    }

    fn keystones(&self) -> i32 {
        self.state.keystones()
    }

    fn shard_slots(&self) -> i32 {
        self.state.shard_slots()
    }

    fn base_max_health(&self) -> f32 {
        self.state.base_max_health()
    }

    fn max_health(&self) -> f32 {
        // TODO does this get optimized into the branchless variant? Maybe just write it branchless...
        // Also I guess this could be cached since settings are immutable?
        // Could store the function pointer for which to invoke...
        if self.settings.difficulty.vitality() {
            self.state.max_health()
        } else {
            self.base_max_health()
        }
    }

    fn base_max_energy(&self) -> f32 {
        self.state.base_max_energy()
    }

    fn max_energy(&self) -> f32 {
        if self.settings.difficulty.energy_shard() {
            self.state.max_energy()
        } else {
            self.base_max_energy()
        }
    }

    fn skill(&self, skill: Skill) -> bool {
        self.state.skill(skill)
    }

    fn shard(&self, shard: Shard) -> bool {
        self.state.shard(shard)
    }

    fn teleporter(&self, teleporter: Teleporter) -> bool {
        self.state.teleporter(teleporter)
    }

    fn clean_water(&self) -> bool {
        self.state.clean_water()
    }

    fn weapon_upgrade(&self, weapon_upgrade: WeaponUpgrade) -> bool {
        self.state.weapon_upgrade(weapon_upgrade)
    }

    fn skills(&self) -> impl Iterator<Item = Skill> + '_ {
        self.state.skills()
    }

    fn shards(&self) -> impl Iterator<Item = Shard> + '_ {
        self.state.shards()
    }

    fn teleporters(&self) -> impl Iterator<Item = Teleporter> + '_ {
        self.state.teleporters()
    }

    fn weapon_upgrades(&self) -> impl Iterator<Item = WeaponUpgrade> + '_ {
        self.state.weapon_upgrades()
    }
}

impl Snapshot for World<'_, '_> {
    fn snapshot(&mut self) {
        self.state.snapshot();
        self.reach.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.state.restore_snapshot();
        self.reach.restore_snapshot();
    }
}

pub struct InventoryDisplay<'world, 'graph, 'settings> {
    world: &'world World<'graph, 'settings>,
}

impl Display for InventoryDisplay<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn comma(f: &mut fmt::Formatter<'_>, first: &mut bool) -> fmt::Result {
            if mem::take(first) {
                Ok(())
            } else {
                write!(f, ", ")
            }
        }

        fn item<T>(f: &mut fmt::Formatter<'_>, first: &mut bool, name: T) -> fmt::Result
        where
            T: Display,
        {
            comma(f, first)?;
            write!(f, "{name}")
        }

        fn amount_item<T>(
            f: &mut fmt::Formatter<'_>,
            first: &mut bool,
            amount: T,
            name: &str,
        ) -> fmt::Result
        where
            T: Display,
        {
            comma(f, first)?;
            write!(f, "{amount} {name}")
        }

        fn resource<const PLURAL_S: bool>(
            f: &mut fmt::Formatter<'_>,
            first: &mut bool,
            amount: i32,
            name: &str,
        ) -> fmt::Result {
            if amount > 0 {
                amount_item(f, first, amount, name)?;

                if PLURAL_S && amount > 1 {
                    write!(f, "s")?;
                }
            }

            Ok(())
        }

        fn iter_item<I, T>(f: &mut fmt::Formatter<'_>, first: &mut bool, mut iter: I) -> fmt::Result
        where
            I: Iterator<Item = T>,
            T: Display,
        {
            iter.try_for_each(|name| item(f, first, name))
        }

        fn bool_item(
            f: &mut fmt::Formatter<'_>,
            first: &mut bool,
            owned: bool,
            name: &str,
        ) -> fmt::Result {
            if owned {
                item(f, first, name)
            } else {
                Ok(())
            }
        }

        let mut first = true;

        amount_item(f, &mut first, self.world.base_max_health(), "Health")?;
        amount_item(f, &mut first, self.world.base_max_energy(), "Energy")?;
        resource::<false>(f, &mut first, self.world.spirit_light(), "Spirit Light")?;
        resource::<false>(f, &mut first, self.world.gorlek_ore(), "Gorlek Ore")?;
        resource::<true>(f, &mut first, self.world.keystones(), "Keystone")?;
        resource::<true>(f, &mut first, self.world.shard_slots(), "Shard Slot")?;
        iter_item(f, &mut first, self.world.skills())?;
        iter_item(f, &mut first, self.world.shards())?;
        iter_item(f, &mut first, self.world.teleporters())?;
        bool_item(f, &mut first, self.world.clean_water(), "Clean Water")?;
        iter_item(f, &mut first, self.world.weapon_upgrades())?;

        Ok(())
    }
}
