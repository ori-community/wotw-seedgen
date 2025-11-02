use rustc_hash::FxHashMap;
use strum::VariantArray;
use wotw_seedgen_assets::UberStateValue;
use wotw_seedgen_data::{Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade};

use crate::{
    compile::{add_float, add_integer, store_boolean, store_float, store_integer},
    output::Event,
    simulate::{Simulate, UberStates, Variables},
};

pub trait Simulation: Sized {
    fn uber_states(&self) -> &UberStates;

    fn uber_states_mut(&mut self) -> &mut UberStates;

    fn variables(&self) -> &Variables;

    fn variables_mut(&mut self) -> &mut Variables;

    fn on_change(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        let _ = (uber_identifier, events);
    }

    #[inline]
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.uber_states().fetch(uber_identifier)
    }

    #[inline]
    fn fetch_boolean(&self, uber_identifier: UberIdentifier) -> bool {
        self.fetch(uber_identifier).as_boolean()
    }

    #[inline]
    fn fetch_integer(&self, uber_identifier: UberIdentifier) -> i32 {
        self.fetch(uber_identifier).as_integer()
    }

    #[inline]
    fn fetch_float(&self, uber_identifier: UberIdentifier) -> f32 {
        self.fetch(uber_identifier).as_float()
    }

    #[inline]
    fn simulate<T: Simulate<Self>>(&mut self, t: &T, events: &[Event]) -> T::Return {
        t.simulate(self, events)
    }

    #[inline]
    fn store_boolean(&mut self, uber_identifier: UberIdentifier, value: bool, events: &[Event]) {
        store_boolean(uber_identifier, value).simulate(self, events);
    }

    #[inline]
    fn store_integer(&mut self, uber_identifier: UberIdentifier, value: i32, events: &[Event]) {
        store_integer(uber_identifier, value).simulate(self, events);
    }

    #[inline]
    fn add_integer(&mut self, uber_identifier: UberIdentifier, add: i32, events: &[Event]) {
        add_integer(uber_identifier, add).simulate(self, events);
    }

    #[inline]
    fn store_float(&mut self, uber_identifier: UberIdentifier, value: f32, events: &[Event]) {
        store_float(uber_identifier, value).simulate(self, events);
    }

    #[inline]
    fn add_float(&mut self, uber_identifier: UberIdentifier, value: f32, events: &[Event]) {
        add_float(uber_identifier, value).simulate(self, events);
    }

    #[inline]
    fn store_spirit_light(&mut self, value: i32, events: &[Event]) {
        self.store_integer(UberIdentifier::SPIRIT_LIGHT, value, events);
    }

    #[inline]
    fn add_spirit_light(&mut self, add: i32, events: &[Event]) {
        self.add_integer(UberIdentifier::SPIRIT_LIGHT, add, events);
    }

    #[inline]
    fn store_gorlek_ore(&mut self, value: i32, events: &[Event]) {
        self.store_integer(UberIdentifier::GORLEK_ORE, value, events);
    }

    #[inline]
    fn add_gorlek_ore(&mut self, add: i32, events: &[Event]) {
        self.add_integer(UberIdentifier::GORLEK_ORE, add, events);
    }

    #[inline]
    fn store_keystones(&mut self, value: i32, events: &[Event]) {
        self.store_integer(UberIdentifier::KEYSTONES, value, events);
    }

    #[inline]
    fn add_keystones(&mut self, add: i32, events: &[Event]) {
        self.add_integer(UberIdentifier::KEYSTONES, add, events);
    }

    #[inline]
    fn store_shard_slots(&mut self, value: i32, events: &[Event]) {
        self.store_integer(UberIdentifier::SHARD_SLOTS, value, events);
    }

    #[inline]
    fn add_shard_slots(&mut self, add: i32, events: &[Event]) {
        self.add_integer(UberIdentifier::SHARD_SLOTS, add, events);
    }

    #[inline]
    fn store_max_health(&mut self, value: i32, events: &[Event]) {
        self.store_integer(UberIdentifier::MAX_HEALTH, value, events);
    }

    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    fn add_max_health(&mut self, add: i32, events: &[Event]) {
        self.add_integer(UberIdentifier::MAX_HEALTH, add, events);
    }

    #[inline]
    fn store_max_energy(&mut self, value: f32, events: &[Event]) {
        self.store_float(UberIdentifier::MAX_ENERGY, value, events);
    }

    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    fn add_max_energy(&mut self, add: f32, events: &[Event]) {
        self.add_float(UberIdentifier::MAX_ENERGY, add, events);
    }

    #[inline]
    fn store_skill(&mut self, skill: Skill, value: bool, events: &[Event]) {
        self.store_boolean(skill.uber_identifier(), value, events);
    }

    #[inline]
    fn store_shard(&mut self, shard: Shard, value: bool, events: &[Event]) {
        self.store_boolean(shard.uber_identifier(), value, events);
    }

    #[inline]
    fn store_teleporter(&mut self, teleporter: Teleporter, value: bool, events: &[Event]) {
        self.store_boolean(teleporter.uber_identifier(), value, events);
    }

    #[inline]
    fn store_clean_water(&mut self, value: bool, events: &[Event]) {
        self.store_boolean(UberIdentifier::CLEAN_WATER, value, events);
    }

    #[inline]
    fn store_weapon_upgrade(
        &mut self,
        weapon_upgrade: WeaponUpgrade,
        value: bool,
        events: &[Event],
    ) {
        self.store_integer(weapon_upgrade.uber_identifier(), i32::from(value), events);
    }

    #[inline]
    fn spirit_light(&self) -> i32 {
        self.uber_states()
            .fetch(UberIdentifier::SPIRIT_LIGHT)
            .expect_integer()
    }

    #[inline]
    fn gorlek_ore(&self) -> i32 {
        self.uber_states()
            .fetch(UberIdentifier::GORLEK_ORE)
            .expect_integer()
    }

    #[inline]
    fn keystones(&self) -> i32 {
        self.uber_states()
            .fetch(UberIdentifier::KEYSTONES)
            .expect_integer()
    }

    #[inline]
    fn shard_slots(&self) -> i32 {
        self.uber_states()
            .fetch(UberIdentifier::SHARD_SLOTS)
            .expect_integer()
    }

    #[inline]
    fn base_max_health(&self) -> f32 {
        self.uber_states()
            .fetch(UberIdentifier::MAX_HEALTH)
            .expect_integer() as f32
    }

    /// Returns the maximum health
    ///
    /// One visual orb in the game represents 10 health
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen_seed_language::simulate::UberStates;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen_seed_language::simulate::{WorldState, Simulation};
    ///
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// let world_state = WorldState::new(uber_states);
    /// assert_eq!(world_state.max_health(), 30.0);
    /// ```
    #[inline]
    fn max_health(&self) -> f32 {
        self.base_max_health() + self.shard(Shard::Vitality) as u8 as f32 * 10.
    }

    #[inline]
    fn base_max_energy(&self) -> f32 {
        self.uber_states()
            .fetch(UberIdentifier::MAX_ENERGY)
            .expect_float()
    }

    /// Returns the maximum energy
    ///
    /// One visual orb in the game represents 1 energy
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen_seed_language::simulate::UberStates;
    /// # use wotw_seedgen_static_assets::UBER_STATE_DATA;
    /// use wotw_seedgen_seed_language::simulate::{WorldState, Simulation};
    ///
    /// # let uber_states = UberStates::new(&*UBER_STATE_DATA);
    /// let world_state = WorldState::new(uber_states);
    /// assert_eq!(world_state.max_energy(), 3.0);
    /// ```
    #[inline]
    fn max_energy(&self) -> f32 {
        self.base_max_energy() + self.shard(Shard::Energy) as u8 as f32
    }

    #[inline]
    fn skill(&self, skill: Skill) -> bool {
        self.uber_states()
            .fetch(skill.uber_identifier())
            .expect_boolean()
    }

    #[inline]
    fn shard(&self, shard: Shard) -> bool {
        self.uber_states()
            .fetch(shard.uber_identifier())
            .expect_boolean()
    }

    #[inline]
    fn teleporter(&self, teleporter: Teleporter) -> bool {
        self.uber_states()
            .fetch(teleporter.uber_identifier())
            .expect_boolean()
    }

    #[inline]
    fn clean_water(&self) -> bool {
        self.uber_states()
            .fetch(UberIdentifier::CLEAN_WATER)
            .expect_boolean()
    }

    #[inline]
    fn weapon_upgrade(&self, weapon_upgrade: WeaponUpgrade) -> bool {
        self.uber_states()
            .fetch(weapon_upgrade.uber_identifier())
            .expect_integer()
            > 0
    }

    fn skills(&self) -> impl Iterator<Item = Skill> + '_ {
        Skill::VARIANTS
            .iter()
            .copied()
            .filter(|skill| self.skill(*skill))
    }

    fn shards(&self) -> impl Iterator<Item = Shard> + '_ {
        Shard::VARIANTS
            .iter()
            .copied()
            .filter(|shard| self.shard(*shard))
    }

    fn teleporters(&self) -> impl Iterator<Item = Teleporter> + '_ {
        Teleporter::VARIANTS
            .iter()
            .copied()
            .filter(|teleporter| self.teleporter(*teleporter))
    }

    fn weapon_upgrades(&self) -> impl Iterator<Item = WeaponUpgrade> + '_ {
        WeaponUpgrade::VARIANTS
            .iter()
            .copied()
            .filter(|weapon_upgrade| self.weapon_upgrade(*weapon_upgrade))
    }

    fn snapshot(&mut self, id: u8) {
        self.uber_states_mut().snapshot(id);
    }

    fn take_snapshot(&mut self, id: u8) -> FxHashMap<UberIdentifier, UberStateValue> {
        self.uber_states_mut().take_snapshot(id)
    }

    fn restore_snapshot(&mut self, id: u8) {
        self.uber_states_mut().restore_snapshot(id);
    }
}
