mod graph;
mod is_met;
mod reached;
mod simulate;
mod uber_states;

use std::{
    fmt::{self, Display},
    mem,
};

use ordered_float::OrderedFloat;
use reached::Reach;
pub use simulate::Simulate;
use strum::VariantArray;
pub use uber_states::{UberStateValue, UberStates};

pub(crate) use graph::{node_condition, node_trigger};

#[cfg(test)]
mod tests;

use crate::{
    logical_difficulty,
    orbs::{self, OrbVariants, Orbs},
};

use rustc_hash::FxHashMap;
use smallvec::{smallvec, SmallVec};
use wotw_seedgen_data::{Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade};
use wotw_seedgen_logic_language::output::{Graph, RefillValue};
use wotw_seedgen_seed_language::output::{
    ArithmeticOperator, CommandFloat, CommandInteger, CommandVoid, Event, Operation,
};
use wotw_seedgen_settings::{Difficulty, WorldSettings};

// TODO A stateful reach check would have some advantages, for instance currently seedgen would not correctly account for "Grant Launch on breaking this Wall"

// TODO design interfaces instead of spamming pub(crate)?
#[derive(Debug, Clone)]
pub struct World<'graph, 'settings> {
    pub(crate) graph: &'graph Graph,
    pub(crate) spawn: usize,
    pub(crate) settings: &'settings WorldSettings,
    pub(crate) uber_states: UberStates,
    pub(crate) reach: Reach,
    updating_reach: bool,
    snapshot: Option<Reach>,
    variables: Variables,
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
    ) -> Self {
        Self {
            graph,
            spawn,
            settings,
            uber_states,
            updating_reach: false,
            reach: Reach::new(graph),
            snapshot: None,
            variables: Default::default(),
        }
    }

    // TODO there are progressions where the requirements is a pure "Impossible". Are we not optimizing those away?
    // TODO it seems like we are returning progressions to nodes that are already reached. Maybe we have to filter that in post since they
    // may have been reached after initially encountering the unmet requirement? This is common for teleporters

    #[inline]
    pub fn simulate<T: Simulate>(&mut self, t: &T, events: &[Event]) -> T::Return {
        t.simulate(self, events)
    }

    pub fn set_boolean(&mut self, uber_identifier: UberIdentifier, value: bool, events: &[Event]) {
        self.simulate(
            &CommandVoid::StoreBoolean {
                uber_identifier,
                value: value.into(),
                trigger_events: true,
            },
            events,
        );
    }
    pub fn set_integer(&mut self, uber_identifier: UberIdentifier, value: i32, events: &[Event]) {
        self.simulate(
            &CommandVoid::StoreInteger {
                uber_identifier,
                value: value.into(),
                trigger_events: true,
            },
            events,
        );
    }
    pub fn set_float(
        &mut self,
        uber_identifier: UberIdentifier,
        value: OrderedFloat<f32>,
        events: &[Event],
    ) {
        self.simulate(
            &CommandVoid::StoreFloat {
                uber_identifier,
                value: value.into(),
                trigger_events: true,
            },
            events,
        );
    }
    pub fn modify_integer(&mut self, uber_identifier: UberIdentifier, add: i32, events: &[Event]) {
        self.simulate(
            &CommandVoid::StoreInteger {
                uber_identifier,
                value: CommandInteger::Arithmetic {
                    operation: Box::new(Operation {
                        left: CommandInteger::FetchInteger { uber_identifier },
                        operator: ArithmeticOperator::Add,
                        right: add.into(),
                    }),
                },
                trigger_events: true,
            },
            events,
        );
    }
    pub fn modify_float(
        &mut self,
        uber_identifier: UberIdentifier,
        add: OrderedFloat<f32>,
        events: &[Event],
    ) {
        self.simulate(
            &CommandVoid::StoreFloat {
                uber_identifier,
                value: CommandFloat::Arithmetic {
                    operation: Box::new(Operation {
                        left: CommandFloat::FetchFloat { uber_identifier },
                        operator: ArithmeticOperator::Add,
                        right: add.into(),
                    }),
                },
                trigger_events: true,
            },
            events,
        );
    }

    #[inline]
    pub fn set_spirit_light(&mut self, value: i32, events: &[Event]) {
        self.set_integer(UberIdentifier::SPIRIT_LIGHT, value, events);
    }
    #[inline]
    pub fn modify_spirit_light(&mut self, add: i32, events: &[Event]) {
        self.modify_integer(UberIdentifier::SPIRIT_LIGHT, add, events);
    }
    #[inline]
    pub fn set_gorlek_ore(&mut self, value: i32, events: &[Event]) {
        self.set_integer(UberIdentifier::GORLEK_ORE, value, events);
    }
    #[inline]
    pub fn modify_gorlek_ore(&mut self, add: i32, events: &[Event]) {
        self.modify_integer(UberIdentifier::GORLEK_ORE, add, events);
    }
    #[inline]
    pub fn set_keystones(&mut self, value: i32, events: &[Event]) {
        self.set_integer(UberIdentifier::KEYSTONES, value, events);
    }
    #[inline]
    pub fn modify_keystones(&mut self, add: i32, events: &[Event]) {
        self.modify_integer(UberIdentifier::KEYSTONES, add, events);
    }
    #[inline]
    pub fn set_shard_slots(&mut self, value: i32, events: &[Event]) {
        self.set_integer(UberIdentifier::SHARD_SLOTS, value, events);
    }
    #[inline]
    pub fn modify_shard_slots(&mut self, add: i32, events: &[Event]) {
        self.modify_integer(UberIdentifier::SHARD_SLOTS, add, events);
    }
    #[inline]
    pub fn set_max_health(&mut self, value: i32, events: &[Event]) {
        self.set_integer(UberIdentifier::MAX_HEALTH, value, events);
    }
    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    pub fn modify_max_health(&mut self, add: i32, events: &[Event]) {
        self.modify_integer(UberIdentifier::MAX_HEALTH, add, events);
    }
    // TODO but where do I *really* want OrderedFloat
    #[inline]
    pub fn set_max_energy(&mut self, value: OrderedFloat<f32>, events: &[Event]) {
        self.set_float(UberIdentifier::MAX_ENERGY, value, events);
    }
    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    pub fn modify_max_energy(&mut self, add: OrderedFloat<f32>, events: &[Event]) {
        self.modify_float(UberIdentifier::MAX_ENERGY, add, events);
    }
    #[inline]
    pub fn set_skill(&mut self, skill: Skill, value: bool, events: &[Event]) {
        self.set_boolean(skill.uber_identifier(), value, events);
    }
    #[inline]
    pub fn set_shard(&mut self, shard: Shard, value: bool, events: &[Event]) {
        self.set_boolean(shard.uber_identifier(), value, events);
    }
    #[inline]
    pub fn set_teleporter(&mut self, teleporter: Teleporter, value: bool, events: &[Event]) {
        self.set_boolean(teleporter.uber_identifier(), value, events);
    }
    #[inline]
    pub fn set_clean_water(&mut self, value: bool, events: &[Event]) {
        self.set_boolean(UberIdentifier::CLEAN_WATER, value, events);
    }
    #[inline]
    pub fn set_weapon_upgrade(
        &mut self,
        weapon_upgrade: WeaponUpgrade,
        value: bool,
        events: &[Event],
    ) {
        self.set_integer(weapon_upgrade.uber_identifier(), i32::from(value), events);
    }

    #[inline]
    pub fn spirit_light(&self) -> i32 {
        self.uber_states[UberIdentifier::SPIRIT_LIGHT].expect_integer()
    }

    #[inline]
    pub fn gorlek_ore(&self) -> i32 {
        self.uber_states[UberIdentifier::GORLEK_ORE].expect_integer()
    }

    #[inline]
    pub fn keystones(&self) -> i32 {
        self.uber_states[UberIdentifier::KEYSTONES].expect_integer()
    }

    #[inline]
    pub fn shard_slots(&self) -> i32 {
        self.uber_states[UberIdentifier::SHARD_SLOTS].expect_integer()
    }

    #[inline]
    pub fn base_max_health(&self) -> f32 {
        self.uber_states[UberIdentifier::MAX_HEALTH].expect_integer() as f32
    }

    /// Returns the maximum health
    ///
    /// One visual orb in the game represents 10 health
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// let world_settings = WorldSettings::default();
    /// let world = World::new(&graph, spawn, &world_settings, uber_states);
    /// assert_eq!(world.max_health(), 30.0);
    /// ```
    pub fn max_health(&self) -> f32 {
        let mut health = self.base_max_health();

        if self.settings.difficulty >= logical_difficulty::VITALITY && self.shard(Shard::Vitality) {
            health += 10.;
        }

        health
    }

    #[inline]
    pub fn base_max_energy(&self) -> f32 {
        *self.uber_states[UberIdentifier::MAX_ENERGY].expect_float()
    }

    /// Returns the maximum energy
    ///
    /// One visual orb in the game represents 1 energy
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// let world_settings = WorldSettings::default();
    /// let world = World::new(&graph, spawn, &world_settings, uber_states);
    /// assert_eq!(world.max_energy(), 3.0);
    /// ```
    pub fn max_energy(&self) -> f32 {
        let mut energy = self.base_max_energy();

        if self.settings.difficulty >= logical_difficulty::ENERGY_SHARD && self.shard(Shard::Energy)
        {
            energy += 1.;
        }

        energy
    }

    /// Returns the maximum health and energy
    pub fn max_orbs(&self) -> Orbs {
        Orbs {
            health: self.max_health(),
            energy: self.max_energy(),
        }
    }

    #[inline]
    pub fn skill(&self, skill: Skill) -> bool {
        self.uber_states[skill.uber_identifier()].expect_boolean()
    }

    #[inline]
    pub fn shard(&self, shard: Shard) -> bool {
        self.uber_states[shard.uber_identifier()].expect_boolean()
    }

    #[inline]
    pub fn teleporter(&self, teleporter: Teleporter) -> bool {
        self.uber_states[teleporter.uber_identifier()].expect_boolean()
    }

    #[inline]
    pub fn clean_water(&self) -> bool {
        self.uber_states[UberIdentifier::CLEAN_WATER].expect_boolean()
    }

    #[inline]
    pub fn weapon_upgrade(&self, weapon_upgrade: WeaponUpgrade) -> bool {
        self.uber_states[weapon_upgrade.uber_identifier()].expect_integer() > 0
    }

    pub fn inventory_display(&self) -> InventoryDisplay {
        InventoryDisplay { world: self }
    }

    pub fn skills(&self) -> impl Iterator<Item = Skill> + '_ {
        Skill::VARIANTS
            .iter()
            .copied()
            .filter(|skill| self.skill(*skill))
    }

    pub fn shards(&self) -> impl Iterator<Item = Shard> + '_ {
        Shard::VARIANTS
            .iter()
            .copied()
            .filter(|shard| self.shard(*shard))
    }

    pub fn teleporters(&self) -> impl Iterator<Item = Teleporter> + '_ {
        Teleporter::VARIANTS
            .iter()
            .copied()
            .filter(|teleporter| self.teleporter(*teleporter))
    }

    pub fn weapon_upgrades(&self) -> impl Iterator<Item = WeaponUpgrade> + '_ {
        WeaponUpgrade::VARIANTS
            .iter()
            .copied()
            .filter(|weapon_upgrade| self.weapon_upgrade(*weapon_upgrade))
    }

    pub fn cap_health<const CHECKPOINT: bool>(&self, orbs: &mut Orbs) {
        // checkpoints don't refill health given by the Vitality shard
        let max_health = if CHECKPOINT {
            self.base_max_health()
        } else {
            self.max_health()
        };

        if !CHECKPOINT
            && self.settings.difficulty >= logical_difficulty::OVERFLOW
            && self.shard(Shard::Overflow)
            && orbs.health > max_health
        {
            orbs.energy += orbs.health - max_health;
        }

        orbs.health = f32::min(orbs.health, max_health);
    }

    pub fn cap_energy<const CHECKPOINT: bool>(&self, orbs: &mut Orbs) {
        // checkpoints do refill energy from the Energy shard
        let max_energy = self.max_energy();

        if !CHECKPOINT
            && self.settings.difficulty >= logical_difficulty::OVERFLOW
            && self.shard(Shard::Overflow)
            && orbs.energy > max_energy
        {
            orbs.health += orbs.energy - max_energy
        }

        orbs.energy = f32::min(orbs.energy, max_energy);
    }

    /// Reduces the [`Orbs`] to the maximum health and energy of this [`Player`] if they exceed it
    ///
    /// This follows the in-game rules when adding health or energy to the in-game player
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// let world_settings = WorldSettings::default();
    /// let world = World::new(&graph, spawn, &world_settings, uber_states);
    ///
    /// let mut orbs = Orbs { health: 90.0, energy: 5.0 };
    /// world.cap_orbs::<false>(&mut orbs);
    /// assert_eq!(orbs, world.max_orbs());
    /// ```
    ///
    /// `CHECKPOINT` represents whether the Orbs are a result of the player respawning on a checkpoint, in which case special rules apply
    ///
    /// ```
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::data::Shard;
    /// use wotw_seedgen::orbs::Orbs;
    /// use wotw_seedgen::settings::{WorldSettings, Difficulty};
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// # let events = [];
    /// let mut world_settings = WorldSettings::default();
    /// world_settings.difficulty = Difficulty::Gorlek;
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states);
    /// world.set_shard(Shard::Vitality, true, &events);
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
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// # let events = [];
    /// let world_settings = WorldSettings::default();
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states);
    /// assert_eq!(world.max_orbs(), Orbs { health: 30.0, energy: 3.0 });
    /// assert_eq!(world.checkpoint_orbs(), Orbs { health: 30.0, energy: 1.0 });
    ///
    /// world.modify_max_health(110, &events);
    /// world.modify_max_energy((12.).into(), &events);
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
    /// # use wotw_seedgen::{World, UberStates};
    /// # use wotw_seedgen_logic_language::output::Graph;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// # let graph = Graph::empty();
    /// # let spawn = 0;
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// # let events = [];
    /// let world_settings = WorldSettings::default();
    /// let mut world = World::new(&graph, spawn, &world_settings, uber_states);
    /// assert_eq!(world.health_plant_drops(), 1.0);
    ///
    /// world.modify_max_health(40, &events);
    /// assert_eq!(world.health_plant_drops(), 2.0);
    ///
    /// world.modify_max_health(90, &events);
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
        if value % 1. == 0.5 && value as u8 % 2 == 0 {
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
                *orb_variants = orbs::either_single(orb_variants, self.checkpoint_orbs())
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

        if self.settings.difficulty >= logical_difficulty::DAMAGE_BUFFS {
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
            if slots > 0 && self.shard(Shard::SpiritSurge) {
                damage_mod += self.spirit_light() as f32 * 0.0001; // TODO but this is capped right
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

        if self.settings.difficulty >= logical_difficulty::RESILIENCE
            && self.shard(Shard::Resilience)
        {
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
        self.destroy_cost_with_any_of(
            target_health,
            self.owned_weapons::<TARGET_IS_WALL>(),
            flying_target,
        )
    }

    pub fn destroy_cost_ranged(&self, target_health: f32, flying_target: bool) -> Option<f32> {
        self.destroy_cost_with_any_of(target_health, self.owned_ranged_weapons(), flying_target)
    }

    pub fn destroy_cost_with(&self, target_health: f32, weapon: Skill, flying_target: bool) -> f32 {
        let (damage, cost) = self.weapon_stats(weapon, flying_target);
        (target_health / damage).ceil() * cost
    }

    /// Returns the energy required to destroy the target with the given combination of weapons, or `None` if `weapons` is empty
    ///
    /// We optimize based on the assumption that `weapons` has energy-less weapons in front
    fn destroy_cost_with_any_of<const N: usize>(
        &self,
        mut target_health: f32,
        weapons: SmallVec<[Skill; N]>,
        flying_target: bool,
    ) -> Option<f32> {
        if weapons.first()?.energy_cost() == 0.0 {
            return Some(0.0);
        }

        let weapon_stats = weapons
            .into_iter()
            .map(|weapon| self.weapon_stats(weapon, flying_target))
            .collect::<SmallVec<[_; 9]>>();

        // Use the best weapon as long as it doesn't "waste" any damage
        let ((damage, mut cost), _) = weapon_stats
            .iter()
            .map(|(damage, cost)| ((*damage, *cost), OrderedFloat(damage / cost)))
            .max_by(|(_, dpe_a), (_, dpe_b)| dpe_a.cmp(dpe_b))?;
        let optimal_hits = (target_health / damage).floor();
        target_health -= optimal_hits * damage;
        cost *= optimal_hits;

        // Figure out the best weapon to deal the last bit of damage
        cost += weapon_stats
            .into_iter()
            .map(|(damage, cost)| OrderedFloat((target_health / damage).ceil() * cost))
            .min()?
            .into_inner();

        // On arbitrary energy costs and damage amounts this procedure might choose suboptimal weapons to use, but for the defaults it should be exhaustive

        Some(cost)
    }

    /// Returns the damage and cost of the weapon after all modifiers
    fn weapon_stats(&self, weapon: Skill, flying_target: bool) -> (f32, f32) {
        let damage_mod = self.damage_mod(flying_target, matches!(weapon, Skill::Bow));

        let damage = weapon.damage(self.settings.difficulty >= logical_difficulty::CHARGE_GRENADE)
            * damage_mod
            + weapon.burn_damage();

        let cost = self.use_cost(weapon);

        (damage, cost)
    }

    pub fn owned_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.owned_weapons_from_fn(logical_difficulty::weapons::<TARGET_IS_WALL>)
    }

    pub fn owned_ranged_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.owned_weapons_from_fn(logical_difficulty::ranged_weapons)
    }

    pub fn owned_shield_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.owned_weapons_from_fn(|_| logical_difficulty::shield_weapons())
    }

    fn owned_weapons_from_fn<const N: usize, F>(&self, weapons_fn: F) -> SmallVec<[Skill; N]>
    where
        F: FnOnce(Difficulty) -> SmallVec<[Skill; N]>,
    {
        let mut weapons = weapons_fn(self.settings.difficulty);
        weapons.retain(|weapon| self.skill(*weapon));
        weapons
    }

    pub fn progression_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.progression_weapons_from_fn(logical_difficulty::weapons::<TARGET_IS_WALL>)
    }

    pub fn ranged_progression_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.progression_weapons_from_fn(logical_difficulty::ranged_weapons)
    }

    pub fn shield_progression_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.progression_weapons_from_fn(|_| logical_difficulty::shield_weapons())
    }

    fn progression_weapons_from_fn<const N: usize, F>(&self, weapons_fn: F) -> SmallVec<[Skill; N]>
    where
        F: FnOnce(Difficulty) -> SmallVec<[Skill; N]>,
    {
        // TODO I find the name of this function confusing
        fn damage_per_energy(weapon: Skill, settings: &WorldSettings) -> f32 {
            // (weapon.damage(unsafe_paths) + weapon.burn_damage()) / weapon.energy_cost()
            (10.0
                / (weapon.damage(settings.difficulty >= logical_difficulty::CHARGE_GRENADE)
                    + weapon.burn_damage()))
            .ceil()
                * weapon.energy_cost()
            // "how much energy do you need to deal 10 damage" leads to a more realistic ordering than pure damage per energy
        }

        let mut weapons = weapons_fn(self.settings.difficulty);
        // TODO check whether creating this map is even worth it
        let dpe_map = weapons
            .iter()
            .map(|weapon| {
                (
                    *weapon,
                    (damage_per_energy(*weapon, self.settings) * 10.0) as u16,
                )
            })
            .collect::<FxHashMap<Skill, u16>>();
        weapons.sort_unstable_by_key(|weapon| dpe_map[weapon]);
        if let Some((index, weapon)) = weapons
            .iter()
            .enumerate()
            .find(|(_, weapon)| self.skill(**weapon))
        {
            let dpe = dpe_map[weapon];
            weapons.truncate(index + 1);
            // maybe there are multiple weapons costing the same and we already skipped over a redundant one
            weapons.swap(index, 0); // if we found something before, there must be at least one element
            let remove_after = weapons
                .iter()
                .rposition(|weapon| dpe_map[weapon] != dpe)
                .unwrap_or(0);
            weapons.truncate(remove_after + 1);
            weapons.swap(0, remove_after);
        }
        weapons
    }

    pub(crate) fn snapshot(&mut self) {
        self.uber_states.snapshot();
        self.snapshot = Some(self.reach.clone());
    }

    pub(crate) fn restore_snapshot(&mut self) {
        self.uber_states.restore_snapshot();
        self.reach = mem::take(&mut self.snapshot).unwrap();
    }
}

