use std::fmt::{self, Display};
use std::ops::ControlFlow;

use super::World;
use crate::logical_difficulty::{LogicalDifficulty, SHIELD_WEAPONS};
use crate::orbs::{self, OrbVariants, Orbs};
use itertools::Itertools;
use log::trace;
use smallvec::SmallVec;
use wotw_seedgen_data::assets::{LocDataEntry, StateDataEntry};
use wotw_seedgen_data::logic_language::output::Node;
use wotw_seedgen_data::Teleporter;
use wotw_seedgen_data::{
    logic_language::output::{Enemy, Requirement},
    seed_language::simulate::Simulation,
    Difficulty, Shard, Skill, UberIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Missing {
    Impossible,
    // UberState(UberIdentifier),
    Boolean(UberIdentifier),
    Integer(UberIdentifier, i32),
    LogicalState(usize),
    Health,
    Energy,
    // TODO if we don't make this type recursive but rather return lists of missing where needed we could try using smallvec
    Any(Vec<Missing>),
}

impl Missing {
    fn uber_state(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        if uber_identifier.is_door() {
            Self::Impossible
        } else {
            match value {
                None => Self::Boolean(uber_identifier),
                Some(value) => Self::Integer(uber_identifier, value),
            }
        }
    }

    fn state(index: usize, node: &Node) -> Self {
        match node {
            Node::Anchor(_) => {
                panic!("state requirement pointed to anchor {}", node.identifier())
            }
            Node::Pickup(LocDataEntry {
                uber_identifier,
                value,
                ..
            })
            | Node::State(StateDataEntry {
                uber_identifier,
                value,
                ..
            }) => Self::uber_state(*uber_identifier, *value),
            Node::LogicalState(_) => Missing::LogicalState(index),
        }
    }

    fn any_boolean<I: IntoIterator<Item = UberIdentifier>>(iter: I) -> Self {
        Self::Any(iter.into_iter().map(Self::Boolean).collect())
    }

    fn any_skill<I: IntoIterator<Item = Skill>>(iter: I) -> Self {
        Self::any_boolean(iter.into_iter().map(Skill::uber_identifier))
    }
}

impl Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Missing::Impossible => "Impossible".fmt(f),
            Missing::Boolean(uber_identifier) => uber_identifier.fmt(f),
            Missing::Integer(uber_identifier, value) => write!(f, "{uber_identifier}>={value}"),
            Missing::LogicalState(state) => write!(f, "{{{state}}}"),
            Missing::Health => "Health".fmt(f),
            Missing::Energy => "Energy".fmt(f),
            Missing::Any(any) => any.iter().format(" or ").fmt(f),
        }
    }
}

impl World<'_, '_> {
    pub fn is_met(
        &self,
        requirement: &Requirement,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        // TODO orbvariants newtype could be cool?
        trace!(
            "checking is_met for {requirement} with {orb_variants}",
            orb_variants = orb_variants.iter().format(" or ")
        );

        // TODO does this optimize cleanly? probably not!
        let flow = self.is_met_impl(requirement, orb_variants);

        match &flow {
            ControlFlow::Continue(()) => trace!(
                "{requirement} was met with {orb_variants}",
                orb_variants = orb_variants.iter().format(" or "),
            ),
            ControlFlow::Break(missing) => trace!("{requirement} was missing {missing}"),
        }

        flow
    }

