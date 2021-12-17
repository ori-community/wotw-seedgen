use rustc_hash::FxHashSet;
use smallvec::{SmallVec, smallvec};

use super::player::Player;
use crate::inventory::Inventory;
use crate::item::{Item, Resource, Skill, Shard, Teleporter};
use crate::util::{Difficulty, Enemy, orbs::{self, Orbs}};

type Itemset = Vec<(Inventory, Orbs)>;

#[derive(Debug, Clone)]
pub enum Requirement {
    Free,
    Impossible,
    Skill(Skill),
    EnergySkill(Skill, f32),
    NonConsumingEnergySkill(Skill),
    SpiritLight(u16),
    Resource(Resource, u16),
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
    fn cost_is_met(cost: f32, player: &Player, orbs: Orbs) -> Option<SmallVec<[Orbs; 3]>> {
        if orbs.energy >= cost { Some(smallvec![
            Orbs {
                energy: -cost,
                ..Orbs::default()
            }
        ])} else if player.difficulty >= Difficulty::Unsafe && player.inventory.has(&Item::Shard(Shard::LifePact), 1) && orbs.energy + orbs.health > cost { Some(smallvec![
            Orbs {
                health: cost - orbs.energy,
                energy: -orbs.energy,
            }
        ])} else { None }
    }
    fn nonconsuming_cost_is_met(cost: f32, player: &Player, orbs: Orbs) -> Option<SmallVec<[Orbs; 3]>> {
        if orbs.energy >= cost || (
            player.difficulty >= Difficulty::Unsafe &&
            player.inventory.has(&Item::Shard(Shard::LifePact), 1) &&
            orbs.energy + orbs.health > cost
        ) {
            Some(smallvec![Orbs::default()])
        } else { None }
    }