#[derive(Debug, Default, Clone)]
struct Variables {
    booleans: FxHashMap<usize, bool>,
    integers: FxHashMap<usize, i32>,
    floats: FxHashMap<usize, OrderedFloat<f32>>,
    strings: FxHashMap<usize, String>,
}
impl Variables {
    fn set_boolean(&mut self, id: usize, value: bool) {
        self.booleans.insert(id, value);
    }
    fn set_integer(&mut self, id: usize, value: i32) {
        self.integers.insert(id, value);
    }
    fn set_float(&mut self, id: usize, value: OrderedFloat<f32>) {
        self.floats.insert(id, value);
    }
    fn set_string(&mut self, id: usize, value: String) {
        self.strings.insert(id, value);
    }
    fn get_boolean(&self, id: &usize) -> bool {
        self.booleans.get(id).copied().unwrap_or_default()
    }
    fn get_integer(&self, id: &usize) -> i32 {
        self.integers.get(id).copied().unwrap_or_default()
    }
    fn get_float(&self, id: &usize) -> OrderedFloat<f32> {
        self.floats.get(id).copied().unwrap_or_default()
    }
    fn get_string(&self, id: &usize) -> String {
        self.strings.get(id).cloned().unwrap_or_default()
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

        fn iter_item<I, T>(f: &mut fmt::Formatter<'_>, first: &mut bool, iter: I) -> fmt::Result
        where
            I: Iterator<Item = T>,
            T: Display,
        {
            iter.map(|name| item(f, first, name)).collect()
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
