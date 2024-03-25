use crate::logical_difficulty;
use crate::orbs::{self, OrbVariants, Orbs};
use crate::world::player::Player;
use rustc_hash::FxHashSet;
use smallvec::smallvec;
use wotw_seedgen_data::{Shard, Skill};
use wotw_seedgen_logic_language::output::{Enemy, Requirement};
use wotw_seedgen_settings::Difficulty;

impl Player<'_> {
    pub fn is_met(
        &self,
        requirement: &Requirement,
        states: &FxHashSet<usize>,
        mut orb_variants: OrbVariants,
    ) -> OrbVariants {
        match requirement {
            Requirement::Free => return orb_variants,
            Requirement::Impossible => {}
            Requirement::Difficulty(difficulty) => {
                if self.settings.difficulty >= *difficulty {
                    return orb_variants;
                }
            }
            Requirement::NormalGameDifficulty => {
                if !self.settings.hard {
                    return orb_variants;
                }
            }
            Requirement::Trick(trick) => {
                if self.settings.tricks.contains(trick) {
                    return orb_variants;
                }
            }
            Requirement::Skill(skill) => {
                if self.inventory.skills.contains(skill) {
                    return orb_variants;
                }
            }
            Requirement::EnergySkill(skill, amount) => {
                if self.inventory.skills.contains(skill) {
                    let cost = self.use_cost(*skill) * *amount;
                    return cost_is_met(cost, self, orb_variants, true);
                }
            }
            Requirement::NonConsumingEnergySkill(skill) => {
                if self.inventory.skills.contains(skill) {
                    let cost = self.use_cost(*skill);
                    return cost_is_met(cost, self, orb_variants, false);
                }
            }
            Requirement::SpiritLight(amount) => {
                if self.inventory.spirit_light >= *amount {
                    return orb_variants;
                }
            }
            Requirement::GorlekOre(amount) => {
                if self.inventory.gorlek_ore >= *amount {
                    return orb_variants;
                }
            }
            Requirement::Keystone(amount) => {
                if self.inventory.keystones >= *amount {
                    return orb_variants;
                }
            }
            Requirement::Shard(shard) => {
                if self.inventory.shards.contains(shard) {
                    return orb_variants;
                }
            }
            Requirement::Teleporter(teleporter) => {
                if self.inventory.teleporters.contains(teleporter) {
                    return orb_variants;
                }
            }
            Requirement::Water => {
                if self.inventory.clean_water {
                    return orb_variants;
                }
            }
            Requirement::State(state) => {
                if states.contains(state) {
                    return orb_variants;
                }
            }
            Requirement::Damage(amount) => {
                let cost = *amount * self.defense_mod();
                return health_is_met(cost, self, orb_variants, true);
            }
            Requirement::Danger(amount) => {
                let cost = *amount * self.defense_mod();
                return health_is_met(cost, self, orb_variants, false);
            }
            Requirement::BreakWall(health) => {
                if let Some(cost) = self.destroy_cost::<true>(*health, false) {
                    return cost_is_met(cost, self, orb_variants, true);
                }
            }
            Requirement::Boss(health) =>
            // TODO rock boss is flying, just placing a todo in case rock boss will be logic relevant someday
            {
                if let Some(cost) = self.destroy_cost::<false>(*health, false) {
                    return cost_is_met(cost, self, orb_variants, true);
                }
            }
            Requirement::Combat(enemies) => {
                // TODO handle nests better
                // Check for movement skills
                if self.settings.difficulty < Difficulty::Unsafe
                    && enemies.iter().any(|(enemy, _)| {
                        (enemy.aerial()
                            && !(self.inventory.skills.contains(&Skill::DoubleJump)
                                || self.inventory.skills.contains(&Skill::Launch)
                                || self.settings.difficulty >= Difficulty::Gorlek
                                    && self.inventory.skills.contains(&Skill::Bash)))
                            || (enemy.dangerous()
                                && !(self.inventory.skills.contains(&Skill::DoubleJump)
                                    || self.inventory.skills.contains(&Skill::Dash)
                                    || self.inventory.skills.contains(&Skill::Bash)
                                    || self.inventory.skills.contains(&Skill::Launch)))
                            || (matches!(enemy, Enemy::Bat)
                                && !self.inventory.skills.contains(&Skill::Bash))
                    })
                {
                    return smallvec![];
                }

                // TODO this might be a try block once that's stable
                let combat_is_met = || {
                    let shield_weapon = self.owned_shield_weapons().first().copied();
                    let mut cost = 0.0;

                    for (enemy, amount) in enemies {
                        let amount = f32::from(*amount);

                        match enemy {
                            Enemy::EnergyRefill => {
                                // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                                // However in between energy refills, the cost is always the same
                                orb_variants = cost_is_met(cost, self, orb_variants, true);
                                if orb_variants.is_empty() {
                                    return None;
                                }
                                for orbs in &mut orb_variants {
                                    self.recharge(orbs, amount);
                                }
                                cost = 0.0;
                                continue;
                            }
                            Enemy::Sandworm => {
                                if self.inventory.skills.contains(&Skill::Burrow) {
                                    continue;
                                } else if self.settings.difficulty < Difficulty::Unsafe {
                                    return None;
                                }
                            }
                            _ => {}
                        }

                        let mut health = enemy.health();

                        if enemy.shielded() {
                            let shield_weapon = shield_weapon?;
                            cost += self.use_cost(shield_weapon) * amount;
                            health = (health - shield_weapon.burn_damage()).max(0.0);
                        } else if enemy.armored() && self.settings.difficulty < Difficulty::Unsafe {
                            health *= 2.0
                        }; // No enemy is shielded and armored

                        let cost_function =
                            if enemy.ranged() && self.settings.difficulty < Difficulty::Unsafe {
                                Player::destroy_cost_ranged
                            } else {
                                Player::destroy_cost::<false>
                            };
                        cost += cost_function(self, health, enemy.flying())? * amount;
                    }

                    Some(cost_is_met(cost, self, orb_variants, true))
                };

                return combat_is_met().unwrap_or_else(|| smallvec![]);
            }
            Requirement::ShurikenBreak(health) => {
                if self.inventory.skills.contains(&Skill::Shuriken) {
                    let clip_mod = if self.settings.difficulty >= Difficulty::Unsafe {
                        2.0
                    } else {
                        3.0
                    };
                    let cost = self.destroy_cost_with(*health, Skill::Shuriken, false) * clip_mod;
                    return cost_is_met(cost, self, orb_variants, true);
                }
            }
            Requirement::SentryBreak(health) => {
                if self.inventory.skills.contains(&Skill::Sentry) {
                    let clip_mod = 6.25;
                    let cost = self.destroy_cost_with(*health, Skill::Sentry, false) * clip_mod;
                    return cost_is_met(cost, self, orb_variants, true);
                }
            }
            Requirement::And(requirements) => {
                for and in requirements {
                    orb_variants = self.is_met(and, states, orb_variants);
                    if orb_variants.is_empty() {
                        break;
                    }
                }
                return orb_variants;
            }
            Requirement::Or(requirements) => {
                let mut cheapest = OrbVariants::new();

                for or in requirements {
                    let orbcost = self.is_met(or, states, orb_variants.clone());
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
            }
        }
        smallvec![]
    }
}