    pub fn is_met(&self, player: &Player, states: &FxHashSet<usize>, orbs: Orbs) -> Option<SmallVec<[Orbs; 3]>> {
        match self {
            Requirement::Free => return Some(smallvec![Orbs::default()]),
            Requirement::Impossible => return None,
            Requirement::Skill(skill) =>
                if player.inventory.has(&Item::Skill(*skill), 1) { return Some(smallvec![Orbs::default()]); },
            Requirement::EnergySkill(skill, amount) =>
                if player.inventory.has(&Item::Skill(*skill), 1) {
                    let cost = player.use_cost(*skill) * *amount;
                    return Requirement::cost_is_met(cost, player, orbs);
                }
            Requirement::NonConsumingEnergySkill(skill) =>
                if player.inventory.has(&Item::Skill(*skill), 1) {
                    let cost = player.use_cost(*skill);
                    return Requirement::nonconsuming_cost_is_met(cost, player, orbs);
                }
            Requirement::SpiritLight(amount) =>
                if player.inventory.has(&Item::SpiritLight(1), *amount) { return Some(smallvec![Orbs::default()]); },
            Requirement::Resource(resource, amount) =>
                if player.inventory.has(&Item::Resource(*resource), *amount) { return Some(smallvec![Orbs::default()]); },
            Requirement::Shard(shard) =>
                if player.inventory.has(&Item::Shard(*shard), 1) { return Some(smallvec![Orbs::default()]); },
            Requirement::Teleporter(teleporter) =>
                if player.inventory.has(&Item::Teleporter(*teleporter), 1) { return Some(smallvec![Orbs::default()]); },
            Requirement::Water =>
                if player.inventory.has(&Item::Water, 1) { return Some(smallvec![Orbs::default()]); },
            Requirement::State(state) =>
                if states.contains(state) { return Some(smallvec![Orbs::default()]); },
            Requirement::Damage(amount) => {
                let cost = *amount * player.defense_mod();
                if orbs.health > cost { return Some(smallvec![
                    Orbs {
                        health: -cost,
                        ..Orbs::default()
                    }
                ])}
                else if player.inventory.has(&Item::Skill(Skill::Regenerate), 1) {
                    let max_health = player.max_health();
                    if max_health > cost {
                        let regens = ((cost - orbs.health) / 30.0).ceil();
                        let max_heal = max_health - orbs.health;
                        if orbs.energy >= regens { return Some(smallvec![
                            Orbs {
                                health: max_heal.min(-cost + 30.0 * regens),
                                energy: -regens,
                            }
                        ])}
                    }
                }
            },
            Requirement::Danger(amount) => {
                let cost = *amount * player.defense_mod();
                if orbs.health > cost {
                    return Some(smallvec![Orbs::default()]);
                }
                else if player.inventory.has(&Item::Skill(Skill::Regenerate), 1) {
                    let max_health = player.max_health();
                    if max_health > cost {
                        let regens = ((cost - orbs.health) / 30.0).ceil();
                        let max_heal = max_health - orbs.health;
                        if orbs.energy >= regens { return Some(smallvec![
                            Orbs {
                                health: max_heal.min(30.0 * regens),
                                energy: -regens,
                            }
                        ])}
                    }
                }
            },
            Requirement::BreakWall(health) =>
                if let Some(weapon) = player.preferred_weapon(true) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    return Requirement::cost_is_met(cost, player, orbs);
                }
            Requirement::Boss(health) =>
                // TODO rock boss is flying, just placing a todo in case rock boss will be logic relevant someday
                if let Some(weapon) = player.preferred_weapon(false) {
                    let cost = player.destroy_cost(*health, weapon, false);
                    return Requirement::cost_is_met(cost, player, orbs);
                }
            Requirement::Combat(enemies) => {  // TODO handle nests better
                if let Some(weapon) = player.preferred_weapon(false) {
                    let (mut aerial, mut dangerous) = (false, false);
                    let mut energy = orbs.energy;

                    let ranged_weapon = player.preferred_ranged_weapon();
                    let shield_weapon = player.preferred_shield_weapon();

                    for (enemy, amount) in enemies {
                        if let Enemy::EnergyRefill = enemy {
                            if energy < 0.0 { return None; }
                            energy = player.max_energy().min(energy + f32::from(*amount));
                            continue;
                        }

                        if enemy.aerial() { aerial = true; }
                        if enemy.dangerous() { dangerous = true; }
                        if player.difficulty < Difficulty::Unsafe && enemy == &Enemy::Bat && !player.inventory.has(&Item::Skill(Skill::Bash), 1) { return None; }
                        if enemy == &Enemy::Sandworm {
                            if player.inventory.has(&Item::Skill(Skill::Burrow), 1) { continue; }
                            else if player.difficulty < Difficulty::Unsafe { return None; }
                        }

                        if enemy.shielded() {
                            if let Some(weapon) = shield_weapon {
                                energy -= player.use_cost(weapon) * f32::from(*amount);
                            } else { return None; }
                        }
                        let armor_mod = if enemy.armored() && player.difficulty < Difficulty::Unsafe { 2.0 } else { 1.0 };

                        let ranged = enemy.ranged();
                        if ranged && ranged_weapon.is_none() { return None; }
                        let used_weapon = if ranged { ranged_weapon.unwrap() } else { weapon };

                        energy -= player.destroy_cost(enemy.health(), used_weapon, enemy.flying()) * f32::from(*amount) * armor_mod;
                    }

                    if player.difficulty < Difficulty::Unsafe && aerial && !(
                        player.inventory.has(&Item::Skill(Skill::DoubleJump), 1) ||
                        player.inventory.has(&Item::Skill(Skill::Launch), 1) ||
                        player.difficulty >= Difficulty::Gorlek && player.inventory.has(&Item::Skill(Skill::Bash), 1)
                    ) { return None; }
                    if player.difficulty < Difficulty::Unsafe && dangerous && !(
                        player.inventory.has(&Item::Skill(Skill::DoubleJump), 1) ||
                        player.inventory.has(&Item::Skill(Skill::Dash), 1) ||
                        player.inventory.has(&Item::Skill(Skill::Bash), 1) ||
                        player.inventory.has(&Item::Skill(Skill::Launch), 1)
                    ) { return None; }

                    let cost = orbs.energy - energy;
                    return Requirement::cost_is_met(cost, player, orbs);
                }
            },
            Requirement::ShurikenBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Shuriken), 1) {
                    let clip_mod = if player.difficulty >= Difficulty::Unsafe { 2.0 } else { 3.0 };
                    let cost = player.destroy_cost(*health, Skill::Shuriken, false) * clip_mod;
                    return Requirement::cost_is_met(cost, player, orbs);
                },
            Requirement::SentryBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Sentry), 1) {
                    let clip_mod = 0.16;
                    let cost = player.destroy_cost(*health, Skill::Sentry, false) * clip_mod;
                    return Requirement::cost_is_met(cost, player, orbs);
                },
            Requirement::And(ands) => {
                let mut best_orbs = smallvec![orbs];

                for and in ands {
                    let mut orbcosts = SmallVec::<[Orbs; 3]>::new();
                    let mut met = false;

                    for orbs in &best_orbs {
                        if let Some(mut orbcost) = and.is_met(player, states, *orbs) {
                            orbcosts.append(&mut orbcost);
                            met = true;
                        }
                    }
                    if !met { return None; }

                    best_orbs = orbs::both(&best_orbs, &orbcosts);
                    best_orbs.retain(|orbs| orbs.health > 0.0 && orbs.energy >= 0.0);
                }

                let cost = orbs::both_single(&best_orbs, Orbs { health: -orbs.health, energy: -orbs.energy });
                return Some(cost);
            },
            Requirement::Or(ors) => {
                let mut cheapest = SmallVec::<[Orbs; 3]>::new();

                for or in ors {
                    if let Some(orbcost) = or.is_met(player, states, orbs) {
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

                if !cheapest.is_empty() {
                    return Some(cheapest);
                }
            },
        }
        None
    }

    fn needed_for_cost(cost: f32, player: &Player) -> Itemset {
        let mut itemsets = vec![(Inventory::default(), Orbs{ energy: -cost, ..Orbs::default() })];

        if player.difficulty >= Difficulty::Unsafe && cost > 0.0 && !player.inventory.has(&Item::Shard(Shard::Overcharge), 1) {
            itemsets.push((Inventory::from(Item::Shard(Shard::Overcharge)), Orbs{ energy: -cost / 2.0, ..Orbs::default() }));
        }

        itemsets
    }
    // TODO damage buff progression?
    fn needed_for_weapon(weapon: Skill, cost: f32, player: &Player) -> Itemset {
        let mut itemsets = Requirement::needed_for_cost(cost, player);

        for (inventory, _) in &mut itemsets {
            inventory.grant(Item::Skill(weapon), 1)
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
            Requirement::Impossible => vec![],
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

                if player.difficulty >= Difficulty::Gorlek && !player.inventory.has(&Item::Shard(Shard::Resilience), 1) {
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
                let clip_mod = if player.difficulty >= Difficulty::Unsafe { 2.0 } else { 3.0 };
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
                    if player.difficulty < Difficulty::Unsafe && enemy == &Enemy::Bat { bash = true; }
                    if enemy == &Enemy::Sandworm { burrow = true; }
                }

                // Skip unneccesary iterations over weapons that are redundant anyway
                let weapons = if melee {
                    player.progression_weapons(false)
                } else { smallvec![Skill::Sword] };
                let ranged_weapons = if ranged {
                    player.ranged_progression_weapons()
                } else { smallvec![Skill::Bow] };
                let shield_weapons = if shielded {
                    player.shield_progression_weapons()
                } else { smallvec![Skill::Hammer] };
                let use_burrow: SmallVec<[_; 2]> = if burrow {
                    if player.difficulty < Difficulty::Unsafe || player.inventory.has(&Item::Skill(Skill::Burrow), 1) {
                        smallvec![true]
                    } else {
                        smallvec![true, false]
                    }
                } else { smallvec![false] };

                // TODO there are redundancies here...
                for weapon in weapons {
                    for ranged_weapon in &ranged_weapons {
                        for shield_weapon in &shield_weapons {
                            for burrow in &use_burrow {
                                let (mut cost, mut highest_cost) = (0.0, 0.0);

                                for (enemy, amount) in enemies {
                                    if let Enemy::EnergyRefill = enemy {
                                        if cost > highest_cost { highest_cost = cost; }
                                        cost = 0_f32.max(cost - f32::from(*amount));
                                        continue;
                                    }

                                    if enemy == &Enemy::Sandworm && *burrow { continue; }

                                    if enemy.shielded() {
                                        cost += player.use_cost(*shield_weapon) * f32::from(*amount);
                                    }
                                    let armor_mod = if enemy.armored() && player.difficulty < Difficulty::Unsafe { 2.0 } else { 1.0 };

                                    let used_weapon = if enemy.ranged() { ranged_weapon } else { &weapon };

                                    cost += player.destroy_cost(enemy.health(), *used_weapon, enemy.flying()) * f32::from(*amount) * armor_mod;
                                }
                                if cost > highest_cost { highest_cost = cost; }

                                let mut itemset = Requirement::needed_for_cost(highest_cost, player);
                                for (inventory, _) in &mut itemset {
                                    if melee { inventory.grant(Item::Skill(weapon), 1) }
                                    if ranged { inventory.grant(Item::Skill(*ranged_weapon), 1) }
                                    if shielded && !inventory.has(&Item::Skill(*shield_weapon), 1) { inventory.grant(Item::Skill(*shield_weapon), 1) }
                                    if *burrow { inventory.grant(Item::Skill(Skill::Burrow), 1) }
                                }

                                itemsets.append(&mut itemset);
                            }
                        }
                    }
                }

                if player.difficulty < Difficulty::Unsafe && aerial {
                    let mut ranged_skills = vec![
                        Item::Skill(Skill::DoubleJump),
                        Item::Skill(Skill::Launch),
                    ];
                    if player.difficulty >= Difficulty::Gorlek { ranged_skills.push(Item::Skill(Skill::Bash)); }

                    if !ranged_skills.iter().any(|skill| player.inventory.has(skill, 1)) {
                        itemsets = Requirement::combine_itemset_items(itemsets, &ranged_skills);
                    }
                }
                if player.difficulty < Difficulty::Unsafe && dangerous {
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
                if player.difficulty < Difficulty::Unsafe && bash && !player.inventory.has(&Item::Skill(Skill::Bash), 1) {
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

    pub fn contained_states(&self) -> Vec<usize> {
        match self {
            Requirement::State(state) => vec![*state],
            Requirement::And(ands) => ands.iter().flat_map(Requirement::contained_states).collect(),
            Requirement::Or(ors) => ors.iter().flat_map(Requirement::contained_states).collect(),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn is_met() {
        let mut player = Player::default();
        player.inventory.grant(Item::Resource(Resource::Health), 1);
        let mut states = FxHashSet::default();
        let orbs = Orbs::default();

        let req = Requirement::Skill(Skill::Blaze);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_some());

        let req = Requirement::And(vec![req, Requirement::Free]);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_some());
        let req = Requirement::Or(vec![req, Requirement::Impossible]);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_some());

        let req = Requirement::EnergySkill(Skill::Blaze, 1.0);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.difficulty = Difficulty::Unsafe;
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -1.0, ..orbs }]));
        player.difficulty = Difficulty::Moki;
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -2.0, ..orbs }]));

        let req = Requirement::State(34);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        states.insert(34);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_some());
        let req = Requirement::State(33);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());

        let req = Requirement::Damage(30.0);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Health), 5);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Health), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { health: -30.0, ..orbs }]));
        let req = Requirement::Damage(60.0);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Health), 6);
        assert_eq!(req.is_met(&player, &states, Orbs { health: 30.0, energy: player.max_energy() }), Some(smallvec![Orbs { health: -30.0, energy: -1.0 }]));
        let req = Requirement::Danger(30.0);
        assert_eq!(req.is_met(&player, &states, Orbs { health: 30.0, energy: player.max_energy() }), Some(smallvec![Orbs { ..orbs }]));
        let req = Requirement::Danger(60.0);
        assert_eq!(req.is_met(&player, &states, Orbs { health: 30.0, energy: player.max_energy() }), Some(smallvec![Orbs { health: 30.0, energy: -1.0 }]));

        player = Player::default();
        let req = Requirement::BreakWall(12.0);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Skill(Skill::Sword), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { ..orbs }]));
        player = Player::default();
        player.inventory.grant(Item::Skill(Skill::Grenade), 1);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 3);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -2.0, ..orbs }]));
        player = Player::default();
        let req = Requirement::BreakWall(16.0);
        player.inventory.grant(Item::Skill(Skill::Grenade), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        player.difficulty = Difficulty::Unsafe;
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -1.0, ..orbs }]));
        player.difficulty = Difficulty::Moki;
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());

        player = Player::default();
        let req = Requirement::ShurikenBreak(12.0);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        player.difficulty = Difficulty::Unsafe;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -2.0, ..orbs }]));
        player.inventory.grant(Item::Resource(Resource::Energy), 6);
        player.difficulty = Difficulty::Moki;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -6.0, ..orbs }]));

        player = Player::default();
        let req = Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]);
        player.inventory.grant(Item::Skill(Skill::Bow), 1);
        player.difficulty = Difficulty::Unsafe;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 7);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -3.25, ..orbs }]));
        player.inventory.grant(Item::Resource(Resource::Energy), 6);
        player.difficulty = Difficulty::Moki;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -6.5, ..orbs }]));
        player = Player::default();
        let req = Requirement::Combat(smallvec![(Enemy::Sandworm, 1), (Enemy::Bat, 1), (Enemy::EnergyRefill, 99), (Enemy::ShieldMiner, 2), (Enemy::EnergyRefill, 1), (Enemy::Balloon, 4)]);
        player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
        player.inventory.grant(Item::Skill(Skill::Spear), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 27);
        player.difficulty = Difficulty::Gorlek;
        player.difficulty = Difficulty::Unsafe;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -14.0, ..orbs }]));
        player.inventory.grant(Item::Resource(Resource::Energy), 37);
        player.inventory.grant(Item::Skill(Skill::Bash), 1);
        player.inventory.grant(Item::Skill(Skill::Launch), 1);
        player.inventory.grant(Item::Skill(Skill::Burrow), 1);
        player.difficulty = Difficulty::Moki;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -33.0, ..orbs }]));
        player = Player::default();
        let req = Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]);
        player.inventory.grant(Item::Skill(Skill::Spear), 1);
        player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        player.difficulty = Difficulty::Gorlek;
        player.difficulty = Difficulty::Unsafe;
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -2.0, ..orbs }]));
        player.difficulty = Difficulty::Moki;
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 11);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Resource(Resource::Energy), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -8.0, ..orbs }]));

        player = Player::default();
        let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
        let b = Requirement::Damage(20.0);
        let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
        let d = Requirement::Damage(10.0);
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 4);
        player.inventory.grant(Item::Resource(Resource::Health), 5);
        let req = Requirement::And(vec![c.clone(), d.clone()]);
        player.difficulty = Difficulty::Unsafe;
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { health: -10.0, energy: -1.0 }]));
        let req = Requirement::Or(vec![a.clone(), b.clone()]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -2.0, ..orbs }, Orbs { health: -20.0, ..orbs }]));
        let req = Requirement::Or(vec![Requirement::And(vec![a.clone(), b.clone()]), Requirement::And(vec![c.clone(), d.clone()]), a.clone(), b.clone()]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -1.0, health: -10.0 }, Orbs { energy: -2.0, ..orbs }, Orbs { health: -20.0, ..orbs }]));
        let req = Requirement::And(vec![Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()])]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -1.0, health: -10.0 }]));
        player.inventory.grant(Item::Resource(Resource::Energy), 8);
        player.inventory.grant(Item::Resource(Resource::Health), 8);
        let req = Requirement::And(vec![Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()]), Requirement::Or(vec![a.clone(), d.clone()]), Requirement::Or(vec![b.clone(), c.clone()])]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -6.0, ..orbs }, Orbs { energy: -4.0, health: -10.0 }, Orbs { health: -60.0, ..orbs }, Orbs { energy: -1.0, health: -40.0 }, Orbs { energy: -2.0, health: -20.0 }]));
        let req = Requirement::Or(vec![Requirement::Free, b.clone()]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs::default()]));
        let req = Requirement::Or(vec![b.clone(), Requirement::Free]);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs::default()]));

        player = Player::default();
        player.difficulty = Difficulty::Unsafe;
        player.inventory.grant(Item::Resource(Resource::Health), 7);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        let req = Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]);
        assert!(req.is_met(&player, &states, player.max_orbs()).is_none());
        player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { energy: -1.0, health: -30.0 }]));

        let req = Requirement::Or(vec![Requirement::Damage(10.0), Requirement::EnergySkill(Skill::Blaze, 1.0)]);
        let req = Requirement::And(vec![req.clone(), req.clone()]);
        player.inventory.grant(Item::Skill(Skill::Blaze), 1);
        player.inventory.grant(Item::Resource(Resource::Energy), 2);
        assert_eq!(req.is_met(&player, &states, player.max_orbs()), Some(smallvec![Orbs { health: -20.0, ..orbs }, Orbs { health: -10.0, energy: -1.0 }, Orbs { energy: -2.0, ..orbs }]));
    }

    #[test]
    fn items_needed() {
        let mut player = Player::default();
        player.spawn(&Settings::default());
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
        player.difficulty = Difficulty::Unsafe;
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::from(Item::Skill(Skill::Grenade)), Orbs { energy: -2.0, ..orbs }),
            (Inventory::from(vec![Item::Skill(Skill::Grenade), Item::Shard(Shard::Overcharge)]), Orbs { energy: -1.0, ..orbs }),
        ]);
        player.difficulty = Difficulty::Moki;

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
        player.difficulty = Difficulty::Gorlek;
        assert_eq!(req.items_needed(&player, &states), vec![
            (Inventory::default(), Orbs { health: -36.0, ..orbs }),
            (Inventory::from(Item::Shard(Shard::Resilience)), Orbs { health: -36.0 * 0.9, ..orbs }),
        ]);
        player.difficulty = Difficulty::Moki;

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
        player.difficulty = Difficulty::Unsafe;
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
        player = Player::default();

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
