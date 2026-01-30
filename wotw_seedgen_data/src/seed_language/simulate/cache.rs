use std::hash::Hash;

use rustc_hash::{FxBuildHasher, FxHashSet};
use strum::VariantArray;

use crate::{
    assets::UberStateValue,
    seed_language::{
        output::Event,
        simulate::{Simulation, Snapshot, Variables},
    },
    CommonUberIdentifier, Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
};

#[derive(Debug, Clone)]
pub struct SimulationCache<S> {
    simulation: S,
    cache: Cache,
    snapshot: Option<Cache>,
}

impl<S: Simulation> SimulationCache<S> {
    pub fn new(simulation: S) -> Self {
        let mut s = Self {
            simulation,
            cache: Cache::default(),
            snapshot: None,
        };
        s.init_cache();
        s
    }

    fn init_cache(&mut self) {
        self.cache.spirit_light = self.simulation.spirit_light();
        self.cache.gorlek_ore = self.simulation.gorlek_ore();
        self.cache.keystones = self.simulation.keystones();
        self.cache.shard_slots = self.simulation.shard_slots();
        self.cache.base_max_health = self.simulation.base_max_health();
        self.cache.base_max_energy = self.simulation.base_max_energy();
        self.cache.skills = self.simulation.skills().collect();
        self.cache.shards = self.simulation.shards().collect();
        self.cache.teleporters = self.simulation.teleporters().collect();
        self.cache.clean_water = self.simulation.clean_water();
        self.cache.weapon_upgrades = self.simulation.weapon_upgrades().collect();
        // TODO too lazy to do this better... I don't think doors belong in states because they must not change anyway or things would break
        for (index, door) in self.cache.doors.iter_mut().enumerate() {
            *door = self.simulation.fetch_integer(UberIdentifier {
                group: 27,
                member: index as i32 + 1,
            });
        }
    }
}

#[derive(Debug, Clone)]
struct Cache {
    spirit_light: i32,
    gorlek_ore: i32,
    keystones: i32,
    shard_slots: i32,
    base_max_health: f32,
    base_max_energy: f32,
    skills: FxHashSet<Skill>,
    shards: FxHashSet<Shard>,
    teleporters: FxHashSet<Teleporter>,
    clean_water: bool,
    weapon_upgrades: FxHashSet<WeaponUpgrade>,
    doors: [i32; 32],
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            spirit_light: 0,
            gorlek_ore: 0,
            keystones: 0,
            shard_slots: 0,
            base_max_health: 0.,
            base_max_energy: 0.,
            skills: FxHashSet::with_capacity_and_hasher(Skill::VARIANTS.len(), FxBuildHasher),
            shards: FxHashSet::with_capacity_and_hasher(Shard::VARIANTS.len(), FxBuildHasher),
            teleporters: FxHashSet::with_capacity_and_hasher(
                Teleporter::VARIANTS.len(),
                FxBuildHasher,
            ),
            clean_water: false,
            weapon_upgrades: FxHashSet::with_capacity_and_hasher(
                WeaponUpgrade::VARIANTS.len(),
                FxBuildHasher,
            ),
            doors: [0; 32],
        }
    }
}

impl Cache {
    fn store(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) {
        let Some(uber_identifier) = CommonUberIdentifier::from_uber_identifier(uber_identifier)
        else {
            return;
        };

        match uber_identifier {
            CommonUberIdentifier::SpiritLight => self.spirit_light = value.expect_integer(),
            CommonUberIdentifier::GorlekOre => self.gorlek_ore = value.expect_integer(),
            CommonUberIdentifier::Keystones => self.keystones = value.expect_integer(),
            CommonUberIdentifier::ShardSlots => self.shard_slots = value.expect_integer(),
            CommonUberIdentifier::CleanWater => self.clean_water = value.expect_boolean(),
            CommonUberIdentifier::MaxHealth => self.base_max_health = value.expect_integer() as f32,
            CommonUberIdentifier::MaxEnergy => self.base_max_energy = value.expect_float(),
            CommonUberIdentifier::Health | CommonUberIdentifier::Energy => {}
            CommonUberIdentifier::Skill(skill) => {
                update_set(&mut self.skills, skill, value.expect_boolean())
            }
            CommonUberIdentifier::Shard(shard) => {
                update_set(&mut self.shards, shard, value.expect_boolean())
            }
            CommonUberIdentifier::Teleporter(teleporter) => {
                update_set(&mut self.teleporters, teleporter, value.expect_boolean())
            }
            CommonUberIdentifier::WeaponUpgrade(weapon_upgrade) => update_set(
                &mut self.weapon_upgrades,
                weapon_upgrade,
                value.expect_integer() > 0,
            ),
        }
    }
}

