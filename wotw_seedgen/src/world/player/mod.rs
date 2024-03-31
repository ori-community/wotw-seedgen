mod is_met;
mod solutions;

// TODO remove maybe
pub(crate) use solutions::filter_redundancies;

#[cfg(test)]
mod tests;

use crate::{
    inventory::Inventory,
    orbs::{OrbVariants, Orbs},
};
use smallvec::SmallVec;
use wotw_seedgen_data::Skill;
use wotw_seedgen_logic_language::output::RefillValue;
use wotw_seedgen_settings::WorldSettings;

/// A logical representation of the in-game player
#[derive(Debug, Clone)]
pub struct Player<'settings> {
    pub inventory: Inventory,
    pub settings: &'settings WorldSettings,
}
impl Player<'_> {
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    ///
    /// The [`Player`] will have an empty Inventory. Use [`Player::new_spawn`] to create a player with starting health and energy like in-game
    pub fn new(settings: &WorldSettings) -> Player {
        Player {
            inventory: Inventory::default(),
            settings,
        }
    }
    /// Returns an instance of [`Player`] with the given [`WorldSettings`]
    ///
    /// The [`Player`] will have six Health and Energy fragments as well as three Shard Slots - these are the resources a player spawns with in-game
    pub fn new_spawn(settings: &WorldSettings) -> Player {
        Player {
            inventory: Inventory::spawn(),
            settings,
        }
    }

    /// Returns the maximum health
    ///
    /// One visual orb in the game represents 10 health
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::new_spawn(&world_settings);
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
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::new_spawn(&world_settings);
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
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// let world_settings = WorldSettings::default();
    /// let player = Player::new_spawn(&world_settings);
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
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::data::Shard;
    /// use wotw_seedgen::orbs::Orbs;
    /// use wotw_seedgen::settings::{WorldSettings, Difficulty};
    ///
    /// let mut world_settings = WorldSettings::default();
    /// world_settings.difficulty = Difficulty::Gorlek;
    /// let mut player = Player::new_spawn(&world_settings);
    /// player.inventory.shards.insert(Shard::Vitality);
    /// let mut orbs = Orbs { health: 90.0, energy: 1.0 };
    ///
    /// player.cap_orbs(&mut orbs, false);
    /// assert_eq!(orbs, Orbs { health: 40.0, energy: 1.0 });
    ///
    /// player.cap_orbs(&mut orbs, true);
    /// assert_eq!(orbs, Orbs { health: 30.0, energy: 1.0 });
    /// ```
    // TODO this didn't end up being used much, maybe it should be used more to have the overflow check?
    pub fn cap_orbs(&self, orbs: &mut Orbs, checkpoint: bool) {
        self.inventory
            .cap_orbs(orbs, checkpoint, self.settings.difficulty)
    }

    /// Returns the [`Orbs`] after respawning on a checkpoint
    ///
    /// This follows the in-game rules when respawning on a checkpoint
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    /// use wotw_seedgen::orbs::Orbs;
    ///
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::new_spawn(&world_settings);
    /// assert_eq!(player.max_orbs(), Orbs { health: 30.0, energy: 3.0 });
    /// assert_eq!(player.checkpoint_orbs(), Orbs { health: 30.0, energy: 1.0 });
    ///
    /// player.inventory.health += 110;
    /// player.inventory.energy += 12.;
    /// assert_eq!(player.max_orbs(), Orbs { health: 140.0, energy: 15.0 });
    /// assert_eq!(player.checkpoint_orbs(), Orbs { health: 42.0, energy: 3.0 });
    /// ```
    pub fn checkpoint_orbs(&self) -> Orbs {
        self.inventory.checkpoint_orbs(self.settings.difficulty)
    }
    /// Returns how many health orbs plants will drop
    ///
    /// This follows the in-game rules when opening a health plant
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::Player;
    /// use wotw_seedgen::settings::WorldSettings;
    ///
    /// let world_settings = WorldSettings::default();
    /// let mut player = Player::new_spawn(&world_settings);
    /// assert_eq!(player.health_plant_drops(), 1.0);
    ///
    /// player.inventory.health += 40;
    /// assert_eq!(player.health_plant_drops(), 2.0);
    ///
    /// player.inventory.health += 90;
    /// assert_eq!(player.health_plant_drops(), 5.0);
    /// ```
    pub fn health_plant_drops(&self) -> f32 {
        self.inventory.health_plant_drops(self.settings.difficulty)
    }

    /// Replenish health, but don't exceed the player's maximum health
    pub fn heal(&self, orbs: &mut Orbs, amount: f32) {
        self.inventory.heal(orbs, amount, self.settings.difficulty);
    }
    /// Replenish energy, but don't exceed the player's maximum energy
    pub fn recharge(&self, orbs: &mut Orbs, amount: f32) {
        self.inventory
            .recharge(orbs, amount, self.settings.difficulty);
    }
    /// Apply the refill from a [`RefillValue`] to a set of [`OrbVariants`]
    pub(crate) fn refill(&self, refill: RefillValue, orb_variants: &mut OrbVariants) {
        self.inventory
            .refill(refill, orb_variants, self.settings.difficulty);
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
    pub fn destroy_cost<const TARGET_IS_WALL: bool>(
        &self,
        target_health: f32,
        flying_target: bool,
    ) -> Option<f32> {
        self.inventory
            .destroy_cost::<TARGET_IS_WALL>(target_health, flying_target, self.settings)
    }
    pub fn destroy_cost_ranged(&self, target_health: f32, flying_target: bool) -> Option<f32> {
        self.inventory
            .destroy_cost_ranged(target_health, flying_target, self.settings)
    }
    pub fn destroy_cost_with(&self, target_health: f32, weapon: Skill, flying_target: bool) -> f32 {
        self.inventory
            .destroy_cost_with(target_health, weapon, flying_target, self.settings)
    }

    pub fn owned_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.inventory
            .owned_weapons::<TARGET_IS_WALL>(self.settings)
    }
    pub fn owned_ranged_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.inventory.owned_ranged_weapons(self.settings)
    }
    pub fn owned_shield_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.inventory.owned_shield_weapons(self.settings)
    }

    pub fn progression_weapons<const TARGET_IS_WALL: bool>(&self) -> SmallVec<[Skill; 9]> {
        self.inventory
            .progression_weapons::<TARGET_IS_WALL>(self.settings)
    }
    pub fn ranged_progression_weapons(&self) -> SmallVec<[Skill; 6]> {
        self.inventory.ranged_progression_weapons(self.settings)
    }
    pub fn shield_progression_weapons(&self) -> SmallVec<[Skill; 4]> {
        self.inventory.shield_progression_weapons(self.settings)
    }
}
