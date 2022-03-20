use smallvec::{SmallVec, smallvec};

use crate::inventory::Inventory;
use crate::item::{Item, Resource, Skill, Shard};
use crate::settings::{Difficulty, WorldSettings};
use crate::util::{self, Orbs};

/// A logical representation of the in-game player
#[derive(Debug, Clone)]
pub struct Player {
    pub inventory: Inventory,
    pub settings: WorldSettings,
}
impl Player {
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    /// 
    /// The [`Player`] will have an empty Inventory. Use [`Player::spawn`] to create a player with starting health and energy like in-game
    pub fn new(settings: WorldSettings) -> Player {
        let inventory = Inventory::default();
        Player { inventory, settings }
    }
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    /// 
    /// The [`Player`] will have six Health and Energy fragments - these are the resources a player spawns with in-game
    pub fn spawn(settings: WorldSettings) -> Player {
        let mut inventory = Inventory::default();
        inventory.grant(Item::Resource(Resource::Health), 6);
        inventory.grant(Item::Resource(Resource::Energy), 6);
        Player { inventory, settings }
    }

    /// Returns the maximum health
    /// 
    /// One visual orb in the game represents 10 health
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// use seedgen::settings::WorldSettings;
    /// 
    /// let player = Player::spawn(WorldSettings::default());
    /// assert_eq!(player.max_health(), 30.0);
    /// ```
    pub fn max_health(&self) -> f32 {
        let mut health = (self.inventory.get(&Item::Resource(Resource::Health)) * 5) as f32;
        if self.settings.difficulty >= Difficulty::Gorlek && self.inventory.has(&Item::Shard(Shard::Vitality), 1) { health += 10.0; }
        health
    }
    /// Returns the maximum energy
    /// 
    /// One visual orb in the game represents 1 energy
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// use seedgen::settings::WorldSettings;
    /// 
    /// let player = Player::spawn(WorldSettings::default());
    /// assert_eq!(player.max_energy(), 3.0);
    /// ```
    pub fn max_energy(&self) -> f32 {
        let mut energy = self.inventory.get(&Item::Resource(Resource::Energy)) as f32 * 0.5;
        if self.settings.difficulty >= Difficulty::Gorlek && self.inventory.has(&Item::Shard(Shard::Energy), 1) { energy += 1.0; }
        energy
    }
    /// Returns the maximum health and energy
    pub fn max_orbs(&self) -> Orbs {
        Orbs {
            energy: self.max_energy(),
            health: self.max_health(),
        }
    }
    /// Reduces the [`Orbs`] to the maximum health and energy of this [`Player`] if they exceed it
    /// 
    /// This follows the in-game rules when adding health or energy to the in-game player
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// use seedgen::settings::WorldSettings;
    /// use seedgen::util::Orbs;
    /// 
    /// let player = Player::spawn(WorldSettings::default());
    /// let mut orbs = Orbs { health: 90.0, energy: 5.0 };
    /// 
    /// player.cap_orbs(&mut orbs, false);
    /// 
    /// assert_eq!(orbs, player.max_orbs());
    /// ```
    /// 
    /// `checkpoint` represents whether the Orbs are a result of the player respawning on a checkpoint, in which case special rules apply
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// # use seedgen::settings::WorldSettings;
    /// # use seedgen::util::Orbs;
    /// use seedgen::Item;
    /// use seedgen::item::Shard;
    /// use seedgen::settings::Difficulty;
    /// 
    /// let mut player = Player::spawn(WorldSettings::default());
    /// player.inventory.grant(Item::Shard(Shard::Vitality), 1);
    /// player.settings.difficulty = Difficulty::Gorlek;
    /// let mut orbs = Orbs { health: 90.0, energy: 1.0 };
    /// 
    /// player.cap_orbs(&mut orbs, false);
    /// assert_eq!(orbs, Orbs { health: 40.0, energy: 1.0 });
    /// 
    /// player.cap_orbs(&mut orbs, true);
    /// assert_eq!(orbs, Orbs { health: 30.0, energy: 1.0 });
    /// ```
    pub fn cap_orbs(&self, orbs: &mut Orbs, checkpoint: bool) {
        // checkpoints don't refill health given by the Vitality shard
        let max_health = if checkpoint {
            (self.inventory.get(&Item::Resource(Resource::Health)) * 5) as f32
        } else {
            self.max_health()
        };
        // (but they do refill energy from the Energy shard...)
        let max_energy = self.max_energy();

        if self.settings.difficulty >= Difficulty::Unsafe && self.inventory.has(&Item::Shard(Shard::Overflow), 1) {
            if orbs.health > max_health {
                orbs.energy += orbs.health - max_health;
            } else if orbs.energy > max_energy {
                orbs.health += orbs.energy - max_energy;
            }
        }

        orbs.health = orbs.health.min(max_health);
        orbs.energy = orbs.energy.min(max_energy);
    }

