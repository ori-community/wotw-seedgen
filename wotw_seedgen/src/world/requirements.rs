use std::slice;

use rustc_hash::FxHashSet;
use smallvec::{SmallVec, smallvec};

use super::player::Player;
use crate::inventory::Inventory;
use crate::item::{Item, Resource, Skill, Shard, Teleporter};
use crate::settings::{Difficulty, Trick, WorldSettings};
use crate::util::{Enemy, orbs::{self, Orbs}};

type Itemset = Vec<(Inventory, Orbs)>;

#[derive(Debug, Clone)]
pub enum Requirement {
    Free,
    Impossible,
    Difficulty(Difficulty),
    NormalGameDifficulty,
    Trick(Trick),
    Skill(Skill),
    EnergySkill(Skill, f32),
    NonConsumingEnergySkill(Skill),
    SpiritLight(u32),
    Resource(Resource, u32),
    Shard(Shard),
    Teleporter(Teleporter),
    Water,
    State(usize),
    Damage(f32),
    Danger(f32),
    Combat(SmallVec<[(Enemy, u8); 12]>),
    Boss(f32),
    BreakWall(f32),
    ShurikenBreak(f32),
    SentryBreak(f32),
    And(Vec<Requirement>),
    Or(Vec<Requirement>),
}
impl Requirement {
    fn cost_is_met(cost: f32, player: &Player, mut orb_variants: SmallVec<[Orbs; 3]>) -> SmallVec<[Orbs; 3]> {
        orb_variants.retain(|orbs| {
            if !Requirement::nonconsuming_cost_is_met_single(cost, player, orbs) { return false }
            if orbs.energy >= cost {
                orbs.energy -= cost;
            } else {
                orbs.energy = 0.0;
                orbs.health -= (cost - orbs.energy) * player.defense_mod();
            }
            true
        });
        orb_variants
    }
    fn nonconsuming_cost_is_met(cost: f32, player: &Player, mut orb_variants: SmallVec<[Orbs; 3]>) -> SmallVec<[Orbs; 3]> {
        orb_variants.retain(|orbs| Requirement::nonconsuming_cost_is_met_single(cost, player, orbs));
        orb_variants
    }
    fn nonconsuming_cost_is_met_single(cost: f32, player: &Player, orbs: &mut Orbs) -> bool {
        if orbs.energy >= cost {
            true
        } else if player.settings.difficulty >= Difficulty::Unsafe
        && player.inventory.has(&Item::Shard(Shard::LifePact), 1) {
            loop {
                let game_thinks_health_cost = cost - orbs.energy;
                let health_cost = game_thinks_health_cost * player.defense_mod();

                let unmet_cost = if orbs.health > game_thinks_health_cost {
                    if orbs.health > health_cost {
                        break true;
                    } else { health_cost }
                } else { game_thinks_health_cost };
                if !Requirement::regenerate_as_needed(unmet_cost, player, orbs) { return false }
            }
        } else { false }
    }
    fn health_is_met(cost: f32, player: &Player, mut orb_variants: SmallVec<[Orbs; 3]>) -> SmallVec<[Orbs; 3]> {
        orb_variants.retain(|orbs| {
            if !Requirement::nonconsuming_health_is_met_single(cost, player, orbs) { return false }
            orbs.health -= cost;
            true
        });
        orb_variants
    }
    fn nonconsuming_health_is_met(cost: f32, player: &Player, mut orb_variants: SmallVec<[Orbs; 3]>) -> SmallVec<[Orbs; 3]> {
        orb_variants.retain(|orbs| Requirement::nonconsuming_health_is_met_single(cost, player, orbs));
        orb_variants
    }
    fn nonconsuming_health_is_met_single(cost: f32, player: &Player, orbs: &mut Orbs) -> bool {
        orbs.health > cost || (
            player.inventory.has(&Item::Skill(Skill::Regenerate), 1) && player.max_health() > cost
            && Requirement::regenerate_as_needed(cost, player, orbs)
        )
    }
    fn regenerate_as_needed(cost: f32, player: &Player, orbs: &mut Orbs) -> bool {
        let mut regens = ((cost - orbs.health) / 30.0).ceil();
        if orbs.health + 30.0 * regens <= cost { regens += 1.0 }
        orbs.heal(30.0 * regens, player);
        let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
        let regen_cost = player.use_cost(Skill::Regenerate);
        // Regenerate is special cased to not allow Life Pact, so we don't go through cost_is_met
        orbs.energy -= regen_cost * regens;
        orbs.energy >= 0.0 && orbs.energy + regen_cost - game_thinks_regen_cost >= 0.0
    }

