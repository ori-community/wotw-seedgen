use std::fmt;

use serde::{Serialize, Deserialize};

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{item::{Item, Resource, Shard, Skill}, settings::{Difficulty, WorldSettings, logical_difficulty}, util::Orbs};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: FxHashMap<Item, u32>,  // TODO what would a switch to usize do here? Often we need a usize and end up casting this
}
impl Inventory {
    pub fn grant(&mut self, mut item: Item, mut amount: u32) {
        if amount == 0 { return }
        let single_instance = item.is_single_instance();  // A `Sword AND Sword` requirement should still just require one Sword
        if single_instance && amount > 1 {
            log::warn!("Granted {} more than once, but that item can only be aquired once...", item);
        }
        if let Item::SpiritLight(stacked_amount) = item {
            amount *= stacked_amount;
            item = Item::SpiritLight(1);
        }
        let prior = self.items.entry(item).or_insert(0);
        if single_instance {
            *prior = amount;
        } else {
            *prior += amount;
        }
    }
    pub fn remove(&mut self, item: &Item, amount: u32) {
        if let Some(prior) = self.items.get_mut(item) {
            if amount >= *prior {
                self.items.remove(item);
            } else {
                *prior -= amount;
            }
        }
    }

    pub fn has(&self, item: &Item, amount: u32) -> bool {
        self.items.get(item).map_or(false, |owned| *owned >= amount)
    }
    pub fn get(&self, item: &Item) -> u32 {
        self.items.get(item).copied().unwrap_or_default()
    }

    // TODO I like the way Pool solved the spirit light issue, maybe inventory should adopt that?
    fn item_size(item: &Item, amount: u32) -> u32 {
        match item {
            // Note that requirement::solutions has logic based on this formula (check_slot_limits)
            Item::SpiritLight(stacked_amount) => (amount * stacked_amount + 39) / 40,  // this will usually demand more than necessary, but with the placeholder system that shouldn't be a problem
            _ => amount,
        }
    }
    pub fn item_count(&self) -> u32 {
        self.items.iter().map(|(item, amount)| Self::item_size(item, *amount)).sum()
    }
    pub fn world_item_count(&self) -> u32 {
        self.items.iter()
            .filter(|&(item, _)| !item.is_multiworld_spread())
            .map(|(item, amount)| Self::item_size(item, *amount))
            .sum()
    }

    pub fn cost(&self) -> u32 {
        self.items.iter().map(|(item, amount)| item.cost() * *amount).sum()
    }

    pub fn contains(&self, other: &Inventory) -> bool {
        if self.items.len() < other.items.len() { return false }
        other.items.iter().all(|(item, amount)| self.has(item, *amount))
    }

    pub fn merge(&mut self, other: Inventory) {
        for (item, amount) in other.items {
            self.grant(item, amount);
        }
    }

    pub fn max_health(&self, difficulty: Difficulty) -> f32 {
        let mut health = (self.get(&Item::Resource(Resource::HealthFragment)) * 5) as f32;
        if difficulty >= logical_difficulty::VITALITY && self.has(&Item::Shard(Shard::Vitality), 1) { health += 10.0; }
        health
    }
    pub fn max_energy(&self, difficulty: Difficulty) -> f32 {
        let mut energy = self.get(&Item::Resource(Resource::EnergyFragment)) as f32 * 0.5;
        if difficulty >= logical_difficulty::ENERGY_SHARD && self.has(&Item::Shard(Shard::Energy), 1) { energy += 1.0; }
        energy
    }
    pub fn max_orbs(&self, difficulty: Difficulty) -> Orbs {
        Orbs {
            energy: self.max_energy(difficulty),
            health: self.max_health(difficulty),
        }
    }

    /// Replenish health, but don't exceed the inventory's maximum health
    pub fn heal(&self, orbs: &mut Orbs, amount: f32, difficulty: Difficulty) {
        orbs.health = (orbs.health + amount).min(self.max_health(difficulty));
    }
    /// Replenish energy, but don't exceed the inventory's maximum energy
    pub fn recharge(&self, orbs: &mut Orbs, amount: f32, difficulty: Difficulty) {
        orbs.energy = (orbs.energy + amount).min(self.max_energy(difficulty));
    }

