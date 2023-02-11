use smallvec::SmallVec;

use crate::inventory::Inventory;
use crate::item::{Item, Resource, Skill, Shard};
use crate::settings::{Difficulty, WorldSettings};
use crate::util::Orbs;

/// A logical representation of the in-game player
#[derive(Debug, Clone)]
pub struct Player<'a> {
    pub inventory: Inventory,
    pub settings: &'a WorldSettings,
}
impl Player<'_> {
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    /// 
    /// The [`Player`] will have an empty Inventory. Use [`Player::spawn`] to create a player with starting health and energy like in-game
    pub fn new(settings: &WorldSettings) -> Player {
        let inventory = Inventory::default();
        Player { inventory, settings }
    }
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    /// 
    /// The [`Player`] will have six Health and Energy fragments - these are the resources a player spawns with in-game
    pub fn spawn(settings: &WorldSettings) -> Player {
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
    /// # use wotw_seedgen::world::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// 
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::spawn(&world_settings);
    /// assert_eq!(player.max_health(), 30.0);
    /// ```
    pub fn max_health(&self) -> f32 {
        self.inventory.max_health(self.settings.difficulty)
    }
    /// Returns the maximum energy
    /// 
    /// One visual orb in the game represents 1 energy
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::world::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// 
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::spawn(&world_settings);
    /// assert_eq!(player.max_energy(), 3.0);
    /// ```
    pub fn max_energy(&self) -> f32 {
        self.inventory.max_energy(self.settings.difficulty)
    }
    /// Returns the maximum health and energy
    pub fn max_orbs(&self) -> Orbs {
        self.inventory.max_orbs(self.settings.difficulty)
    }
    /// Reduces the [`Orbs`] to the maximum health and energy of this [`Player`] if they exceed it
    /// 
    /// This follows the in-game rules when adding health or energy to the in-game player
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use wotw_seedgen::world::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::util::Orbs;
    /// 
    /// let world_settings = WorldSettings::default();
    /// let player = Player::spawn(&world_settings);
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
    /// # use wotw_seedgen::world::Player;
    /// # use wotw_seedgen::settings::WorldSettings;
    /// # use wotw_seedgen::util::Orbs;
    /// use wotw_seedgen::Item;
    /// use wotw_seedgen::item::Shard;
    /// use wotw_seedgen::settings::Difficulty;
    /// 
    /// let mut world_settings = WorldSettings::default();
    /// world_settings.difficulty = Difficulty::Gorlek;
    /// let mut player = Player::spawn(&world_settings);
    /// player.inventory.grant(Item::Shard(Shard::Vitality), 1);
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
    /// # use wotw_seedgen::world::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::util::Orbs;
    /// use wotw_seedgen::Item;
    /// use wotw_seedgen::item::Resource;
    /// 
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::spawn(&world_settings);
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
    /// # use wotw_seedgen::world::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::util::Orbs;
    /// use wotw_seedgen::Item;
    /// use wotw_seedgen::item::Resource;
    /// 
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::spawn(&world_settings);
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
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::float_cmp)]
        if value % 1. == 0.5 && value as u8 % 2 == 0 {
            value.floor()
        } else {
            value.round()
        }
    }

    /// Replenish health, but don't exceed the player's maximum health
    pub fn heal(&self, orbs: &mut Orbs, amount: f32) {
        self.inventory.heal(orbs, amount, self.settings.difficulty);
    }
    /// Replenish energy, but don't exceed the player's maximum energy
    pub fn recharge(&self, orbs: &mut Orbs, amount: f32) {
        self.inventory.recharge(orbs, amount, self.settings.difficulty);
    }

    pub fn damage_mod(&self, flying_target: bool, bow: bool) -> f32 {
        self.inventory.damage_mod(flying_target, bow, self.settings)
    }
    pub fn defense_mod(&self) -> f32 {
        self.inventory.defense_mod(self.settings)
    }
    pub fn energy_mod(&self) -> f32 {
        self.inventory.energy_mod(self.settings)
    }

    pub fn use_cost(&self, skill: Skill) -> f32 {
        self.inventory.use_cost(skill, self.settings)
    }
    pub fn destroy_cost<const TARGET_IS_WALL: bool>(&self, target_health: f32, flying_target: bool) -> Option<f32> {
        self.inventory.destroy_cost::<TARGET_IS_WALL>(target_health, flying_target, self.settings)
    }
    pub fn destroy_cost_ranged(&self, target_health: f32, flying_target: bool) -> Option<f32> {
        self.inventory.destroy_cost_ranged(target_health, flying_target, self.settings)
    }
    pub fn destroy_cost_with(&self, target_health: f32, weapon: Skill, flying_target: bool) -> f32 {
        self.inventory.destroy_cost_with(target_health, weapon, flying_target, self.settings)
    }

    pub fn owned_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.inventory.owned_weapons::<TARGET_IS_WALL>(self.settings)
    }
    pub fn owned_ranged_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.inventory.owned_ranged_weapons(self.settings)
    }
    pub fn owned_shield_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.inventory.owned_shield_weapons(self.settings)
    }

    pub fn progression_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.inventory.progression_weapons::<TARGET_IS_WALL>(self.settings)
    }
    pub fn ranged_progression_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.inventory.ranged_progression_weapons(self.settings)
    }
    pub fn shield_progression_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.inventory.shield_progression_weapons(self.settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use smallvec::smallvec;
    use crate::item::BonusItem;

    #[test]
    fn inventory() {
        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);
        player.inventory.grant(Item::BonusItem(BonusItem::Relic), 2);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        assert!(player.inventory.has(&Item::BonusItem(BonusItem::Relic), 1));
        assert!(player.inventory.has(&Item::BonusItem(BonusItem::Relic), 2));
        assert!(player.inventory.has(&Item::Skill(Skill::Shuriken), 1));
        assert!(!player.inventory.has(&Item::Skill(Skill::Bash), 0));
    }

    #[test]
    fn weapon_preference() {
        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);
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
        assert_eq!(player.progression_weapons::<false>(), weapons);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        let weapons: SmallVec<[_; 5]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ];
        assert_eq!(player.progression_weapons::<false>(), weapons);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        let weapons: SmallVec<[_; 5]> = smallvec![
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ];
        assert_eq!(player.progression_weapons::<false>(), weapons);
    }

    #[test]
    fn max_energy() {
        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);
        assert_eq!(player.max_energy(), 0.0);
        for _ in 0..10 { player.inventory.grant(Item::Resource(Resource::Energy), 1); }
        player.inventory.grant(Item::Shard(Shard::Energy), 1);
        assert_eq!(player.max_energy(), 5.0);
        let world_settings = WorldSettings { difficulty: Difficulty::Gorlek, ..WorldSettings::default() };
        player.settings = &world_settings;
        assert_eq!(player.max_energy(), 6.0);
    }

    #[test]
    fn refill_orbs() {
        let world_settings = WorldSettings::default();
        let mut player = Player::spawn(&world_settings);

        let expected = [30.,35.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,40.,41.,42.,44.,45.,47.,48.,50.,52.,53.,55.,56.,58.,59.,61.,62.,64.,65.,66.,68.,69.];
        for health in expected {
            assert_eq!(player.checkpoint_orbs().health, health);
            player.inventory.grant(Item::Resource(Resource::Health), 1);
        }

        player = Player::new(&world_settings);

        let expected = [0.,0.,0.,0.,1.,1.,1.,1.,1.,2.,2.,2.,2.,2.,2.,2.,3.,3.,3.,3.,3.,4.,4.,4.,4.,4.,4.,4.,5.,5.,5.,5.,5.,6.,6.,6.,6.,6.,6.,6.,7.,7.,7.,7.,7.,8.,8.];
        for drops in expected {
            assert_eq!(player.health_plant_drops(), drops);
            player.inventory.grant(Item::Resource(Resource::Health), 1);
        }

        let world_settings = WorldSettings { difficulty: Difficulty::Gorlek, ..WorldSettings::default() };
        player = Player::new(&world_settings);

        player.inventory.grant(Item::Shard(Shard::Energy), 1);
        player.inventory.grant(Item::Shard(Shard::Vitality), 1);

        assert_eq!(player.checkpoint_orbs(), Orbs { energy: 1.0, health: 0.0 });

        player.inventory.grant(Item::Resource(Resource::Health), 7);

        assert_eq!(player.checkpoint_orbs(), Orbs { health: 35.0, energy: 1.0 });

        player.inventory.grant(Item::Resource(Resource::Health), 21);

        assert_eq!(player.checkpoint_orbs(), Orbs { health: 45.0, energy: 1.0 });
    }
}
