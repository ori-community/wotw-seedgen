use super::*;

use rustc_hash::FxHashSet;
use smallvec::smallvec;

use crate::item::Item;
use crate::settings::logical_difficulty;
use crate::world::player::Player;
use crate::util::orbs::{self, Orbs, OrbVariants};

impl Requirement {
    pub fn is_met(&self, player: &Player, states: &FxHashSet<usize>, mut orb_variants: OrbVariants) -> OrbVariants {
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
                    return cost_is_met(cost, player, orb_variants, true);
                }
            Requirement::NonConsumingEnergySkill(skill) =>
                if player.inventory.has(&Item::Skill(*skill), 1) {
                    let cost = player.use_cost(*skill);
                    return cost_is_met(cost, player, orb_variants, false);
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
                return health_is_met(cost, player, orb_variants, true);
            },
            Requirement::Danger(amount) => {
                let cost = *amount * player.defense_mod();
                return health_is_met(cost, player, orb_variants, false);
            },
            Requirement::BreakWall(health) =>
                if let Some(cost) = player.destroy_cost::<true>(*health, false) {
                    return cost_is_met(cost, player, orb_variants, true);
                }
            Requirement::Boss(health) =>
                // TODO rock boss is flying, just placing a todo in case rock boss will be logic relevant someday
                if let Some(cost) = player.destroy_cost::<false>(*health, false) {
                    return cost_is_met(cost, player, orb_variants, true);
                }
            Requirement::Combat(enemies) => {
                // Check for movement skills
                if player.settings.difficulty < Difficulty::Unsafe && enemies.iter().any(|(enemy, _)| {
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
                    || (matches!(enemy, Enemy::Bat) && !player.inventory.has(&Item::Skill(Skill::Bash), 1))
                }) { return smallvec![] }

                // TODO this might be a try block once that's stable
                let combat_is_met = || {
                    let shield_weapon = player.owned_shield_weapons().first().copied();
                    let mut cost = 0.0;

                    for (enemy, amount) in enemies {
                        let amount = f32::from(*amount);

                        match enemy {
                            Enemy::EnergyRefill => {
                                // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                                // However in between energy refills, the cost is always the same
                                orb_variants = cost_is_met(cost, player, orb_variants, true);
                                if orb_variants.is_empty() { return None }
                                for orbs in &mut orb_variants {
                                    player.recharge(orbs, amount);
                                }
                                cost = 0.0;
                                continue;
                            },
                            Enemy::Sandworm => {
                                if player.inventory.has(&Item::Skill(Skill::Burrow), 1) { continue }
                                else if player.settings.difficulty < Difficulty::Unsafe { return None }
                            },
                            _ => {},
                        }

                        if enemy.shielded() {
                            cost += player.use_cost(shield_weapon?) * amount;  // TODO grenade also burns
                        }
                        let mut health = enemy.health();
                        if enemy.armored() && player.settings.difficulty < Difficulty::Unsafe { health *= 2.0 };
                        let cost_function = if enemy.ranged() && player.settings.difficulty < Difficulty::Unsafe { Player::destroy_cost_ranged } else { Player::destroy_cost::<false> };
                        cost += cost_function(player, health, enemy.flying())? * amount;
                    }

                    Some(cost_is_met(cost, player, orb_variants, true))
                };

                return combat_is_met().unwrap_or_else(|| smallvec![]);
            },
            Requirement::ShurikenBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Shuriken), 1) {
                    let clip_mod = if player.settings.difficulty >= Difficulty::Unsafe { 2.0 } else { 3.0 };
                    let cost = player.destroy_cost_with(*health, Skill::Shuriken, false) * clip_mod;
                    return cost_is_met(cost, player, orb_variants, true);
                },
            Requirement::SentryBreak(health) =>
                if player.inventory.has(&Item::Skill(Skill::Sentry), 1) {
                    let clip_mod = 6.25;
                    let cost = player.destroy_cost_with(*health, Skill::Sentry, false) * clip_mod;
                    return cost_is_met(cost, player, orb_variants, true);
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
}

fn cost_is_met(cost: f32, player: &Player, mut orb_variants: OrbVariants, consuming: bool) -> OrbVariants {
    let mut added_orb_variants = vec![];

    fn orbs_meet_cost(orbs: &mut Orbs, added_orb_variants: &mut Vec<Orbs>, cost: f32, player: &Player, consuming: bool) -> bool {
        let has_life_pact = player.settings.difficulty >= logical_difficulty::LIFE_PACT && player.inventory.has(&Item::Shard(Shard::LifePact), 1);
        if has_life_pact && consuming && player.inventory.has(&Item::Skill(Skill::Regenerate), 1) {
            // Health is worth more than Energy with Life Pact and if we wait too long we might be unable to Regenerate later
            let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
            let regen_cost = player.use_cost(Skill::Regenerate);
            let higher_cost = regen_cost.max(game_thinks_regen_cost);

            // TODO if we fix the incorrect affordability calculations this might want to take the defense mod into account
            if orbs.energy >= higher_cost && player.max_health() - orbs.health > regen_cost {
                let mut new_orbs = *orbs;
                new_orbs.energy -= regen_cost;
                player.heal(&mut new_orbs, 30.0);
                if orbs_meet_cost(&mut new_orbs, added_orb_variants, cost, player, consuming) {
                    added_orb_variants.push(new_orbs);
                }
            }
        }

        if orbs.energy >= cost {
            if consuming { orbs.energy -= cost; }
            true
        } else if has_life_pact {
            loop {
                let missing_energy = cost - orbs.energy;
                let game_thinks_health_cost = missing_energy * 10.0;  // A health orb is ten times as much as an energy orb, but the game considers orbs equal for Life Pact
                let health_cost = game_thinks_health_cost * player.defense_mod();
                let higher_cost = health_cost.max(game_thinks_health_cost);  // we have to meet both

                if orbs.health > higher_cost {
                    orbs.health -= health_cost;
                    if consuming { orbs.energy = 0.0; }
                    else { player.recharge(orbs, missing_energy); }  // The game doesn't refund the health, it refunds it as energy
                    break true;
                }
                if !regenerate_as_needed(higher_cost, player, orbs) { return false }
            }
        } else { false }
    }

    orb_variants.retain(|orbs| orbs_meet_cost(orbs, &mut added_orb_variants, cost, player, consuming));
    orb_variants.extend(added_orb_variants);
    orb_variants
}

fn health_is_met(cost: f32, player: &Player, mut orb_variants: OrbVariants, consuming: bool) -> OrbVariants {
    orb_variants.retain(|orbs| {
        let met = orbs.health > cost
        || (player.inventory.has(&Item::Skill(Skill::Regenerate), 1) && player.max_health() > cost && regenerate_as_needed(cost, player, orbs));
        if consuming { orbs.health -= cost }
        met
    });
    orb_variants
}

fn regenerate_as_needed(cost: f32, player: &Player, orbs: &mut Orbs) -> bool {
    let mut regens = ((cost - orbs.health) / 30.0).ceil();
    if orbs.health + 30.0 * regens <= cost { regens += 1.0 }
    player.heal(orbs, 30.0 * regens);
    let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
    let regen_cost = player.use_cost(Skill::Regenerate);
    // Regenerate is special cased to not allow Life Pact, so we don't go through cost_is_met
    orbs.energy -= regen_cost * regens;
    orbs.energy >= 0.0 && orbs.energy + regen_cost - game_thinks_regen_cost >= 0.0  // On the final regenerate we have to make sure the the game is happy with our amount of resources
}
