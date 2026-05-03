use std::fmt::{self, Display};
use std::ops::ControlFlow;

use super::World;
use crate::logical_difficulty::{LogicalDifficulty, SHIELD_WEAPONS};
use crate::orbs::{self, format_orb_variants, OrbVariants, Orbs};
use crate::world::GraphRef;
use itertools::Itertools;
use log::trace;
use ordered_float::OrderedFloat;
use smallvec::SmallVec;
use wotw_seedgen_data::assets::{LocDataEntry, StateDataEntry};
use wotw_seedgen_data::logic_language::output::Node;
use wotw_seedgen_data::Teleporter;
use wotw_seedgen_data::{
    logic_language::output::{Enemy, Requirement},
    seed_language::simulate::Simulation,
    Difficulty, EqIgnore, Shard, Skill, UberIdentifier,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Missing<'graph> {
    Impossible,
    // UberState(UberIdentifier),
    Boolean(UberIdentifier),
    Integer(UberIdentifier, i32),
    LogicalState(usize),
    Health(OrderedFloat<f32>),
    Energy(OrderedFloat<f32>),
    WallWeapon,
    EnemyWeapon,
    // TODO which weapons are better depends on the health of the wall.
    // Noticed this with Grenade being offered as better than Bow because of DPE
    // but for common wall health values this is almost always wrong.
    EnergyOrBetterWallWeapon(OrderedFloat<f32>),
    EnergyOrBetterEnemyWeapon(OrderedFloat<f32>),
    EnergyOrBurrowOrBetterEnemyWeapon(OrderedFloat<f32>),
    // TODO if we don't make this type recursive but rather return lists of missing where needed we could try using smallvec
    Any(Vec<Missing<'graph>>),
    Or(
        Vec<(Missing<'graph>, GraphRef<'graph, Requirement>)>,
        EqIgnore<OrbVariants>,
    ),
}

impl Missing<'_> {
    fn uber_state(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        if uber_identifier.is_entrance() {
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

    fn weapon<const TARGET_IS_WALL: bool>() -> Self {
        if TARGET_IS_WALL {
            Self::WallWeapon
        } else {
            Self::EnemyWeapon
        }
    }

    fn energy_or_better_weapon<const TARGET_IS_WALL: bool>() -> fn(OrderedFloat<f32>) -> Self {
        if TARGET_IS_WALL {
            Self::EnergyOrBetterWallWeapon
        } else {
            Self::EnergyOrBetterEnemyWeapon
        }
    }

    fn energy_or_better_enemy_weapon(burrow_reduces_cost: bool) -> fn(OrderedFloat<f32>) -> Self {
        if burrow_reduces_cost {
            Self::EnergyOrBurrowOrBetterEnemyWeapon
        } else {
            Self::EnergyOrBetterEnemyWeapon
        }
    }
}

impl Display for Missing<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Missing::Impossible => "Impossible".fmt(f),
            Missing::Boolean(uber_identifier) => uber_identifier.fmt(f),
            Missing::Integer(uber_identifier, amount) => write!(f, "{uber_identifier}*{amount}"),
            Missing::LogicalState(state) => write!(f, "{{{state}}}"),
            Missing::Health(amount) => write!(f, "Health*{amount}"),
            Missing::Energy(amount) => write!(f, "Energy*{amount}"),
            Missing::WallWeapon => "WallWeapon".fmt(f),
            Missing::EnemyWeapon => "EnemyWeapon".fmt(f),
            Missing::EnergyOrBetterWallWeapon(amount) => {
                write!(f, "EnergyOrBetterWallWeapon*{amount}")
            }
            Missing::EnergyOrBetterEnemyWeapon(amount) => {
                write!(f, "EnergyOrBetterEnemyWeapon*{amount}")
            }
            Missing::EnergyOrBurrowOrBetterEnemyWeapon(amount) => {
                write!(f, "EnergyOrBurrowOrBetterEnemyWeapon*{amount}")
            }
            Missing::Any(any) => any.iter().format(" or ").fmt(f),
            Missing::Or(ors, orbs) => write!(
                f,
                "{ors} [{orbs}]",
                ors = ors.iter().format_with(" or ", |(missing, requirement), f| {
                    f(&format_args!("{missing} -> {}", requirement.0))
                }),
                orbs = format_orb_variants(&orbs.0)
            ),
        }
    }
}