    fn is_met_impl(
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
                self.cost_met::<true>(cost, orb_variants)
            }
            Requirement::NonConsumingEnergySkill(skill) => {
                self.skill_met(*skill)?;

                let cost = self.use_cost(*skill);
                self.cost_met::<false>(cost, orb_variants)
            }
            Requirement::SpiritLight(amount) => self.integer_met(
                self.spirit_light(),
                *amount as i32,
                UberIdentifier::SPIRIT_LIGHT,
            ),
            Requirement::GorlekOre(amount) => self.integer_met(
                self.gorlek_ore(),
                *amount as i32,
                UberIdentifier::GORLEK_ORE,
            ),
            Requirement::Keystone(amount) => {
                self.integer_met(self.keystones(), *amount as i32, UberIdentifier::KEYSTONES)
            }
            Requirement::Shard(shard) => self.shard_met(*shard),
            Requirement::Teleporter(teleporter) => self.teleporter_met(*teleporter),
            Requirement::Water => self.boolean_met(self.clean_water(), UberIdentifier::CLEAN_WATER),
            Requirement::State(state) => {
                if self.has_reached(*state) {
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(Missing::state(*state, &self.graph.nodes[*state]))
                }
            }
            Requirement::Damage(amount) => {
                let cost = *amount * self.defense_mod();
                self.health_met::<true>(cost, orb_variants)
            }
            Requirement::Danger(amount) => {
                let cost = *amount * self.defense_mod();
                self.health_met::<false>(cost, orb_variants)
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

                let shield_weapon = self.owned_shield_weapons().next();
                let mut cost = 0.0;

                for (enemy, amount) in enemies {
                    let amount = f32::from(*amount);

                    match enemy {
                        Enemy::EnergyRefill => {
                            // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                            // However in between energy refills, the cost is always the same
                            self.cost_met_or_better_weapons::<true>(cost, orb_variants)?;

                            for orbs in &mut *orb_variants {
                                self.recharge(orbs, amount);
                            }

                            cost = 0.0;
                            continue;
                        }
                        Enemy::Sandworm => {
                            if self.skill(Skill::Burrow) {
                                continue;
                            // TODO put all such comparisons into logical_difficulty?
                            } else if self.settings.difficulty < Difficulty::Unsafe {
                                return ControlFlow::Break(Missing::Boolean(
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
                            return ControlFlow::Break(Missing::any_skill(SHIELD_WEAPONS));
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
                            Missing::any_skill(self.settings.difficulty.ranged_weapons_iter())
                        } else {
                            Missing::any_skill(self.settings.difficulty.weapons_iter::<false>())
                        };

                        return ControlFlow::Break(missing);
                    };

                    cost += enemy_cost * amount;
                }

                self.cost_met_or_better_weapons::<true>(cost, orb_variants)
            }
            Requirement::ShurikenBreak(health) => {
                self.skill_met(Skill::Shuriken)?;

                let clip_mod = if self.settings.difficulty >= Difficulty::Unsafe {
                    2.0
                } else {
                    3.0
                };
                let cost = self.destroy_cost_with(*health, Skill::Shuriken, false) * clip_mod;

                self.cost_met::<true>(cost, orb_variants)
            }
            Requirement::SentryBreak(health) => {
                self.skill_met(Skill::Sentry)?;

                let clip_mod = 6.25;
                let cost = self.destroy_cost_with(*health, Skill::Sentry, false) * clip_mod;

                self.cost_met::<true>(cost, orb_variants)
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
        self.boolean_met(self.skill(skill), skill.uber_identifier())
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
        self.boolean_met(self.shard(shard), shard.uber_identifier())
    }

    fn teleporter_met(&self, teleporter: Teleporter) -> ControlFlow<Missing> {
        self.boolean_met(self.teleporter(teleporter), teleporter.uber_identifier())
    }

    fn boolean_met(
        &self,
        condition: bool,
        uber_identifier: UberIdentifier,
    ) -> ControlFlow<Missing> {
        if condition {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::Boolean(uber_identifier))
        }
    }

    fn integer_met(
        &self,
        current: i32,
        expected: i32,
        uber_identifier: UberIdentifier,
    ) -> ControlFlow<Missing> {
        let missing = expected - current;

        if missing <= 0 {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::Integer(uber_identifier, missing))
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

            // TODO don't have to go through all enemies if one of these breaks?
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

    // TODO these seem similar in nature to the different weapon arrays which come out of LogicalDifficulty, maybe they should be there?
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
            return ControlFlow::Break(Missing::any_skill(
                self.settings.difficulty.weapons_iter::<TARGET_IS_WALL>(),
            ));
        };

        self.cost_met_or_better_weapons::<TARGET_IS_WALL>(cost, orb_variants)
    }

    fn cost_met_or_better_weapons<const TARGET_IS_WALL: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        // TODO while it has improved, this still harms performance heavily in some cases because it generates solutions
        // like Spear + Grenade + Bow etc. trying to suggest better weapons even though those were already covered earlier
        // but it's important for correctness, otherwise destroy requirements that initially try to solve with an energy weapon may never complete
        // maybe it's just more general improvements needed in solutions, or maybe partial solutions should
        // remember whether they have branched into better weapons already and enable a shortcut to cost_met here
        self.cost_met::<true>(cost, orb_variants)
            .map_break(|missing| {
                let mut missing = vec![missing];

                missing.extend(
                    self.better_weapons::<TARGET_IS_WALL>()
                        .map(|weapon| Missing::Boolean(weapon.uber_identifier())),
                );

                Missing::Any(missing)
            })
    }

