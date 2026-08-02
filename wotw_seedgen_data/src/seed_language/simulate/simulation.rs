use crate::{
    assets::UberStateValue,
    seed_language::{
        output::{CommandsOutput, Trigger},
        simulate::{
            condition_values::ConditionValues, set_uber_state, Heap, Simulate, Stack, UberStates,
        },
    },
    Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
};
use strum::VariantArray;

pub trait Simulation: Sized {
    fn stack(&self) -> &Stack;

    fn stack_mut(&mut self) -> &mut Stack;

    fn heap(&self) -> &Heap;

    fn heap_mut(&mut self) -> &mut Heap;

    fn uber_states(&self) -> &UberStates;

    fn uber_states_mut(&mut self) -> &mut UberStates;

    fn condition_values(&mut self) -> &mut ConditionValues;

    #[inline]
    fn register_trigger(&mut self, trigger: &mut Trigger, event_index: usize) {
        trigger.register(event_index, self);
    }

    #[inline]
    fn on_change(&mut self, uber_identifier: UberIdentifier, output: &CommandsOutput) {
        let _ = (uber_identifier, output);
    }

    #[inline]
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.uber_states().fetch(uber_identifier)
    }

    #[inline]
    fn store_impl(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) -> &[usize] {
        self.uber_states_mut().store(uber_identifier, value)
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
    fn simulate<T: Simulate<Self>>(&mut self, t: &T, output: &CommandsOutput) -> T::Return {
        t.simulate(self, output)
    }

    #[inline]
    fn store_boolean(
        &mut self,
        uber_identifier: UberIdentifier,
        value: bool,
        output: &CommandsOutput,
    ) {
        set_uber_state(self, output, uber_identifier, value.into(), true);
    }

    #[inline]
    fn store_integer(
        &mut self,
        uber_identifier: UberIdentifier,
        value: i32,
        output: &CommandsOutput,
    ) {
        set_uber_state(self, output, uber_identifier, value.into(), true);
    }

    #[inline]
    fn add_integer(&mut self, uber_identifier: UberIdentifier, add: i32, output: &CommandsOutput) {
        self.store_integer(
            uber_identifier,
            self.fetch_integer(uber_identifier) + add,
            output,
        );
    }

    #[inline]
    fn store_float(
        &mut self,
        uber_identifier: UberIdentifier,
        value: f32,
        output: &CommandsOutput,
    ) {
        set_uber_state(self, output, uber_identifier, value.into(), true);
    }

    #[inline]
    fn add_float(&mut self, uber_identifier: UberIdentifier, add: f32, output: &CommandsOutput) {
        // add_float(uber_identifier, add).simulate(self, output);
        self.store_float(
            uber_identifier,
            self.fetch_float(uber_identifier) + add,
            output,
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
    fn store_spirit_light(&mut self, value: i32, output: &CommandsOutput) {
        self.store_integer(UberIdentifier::SPIRIT_LIGHT, value, output);
    }

    #[inline]
    fn add_spirit_light(&mut self, add: i32, output: &CommandsOutput) {
        self.add_integer(UberIdentifier::SPIRIT_LIGHT, add, output);
    }

    #[inline]
    fn store_gorlek_ore(&mut self, value: i32, output: &CommandsOutput) {
        self.store_integer(UberIdentifier::GORLEK_ORE, value, output);
    }

    #[inline]
    fn add_gorlek_ore(&mut self, add: i32, output: &CommandsOutput) {
        self.add_integer(UberIdentifier::GORLEK_ORE, add, output);
    }

    #[inline]
    fn store_keystones(&mut self, value: i32, output: &CommandsOutput) {
        self.store_integer(UberIdentifier::KEYSTONES, value, output);
    }

    #[inline]
    fn add_keystones(&mut self, add: i32, output: &CommandsOutput) {
        self.add_integer(UberIdentifier::KEYSTONES, add, output);
    }

    #[inline]
    fn store_shard_slots(&mut self, value: i32, output: &CommandsOutput) {
        self.store_integer(UberIdentifier::SHARD_SLOTS, value, output);
    }

    #[inline]
    fn add_shard_slots(&mut self, add: i32, output: &CommandsOutput) {
        self.add_integer(UberIdentifier::SHARD_SLOTS, add, output);
    }

    #[inline]
    fn store_base_max_health(&mut self, value: i32, output: &CommandsOutput) {
        self.store_integer(UberIdentifier::BASE_MAX_HEALTH, value, output);
    }

    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    fn add_base_max_health(&mut self, add: i32, output: &CommandsOutput) {
        self.add_integer(UberIdentifier::BASE_MAX_HEALTH, add, output);
    }

    #[inline]
    fn store_base_max_energy(&mut self, value: f32, output: &CommandsOutput) {
        self.store_float(UberIdentifier::BASE_MAX_ENERGY, value, output);
    }

    // TODO check that uses scaled correctly since they might have used the number of fragments before
    #[inline]
    fn add_base_max_energy(&mut self, add: f32, output: &CommandsOutput) {
        self.add_float(UberIdentifier::BASE_MAX_ENERGY, add, output);
    }

    #[inline]
    fn store_skill(&mut self, skill: Skill, value: bool, output: &CommandsOutput) {
        self.store_boolean(skill.uber_identifier(), value, output);
    }

    #[inline]
    fn store_shard(&mut self, shard: Shard, value: bool, output: &CommandsOutput) {
        self.store_boolean(shard.uber_identifier(), value, output);
    }

    #[inline]
    fn store_teleporter(&mut self, teleporter: Teleporter, value: bool, output: &CommandsOutput) {
        self.store_boolean(teleporter.uber_identifier(), value, output);
    }

    #[inline]
    fn store_clean_water(&mut self, value: bool, output: &CommandsOutput) {
        self.store_boolean(UberIdentifier::CLEAN_WATER, value, output);
    }

    #[inline]
    fn store_weapon_upgrade(
        &mut self,
        weapon_upgrade: WeaponUpgrade,
        value: bool,
        output: &CommandsOutput,
    ) {
        self.store_integer(weapon_upgrade.uber_identifier(), i32::from(value), output);
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
        self.fetch(UberIdentifier::BASE_MAX_HEALTH).expect_integer() as f32
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
        self.base_max_health() + f32::from(u8::from(self.shard(Shard::Vitality))) * 10.
    }

    #[inline]
    fn base_max_energy(&self) -> f32 {
        self.fetch(UberIdentifier::BASE_MAX_ENERGY).expect_float()
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
        self.base_max_energy() + f32::from(u8::from(self.shard(Shard::Energy)))
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

    // mirrors https://github.com/ori-community/wotw-rando-client/blob/v5/src/Randomizer/uber_states/uber_state_intercepts.cpp
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