impl<'graph> World<'graph, '_> {
    pub fn is_met(
        &self,
        requirement: &'graph Requirement,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
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
        requirement: &'graph Requirement,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        match requirement {
            Requirement::Free => ControlFlow::Continue(()),
            Requirement::Impossible => ControlFlow::Break(Missing::Impossible),
            Requirement::Difficulty(difficulty) => {
                self.setting_met(self.settings.difficulty >= *difficulty)
            }
            Requirement::NormalGameDifficulty => self.setting_met(!self.settings.hard),
            Requirement::Trick(trick) => self.setting_met(self.settings.tricks.contains(trick)),
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
                let mut burrow_reduces_cost = false;

                for (enemy, amount) in enemies {
                    let amount = f32::from(*amount);

                    match enemy {
                        Enemy::EnergyRefill => {
                            // It is possible for the total cost of a combat requirement to be different across orb variants because some of them may max out during energy refills
                            // However in between energy refills, the cost is always the same
                            self.cost_met_or_better_weapon::<true>(cost, orb_variants)?;

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
                                return ControlFlow::Break(Missing::Boolean(Skill::BURROW_ID));
                            } else {
                                burrow_reduces_cost = true;
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
                            // TODO same optimization for ranged / shield weapons?
                            Missing::EnemyWeapon
                        };

                        return ControlFlow::Break(missing);
                    };

                    cost += enemy_cost * amount;
                }

                // TODO what if what we need is specifically a better shield / ranged weapon?
                self.cost_met_or::<true>(
                    cost,
                    orb_variants,
                    Missing::energy_or_better_enemy_weapon(burrow_reduces_cost),
                )
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
                            missing.push((or_missing, GraphRef(or)));
                        }
                    }
                }

                if cheapest.is_empty() {
                    // TODO can we avoid cloning orb variants, for example by checking
                    // if the orbs are maxed?
                    ControlFlow::Break(Missing::Or(missing, EqIgnore(orb_variants.clone())))
                } else {
                    *orb_variants = cheapest;
                    ControlFlow::Continue(())
                }
            }
        }
    }

    fn setting_met(&self, condition: bool) -> ControlFlow<Missing<'graph>> {
        if condition {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::Impossible)
        }
    }

    fn skill_met(&self, skill: Skill) -> ControlFlow<Missing<'graph>> {
        self.boolean_met(self.skill(skill), skill.uber_identifier())
    }

    fn any_skill_met<T>(&self, skills: T) -> ControlFlow<Missing<'graph>>
    where
        T: IntoIterator<Item = Skill> + Copy,
    {
        if skills.into_iter().any(|skill| self.skill(skill)) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::any_skill(skills))
        }
    }

    fn shard_met(&self, shard: Shard) -> ControlFlow<Missing<'graph>> {
        self.boolean_met(self.shard(shard), shard.uber_identifier())
    }

    fn teleporter_met(&self, teleporter: Teleporter) -> ControlFlow<Missing<'graph>> {
        self.boolean_met(self.teleporter(teleporter), teleporter.uber_identifier())
    }

    fn boolean_met(
        &self,
        condition: bool,
        uber_identifier: UberIdentifier,
    ) -> ControlFlow<Missing<'graph>> {
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
    ) -> ControlFlow<Missing<'graph>> {
        let missing = expected - current;

        if missing <= 0 {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(Missing::Integer(uber_identifier, missing))
        }
    }

    // TODO use more arrayvec instead of smallvec
    fn enemy_movement_met(
        &self,
        enemies: &SmallVec<[(Enemy, u8); 12]>,
    ) -> ControlFlow<Missing<'graph>> {
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
    fn aerial_met(&self) -> ControlFlow<Missing<'graph>> {
        if self.settings.difficulty < Difficulty::Gorlek {
            self.any_skill_met([Skill::DoubleJump, Skill::Launch])
        } else {
            self.any_skill_met([Skill::DoubleJump, Skill::Launch, Skill::Bash])
        }
    }

    fn dangerous_met(&self) -> ControlFlow<Missing<'graph>> {
        self.any_skill_met([Skill::DoubleJump, Skill::Dash, Skill::Bash, Skill::Launch])
    }

    fn destroy_cost_met<const TARGET_IS_WALL: bool>(
        &self,
        target_health: f32,
        flying_target: bool,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        let Some(cost) = self.destroy_cost::<TARGET_IS_WALL>(target_health, flying_target) else {
            return ControlFlow::Break(Missing::weapon::<TARGET_IS_WALL>());
        };

        self.cost_met_or_better_weapon::<TARGET_IS_WALL>(cost, orb_variants)
    }

    fn cost_met_or_better_weapon<const TARGET_IS_WALL: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        self.cost_met_or::<true>(
            cost,
            orb_variants,
            Missing::energy_or_better_weapon::<TARGET_IS_WALL>(),
        )
    }

    fn cost_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        self.cost_met_or::<CONSUMING>(cost, orb_variants, Missing::Energy)
    }

    fn cost_met_or<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
        energy: fn(OrderedFloat<f32>) -> Missing<'graph>,
    ) -> ControlFlow<Missing<'graph>> {
        let mut missing = MissingOrbStats::new();

        if CONSUMING
            && self.settings.difficulty.life_pact()
            && self.shard(Shard::LifePact)
            && self.skill(Skill::Regenerate)
        {
            let mut new_orb_variants = OrbVariants::new();
            for orbs in &*orb_variants {
                self.regenerate_preemptively(*orbs, &mut new_orb_variants, &mut missing);
            }

            *orb_variants = orbs::either(orb_variants, &new_orb_variants);
        }

        orb_variants.retain(|orbs| self.orbs_meet_cost::<CONSUMING>(orbs, cost, &mut missing));

        break_if_empty(orb_variants, || missing.finish(energy))
    }

    fn regenerate_preemptively(
        &self,
        mut orbs: Orbs,
        new_orb_variants: &mut OrbVariants,
        missing: &mut MissingOrbStats,
    ) {
        // TODO if we fix the incorrect affordability calculations this might want to take the defense mod into account
        let heal_potential = self.max_health() - orbs.health;
        // Health is worth more than Energy with Life Pact and if we wait too long we might be unable to Regenerate later
        let regen_cost = self.use_cost(Skill::Regenerate);

        if heal_potential > regen_cost {
            let max_heals = ((heal_potential - regen_cost) / 30.).ceil() as u32;
            let higher_cost = regen_cost.max(Skill::Regenerate.energy_cost());

            for _ in 1..=max_heals {
                if orbs.energy >= higher_cost {
                    orbs.energy -= regen_cost;
                    self.heal(&mut orbs, 30.0);
                    trace!("adding regenerate option {orbs} to keep life pact enabled");
                    new_orb_variants.push(orbs);
                } else {
                    missing.energy = f32::max(missing.energy, higher_cost - orbs.energy);
                    break;
                }
            }
        }
    }

    fn orbs_meet_cost<const CONSUMING: bool>(
        &self,
        orbs: &mut Orbs,
        cost: f32,
        missing: &mut MissingOrbStats,
    ) -> bool {
        trace!("checking cost_met for cost {cost} with {orbs}");

        let met = orbs.energy >= cost || {
            missing.energy = f32::max(missing.energy, cost - orbs.energy);

            self.settings.difficulty.life_pact()
                && self.shard(Shard::LifePact)
                && match self.pay_life_pact::<CONSUMING>(orbs, cost) {
                    Ok(()) => return true,
                    Err(missing_health) => {
                        missing.health = f32::max(missing.health, missing_health);

                        if CONSUMING {
                            // already have paths for preemptive healing
                            false
                        } else {
                            self.max_health() > missing_health && {
                                if self.skill(Skill::Regenerate) {
                                    match self
                                        .regenerate_as_needed(missing_health, orbs)
                                        .and_then(|()| self.pay_life_pact::<CONSUMING>(orbs, cost))
                                    {
                                        Ok(()) => return true,
                                        Err(_) => false,
                                    }
                                } else {
                                    missing.regen = true;
                                    false
                                }
                            }
                        }
                    }
                }
        };

        if CONSUMING {
            orbs.energy -= cost;
        }

        met
    }

    fn pay_life_pact<const CONSUMING: bool>(&self, orbs: &mut Orbs, cost: f32) -> Result<(), f32> {
        let missing_energy = cost - orbs.energy;
        let game_thinks_health_cost = missing_energy * 10.0; // A health orb is ten times as much as an energy orb, but the game considers orbs equal for Life Pact
        let health_cost = game_thinks_health_cost * self.defense_mod();
        // TODO the higher cost only matters on the final use, not the entire cost
        let higher_cost = health_cost.max(game_thinks_health_cost); // we have to meet both

        if orbs.health > higher_cost {
            orbs.health -= health_cost;

            if CONSUMING {
                orbs.energy = 0.0;
            } else {
                // The game doesn't refund the health, it refunds it as energy
                self.recharge(orbs, missing_energy);
            }

            Ok(())
        } else {
            Err(higher_cost - orbs.health)
        }
    }

    fn health_met<const CONSUMING: bool>(
        &self,
        cost: f32,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        let mut missing = MissingOrbStats::new();

        orb_variants.retain(|orbs| self.orbs_meet_health::<CONSUMING>(cost, orbs, &mut missing));

        break_if_empty(orb_variants, || missing.finish(Missing::Energy))
    }

    fn orbs_meet_health<const CONSUMING: bool>(
        &self,
        cost: f32,
        orbs: &mut Orbs,
        missing: &mut MissingOrbStats,
    ) -> bool {
        trace!("checking health_met for cost {cost} with {orbs}");

        let met = orbs.health > cost || {
            missing.health = f32::max(missing.health, cost - orbs.health);

            self.max_health() > cost && {
                if self.skill(Skill::Regenerate) {
                    match self.regenerate_as_needed(cost, orbs) {
                        Ok(()) => true,
                        Err(energy) => {
                            missing.energy = f32::max(missing.energy, energy);
                            false
                        }
                    }
                } else {
                    missing.regen = true;
                    false
                }
            }
        };

        if CONSUMING {
            orbs.health -= cost
        }

        met
    }

    fn regenerate_as_needed(&self, cost: f32, orbs: &mut Orbs) -> Result<(), f32> {
        trace!("attempting to regenerate to meet cost {cost} with {orbs}");

        let mut regens = ((cost - orbs.health) / 30.0).ceil();
        if orbs.health + 30.0 * regens <= cost {
            regens += 1.0
        }
        self.heal(orbs, 30.0 * regens);
        let regen_cost = self.use_cost(Skill::Regenerate);
        // Regenerate is special cased to not allow Life Pact, so we don't go through cost_is_met
        orbs.energy -= regen_cost * regens;

        let remaining = f32::min(
            orbs.energy,
            // On the final regenerate we have to make sure the the game is happy with our amount of resources
            orbs.energy + regen_cost - Skill::Regenerate.energy_cost(),
        );

        if remaining >= 0.0 {
            Ok(())
        } else {
            Err(-remaining)
        }
    }
}

fn break_if_empty<'graph, M>(orb_variants: &OrbVariants, missing: M) -> ControlFlow<Missing<'graph>>
where
    M: FnOnce() -> Missing<'graph>,
{
    if orb_variants.is_empty() {
        ControlFlow::Break(missing())
    } else {
        ControlFlow::Continue(())
    }
}

struct MissingOrbStats {
    health: f32,
    energy: f32,
    regen: bool,
}

impl MissingOrbStats {
    fn new() -> Self {
        Self {
            health: f32::MIN,
            energy: f32::MIN,
            regen: false,
        }
    }

    fn finish<'graph>(self, energy: fn(OrderedFloat<f32>) -> Missing<'graph>) -> Missing<'graph> {
        let mut missing = vec![];

        if self.health > f32::MIN {
            missing.push(Missing::Health(self.health.into()));
        }
        if self.energy > f32::MIN {
            missing.push(energy(self.energy.into()));
        }
        if self.regen {
            missing.push(Missing::Boolean(Skill::REGENERATE_ID));
        }

        Missing::Any(missing)
    }
}