impl<S: Simulation> Simulation for SimulationCache<S> {
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.simulation.fetch(uber_identifier)
    }

    fn store_impl(
        &mut self,
        uber_identifier: UberIdentifier,
        value: UberStateValue,
    ) -> impl Iterator<Item = usize> + '_ {
        self.cache.store(uber_identifier, value);
        self.simulation.store_impl(uber_identifier, value)
    }

    fn on_change(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        self.simulation.on_change(uber_identifier, events);
    }

    fn variables(&self) -> &Variables {
        self.simulation.variables()
    }

    fn variables_mut(&mut self) -> &mut Variables {
        self.simulation.variables_mut()
    }

    fn store_spirit_light(&mut self, value: i32, events: &[Event]) {
        self.cache.spirit_light = value;
        self.simulation
            .store_integer(UberIdentifier::SPIRIT_LIGHT, value, events);
    }

    fn add_spirit_light(&mut self, add: i32, events: &[Event]) {
        self.cache.spirit_light += add;
        self.simulation
            .add_integer(UberIdentifier::SPIRIT_LIGHT, add, events);
    }

    fn store_gorlek_ore(&mut self, value: i32, events: &[Event]) {
        self.cache.gorlek_ore = value;
        self.simulation
            .store_integer(UberIdentifier::GORLEK_ORE, value, events);
    }

    fn add_gorlek_ore(&mut self, add: i32, events: &[Event]) {
        self.cache.gorlek_ore += add;
        self.simulation
            .add_integer(UberIdentifier::GORLEK_ORE, add, events);
    }

    fn store_keystones(&mut self, value: i32, events: &[Event]) {
        self.cache.keystones = value;
        self.simulation
            .store_integer(UberIdentifier::KEYSTONES, value, events);
    }

    fn add_keystones(&mut self, add: i32, events: &[Event]) {
        self.cache.keystones += add;
        self.simulation
            .add_integer(UberIdentifier::KEYSTONES, add, events);
    }

    fn store_shard_slots(&mut self, value: i32, events: &[Event]) {
        self.cache.shard_slots = value;
        self.simulation
            .store_integer(UberIdentifier::SHARD_SLOTS, value, events);
    }

    fn add_shard_slots(&mut self, add: i32, events: &[Event]) {
        self.cache.shard_slots += add;
        self.simulation
            .add_integer(UberIdentifier::SHARD_SLOTS, add, events);
    }

    fn store_max_health(&mut self, value: i32, events: &[Event]) {
        self.cache.base_max_health = value as f32;
        self.simulation
            .store_integer(UberIdentifier::MAX_HEALTH, value, events);
    }

    fn add_max_health(&mut self, add: i32, events: &[Event]) {
        self.cache.base_max_health += add as f32;
        self.simulation
            .add_integer(UberIdentifier::MAX_HEALTH, add, events);
    }

    fn store_max_energy(&mut self, value: f32, events: &[Event]) {
        self.cache.base_max_energy = value;
        self.simulation
            .store_float(UberIdentifier::MAX_ENERGY, value, events);
    }

    fn add_max_energy(&mut self, add: f32, events: &[Event]) {
        self.cache.base_max_energy += add;
        self.simulation
            .add_float(UberIdentifier::MAX_ENERGY, add, events);
    }

    fn store_skill(&mut self, skill: Skill, value: bool, events: &[Event]) {
        update_set(&mut self.cache.skills, skill, value);
        self.simulation
            .store_boolean(skill.uber_identifier(), value, events);
    }

    fn store_shard(&mut self, shard: Shard, value: bool, events: &[Event]) {
        update_set(&mut self.cache.shards, shard, value);
        self.simulation
            .store_boolean(shard.uber_identifier(), value, events);
    }

    fn store_teleporter(&mut self, teleporter: Teleporter, value: bool, events: &[Event]) {
        update_set(&mut self.cache.teleporters, teleporter, value);
        self.simulation
            .store_boolean(teleporter.uber_identifier(), value, events);
    }

    fn store_clean_water(&mut self, value: bool, events: &[Event]) {
        self.cache.clean_water = value;
        self.simulation
            .store_boolean(UberIdentifier::CLEAN_WATER, value, events);
    }

    fn store_weapon_upgrade(
        &mut self,
        weapon_upgrade: WeaponUpgrade,
        value: bool,
        events: &[Event],
    ) {
        update_set(&mut self.cache.weapon_upgrades, weapon_upgrade, value);
        self.simulation
            .store_integer(weapon_upgrade.uber_identifier(), i32::from(value), events);
    }

    fn spirit_light(&self) -> i32 {
        self.cache.spirit_light
    }

    fn gorlek_ore(&self) -> i32 {
        self.cache.gorlek_ore
    }

    fn keystones(&self) -> i32 {
        self.cache.keystones
    }

    fn shard_slots(&self) -> i32 {
        self.cache.shard_slots
    }

    fn base_max_health(&self) -> f32 {
        self.cache.base_max_health
    }

    fn base_max_energy(&self) -> f32 {
        self.cache.base_max_energy
    }

    fn skill(&self, skill: Skill) -> bool {
        self.cache.skills.contains(&skill)
    }

    fn shard(&self, shard: Shard) -> bool {
        self.cache.shards.contains(&shard)
    }

    fn teleporter(&self, teleporter: Teleporter) -> bool {
        self.cache.teleporters.contains(&teleporter)
    }

    fn clean_water(&self) -> bool {
        self.cache.clean_water
    }

    fn weapon_upgrade(&self, weapon_upgrade: WeaponUpgrade) -> bool {
        self.cache.weapon_upgrades.contains(&weapon_upgrade)
    }

    fn skills(&self) -> impl Iterator<Item = Skill> + '_ {
        self.cache.skills.iter().copied()
    }

    fn shards(&self) -> impl Iterator<Item = Shard> + '_ {
        self.cache.shards.iter().copied()
    }

    fn teleporters(&self) -> impl Iterator<Item = Teleporter> + '_ {
        self.cache.teleporters.iter().copied()
    }

    fn weapon_upgrades(&self) -> impl Iterator<Item = WeaponUpgrade> + '_ {
        self.cache.weapon_upgrades.iter().copied()
    }
}

impl<S: Snapshot> Snapshot for SimulationCache<S> {
    fn snapshot(&mut self) {
        self.snapshot = Some(self.cache.clone());
        self.simulation.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.cache = self.snapshot.take().unwrap();
        self.simulation.restore_snapshot();
    }
}

fn update_set<V: Eq + Hash>(set: &mut FxHashSet<V>, item: V, value: bool) {
    if value {
        set.insert(item);
    } else {
        set.remove(&item);
    }
}