    pub fn damage_mod(&self, flying_target: bool, bow: bool, settings: &WorldSettings) -> f32 {
        let mut damage_mod = 1.0;

        if settings.difficulty >= logical_difficulty::DAMAGE_BUFFS {
            if self.has(&Item::Skill(Skill::GladesAncestralLight), 1) { damage_mod += 0.25; }
            if self.has(&Item::Skill(Skill::InkwaterAncestralLight), 1) { damage_mod += 0.25; }

            let mut slots = self.get(&Item::Resource(Resource::ShardSlot));
            let mut splinter = false;

            if flying_target && slots > 0 && self.has(&Item::Shard(Shard::Wingclip), 1) { damage_mod += 1.0; slots -= 1; }
            if slots > 0 && bow && self.has(&Item::Shard(Shard::Splinter), 1) { splinter = true; slots -= 1; }
            if slots > 0 && self.has(&Item::Shard(Shard::SpiritSurge), 1) { damage_mod += (self.get(&Item::SpiritLight(1)) / 10000) as f32; slots -= 1; }
            if slots > 0 && self.has(&Item::Shard(Shard::LastStand), 1) { damage_mod += 0.2; slots -= 1; }
            if slots > 0 && self.has(&Item::Shard(Shard::Reckless), 1) { damage_mod += 0.15; slots -= 1; }
            if slots > 0 && self.has(&Item::Shard(Shard::Lifeforce), 1) { damage_mod += 0.1; slots -= 1; }
            if slots > 0 && self.has(&Item::Shard(Shard::Finesse), 1) { damage_mod += 0.05; }
            if splinter { damage_mod *= 1.5; }  // Splinter stacks multiplicatively where other buffs stack additively
        }

        damage_mod
    }
    pub fn energy_mod(&self, settings: &WorldSettings) -> f32 {
        let mut energy_mod = 1.0;
        if settings.difficulty < Difficulty::Unsafe { energy_mod *= 2.0; }
        else if self.has(&Item::Shard(Shard::Overcharge), 1) { energy_mod *= 0.5; }
        energy_mod
    }
    pub fn defense_mod(&self, settings: &WorldSettings) -> f32 {
        let mut defense_mod = if settings.difficulty >= logical_difficulty::RESILIENCE && self.has(&Item::Shard(Shard::Resilience), 1) { 0.9 } else { 1.0 };
        if settings.hard { defense_mod *= 2.0; }
        defense_mod
    }