    fn better_weapons<const TARGET_IS_WALL: bool>(&self) -> impl Iterator<Item = Skill> + '_ {
        let mut lowest_cost = Skill::Spear.energy_cost();
        let mut highest_dpe =
            Skill::Sentry.damage_per_energy(self.settings.difficulty.charge_grenade());

        for owned in self.owned_weapons::<TARGET_IS_WALL>() {
            let cost = owned.energy_cost();
            lowest_cost = lowest_cost.min(cost);
            highest_dpe = highest_dpe
                .max(owned.total_damage(self.settings.difficulty.charge_grenade()) / cost);
        }

        self.settings
            .difficulty
            .weapons_iter::<TARGET_IS_WALL>()
            .filter(move |weapon| {
                weapon.energy_cost() < lowest_cost
                    || weapon.damage_per_energy(self.settings.difficulty.charge_grenade())
                        > highest_dpe
            })
    }

    fn cost_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        let mut added_orb_variants = vec![];

        orb_variants
            .retain(|orbs| self.orbs_meet_cost::<CONSUMING>(orbs, &mut added_orb_variants, cost));
        orb_variants.extend(added_orb_variants);

        break_if_empty(orb_variants, Missing::Energy)
    }

    fn orbs_meet_cost<const CONSUMING: bool>(
        &self,
        orbs: &mut Orbs,
        added_orb_variants: &mut Vec<Orbs>,
        cost: f32,
    ) -> bool {
        trace!("checking orbs_meet_cost for cost {cost} with {orbs}");

        let has_life_pact = self.settings.difficulty.life_pact() && self.shard(Shard::LifePact);
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
                trace!("adding regenerate option {new_orbs} to keep life pact enabled");
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
                        // The game doesn't refund the health, it refunds it as energy
                        self.recharge(orbs, missing_energy);
                    }

                    break true;
                }

                // TODO is this path not redundant with the preemptive regeneration?
                if !self.regenerate_as_needed(higher_cost, orbs) {
                    return false;
                }
            }
        } else {
            false
        }
    }

    fn health_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing> {
        orb_variants.retain(|orbs| self.orbs_meet_health::<CONSUMING>(cost, orbs));
        break_if_empty(orb_variants, Missing::Health)
    }

    fn orbs_meet_health<const CONSUMING: bool>(&self, cost: f32, orbs: &mut Orbs) -> bool {
        trace!("checking health_met for cost {cost} with {orbs}");

        if orbs.health > cost
            || (self.skill(Skill::Regenerate)
                && self.max_health() > cost
                && self.regenerate_as_needed(cost, orbs))
        {
            if CONSUMING {
                orbs.health -= cost
            }
            true
        } else {
            false
        }
    }

    fn regenerate_as_needed(&self, cost: f32, orbs: &mut Orbs) -> bool {
        trace!("attempting to regenerate to meet cost {cost} with {orbs}");

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
