use crate::{
    assets::UberStateValue,
    seed_language::{
        output::{Event, Trigger},
        simulate::{condition_values::ConditionValues, set_uber_state, Simulate, Variables},
    },
    Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
};
use strum::VariantArray;

pub trait Simulation: Sized {
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue;

    fn store_impl(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) -> &[usize];

    fn on_change(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        let _ = (uber_identifier, events);
    }

    fn variables(&self) -> &Variables;

    fn variables_mut(&mut self) -> &mut Variables;

    fn condition_values(&mut self) -> &mut ConditionValues;

    fn register_trigger(&mut self, trigger: &mut Trigger) {
        if let Trigger::Condition(condition) = trigger {
            let initial_value = condition.condition.simulate(self, &[]);
            let id = self.condition_values().register(initial_value);
            condition.id = Some(id);
        }
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
        set_uber_state(self, events, uber_identifier, value.into(), true);
    }

    #[inline]
    fn store_integer(&mut self, uber_identifier: UberIdentifier, value: i32, events: &[Event]) {
        set_uber_state(self, events, uber_identifier, value.into(), true);
    }

    #[inline]
    fn add_integer(&mut self, uber_identifier: UberIdentifier, add: i32, events: &[Event]) {
        self.store_integer(
            uber_identifier,
            self.fetch_integer(uber_identifier) + add,
            events,
        );
    }

    #[inline]
    fn store_float(&mut self, uber_identifier: UberIdentifier, value: f32, events: &[Event]) {
        set_uber_state(self, events, uber_identifier, value.into(), true);
    }

    #[inline]
    fn add_float(&mut self, uber_identifier: UberIdentifier, add: f32, events: &[Event]) {
        // add_float(uber_identifier, add).simulate(self, events);
        self.store_float(
            uber_identifier,
            self.fetch_float(uber_identifier) + add,
            events,
        );
    }

    #[inline]
    fn loc_data_condition_met(&self, uber_identifier: UberIdentifier, value: Option<i32>) -> bool {
        match value {
            None => self.fetch_boolean(uber_identifier),
            Some(value) => self.fetch_integer(uber_identifier) >= value,
        }
    }

    // TODO less hardcoded solution? Entrances are not allowed to change anyway, they just have to be set at the start
    #[inline]
    fn entrance_condition_met(&self, uber_identifier: UberIdentifier, value: i32) -> bool {
        self.fetch_integer(uber_identifier) == value
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
        self.fetch(UberIdentifier::SPIRIT_LIGHT).expect_integer()
    }

    #[inline]
    fn gorlek_ore(&self) -> i32 {
        self.fetch(UberIdentifier::GORLEK_ORE).expect_integer()
    }

    #[inline]
    fn keystones(&self) -> i32 {
        self.fetch(UberIdentifier::KEYSTONES).expect_integer()
    }

    #[inline]
    fn shard_slots(&self) -> i32 {
        self.fetch(UberIdentifier::SHARD_SLOTS).expect_integer()
    }

    #[inline]
    fn base_max_health(&self) -> f32 {
        self.fetch(UberIdentifier::MAX_HEALTH).expect_integer() as f32
    }

    /// Returns the maximum health
    ///
    /// One visual orb in the game represents 10 health
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen_data::seed_language::simulate::{WorldState, Simulation};
    ///
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let world_state = WorldState::new(uber_states, &mut events);
    /// assert_eq!(world_state.max_health(), 30.0);
    /// ```
    #[inline]
    fn max_health(&self) -> f32 {
        self.base_max_health() + self.shard(Shard::Vitality) as u8 as f32 * 10.
    }

    #[inline]
    fn base_max_energy(&self) -> f32 {
        self.fetch(UberIdentifier::MAX_ENERGY).expect_float()
    }

    /// Returns the maximum energy
    ///
    /// One visual orb in the game represents 1 energy
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen_data::seed_language::simulate::UberStates;
    /// # use wotw_seedgen_data::assets::{AssetFileAccess, LocData, StateData, TEST_ASSETS};
    /// use wotw_seedgen_data::seed_language::simulate::{WorldState, Simulation};
    ///
    /// # let uber_states = TEST_ASSETS.uber_states.clone();
    /// # let mut events = [];
    /// let world_state = WorldState::new(uber_states, &mut events);
    /// assert_eq!(world_state.max_energy(), 3.0);
    /// ```
    #[inline]
    fn max_energy(&self) -> f32 {
        self.base_max_energy() + self.shard(Shard::Energy) as u8 as f32
    }

    #[inline]
    fn skill(&self, skill: Skill) -> bool {
        self.fetch(skill.uber_identifier()).expect_boolean()
    }

    // TODO support spawning with reduced shard slots?
    #[inline]
    fn shard(&self, shard: Shard) -> bool {
        self.fetch(shard.uber_identifier()).expect_boolean()
    }

    #[inline]
    fn teleporter(&self, teleporter: Teleporter) -> bool {
        self.fetch(teleporter.uber_identifier()).expect_boolean()
    }

    #[inline]
    fn clean_water(&self) -> bool {
        self.fetch(UberIdentifier::CLEAN_WATER).expect_boolean()
    }

    #[inline]
    fn weapon_upgrade(&self, weapon_upgrade: WeaponUpgrade) -> bool {
        self.fetch(weapon_upgrade.uber_identifier())
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

    // mirrors https://github.com/ori-community/wotw-rando-client/blob/v5/projects/Randomizer/uber_states/uber_state_intercepts.cpp
    fn should_prevent_store(&self, uber_identifier: UberIdentifier, value: UberStateValue) -> bool {
        const WELLSPRING_QUEST: UberIdentifier = UberIdentifier::new(937, 34641);
        const KU_QUEST: UberIdentifier = UberIdentifier::new(14019, 34504);

        match uber_identifier {
            WELLSPRING_QUEST => self.fetch(WELLSPRING_QUEST) >= value.as_integer(),
            KU_QUEST => value <= 4,
            _ => false,
        }
    }
}