    pub fn use_cost(&self, weapon: Skill, settings: &WorldSettings) -> f32 {
        weapon.energy_cost() * self.energy_mod(settings)
    }
    /// Returns the damage and cost of the weapon after all modifiers
    pub fn weapon_stats(&self, weapon: Skill, flying_target: bool, settings: &WorldSettings) -> (f32, f32) {
        let damage_mod = self.damage_mod(flying_target, matches!(weapon, Skill::Bow), settings);
        let damage = weapon.damage(settings) * damage_mod + weapon.burn_damage();
        let cost = weapon.energy_cost() * self.energy_mod(settings);
        (damage, cost)
    }
    /// Returns the energy required to destroy the target with the given weapon
    pub fn destroy_cost_with(&self, target_health: f32, weapon: Skill, flying_target: bool, settings: &WorldSettings) -> f32 {
        let (damage, cost) = self.weapon_stats(weapon, flying_target, settings);
        (target_health / damage).ceil() * cost
    }
    /// Returns the energy required to destroy the target with the given combination of weapons, or `None` if `weapons` is empty
    /// 
    /// We optimize based on the assumption that `weapons` has energy-less weapons in front
    fn destroy_cost_with_any_of<const N: usize>(&self, mut target_health: f32, weapons: SmallVec<[Skill; N]>, flying_target: bool, settings: &WorldSettings) -> Option<f32> {
        if weapons.first()?.energy_cost() == 0.0 { return Some(0.0) }

        let weapon_stats = weapons.into_iter().map(|weapon| self.weapon_stats(weapon, flying_target, settings)).collect::<SmallVec<[_; 9]>>();

        // Use the best weapon as long as it doesn't "waste" any damage
        use decorum::cmp::FloatOrd;
        let ((damage, mut cost), _) = weapon_stats.iter()
            .map(|(damage, cost)| ((*damage, *cost), damage / cost))
            .max_by(|(_, dpe_a), (_, dpe_b)| dpe_a.float_cmp(dpe_b))?;
        let optimal_hits = (target_health / damage).floor();
        target_health -= optimal_hits * damage;
        cost *= optimal_hits;

        // Figure out the best weapon to deal the last bit of damage
        cost += weapon_stats.into_iter().map(|(damage, cost)| ((target_health / damage).ceil() * cost)).min_by(f32::float_cmp)?;

        // On arbitrary energy costs and damage amounts this procedure might choose suboptimal weapons to use, but for the defaults it should be exhaustive

        Some(cost)
    }
    /// Returns the energy required to destroy the target, or `None` if no weapons are available to attack the target
    pub fn destroy_cost<const TARGET_IS_WALL: bool>(&self, target_health: f32, flying_target: bool, settings: &WorldSettings) -> Option<f32> {
        self.destroy_cost_with_any_of(target_health, self.owned_weapons::<TARGET_IS_WALL>(settings), flying_target, settings)
    }
    /// Returns the energy required to destroy the target with a ranged weapon, or `None` if no weapons are available to attack the target
    pub fn destroy_cost_ranged(&self, target_health: f32, flying_target: bool, settings: &WorldSettings) -> Option<f32> {
        self.destroy_cost_with_any_of(target_health, self.owned_ranged_weapons(settings), flying_target, settings)
    }

    fn owned_weapons_from_fn<const N: usize, F>(&self, weapons_fn: F, settings: &WorldSettings) -> SmallVec<[Skill; N]>
    where F: FnOnce(Difficulty) -> SmallVec<[Skill; N]>
    {
        let mut weapons = weapons_fn(settings.difficulty);
        weapons.retain(|weapon| self.has(&Item::Skill(*weapon), 1));
        weapons
    }
    // TODO would it be worth to precompile the resulting slices for all variants?
    pub fn owned_weapons<const TARGET_IS_WALL: bool>(&self, settings: &WorldSettings) -> SmallVec<[Skill; 9]> {
        self.owned_weapons_from_fn(Difficulty::weapons::<TARGET_IS_WALL>, settings)
    }
    pub fn owned_ranged_weapons(&self, settings: &WorldSettings) -> SmallVec<[Skill; 6]> {
        self.owned_weapons_from_fn(Difficulty::ranged_weapons, settings)
    }
    pub fn owned_shield_weapons(&self, settings: &WorldSettings) -> SmallVec<[Skill; 4]> {
        self.owned_weapons_from_fn(Difficulty::shield_weapons, settings)
    }