    /// Returns the [`Orbs`] after respawning on a checkpoint
    /// 
    /// This follows the in-game rules when respawning on a checkpoint
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// use seedgen::settings::WorldSettings;
    /// use seedgen::util::Orbs;
    /// use seedgen::Item;
    /// use seedgen::item::Resource;
    /// 
    /// let mut player = Player::spawn(WorldSettings::default());
    /// assert_eq!(player.max_orbs(), Orbs { health: 30.0, energy: 3.0 });
    /// assert_eq!(player.checkpoint_orbs(), Orbs { health: 30.0, energy: 1.0 });
    /// 
    /// player.inventory.grant(Item::Resource(Resource::Health), 22);
    /// player.inventory.grant(Item::Resource(Resource::Energy), 24);
    /// assert_eq!(player.max_orbs(), Orbs { health: 140.0, energy: 15.0 });
    /// assert_eq!(player.checkpoint_orbs(), Orbs { health: 42.0, energy: 3.0 });
    /// ```
    pub fn checkpoint_orbs(&self) -> Orbs {
        let health_refill = (self.max_health() * 0.3).ceil().max(40.0);
        let energy_refill = (self.max_energy() * 0.2).ceil().max(1.0);

        let mut orbs = Orbs {
            health: health_refill,
            energy: energy_refill,
        };

        self.cap_orbs(&mut orbs, true);
        orbs
    }
    /// Returns how many health orbs plants will drop
    /// 
    /// This follows the in-game rules when opening a health plant
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use seedgen::world::Player;
    /// use seedgen::settings::WorldSettings;
    /// use seedgen::util::Orbs;
    /// use seedgen::Item;
    /// use seedgen::item::Resource;
    /// 
    /// let mut player = Player::spawn(WorldSettings::default());
    /// assert_eq!(player.health_plant_drops(), 1.0);
    /// 
    /// player.inventory.grant(Item::Resource(Resource::Health), 8);
    /// assert_eq!(player.health_plant_drops(), 2.0);
    /// 
    /// player.inventory.grant(Item::Resource(Resource::Health), 18);
    /// assert_eq!(player.health_plant_drops(), 5.0);
    /// ```
    pub fn health_plant_drops(&self) -> f32 {
        let value = self.max_health() / 30.0;
        // the game rounds to even
        if value % 1. == 0.5 && value as u8 % 2 == 0 {
            value.floor()
        } else {
            value.round()
        }
    }

    pub fn damage_mod(&self, flying_target: bool, bow: bool) -> f32 {
        let mut damage_mod = 1.0;

        if self.settings.difficulty >= Difficulty::Gorlek {
            damage_mod += 0.25 * self.inventory.get(&Item::Skill(Skill::AncestralLight)) as f32;
        }

        if self.settings.difficulty >= Difficulty::Unsafe {
            let mut slots = self.inventory.get(&Item::Resource(Resource::ShardSlot));
            let mut splinter = false;

            if flying_target && slots > 0 && self.inventory.has(&Item::Shard(Shard::Wingclip), 1) { damage_mod += 1.0; slots -= 1; }
            if slots > 0 && bow && self.inventory.has(&Item::Shard(Shard::Splinter), 1) { splinter = true; slots -= 1; }
            if slots > 0 && self.inventory.has(&Item::Shard(Shard::SpiritSurge), 1) { damage_mod += (self.inventory.get(&Item::SpiritLight(1)) / 10000) as f32; slots -= 1; }
            if slots > 0 && self.inventory.has(&Item::Shard(Shard::LastStand), 1) { damage_mod += 0.2; slots -= 1; }
            if slots > 0 && self.inventory.has(&Item::Shard(Shard::Reckless), 1) { damage_mod += 0.15; slots -= 1; }
            if slots > 0 && self.inventory.has(&Item::Shard(Shard::Lifeforce), 1) { damage_mod += 0.1; slots -= 1; }
            if slots > 0 && self.inventory.has(&Item::Shard(Shard::Finesse), 1) { damage_mod += 0.05; }
            if splinter { damage_mod *= 1.5; }  // Splinter stacks multiplicatively where other buffs stack additively
        }

        damage_mod
    }
    pub fn defense_mod(&self) -> f32 {
        let mut defense_mod = if self.settings.difficulty >= Difficulty::Gorlek && self.inventory.has(&Item::Shard(Shard::Resilience), 1) { 0.9 } else { 1.0 };
        if self.settings.hard { defense_mod *= 2.0; }
        defense_mod
    }
    pub fn energy_mod(&self) -> f32 {
        let mut energy_mod = 1.0;
        if self.settings.difficulty < Difficulty::Unsafe { energy_mod *= 2.0; }
        else if self.inventory.has(&Item::Shard(Shard::Overcharge), 1) { energy_mod *= 0.5; }
        energy_mod
    }