#[must_use]
fn cost_is_met(
    cost: f32,
    player: &Player,
    mut orb_variants: OrbVariants,
    consuming: bool,
) -> OrbVariants {
    let mut added_orb_variants = vec![];

    fn orbs_meet_cost(
        orbs: &mut Orbs,
        added_orb_variants: &mut Vec<Orbs>,
        cost: f32,
        player: &Player,
        consuming: bool,
    ) -> bool {
        let has_life_pact = player.settings.difficulty >= logical_difficulty::LIFE_PACT
            && player.inventory.shards.contains(&Shard::LifePact);
        if has_life_pact && consuming && player.inventory.skills.contains(&Skill::Regenerate) {
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
            if consuming {
                orbs.energy -= cost;
            }
            true
        } else if has_life_pact {
            loop {
                let missing_energy = cost - orbs.energy;
                let game_thinks_health_cost = missing_energy * 10.0; // A health orb is ten times as much as an energy orb, but the game considers orbs equal for Life Pact
                let health_cost = game_thinks_health_cost * player.defense_mod();
                let higher_cost = health_cost.max(game_thinks_health_cost); // we have to meet both

                if orbs.health > higher_cost {
                    orbs.health -= health_cost;
                    if consuming {
                        orbs.energy = 0.0;
                    } else {
                        player.recharge(orbs, missing_energy);
                    } // The game doesn't refund the health, it refunds it as energy
                    break true;
                }
                if !regenerate_as_needed(higher_cost, player, orbs) {
                    return false;
                }
            }
        } else {
            false
        }
    }

    orb_variants
        .retain(|orbs| orbs_meet_cost(orbs, &mut added_orb_variants, cost, player, consuming));
    orb_variants.extend(added_orb_variants);
    orb_variants
}

fn health_is_met(
    cost: f32,
    player: &Player,
    mut orb_variants: OrbVariants,
    consuming: bool,
) -> OrbVariants {
    orb_variants.retain(|orbs| {
        let met = orbs.health > cost
            || (player.inventory.skills.contains(&Skill::Regenerate)
                && player.max_health() > cost
                && regenerate_as_needed(cost, player, orbs));
        if consuming {
            orbs.health -= cost
        }
        met
    });
    orb_variants
}

fn regenerate_as_needed(cost: f32, player: &Player, orbs: &mut Orbs) -> bool {
    let mut regens = ((cost - orbs.health) / 30.0).ceil();
    if orbs.health + 30.0 * regens <= cost {
        regens += 1.0
    }
    player.heal(orbs, 30.0 * regens);
    let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
    let regen_cost = player.use_cost(Skill::Regenerate);
    // Regenerate is special cased to not allow Life Pact, so we don't go through cost_is_met
    orbs.energy -= regen_cost * regens;
    orbs.energy >= 0.0 && orbs.energy + regen_cost - game_thinks_regen_cost >= 0.0
    // On the final regenerate we have to make sure the the game is happy with our amount of resources
}