    pub fn is_met(&self, player: &Player, states: &FxHashSet<usize>, mut orb_variants: SmallVec<[Orbs; 3]>) -> SmallVec<[Orbs; 3]> {
        match self {
            Requirement::Free => return orb_variants,
            Requirement::Impossible => {},
            Requirement::Difficulty(difficulty) =>
                if player.settings.difficulty >= *difficulty { return orb_variants }
            Requirement::NormalGameDifficulty =>
                if !player.settings.hard { return orb_variants }
            Requirement::Trick(trick) =>
                if player.settings.tricks.contains(trick) { return orb_variants }
            Requirement::Skill(skill) =>
                if player.inventory.has(&Item::Skill(*skill), 1) { return orb_variants },
            Requirement::EnergySkill(skill, amount) =>
                if player.inventory.has(&Item::Skill(*skill), 1) {
                    let cost = player.use_cost(*skill) * *amount;
                    return Requirement::cost_is_met(cost, player, orb_variants);
                }
            Requirement::NonConsumingEnergySkill(skill) =>
                if player.inventory.has(&Item::Skill(*skill), 1) {
                    let cost = player.use_cost(*skill);
                    return Requirement::nonconsuming_cost_is_met(cost, player, orb_variants);
                }
            Requirement::SpiritLight(amount) =>
                if player.inventory.has(&Item::SpiritLight(1), *amount) { return orb_variants },
            Requirement::Resource(resource, amount) =>
                if player.inventory.has(&Item::Resource(*resource), *amount) { return orb_variants },
            Requirement::Shard(shard) =>
                if player.inventory.has(&Item::Shard(*shard), 1) { return orb_variants },
            Requirement::Teleporter(teleporter) =>
                if player.inventory.has(&Item::Teleporter(*teleporter), 1) { return orb_variants },
            Requirement::Water =>
                if player.inventory.has(&Item::Water, 1) { return orb_variants },
            Requirement::State(state) =>
                if states.contains(state) { return orb_variants },
            Requirement::Damage(amount) => {
                let cost = *amount * player.defense_mod();
                return Requirement::health_is_met(cost, player, orb_variants);
            },
            Requirement::Danger(amount) => {
                let cost = *amount * player.defense_mod();
                return Requirement::nonconsuming_health_is_met(cost, player, orb_variants);
            },
            Requirement::BreakWall(health) =>
                if let Some(weapon) = player.preferred_weapon(true) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    return Requirement::cost_is_met(cost, player, orb_variants);
                }
            Requirement::Boss(health) =>
                // TODO rock boss is flying, just placing a todo in case rock boss will be logic relevant someday
                if let Some(weapon) = player.preferred_weapon(false) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    return Requirement::cost_is_met(cost, player, orb_variants);
                }
            Requirement::Combat(enemies) => {
                if let Some(weapon) = player.preferred_weapon(false) {
                    let ranged_weapon = player.preferred_ranged_weapon();
                    let shield_weapon = player.preferred_shield_weapon();

                    // Short circuit when missing skills
                    for (enemy, _) in enemies {
                        if player.settings.difficulty < Difficulty::Unsafe && (
                            (enemy.aerial() && !(
                                player.inventory.has(&Item::Skill(Skill::DoubleJump), 1)
                                || player.inventory.has(&Item::Skill(Skill::Launch), 1)
                                || player.settings.difficulty >= Difficulty::Gorlek && player.inventory.has(&Item::Skill(Skill::Bash), 1)
                            ))
                            || (enemy.dangerous() && !(
                                player.inventory.has(&Item::Skill(Skill::DoubleJump), 1)
                                || player.inventory.has(&Item::Skill(Skill::Dash), 1)
                                || player.inventory.has(&Item::Skill(Skill::Bash), 1)
                                || player.inventory.has(&Item::Skill(Skill::Launch), 1)
                            ))
                            || ((*enemy == Enemy::Bat && !player.inventory.has(&Item::Skill(Skill::Bash), 1)))
                            || (*enemy == Enemy::Sandworm && !player.inventory.has(&Item::Skill(Skill::Burrow), 1))
                        )
                        || (enemy.shielded() && shield_weapon.is_none())
                        || (enemy.ranged() && ranged_weapon.is_none())
                        { return smallvec![] }
                    }

                    let mut cost = 0.;

                    for (enemy, amount) in enemies {
                        if let Enemy::EnergyRefill = enemy {
                            // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                            // However in between energy refills, the cost is always the same
                            orb_variants = Requirement::cost_is_met(cost, player, orb_variants);
                            if orb_variants.is_empty() { return orb_variants }
                            for orbs in &mut orb_variants {
                                orbs.recharge(f32::from(*amount), player);
                            }
                            cost = 0.;
                            continue;
                        }

                        if enemy.shielded() {
                            cost += player.use_cost(shield_weapon.unwrap()) * f32::from(*amount);
                        }
                        let armor_mod = if enemy.armored() && player.settings.difficulty < Difficulty::Unsafe { 2.0 } else { 1.0 };
                        let ranged = enemy.ranged();
                        let used_weapon = if ranged { ranged_weapon.unwrap() } else { weapon };

                        cost += player.destroy_cost(enemy.health(), used_weapon, enemy.flying()) * f32::from(*amount) * armor_mod;
                    }

                    return Requirement::cost_is_met(cost, player, orb_variants);
                }
            },
            Requirement::ShurikenBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Shuriken), 1) {
                    let clip_mod = if player.settings.difficulty >= Difficulty::Unsafe { 2.0 } else { 3.0 };
                    let cost = player.destroy_cost(*health, Skill::Shuriken, false) * clip_mod;
                    return Requirement::cost_is_met(cost, player, orb_variants);
                },
            Requirement::SentryBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Sentry), 1) {
                    let clip_mod = 0.16;
                    let cost = player.destroy_cost(*health, Skill::Sentry, false) * clip_mod;
                    return Requirement::cost_is_met(cost, player, orb_variants);
                },
            Requirement::And(requirements) => {
                for and in requirements {
                    orb_variants = and.is_met(player, states, orb_variants);
                }
                return orb_variants;
            },
            Requirement::Or(requirements) => {
                let mut cheapest = SmallVec::<[Orbs; 3]>::new();

                for or in requirements {
                    let orbcost = or.is_met(player, states, orb_variants.clone());
                    if !orbcost.is_empty() {
                        if cheapest.is_empty() {
                            cheapest = orbcost;
                        } else {
                            cheapest = orbs::either(&cheapest, &orbcost);
                        }
                        if cheapest[0] == Orbs::default() {
                            break;
                        }
                    }
                }

                return cheapest;
            },
        }
        smallvec![]
    }

    fn needed_for_cost(cost: f32, player: &Player) -> Itemset {
        let mut itemsets = vec![(Inventory::default(), Orbs{ energy: -cost, ..Orbs::default() })];

        if player.settings.difficulty >= Difficulty::Unsafe && cost > 0.0 && !player.inventory.has(&Item::Shard(Shard::Overcharge), 1) {
            itemsets.push((Inventory::from(Item::Shard(Shard::Overcharge)), Orbs{ energy: -cost / 2.0, ..Orbs::default() }));
        }

        itemsets
    }
    // TODO damage buff progression?
    fn needed_for_weapon(weapon: Skill, cost: f32, player: &Player) -> Itemset {
        let mut itemsets = Requirement::needed_for_cost(cost, player);

        for (inventory, _) in &mut itemsets {
            inventory.grant(Item::Skill(weapon), 1);
        }

        itemsets
    }

    fn combine_itemsets(left: Itemset, right: &[(Inventory, Orbs)]) -> Itemset {
        let mut combined = Vec::new();
        for (left_inventory, left_orbs) in left {
            for (right_inventory, right_orbs) in right {
                let inventory = left_inventory.merge(right_inventory);
                let orbs = left_orbs + *right_orbs;
                combined.push((inventory, orbs));
            }
        };
        combined
    }
    fn combine_itemset_items(left: Itemset, right: &[Item]) -> Itemset {
        let mut combined = Vec::new();
        for (left_inventory, left_orbs) in left {
            for item in right {
                let mut inventory = left_inventory.clone();
                inventory.grant(item.clone(), 1);
                combined.push((inventory, left_orbs));
            }
        };
        combined
    }
    fn combine_itemset_item(left: &mut Itemset, right: &Item) {
        for (left_inventory, _) in left {
            left_inventory.grant(right.clone(), 1);
        };
    }

    pub fn items_needed(&self, player: &Player, states: &[usize]) -> Itemset {
        match self {
            Requirement::Free => vec![(Inventory::default(), Orbs::default())],
            Requirement::Impossible | Requirement::Difficulty(_) | Requirement::NormalGameDifficulty | Requirement::Trick(_) => vec![],
            Requirement::Skill(skill) => vec![(Inventory::from(Item::Skill(*skill)), Orbs::default())],
            Requirement::EnergySkill(skill, amount) => {
                let cost = player.use_cost(*skill) * *amount;
                let mut itemsets = Requirement::needed_for_cost(cost, player);
                Requirement::combine_itemset_item(&mut itemsets, &Item::Skill(*skill));

                itemsets
            },
            Requirement::NonConsumingEnergySkill(skill) => {
                let cost = player.use_cost(*skill);
                let mut itemsets = Requirement::needed_for_cost(cost, player);
                Requirement::combine_itemset_item(&mut itemsets, &Item::Skill(*skill));

                itemsets
            },
            Requirement::SpiritLight(amount) => vec![(Inventory::from((Item::SpiritLight(1), *amount)), Orbs::default())],
            Requirement::Resource(resource, amount) => vec![(Inventory::from((Item::Resource(*resource), *amount)), Orbs::default())],
            Requirement::Shard(shard) => vec![(Inventory::from(Item::Shard(*shard)), Orbs::default())],
            Requirement::Teleporter(teleporter) => vec![(Inventory::from(Item::Teleporter(*teleporter)), Orbs::default())],
            Requirement::Water => vec![(Inventory::from(Item::Water), Orbs::default())],
            Requirement::State(state) =>
                if states.contains(state) { vec![(Inventory::default(), Orbs::default())] } else { vec![] },
            Requirement::Damage(amount) | Requirement::Danger(amount) => {
                let mut itemsets = Vec::new();

                let cost = *amount * player.defense_mod();

                itemsets.push((Inventory::default(), Orbs { health: -cost, ..Orbs::default() }));

                if player.settings.difficulty >= Difficulty::Gorlek && !player.inventory.has(&Item::Shard(Shard::Resilience), 1) {
                    let resilience_cost = cost * 0.9;

                    itemsets.push((Inventory::from(Item::Shard(Shard::Resilience)), Orbs { health: -resilience_cost, ..Orbs::default() }));
                }

                itemsets
            },
            Requirement::BreakWall(health) => {
                // TODO damage buff progressions
                let mut itemsets = Vec::new();

                for weapon in player.progression_weapons(true) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    itemsets.append(&mut Requirement::needed_for_weapon(weapon, cost, player));
                }

                itemsets
            },
            Requirement::Boss(health) => {
                let mut itemsets = Vec::new();

                for weapon in player.progression_weapons(false) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    itemsets.append(&mut Requirement::needed_for_weapon(weapon, cost, player));
                }

                itemsets
            },
            Requirement::ShurikenBreak(health) => {
                let clip_mod = if player.settings.difficulty >= Difficulty::Unsafe { 2.0 } else { 3.0 };
                let cost = player.destroy_cost(*health, Skill::Shuriken, false) * clip_mod;
                Requirement::needed_for_weapon(Skill::Shuriken, cost, player)
            },
            Requirement::SentryBreak(health) => {
                let clip_mod = 0.16;
                let cost = player.destroy_cost(*health, Skill::Sentry, false) * clip_mod;
                Requirement::needed_for_weapon(Skill::Sentry, cost, player)
            },
            Requirement::Combat(enemies) => {
                let mut itemsets = Vec::<(Inventory, Orbs)>::new();

                let (mut aerial, mut dangerous, mut ranged, mut melee, mut shielded, mut bash, mut burrow) = (false, false, false, false, false, false, false);

                for (enemy, _) in enemies {
                    if enemy.aerial() { aerial = true; }
                    if enemy.dangerous() { dangerous = true; }
                    if enemy.ranged() { ranged = true; }
                    else { melee = true; }
                    if enemy.shielded() { shielded = true; }
                    if player.settings.difficulty < Difficulty::Unsafe && enemy == &Enemy::Bat { bash = true; }
                    if enemy == &Enemy::Sandworm { burrow = true; }
                }

                // Skip unneccesary iterations over weapons that are redundant anyway
                let weapons = if melee {
                    player.progression_weapons(false)
                } else { smallvec![Skill::Sword] };
                let ranged_weapons = if ranged {
                    player.ranged_progression_weapons()
                } else { smallvec![Skill::Spear] };
                let shield_weapons = if shielded {
                    player.shield_progression_weapons()
                } else { smallvec![Skill::Spear] };
                let use_burrow: SmallVec<[_; 2]> = if burrow {
                    if player.settings.difficulty < Difficulty::Unsafe || player.inventory.has(&Item::Skill(Skill::Burrow), 1) {
                        smallvec![true]
                    } else {
                        smallvec![true, false]
                    }
                } else { smallvec![false] };

                // Filter combinations of weapons for redundancies
                let weapons_len = weapons.len();
                let mut weapon_combinations = Vec::<SmallVec<[_; 3]>>::with_capacity(weapons_len * ranged_weapons.len() * shield_weapons.len());
                for ranged_weapon in ranged_weapons {
                    for &shield_weapon in &shield_weapons {
                        let weapon_position = weapons.iter()
                            .position(|&weapon| weapon == ranged_weapon || weapon == shield_weapon)
                            .map_or(weapons_len, |index| (index + 1).min(weapons_len));
                        for weapon in &weapons[0..weapon_position] {
                            weapon_combinations.push(smallvec![*weapon, ranged_weapon, shield_weapon]);
                        }
                    }
                }

                for weapons in weapon_combinations {
                    let weapon = weapons[0];
                    let ranged_weapon = weapons[1];
                    let shield_weapon = weapons[2];
                    for &burrow in &use_burrow {
                        let (mut cost, mut highest_cost) = (0.0, 0.0);

                        for (enemy, amount) in enemies {
                            if let Enemy::EnergyRefill = enemy {
                                if cost > highest_cost { highest_cost = cost; }
                                cost = 0_f32.max(cost - f32::from(*amount));
                                continue;
                            }

                            if enemy == &Enemy::Sandworm && burrow { continue; }

                            if enemy.shielded() {
                                cost += player.use_cost(shield_weapon) * f32::from(*amount);
                            }
                            let armor_mod = if enemy.armored() && player.settings.difficulty < Difficulty::Unsafe { 2.0 } else { 1.0 };

                            let used_weapon = if enemy.ranged() { ranged_weapon } else { weapon };

                            cost += player.destroy_cost(enemy.health(), used_weapon, enemy.flying()) * f32::from(*amount) * armor_mod;
                        }
                        if cost > highest_cost { highest_cost = cost; }

                        let mut itemset = Requirement::needed_for_cost(highest_cost, player);
                        for (inventory, _) in &mut itemset {
                            if melee { inventory.grant(Item::Skill(weapon), 1) }
                            if ranged { inventory.grant(Item::Skill(ranged_weapon), 1) }
                            if shielded && !inventory.has(&Item::Skill(shield_weapon), 1) { inventory.grant(Item::Skill(shield_weapon), 1) }
                            if burrow { inventory.grant(Item::Skill(Skill::Burrow), 1) }
                        }

                        itemsets.append(&mut itemset);
                    }
                }

                if player.settings.difficulty < Difficulty::Unsafe && aerial {
                    let mut ranged_skills = vec![
                        Item::Skill(Skill::DoubleJump),
                        Item::Skill(Skill::Launch),
                    ];
                    if player.settings.difficulty >= Difficulty::Gorlek { ranged_skills.push(Item::Skill(Skill::Bash)); }

                    if !ranged_skills.iter().any(|skill| player.inventory.has(skill, 1)) {
                        itemsets = Requirement::combine_itemset_items(itemsets, &ranged_skills);
                    }
                }
                if player.settings.difficulty < Difficulty::Unsafe && dangerous {
                    let evasion_skills = [
                        Item::Skill(Skill::DoubleJump),
                        Item::Skill(Skill::Dash),
                        Item::Skill(Skill::Bash),
                        Item::Skill(Skill::Launch),
                    ];

                    if !evasion_skills.iter().any(|skill| player.inventory.has(skill, 1)) {
                        itemsets = Requirement::combine_itemset_items(itemsets, &evasion_skills);
                    }
                }
                if player.settings.difficulty < Difficulty::Unsafe && bash && !player.inventory.has(&Item::Skill(Skill::Bash), 1) {
                    Requirement::combine_itemset_item(&mut itemsets, &Item::Skill(Skill::Bash));
                }

                itemsets
            },
            Requirement::And(ands) => {
                let mut tail = ands.iter().map(|and| and.items_needed(player, states));
                let head = tail.next().unwrap_or_default();
                tail.fold(head, |acc, next| {
                    Requirement::combine_itemsets(acc, &next)
                })
            },
            Requirement::Or(ors) => {
                ors.iter()
                    .flat_map(|or| or.items_needed(player, states))
                    .collect()
            },
        }
    }

    /// Checks whether this [`Requirement`] is possible to meet with the given settings
    pub(crate) fn is_possible_for(&self, settings: &WorldSettings) -> bool {
        match self {
            Requirement::Impossible => false,
            Requirement::Difficulty(difficulty) => settings.difficulty >= *difficulty,
            Requirement::NormalGameDifficulty => !settings.hard,
            Requirement::Trick(trick) => settings.tricks.contains(trick),
            Requirement::And(nested) => nested.iter().all(|requirement| requirement.is_possible_for(settings)),
            Requirement::Or(nested) => nested.iter().any(|requirement| requirement.is_possible_for(settings)),
            _ => true,
        }
    }

    pub(crate) fn contained_requirements<'a, 'b>(&'a self, settings: &'b WorldSettings) -> ContainedRequirements<'a, 'b> {
        ContainedRequirements::new(self, settings)
    }
}