    pub fn use_cost(&self, skill: Skill) -> f32 {
        skill.energy_cost() * self.energy_mod()
    }
    pub fn destroy_cost(&self, health: f32, skill: Skill, flying_target: bool) -> f32 {
        let damage = skill.damage(Difficulty::Unsafe) * self.damage_mod(flying_target, matches!(skill, Skill::Bow)) + skill.burn_damage();  // Burn damage is unaffected by damage buffs
        (health / damage).ceil() * self.use_cost(skill)
    }

    fn weapons_by_dpe(&self, wall: bool) -> SmallVec<[Skill; 8]> {
        let mut weapons: SmallVec<[_; 8]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Spear,
        ];
        if !wall { weapons.push(Skill::Flash); }
        if self.settings.difficulty >= Difficulty::Unsafe { weapons.push(Skill::Sentry); }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        weapons.sort_unstable_by_key(|&weapon| (weapon.damage_per_energy(self.settings.difficulty) * 10.0) as u16);
        weapons
    }
    fn ranged_weapons_by_dpe(&self) -> SmallVec<[Skill; 2]> {
        let mut weapons: SmallVec<[_; 2]> = smallvec![
            Skill::Bow,
            Skill::Spear,
        ];
        if self.settings.difficulty >= Difficulty::Gorlek {
            weapons.push(Skill::Grenade);
            weapons.push(Skill::Shuriken);
        }
        if self.settings.difficulty >= Difficulty::Unsafe {
            weapons.push(Skill::Flash);
            weapons.push(Skill::Blaze);
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        weapons.sort_unstable_by_key(|&weapon| (weapon.damage_per_energy(self.settings.difficulty) * 10.0) as u16);
        weapons
    }
    fn shield_weapons_by_dpe(&self) -> SmallVec<[Skill; 4]> {
        let mut weapons: SmallVec<[_; 4]> = smallvec![
            Skill::Hammer,
            Skill::Launch,
            Skill::Grenade,
            Skill::Spear,
        ];

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        weapons.sort_unstable_by_key(|&weapon| (weapon.damage_per_energy(self.settings.difficulty) * 10.0) as u16);
        weapons
    }

    fn preferred_among_weapons<W>(&self, weapons: W) -> Option<Skill>
    where W: IntoIterator<Item=Skill>,
    {
        for weapon in weapons {
            if self.inventory.has(&Item::Skill(weapon), 1) {
                return Some(weapon);
            }
        }
        None
    }
    pub fn preferred_weapon(&self, wall: bool) -> Option<Skill> {
        self.preferred_among_weapons(self.weapons_by_dpe(wall))
    }
    pub fn preferred_ranged_weapon(&self) -> Option<Skill> {
        self.preferred_among_weapons(self.ranged_weapons_by_dpe())
    }
    pub fn preferred_shield_weapon(&self) -> Option<Skill> {
        self.preferred_among_weapons(self.shield_weapons_by_dpe())
    }

    fn progression_among_weapons<W>(&self, weapons: W) -> SmallVec<[Skill; 8]>
    where W: IntoIterator<Item=Skill>,
    {
        let mut progression_weapons = SmallVec::new();

        for weapon in weapons {
            progression_weapons.push(weapon);
            if self.inventory.has(&Item::Skill(weapon), 1) {
                break;
            }
        }

        progression_weapons
    }
    pub fn progression_weapons(&self, wall: bool) -> SmallVec<[Skill; 8]> {
        self.progression_among_weapons(self.weapons_by_dpe(wall))
    }
    pub fn ranged_progression_weapons(&self) -> SmallVec<[Skill; 8]> {
        self.progression_among_weapons(self.ranged_weapons_by_dpe())
    }
    pub fn shield_progression_weapons(&self) -> SmallVec<[Skill; 8]> {
        self.progression_among_weapons(self.shield_weapons_by_dpe())
    }

    pub fn missing_items(&self, needed: &mut Inventory) {
        for (item, amount) in &mut needed.items {
            let owned = self.inventory.get(item);
            *amount -= owned.min(*amount);
        }

        needed.items.retain(|_, amount| *amount > 0);
    }
    pub fn missing_for_orbs(&self, needed: &Inventory, orb_cost: Orbs, current_orbs: Orbs) -> Vec<Inventory> {
        let orbs = current_orbs + orb_cost;
        let mut inventories = Vec::new();

        if orbs.health <= 0.0 {
            #[allow(clippy::cast_possible_truncation)]
            let health_fragments = util::float_to_int(((-orbs.health + 0.1) / 5.0).ceil()).unwrap();
            inventories.push(Inventory::from((Item::Resource(Resource::Health), health_fragments)));

            let max_heal = self.max_health() - current_orbs.health;
            if max_heal > -orbs.health {
                let has_regen = self.inventory.has(&Item::Skill(Skill::Regenerate), 1);
                let max_regens = (-orbs.health / 30.0).ceil();

                let mut regens = 1.0;
                while regens <= max_regens {
                    let mut regen_inventory = Inventory::default();
                    if !has_regen {
                        regen_inventory.grant(Item::Skill(Skill::Regenerate), 1);
                    }

                    let regen_orbs = orbs + Orbs { health: (30.0 * regens), energy: -1.0 * regens };

                    if regen_orbs.health <= 0.0 {
                        #[allow(clippy::cast_possible_truncation)]
                        let health_fragments = util::float_to_int(((-regen_orbs.health + 0.1) / 5.0).ceil()).unwrap();
                        regen_inventory.grant(Item::Resource(Resource::Health), health_fragments);
                    }
                    if regen_orbs.energy < 0.0 {
                        #[allow(clippy::cast_possible_truncation)]
                        let energy_fragments = util::float_to_int((-regen_orbs.energy * 2.0).ceil()).unwrap();
                        regen_inventory.grant(Item::Resource(Resource::Energy), energy_fragments);
                    }

                    inventories.push(regen_inventory);

                    regens += 1.0;
                }
            }
        } else {
            inventories.push(Inventory::default());
        }

        if orbs.energy < 0.0 {
            #[allow(clippy::cast_possible_truncation)]
            let energy_fragments = util::float_to_int((-orbs.energy * 2.0).ceil()).unwrap();

            for inventory in &mut inventories {
                inventory.grant(Item::Resource(Resource::Energy), energy_fragments);
            }
        }

        for inventory in &mut inventories {
            for (item, amount) in needed.items.clone() {
                inventory.grant(item, amount);
            }
        }

        inventories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::item::BonusItem;

    #[test]
    fn inventory() {
        let mut player = Player::new(WorldSettings::default());
        player.inventory.grant(Item::BonusItem(BonusItem::Relic), 2);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        assert!(player.inventory.has(&Item::BonusItem(BonusItem::Relic), 1));
        assert!(player.inventory.has(&Item::BonusItem(BonusItem::Relic), 2));
        assert!(player.inventory.has(&Item::Skill(Skill::Shuriken), 1));
        assert!(!player.inventory.has(&Item::Skill(Skill::Bash), 0));
    }

    #[test]
    fn weapon_preference() {
        let mut player = Player::new(WorldSettings::default());
        assert_eq!(player.preferred_weapon(true), None);
        assert_eq!(player.preferred_ranged_weapon(), None);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        assert_eq!(player.preferred_weapon(true), Some(Skill::Shuriken));
        assert_eq!(player.preferred_ranged_weapon(), None);
        player.settings.difficulty = Difficulty::Gorlek;
        assert_eq!(player.preferred_ranged_weapon(), Some(Skill::Shuriken));
        player.inventory.grant(Item::Skill(Skill::Spear), 1);
        assert_eq!(player.preferred_weapon(true), Some(Skill::Shuriken));
        assert_eq!(player.preferred_ranged_weapon(), Some(Skill::Shuriken));
        player.inventory.grant(Item::Skill(Skill::Sword), 1);
        assert_eq!(player.preferred_weapon(true), Some(Skill::Sword));

        player = Player::new(WorldSettings::default());
        let weapons: SmallVec<[_; 8]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Flash,
            Skill::Spear,
        ];
        assert_eq!(player.progression_weapons(false), weapons);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        let weapons: SmallVec<[_; 5]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ];
        assert_eq!(player.progression_weapons(false), weapons);
        player.settings.difficulty = Difficulty::Unsafe;
        let weapons: SmallVec<[_; 5]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ];
        assert_eq!(player.progression_weapons(false), weapons);
    }

    #[test]
    fn max_energy() {
        let mut player = Player::new(WorldSettings::default());
        assert_eq!(player.max_energy(), 0.0);
        for _ in 0..10 { player.inventory.grant(Item::Resource(Resource::Energy), 1); }
        player.inventory.grant(Item::Shard(Shard::Energy), 1);
        assert_eq!(player.max_energy(), 5.0);
        player.settings.difficulty = Difficulty::Gorlek;
        assert_eq!(player.max_energy(), 6.0);
    }

    #[test]
    fn destroy_cost() {
        let mut player = Player::new(WorldSettings::default());
        assert_eq!(player.destroy_cost(10.0, Skill::Bow, false), 1.5);
        assert_eq!(player.destroy_cost(10.0, Skill::Spear, true), 4.0);
        assert_eq!(player.destroy_cost(0.0, Skill::Spear, false), 0.0);
        player.inventory.grant(Item::Skill(Skill::AncestralLight), 2);
        player.settings.difficulty = Difficulty::Unsafe;
        player.inventory.grant(Item::Shard(Shard::Wingclip), 1);
        player.inventory.grant(Item::Resource(Resource::ShardSlot), 1);
        assert_eq!(player.destroy_cost(10.0, Skill::Bow, true), 0.25);
        assert_eq!(player.destroy_cost(1.0, Skill::Spear, false), 2.0);
    }

    #[test]
    fn refill_orbs() {
        let mut player = Player::spawn(WorldSettings::default());

        let expected = [30.,35.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,41.,42.,44.,45.,47.,48.,50.,52.,53.,55.,56.,58.,59.,61.,62.,64.,65.,66.,68.,69.];
        for health in expected {
            assert_eq!(player.checkpoint_orbs().health, health);
            player.inventory.grant(Item::Resource(Resource::Health), 1);
        }

        player = Player::new(WorldSettings::default());

        let expected = [0.,0.,0.,0.,1.,1.,1.,1.,1.,2.,2.,2.,2.,2.,2.,2.,3.,3.,3.,3.,3.,4.,4.,4.,4.,4.,4.,4.,5.,5.,5.,5.,5.,6.,6.,6.,6.,6.,6.,6.,7.,7.,7.,7.,7.,8.,8.];
        for drops in expected {
            assert_eq!(player.health_plant_drops(), drops);
            player.inventory.grant(Item::Resource(Resource::Health), 1);
        }

        player = Player::new(WorldSettings::default());
        player.settings.difficulty = Difficulty::Gorlek;

        player.inventory.grant(Item::Shard(Shard::Energy), 1);
        player.inventory.grant(Item::Shard(Shard::Vitality), 1);

        assert_eq!(player.checkpoint_orbs(), Orbs { energy: 1.0, health: 0.0 });

        player.inventory.grant(Item::Resource(Resource::Health), 7);

        assert_eq!(player.checkpoint_orbs(), Orbs { health: 35.0, energy: 1.0 });

        player.inventory.grant(Item::Resource(Resource::Health), 21);

        assert_eq!(player.checkpoint_orbs(), Orbs { health: 45.0, energy: 1.0 });
    }
}
