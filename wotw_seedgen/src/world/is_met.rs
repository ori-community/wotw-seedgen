use std::ops::ControlFlow;

use super::World;
use crate::logical_difficulty;
use crate::orbs::{self, OrbVariants, Orbs};
use smallvec::SmallVec;
use wotw_seedgen_data::{
    logic_language::output::{Enemy, Requirement},
    seed_language::simulate::Simulation,
    Difficulty, Shard, Skill, UberIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Missing {
    Impossible,
    UberState(UberIdentifier),
    LogicalState(usize),
    Orbs,
    Any(Vec<Missing>),
}

impl Missing {
    fn any<I: IntoIterator<Item = UberIdentifier>>(iter: I) -> Self {
        Self::Any(iter.into_iter().map(Self::UberState).collect())
    }

    fn any_skill<I: IntoIterator<Item = Skill>>(iter: I) -> Self {
        Self::any(iter.into_iter().map(Skill::uber_identifier))
    }
}

impl World<'_, '_> {
    // TODO does controlflow have must_use?
    pub fn is_met(
        &self,
        requirement: &Requirement,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        match requirement {
            Requirement::Free => ControlFlow::Continue(()),
            Requirement::Impossible => ControlFlow::Break(Missing::Impossible),
            Requirement::Difficulty(difficulty) => {
                self.setting_met(self.settings.difficulty >= *difficulty)
            }
            Requirement::NormalGameDifficulty => self.setting_met(self.settings.hard),
            Requirement::Trick(trick) => self.setting_met(!self.settings.tricks.contains(trick)),
            Requirement::Skill(skill) => self.skill_met(*skill),
            Requirement::EnergySkill(skill, amount) => {
                self.skill_met(*skill)?;

                let cost = self.use_cost(*skill) * *amount;
                self.cost_is_met::<true>(cost, orb_variants)
            }
            Requirement::NonConsumingEnergySkill(skill) => {
                self.skill_met(*skill)?;

                let cost = self.use_cost(*skill);
                self.cost_is_met::<false>(cost, orb_variants)
            }
            Requirement::SpiritLight(amount) => self.uber_state_met(
                self.spirit_light() >= *amount as i32,
                UberIdentifier::SPIRIT_LIGHT,
            ),
            Requirement::GorlekOre(amount) => self.uber_state_met(
                self.gorlek_ore() >= *amount as i32,
                UberIdentifier::GORLEK_ORE,
            ),
            Requirement::Keystone(amount) => self.uber_state_met(
                self.keystones() >= *amount as i32,
                UberIdentifier::KEYSTONES,
            ),
            Requirement::Shard(shard) => self.shard_met(*shard),
            Requirement::Teleporter(teleporter) => {
                self.uber_state_met(self.teleporter(*teleporter), teleporter.uber_identifier())
            }
            Requirement::Water => {
                self.uber_state_met(self.clean_water(), UberIdentifier::CLEAN_WATER)
            }
            Requirement::State(state) => {
                if self.has_reached(*state) {
                    ControlFlow::Continue(())
                } else {
                    let missing = self.graph.nodes[*state]
                        .uber_identifier()
                        .map_or(Missing::LogicalState(*state), Missing::UberState);
                    ControlFlow::Break(missing)
                }
            }
            Requirement::Damage(amount) => {
                let cost = *amount * self.defense_mod();
                self.health_is_met::<true>(cost, orb_variants)
            }
            Requirement::Danger(amount) => {
                let cost = *amount * self.defense_mod();
                self.health_is_met::<false>(cost, orb_variants)
            }
            Requirement::BreakWall(health) => {
                self.destroy_cost_met::<true>(*health, false, orb_variants)
            }
            Requirement::Boss(health) => {
                // TODO rock boss is flying, just placing a todo in case rock boss will be logic relevant someday
                self.destroy_cost_met::<false>(*health, false, orb_variants)
            }
            Requirement::Combat(enemies) => {
                // TODO handle nests better
                self.enemy_movement_met(enemies)?;

                let shield_weapon = self.owned_shield_weapons().first().copied();
                let mut cost = 0.0;

                for (enemy, amount) in enemies {
                    let amount = f32::from(*amount);

                    match enemy {
                        Enemy::EnergyRefill => {
                            // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                            // However in between energy refills, the cost is always the same
                            self.cost_is_met::<true>(cost, orb_variants)?;

                            for orbs in &mut *orb_variants {
                                self.recharge(orbs, amount);
                            }

                            cost = 0.0;
                            continue;
                        }
                        Enemy::Sandworm => {
                            if self.skill(Skill::Burrow) {
                                continue;
                            } else if self.settings.difficulty < Difficulty::Unsafe {
                                return ControlFlow::Break(Missing::UberState(
                                    Skill::Burrow.uber_identifier(),
                                ));
                            }
                        }
                        _ => {}
                    }

                    let mut health = enemy.health();

                    if enemy.shielded() {
                        let Some(shield_weapon) = shield_weapon else {
                            // TODO precompiled slices for weapon identifiers?
                            return ControlFlow::Break(Missing::any_skill(
                                logical_difficulty::shield_weapons(),
                            ));
                        };
                        cost += self.use_cost(shield_weapon) * amount;
                        health = (health - shield_weapon.burn_damage()).max(0.0);
                    }
                    // No enemy is shielded and armored
                    else if enemy.armored() && self.settings.difficulty < Difficulty::Unsafe {
                        health *= 2.0
                    };

                    let ranged_weapon =
                        enemy.ranged() && self.settings.difficulty < Difficulty::Unsafe;
                    let cost_function = if ranged_weapon {
                        World::destroy_cost_ranged
                    } else {
                        World::destroy_cost::<false>
                    };

                    let Some(enemy_cost) = cost_function(self, health, enemy.flying()) else {
                        let missing = if ranged_weapon {
                            Missing::any_skill(logical_difficulty::ranged_weapons(
                                self.settings.difficulty,
                            ))
                        } else {
                            Missing::any_skill(logical_difficulty::weapons::<false>(
                                self.settings.difficulty,
                            ))
                        };

                        return ControlFlow::Break(missing);
                    };

                    cost += enemy_cost * amount;
                }

                self.cost_is_met::<true>(cost, orb_variants)
            }
            Requirement::ShurikenBreak(health) => {
                self.skill_met(Skill::Shuriken)?;

                let clip_mod = if self.settings.difficulty >= Difficulty::Unsafe {
                    2.0
                } else {
                    3.0
                };
                let cost = self.destroy_cost_with(*health, Skill::Shuriken, false) * clip_mod;

                self.cost_is_met::<true>(cost, orb_variants)
            }
            Requirement::SentryBreak(health) => {
                self.skill_met(Skill::Sentry)?;

                let clip_mod = 6.25;
                let cost = self.destroy_cost_with(*health, Skill::Sentry, false) * clip_mod;

                self.cost_is_met::<true>(cost, orb_variants)
            }
            Requirement::And(requirements) => {
                for and in requirements {
                    self.is_met(and, orb_variants)?;
                }

                ControlFlow::Continue(())
            }
            Requirement::Or(requirements) => {
                let mut cheapest = OrbVariants::new();
                let mut missing = vec![];

                for or in requirements {
                    let mut orb_variants_after = orb_variants.clone();
                    match self.is_met(or, &mut orb_variants_after) {
                        ControlFlow::Continue(()) => {
                            if cheapest.is_empty() {
                                cheapest = orb_variants_after;
                            } else {
                                cheapest = orbs::either(&cheapest, &orb_variants_after);
                            }
                            if cheapest[0] == Orbs::default() {
                                break;
                            }
                        }
                        ControlFlow::Break(or_missing) => {
                            if !missing.contains(&or_missing) {
                                missing.push(or_missing);
                            }
                        }
                    }
                }

                *orb_variants = cheapest;

                break_if_empty(orb_variants, Missing::Any(missing))
            }
        }
    }

    fn setting_met(&self, condition: bool) -> ControlFlow<Missing> {
        if condition {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::Impossible)
        }
    }

    fn skill_met(&self, skill: Skill) -> ControlFlow<Missing> {
        self.uber_state_met(self.skill(skill), skill.uber_identifier())
    }

    fn any_skill_met<T>(&self, skills: T) -> ControlFlow<Missing>
    where
        T: IntoIterator<Item = Skill> + Copy,
    {
        if skills.into_iter().any(|skill| self.skill(skill)) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::any_skill(skills))
        }
    }

    fn shard_met(&self, shard: Shard) -> ControlFlow<Missing> {
        self.uber_state_met(self.shard(shard), shard.uber_identifier())
    }

    fn uber_state_met(
        &self,
        condition: bool,
        uber_identifier: UberIdentifier,
    ) -> ControlFlow<Missing> {
        if condition {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::UberState(uber_identifier))
        }
    }

    // TODO use more arrayvec instead of smallvec
    fn enemy_movement_met(&self, enemies: &SmallVec<[(Enemy, u8); 12]>) -> ControlFlow<Missing> {
        if self.settings.difficulty < Difficulty::Unsafe {
            let mut aerial = false;
            let mut dangerous = false;
            let mut bat = false;

            for (enemy, _) in enemies {
                aerial |= enemy.aerial();
                dangerous |= enemy.dangerous();
                bat |= matches!(enemy, Enemy::Bat);
            }

            if aerial {
                self.aerial_met()?;
            }
            if dangerous {
                self.dangerous_met()?;
            }
            if bat {
                self.skill_met(Skill::Bash)?;
            }
        }

        ControlFlow::Continue(())
    }

    fn aerial_met(&self) -> ControlFlow<Missing> {
        if self.settings.difficulty < Difficulty::Gorlek {
            self.any_skill_met([Skill::DoubleJump, Skill::Launch])
        } else {
            self.any_skill_met([Skill::DoubleJump, Skill::Launch, Skill::Bash])
        }
    }

    fn dangerous_met(&self) -> ControlFlow<Missing> {
        self.any_skill_met([Skill::DoubleJump, Skill::Dash, Skill::Bash, Skill::Launch])
    }

    fn destroy_cost_met<const TARGET_IS_WALL: bool>(
        &self,
        target_health: f32,
        flying_target: bool,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        let Some(cost) = self.destroy_cost::<TARGET_IS_WALL>(target_health, flying_target) else {
            let states = logical_difficulty::weapons::<TARGET_IS_WALL>(self.settings.difficulty);
            return ControlFlow::Break(Missing::any_skill(states));
        };

        self.cost_is_met::<true>(cost, orb_variants)
    }

    fn cost_is_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        let mut added_orb_variants = vec![];

        orb_variants
            .retain(|orbs| self.orbs_meet_cost::<CONSUMING>(orbs, &mut added_orb_variants, cost));
        orb_variants.extend(added_orb_variants);

        break_if_empty(orb_variants, Missing::Orbs)
    }

    fn orbs_meet_cost<const CONSUMING: bool>(
        &self,
        orbs: &mut Orbs,
        added_orb_variants: &mut Vec<Orbs>,
        cost: f32,
    ) -> bool {
        let has_life_pact = self.settings.difficulty >= logical_difficulty::LIFE_PACT
            && self.shard(Shard::LifePact);
        if has_life_pact && CONSUMING && self.skill(Skill::Regenerate) {
            // Health is worth more than Energy with Life Pact and if we wait too long we might be unable to Regenerate later
            let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
            let regen_cost = self.use_cost(Skill::Regenerate);
            let higher_cost = regen_cost.max(game_thinks_regen_cost);

            // TODO if we fix the incorrect affordability calculations this might want to take the defense mod into account
            if orbs.energy >= higher_cost && self.max_health() - orbs.health > regen_cost {
                let mut new_orbs = *orbs;
                new_orbs.energy -= regen_cost;
                self.heal(&mut new_orbs, 30.0);
                if self.orbs_meet_cost::<CONSUMING>(&mut new_orbs, added_orb_variants, cost) {
                    added_orb_variants.push(new_orbs);
                }
            }
        }

        if orbs.energy >= cost {
            if CONSUMING {
                orbs.energy -= cost;
            }
            true
        } else if has_life_pact {
            loop {
                let missing_energy = cost - orbs.energy;
                let game_thinks_health_cost = missing_energy * 10.0; // A health orb is ten times as much as an energy orb, but the game considers orbs equal for Life Pact
                let health_cost = game_thinks_health_cost * self.defense_mod();
                let higher_cost = health_cost.max(game_thinks_health_cost); // we have to meet both

                if orbs.health > higher_cost {
                    orbs.health -= health_cost;
                    if CONSUMING {
                        orbs.energy = 0.0;
                    } else {
                        self.recharge(orbs, missing_energy);
                    } // The game doesn't refund the health, it refunds it as energy
                    break true;
                }
                if !self.regenerate_as_needed(higher_cost, orbs) {
                    return false;
                }
            }
        } else {
            false
        }
    }

    fn health_is_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        orb_variants.retain(|orbs| {
            let met = orbs.health > cost
                || (self.skill(Skill::Regenerate)
                    && self.max_health() > cost
                    && self.regenerate_as_needed(cost, orbs));
            if CONSUMING {
                orbs.health -= cost
            }
            met
        });

        break_if_empty(orb_variants, Missing::Orbs)
    }

    fn regenerate_as_needed(&self, cost: f32, orbs: &mut Orbs) -> bool {
        let mut regens = ((cost - orbs.health) / 30.0).ceil();
        if orbs.health + 30.0 * regens <= cost {
            regens += 1.0
        }
        self.heal(orbs, 30.0 * regens);
        let game_thinks_regen_cost = Skill::Regenerate.energy_cost();
        let regen_cost = self.use_cost(Skill::Regenerate);
        // Regenerate is special cased to not allow Life Pact, so we don't go through cost_is_met
        orbs.energy -= regen_cost * regens;
        orbs.energy >= 0.0 && orbs.energy + regen_cost - game_thinks_regen_cost >= 0.0
        // On the final regenerate we have to make sure the the game is happy with our amount of resources
    }
}

fn break_if_empty(orb_variants: &OrbVariants, missing: Missing) -> ControlFlow<Missing> {
    if orb_variants.is_empty() {
        ControlFlow::Break(missing)
    } else {
        ControlFlow::Continue(())
    }
}