pub(crate) struct ContainedRequirements<'a, 'b> {
    nested: Vec<slice::Iter<'a, Requirement>>,
    settings: &'b WorldSettings,
}
impl<'a, 'b> ContainedRequirements<'a, 'b> {
    pub(crate) fn new(requirement: &'a Requirement, settings: &'b WorldSettings) -> ContainedRequirements<'a, 'b> {
        ContainedRequirements {
            nested: vec![slice::from_ref(requirement).iter()],
            settings,
        }
    }
}
impl<'a> Iterator for ContainedRequirements<'a, '_> {
    type Item = &'a Requirement;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: loop {
            let current = self.nested.last_mut()?;
            loop {
                match current.next() {
                    Some(requirement) => {
                        if requirement.is_possible_for(&self.settings) {
                            match requirement {
                                Requirement::And(nested) | Requirement::Or(nested) => self.nested.push(nested.iter()),
                                req @ _ => return Some(req)
                            }
                            continue 'outer;
                        }
                    },
                    None => {
                        self.nested.pop();
                        continue 'outer;
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::WorldSettings;

    #[test]
    fn is_met() {
        macro_rules! test {
            ($player:expr, $states:expr, $req:expr, [...]) => {
                assert!(!$req.is_met($player, $states, smallvec![$player.max_orbs()]).is_empty());
            };
            ($player:expr, $states:expr, $req:expr, [$player_orbs:expr], [...]) => {
                assert!(!$req.is_met($player, $states, smallvec![$player_orbs]).is_empty());
            };
            ($player:expr, $states:expr, $req:expr, [$player_orbs:expr], [$($orbs:expr),* $(,)?]) => {
                {
                    let sort = |mut orbs: smallvec::SmallVec<[Orbs; 3]>| { orbs.sort_unstable_by(|a, b| a.health.partial_cmp(&b.health).expect("non-real orb value")); orbs };
                    assert_eq!(sort($req.is_met($player, $states, smallvec![$player_orbs])), sort(smallvec![$($player_orbs + $orbs),*]));
                }
            };
            ($player:expr, $states:expr, $req:expr, [$($orbs:tt)*]) => {
                test!($player, $states, $req, [$player.max_orbs()], [$($orbs)*]);
            };
        }

        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);

        player.inventory.grant(Item::Resource(Resource::Health), 1);
        let mut states = FxHashSet::default();
        let orbs = Orbs::default();

        test!(&player, &states, Requirement::Skill(Skill::Blaze), []);
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        test!(&player, &states, Requirement::Skill(Skill::Blaze), [...]);

        test!(&player, &states, Requirement::And(vec![Requirement::Skill(Skill::Blaze), Requirement::Free]), [...]);
        test!(&player, &states, Requirement::Or(vec![Requirement::Skill(Skill::Blaze), Requirement::Impossible]), [...]);

        test!(&player, &states, Requirement::EnergySkill(Skill::Blaze, 1.0), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        test!(&player, &states, Requirement::EnergySkill(Skill::Blaze, 1.0), []);

        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::EnergySkill(Skill::Blaze, 1.0), [Orbs { energy: -1.0, ..orbs }]);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        test!(&player, &states, Requirement::EnergySkill(Skill::Blaze, 1.0), [Orbs { energy: -2.0, ..orbs }]);

        test!(&player, &states, Requirement::State(34), []);
        states.insert(34);
        test!(&player, &states, Requirement::State(34), [...]);
        test!(&player, &states, Requirement::State(33), []);

        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::Damage(30.0), []);
        player.inventory.grant(Item::Resource(Resource::Health), 5);
        test!(&player, &states, Requirement::Damage(30.0), []);
        player.inventory.grant(Item::Resource(Resource::Health), 1);
        test!(&player, &states, Requirement::Damage(30.0), [Orbs { health: -30.0, ..orbs }]);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
        test!(&player, &states, Requirement::Damage(60.0), []);
        player.inventory.grant(Item::Resource(Resource::Health), 6);
        test!(&player, &states, Requirement::Damage(60.0), [Orbs { health: 30.0, energy: player.max_energy() }], [Orbs { health: -25.0, energy: -2.0 }]);
        test!(&player, &states, Requirement::Danger(30.0), [Orbs { health: 30.0, energy: player.max_energy() }], [Orbs { health: 30.0, energy: -1.0 }]);
        test!(&player, &states, Requirement::Danger(60.0), [Orbs { health: 30.0, energy: player.max_energy() }], [Orbs { health: 35.0, energy: -2.0 }]);

        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player = Player::new(&world_settings);
        test!(&player, &states, Requirement::BreakWall(12.0), []);
        player.inventory.grant(Item::Skill(Skill::Sword), 1);
        test!(&player, &states, Requirement::BreakWall(12.0), [player.max_orbs()]);
        player = Player::new(&world_settings);
        player.inventory.grant(Item::Skill(Skill::Grenade), 1);
        test!(&player, &states, Requirement::BreakWall(12.0), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 3);
        test!(&player, &states, Requirement::BreakWall(12.0), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        test!(&player, &states, Requirement::BreakWall(12.0), [Orbs { energy: -2.0, ..orbs }]);
        player = Player::new(&world_settings);
        player.inventory.grant(Item::Skill(Skill::Grenade), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::BreakWall(16.0), [Orbs { energy: -1.0, ..orbs }]);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        test!(&player, &states, Requirement::BreakWall(12.0), []);

        player = Player::new(&world_settings);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::ShurikenBreak(12.0), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        test!(&player, &states, Requirement::ShurikenBreak(12.0), [Orbs { energy: -2.0, ..orbs }]);
        player.inventory.grant(Item::Resource(Resource::Energy), 6);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::ShurikenBreak(12.0), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        test!(&player, &states, Requirement::ShurikenBreak(12.0), [Orbs { energy: -6.0, ..orbs }]);

        player = Player::new(&world_settings);
        player.inventory.grant(Item::Skill(Skill::Bow), 1);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 7);
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]), [Orbs { energy: -3.25, ..orbs }]);
        player.inventory.grant(Item::Resource(Resource::Energy), 6);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]), []);
        player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]), [Orbs { energy: -6.5, ..orbs }]);
        player = Player::new(&world_settings);
        let req = Requirement::Combat(smallvec![(Enemy::Sandworm, 1), (Enemy::Bat, 1), (Enemy::EnergyRefill, 99), (Enemy::ShieldMiner, 2), (Enemy::EnergyRefill, 1), (Enemy::Balloon, 4)]);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        player.inventory.grant(Item::Skill(Skill::Spear), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 27);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, &req, []);
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        test!(&player, &states, &req, [Orbs { energy: -14.0, ..orbs }]);
        player.inventory.grant(Item::Resource(Resource::Energy), 37);
        player.inventory.grant(Item::Skill(Skill::Bash), 1);
        player.inventory.grant(Item::Skill(Skill::Launch), 1);
        player.inventory.grant(Item::Skill(Skill::Burrow), 1);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, &req, []);
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        test!(&player, &states, &req, [Orbs { energy: -33.0, ..orbs }]);
        player = Player::new(&world_settings);
        player.inventory.grant(Item::Skill(Skill::Spear), 1);
        player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        let world_settings = WorldSettings { difficulty: Difficulty::Gorlek, ..WorldSettings::default() };
        player.settings = &world_settings;
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]), [Orbs { energy: -2.0, ..orbs }]);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 11);
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]), []);
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        test!(&player, &states, Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]), [Orbs { energy: -8.0, ..orbs }]);

        player = Player::new(&world_settings);
        let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
        let b = Requirement::Damage(20.0);
        let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
        let d = Requirement::Damage(10.0);
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        player.inventory.grant(Item::Resource(Resource::Health), 5);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        test!(&player, &states, Requirement::And(vec![c.clone(), d.clone()]), [Orbs { health: -10.0, energy: -1.0 }]);
        test!(&player, &states, Requirement::Or(vec![a.clone(), b.clone()]), [Orbs { energy: -2.0, ..orbs }, Orbs { health: -20.0, ..orbs }]);
        test!(&player, &states, Requirement::Or(vec![Requirement::And(vec![a.clone(), b.clone()]), Requirement::And(vec![c.clone(), d.clone()]), a.clone(), b.clone()]),
            [Orbs { energy: -1.0, health: -10.0 }, Orbs { energy: -2.0, ..orbs }, Orbs { health: -20.0, ..orbs }]);
        test!(&player, &states, Requirement::And(vec![Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()])]),
            [Orbs { energy: -1.0, health: -10.0 }]);
        player.inventory.grant(Item::Resource(Resource::Energy), 8);
        player.inventory.grant(Item::Resource(Resource::Health), 8);
        test!(&player, &states, Requirement::And(vec![Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()]), Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()])]),
            [Orbs { energy: -6.0, ..orbs }, Orbs { energy: -4.0, health: -10.0 }, Orbs { health: -60.0, ..orbs }, Orbs { energy: -1.0, health: -40.0 }, Orbs { energy: -2.0, health: -20.0 }]);
        test!(&player, &states, Requirement::Or(vec![Requirement::Free, b.clone()]), [Orbs::default()]);
        test!(&player, &states, Requirement::Or(vec![b.clone(), Requirement::Free]), [Orbs::default()]);

        player = Player::new(&world_settings);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        player.inventory.grant(Item::Resource(Resource::Health), 7);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        test!(&player, &states, Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]), []);
        player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
        test!(&player, &states, Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]), [Orbs { energy: -1.0, health: -30.0 }]);

        let req = Requirement::Or(vec![Requirement::Damage(10.0), Requirement::EnergySkill(Skill::Blaze, 1.0)]);
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        test!(&player, &states, Requirement::And(vec![req.clone(), req.clone()]), [Orbs { health: -20.0, ..orbs }, Orbs { health: -10.0, energy: -1.0 }, Orbs { energy: -2.0, ..orbs }]);
    }

    #[test]
    fn items_needed() {
        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);
        let states = Vec::default();
        let orbs = Orbs::default();

        let req = Requirement::Free;
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::default(), orbs)]);
        let req = Requirement::Impossible;
        assert_eq!(req.items_needed(&player, &states), vec![]);
        let req = Requirement::Or(vec![Requirement::Free, Requirement::Impossible]);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::default(), orbs)]);
        let req = Requirement::And(vec![Requirement::Free, Requirement::Impossible]);
        assert_eq!(req.items_needed(&player, &states), vec![]);

        let req = Requirement::Skill(Skill::Dash);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Skill(Skill::Dash)), orbs)]);
        let req = Requirement::Or(vec![Requirement::Skill(Skill::Dash), Requirement::Skill(Skill::Bash)]);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Skill(Skill::Dash)), orbs), (Inventory::from(Item::Skill(Skill::Bash)), orbs)]);
        let req = Requirement::And(vec![Requirement::Skill(Skill::Dash), Requirement::Skill(Skill::Bash)]);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(vec![Item::Skill(Skill::Dash), Item::Skill(Skill::Bash)]), orbs)]);

        let req = Requirement::EnergySkill(Skill::Grenade, 2.0);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -4.0, ..orbs })]);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Grenade), Item::Shard(Shard::Overcharge)]), Orbs { energy: -1.0, ..orbs }),
        ]);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;

        let req = Requirement::Resource(Resource::ShardSlot, 3);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from((Item::Resource(Resource::ShardSlot), 3)), orbs)]);
        let req = Requirement::Shard(Shard::Overflow);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Shard(Shard::Overflow)), orbs)]);
        let req = Requirement::Teleporter(Teleporter::Glades);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Teleporter(Teleporter::Glades)), orbs)]);
        let req = Requirement::Water;
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::from(Item::Water), orbs)]);

        let req = Requirement::Damage(36.0);
        assert_eq!(req.items_needed(&player, &states), vec![(Inventory::default(), Orbs { health: -36.0, ..orbs })]);
        let world_settings = WorldSettings { difficulty: Difficulty::Gorlek, ..WorldSettings::default() };
        player.settings = &world_settings;
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::default(), Orbs { health: -36.0, ..orbs }),
            (Inventory::from(Item::Shard(Shard::Resilience)), Orbs { health: -36.0 * 0.9, ..orbs }),
        ]);
        let world_settings = WorldSettings { difficulty: Difficulty::Moki, ..WorldSettings::default() };
        player.settings = &world_settings;

        let req = Requirement::BreakWall(12.0);
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Sword)), orbs),
            (Inventory::from(Item::Skill(Skill::Hammer)), orbs),
            (Inventory::from(Item::Skill(Skill::Bow)), Orbs { energy: -1.5, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Shuriken)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Blaze)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Spear)), Orbs { energy: -4.0, ..orbs }),
        ]);
        let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
        player.settings = &world_settings;
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Sword)), orbs),
            (Inventory::from(Item::Skill(Skill::Hammer)), orbs),
            (Inventory::from(Item::Skill(Skill::Bow)), Orbs { energy: -0.75, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Bow), Item::Shard(Shard::Overcharge)]), Orbs { energy: -0.75 * 0.5, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -1.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Grenade), Item::Shard(Shard::Overcharge)]), Orbs { energy: -0.5, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Shuriken)), Orbs { energy: -1.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Shuriken), Item::Shard(Shard::Overcharge)]), Orbs { energy: -0.5, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Blaze)), Orbs { energy: -1.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Blaze), Item::Shard(Shard::Overcharge)]), Orbs { energy: -0.5, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Spear)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Spear), Item::Shard(Shard::Overcharge)]), Orbs { energy: -1.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Sentry)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Sentry), Item::Shard(Shard::Overcharge)]), Orbs { energy: -1.0, ..orbs }),
        ]);
        player.inventory.grant(Item::Skill(Skill::Bow), 1);
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Sword)), orbs),
            (Inventory::from(Item::Skill(Skill::Hammer)), orbs),
            (Inventory::from(Item::Skill(Skill::Bow)), Orbs { energy: -0.75, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Bow), Item::Shard(Shard::Overcharge)]), Orbs { energy: -0.75 * 0.5, ..orbs }),
        ]);

        let req = Requirement::Combat(smallvec![(Enemy::Slug, 1)]);
        let world_settings = WorldSettings::default();
        let player = Player::new(&world_settings);

        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Sword)), orbs),
            (Inventory::from(Item::Skill(Skill::Hammer)), orbs),
            (Inventory::from(Item::Skill(Skill::Bow)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Shuriken)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Blaze)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Flash)), Orbs { energy: -4.0, ..orbs }),
            (Inventory::from(Item::Skill(Skill::Spear)), Orbs { energy: -4.0, ..orbs }),
        ]);
    }
}