    fn progression_weapons_from_fn<const N: usize, F>(&self, weapons_fn: F, settings: &WorldSettings) -> SmallVec<[Skill; N]>
    where F: FnOnce(Difficulty) -> SmallVec<[Skill; N]>
    {
        let mut weapons = weapons_fn(settings.difficulty);
        // TODO check whether creating this map is even worth it
        let dpe_map = weapons.iter().map(|weapon| (*weapon, (weapon.damage_per_energy(settings) * 10.0) as u16)).collect::<FxHashMap<Skill, u16>>();
        weapons.sort_unstable_by_key(|weapon| dpe_map[weapon]);
        if let Some((index, weapon)) = weapons.iter().enumerate().find(|(_, weapon)| self.has(&Item::Skill(**weapon), 1)) {
            let dpe = dpe_map[weapon];
            weapons.truncate(index + 1);
            // maybe there are multiple weapons costing the same and we already skipped over a redundant one
            weapons.swap(index, 0);  // if we found something before, there must be at least one element
            let remove_after = weapons.iter().rposition(|weapon| dpe_map[weapon] != dpe).unwrap_or(0);
            weapons.truncate(remove_after + 1);
            weapons.swap(0, remove_after);
        }
        weapons
    }
    pub fn progression_weapons<const TARGET_IS_WALL: bool>(&self, settings: &WorldSettings) -> SmallVec<[Skill; 9]> {
        self.progression_weapons_from_fn(Difficulty::weapons::<TARGET_IS_WALL>, settings)
    }
    pub fn ranged_progression_weapons(&self, settings: &WorldSettings) -> SmallVec<[Skill; 6]> {
        self.progression_weapons_from_fn(Difficulty::ranged_weapons, settings)
    }
    pub fn shield_progression_weapons(&self, settings: &WorldSettings) -> SmallVec<[Skill; 4]> {
        self.progression_weapons_from_fn(Difficulty::shield_weapons, settings)
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO could try iter::intersperse here once it's stabilized
        let mut first = true;
        for (item, amount) in &self.items {
            if first { first = false }
            else { write!(f, ", ")?; }
            if *amount != 1 { write!(f, "{amount} ")?; }
            item.fmt(f)?;
        }
        Ok(())
    }
}

impl From<Item> for Inventory {
    fn from(item: Item) -> Inventory {
        let mut inventory = Inventory::default();
        inventory.grant(item, 1);
        inventory
    }
}
impl From<(Item, u32)> for Inventory {
    fn from(item_amount: (Item, u32)) -> Inventory {
        let mut inventory = Inventory::default();
        let (item, amount) = item_amount;
        inventory.grant(item, amount);
        inventory
    }
}
impl FromIterator<Item> for Inventory {
    fn from_iter<T: IntoIterator<Item = Item>>(items: T) -> Inventory {
        let mut inventory = Inventory::default();
        for item in items {
            inventory.grant(item, 1);
        }
        inventory
    }
}
impl FromIterator<(Item, u32)> for Inventory {
    fn from_iter<T: IntoIterator<Item = (Item, u32)>>(items: T) -> Inventory {
        let mut inventory = Inventory::default();
        for (item, amount) in items {
            inventory.grant(item, amount);
        }
        inventory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destroy_cost() {
        let mut world_settings = WorldSettings::default();
        let mut inventory = Inventory::default();
        assert_eq!(inventory.destroy_cost::<false>(10.0, false, &world_settings), None);
        inventory.grant(Item::Skill(Skill::Spear), 1);
        assert_eq!(inventory.destroy_cost::<false>(10.0, true, &world_settings), Some(4.0));
        assert_eq!(inventory.destroy_cost::<false>(0.0, false, &world_settings), Some(0.0));
        inventory.grant(Item::Skill(Skill::Bow), 1);
        assert_eq!(inventory.destroy_cost::<false>(10.0, false, &world_settings), Some(1.5));
        world_settings.difficulty = Difficulty::Unsafe;
        inventory.grant(Item::Skill(Skill::GladesAncestralLight), 1);
        inventory.grant(Item::Skill(Skill::InkwaterAncestralLight), 1);
        inventory.grant(Item::Shard(Shard::Wingclip), 1);
        inventory.grant(Item::Resource(Resource::ShardSlot), 1);
        inventory.remove(&Item::Skill(Skill::Bow), 1);
        assert_eq!(inventory.destroy_cost::<false>(1.0, false, &world_settings), Some(2.0));
        inventory.grant(Item::Skill(Skill::Bow), 1);
        assert_eq!(inventory.destroy_cost::<false>(10.0, true, &world_settings), Some(0.25));
        inventory.items.clear();
        inventory.grant(Item::Skill(Skill::Grenade), 1);
        inventory.grant(Item::Skill(Skill::Shuriken), 1);
        assert_eq!(inventory.destroy_cost::<false>(20.0, false, &world_settings), Some(1.5));
        assert_eq!(inventory.destroy_cost::<false>(24.0, false, &world_settings), Some(1.5));
        assert_eq!(inventory.destroy_cost::<false>(34.0, false, &world_settings), Some(2.0));
    }
}
