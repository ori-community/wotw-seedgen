use super::*;

use std::iter;

use rustc_hash::FxHashSet;
use smallvec::smallvec;
use wotw_seedgen_data::{Shard, Teleporter};
use wotw_seedgen_logic_language::output::{Enemy, Requirement};
use wotw_seedgen_settings::Difficulty;

use crate::inventory::{item_count_from_spirit_light_amount, Inventory};
use crate::orbs::{OrbVariants, Orbs};
use crate::world::player::Player;

// Notes on a concept that I don't want to implement right now
// Instead of generating all possible solutions, and later choosing a random one, we could make random choices along the way
// For instance, a damage requirement could choose to solve with resilience 10% of the time (and communicate the result to all further damage requirements in the chain)
// We would then only follow through on that choice, and optimally keep working on just one solutions throughout the requirement chain
// The catch with this is that not all solutions are practical, the available item slots and pool eliminate anywhere between none and all solutions
// If we ever arrive at a point where the constraints eliminate the solution we're working on, we would have to pivot to another solution
// We'd still need to remember entry points that can potentially generate all possible solutions, but we could do so lazily
// Worst-case (no solution is practical) we still go through all the options, this could be mitigated by exiting out of impractical solutions as early as possible
// But in many cases this could significantly improve performance when choosing solutions

impl Player<'_> {
    /// Returns a set of [`Inventory`]s that would solve this [`Requirement`] if they were to be granted to the [`Player`]
    ///
    /// The solutions are checked to fit into the given amount of item slots. The returned `Vec` may be empty if no solution works with the available slots.
    ///
    /// The returned options are not guaranteed to be exhaustive across all the possible ways to solve the requirement.
    pub fn solutions(
        &self,
        requirement: &Requirement,
        states: &FxHashSet<usize>,
        orb_variants: OrbVariants,
        slots: usize,
        world_slots: usize,
    ) -> Vec<Inventory> {
        // We only need to clone this in debug builds where we want to display it later when some checks fail
        #[cfg(debug_assertions)]
        let cloned_if_debug = orb_variants.clone();
        #[cfg(not(debug_assertions))]
        let cloned_if_debug = orb_variants;
        let mut solutions = cloned_if_debug.into_iter().flat_map(|orbs| {
            let mut solutions = vec![TaggedSolution::new(self.inventory.clone(), orbs)];
            self.find_solutions(&mut solutions, requirement, states, slots, world_slots, self.inventory.world_item_count());

            solutions.into_iter().map(|solution| {
                #[cfg(debug_assertions)] assert!(solution.orbs.health >= 0.0 && solution.orbs.energy >= 0.0, "Negative health or energy after creating solution!\n\nRequirement: {self:?}\nSolution: {solution:?}\nPlayer inventory: {}\nOrb Variants: {orb_variants:?}", self.inventory);

                solution.inventory - self.inventory.clone()
            })
        }).collect::<Vec<_>>();

        filter_redundancies(&mut solutions);

        #[cfg(debug_assertions)]
        self.check_solutions(&solutions, requirement, states, orb_variants);

        solutions
    }

    fn find_solutions(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        requirement: &Requirement,
        states: &FxHashSet<usize>,
        slots: usize,
        world_slots: usize,
        initial_item_count: usize,
    ) {
        match requirement {
            Requirement::Free => {}
            Requirement::Impossible
            | Requirement::Difficulty(_)
            | Requirement::NormalGameDifficulty
            | Requirement::Trick(_) => solutions.clear(),
            Requirement::Skill(skill) => require_skill(solutions, *skill),
            Requirement::EnergySkill(skill, amount) => {
                self.needed_for_energy_skill(solutions, *skill, *amount, true)
            }
            Requirement::NonConsumingEnergySkill(skill) => {
                self.needed_for_energy_skill(solutions, *skill, 1.0, false)
            }
            Requirement::SpiritLight(amount) => require_spirit_light(solutions, *amount),
            Requirement::GorlekOre(amount) => require_gorlek_ore(solutions, *amount),
            Requirement::Keystone(amount) => require_keystones(solutions, *amount),
            Requirement::Shard(shard) => require_shard(solutions, *shard),
            Requirement::Teleporter(teleporter) => require_teleporter(solutions, *teleporter),
            Requirement::Water => require_clean_water(solutions),
            Requirement::State(state) => {
                if !states.contains(state) {
                    solutions.clear()
                }
            }
            Requirement::Damage(amount) => {
                self.needed_for_health(solutions, all(solutions), *amount, true)
            }
            Requirement::Danger(amount) => {
                self.needed_for_health(solutions, all(solutions), *amount, false)
            }
            Requirement::BreakWall(health) => self.needed_to_destroy::<true>(solutions, *health),
            Requirement::Boss(health) => self.needed_to_destroy::<false>(solutions, *health),
            Requirement::ShurikenBreak(health) => self.needed_to_destroy_with(
                solutions,
                Skill::Shuriken,
                if self.settings.difficulty >= Difficulty::Unsafe {
                    2.0
                } else {
                    3.0
                },
                *health,
            ),
            Requirement::SentryBreak(health) => {
                self.needed_to_destroy_with(solutions, Skill::Sentry, 6.25, *health)
            }
            Requirement::Combat(enemies) => self.needed_for_combat(solutions, enemies),
            Requirement::And(ands) => ands.iter().for_each(|and| {
                self.find_solutions(
                    solutions,
                    and,
                    states,
                    slots,
                    world_slots,
                    initial_item_count,
                )
            }),
            Requirement::Or(ors) => alternate_solutions(solutions, ors, |solutions, or| {
                self.find_solutions(
                    solutions,
                    or,
                    states,
                    slots,
                    world_slots,
                    initial_item_count,
                )
            }),
        }
        self.check_slot_limits(solutions, slots, world_slots, initial_item_count);
    }

    #[cfg(debug_assertions)]
    fn check_solutions(
        &self,
        solutions: &[Inventory],
        requirement: &Requirement,
        states: &FxHashSet<usize>,
        orb_variants: OrbVariants,
    ) {
        for solution in solutions {
            let mut solution_player = self.clone();
            let solution_extra_orbs = solution.max_orbs(self.settings.difficulty);
            solution_player.inventory += solution.clone();
            let solution_orb_variants = orb_variants
                .iter()
                .map(|orbs| *orbs + solution_extra_orbs)
                .collect();
            let is_met = !solution_player
                .is_met(requirement, states, solution_orb_variants)
                .is_empty();

            assert!(is_met, "Solution doesn't solve requirement!\nRequirement: {requirement:?}\nSolution: {solution}\nPlayer inventory: {}\nOrb Variants: {orb_variants:?}", self.inventory);
        }
    }

    fn needed_for_health(
        &self,
        total_solutions: &mut Vec<TaggedSolution>,
        solutions: Vec<usize>,
        mut amount: f32,
        consuming: bool,
    ) {
        let game_thinks_regenerate_cost = Skill::Regenerate.energy_cost();
        let mut regenerate_cost = game_thinks_regenerate_cost;
        if self.settings.difficulty < Difficulty::Unsafe {
            regenerate_cost *= 2.0;
        }

        if self.settings.hard {
            amount *= 2.0
        }

        self.call_for_alternatives(
            total_solutions,
            solutions,
            Tag::UseResilience,
            0.9,
            1.0,
            |total_solutions, solutions, defense_mod| {
                let amount = amount * defense_mod;

                // TODO I think you could eliminate all the fragment logic from here now and put the conversions into the inventory functions
                // would have to think about the health clause though to make sure it never expects going to 0 health
                let health_fragments_solution = |mut orbs: Orbs| {
                    let missing_health = amount - orbs.health;
                    let missing_health_fragments = (missing_health * 0.2).ceil().max(0.0);
                    let mut health_increase = missing_health_fragments * 5.0;
                    if missing_health >= health_increase {
                        health_increase += 5.0
                    }
                    orbs.health += health_increase; // granting fragments increases max health as well
                    if consuming {
                        orbs.health -= amount
                    }
                    (health_increase as usize, orbs)
                };

                self.call_for_alternatives(
                    total_solutions,
                    solutions,
                    Tag::UseOvercharge,
                    0.5,
                    1.0,
                    |total_solutions, solutions, cost_mod| {
                        // We are accepting some redundancies here if the player doesn't actually need to heal
                        let regenerate_cost = regenerate_cost * cost_mod;

                        for index in solutions {
                            let solution = &total_solutions[index];
                            let max_health =
                                solution.inventory.max_health(self.settings.difficulty);

                            let (health_increase, orbs) = health_fragments_solution(solution.orbs);

                            let higher_cost = regenerate_cost.max(game_thinks_regenerate_cost);
                            // If this solution used Life Pact before, that implies it used up all of its energy and Regenerate prohibits using health to pay for it
                            // If we tried to grant extra energy fragments to pay for the Regenerate, that extra energy would just get eaten up by the earlier cost that Life Pact payed for
                            // We can assume that some other solution contains a variant where enough energy was granted not to need Life Pact, we'll just put the Regenerate solutions into that other solution
                            // Technically there are still ways to Regenerate using nonconsuming energy weapons that refund the payed health as energy, but we don't consider this for logic because it is softlockable with efficiency upgrades
                            if solution.health_payed_for_life_pact == 0.0 {
                                // We don't use the tag here because in this context Regenerate never has a strict advantage, we'd never to cover all cases anyway
                                let max_regens =
                                    ((max_health - solution.orbs.health) * (1.0 / 30.0)).ceil()
                                        as usize;
                                if max_regens > 0 {
                                    let mut total_energy_increase = 0.;
                                    for _ in 1..=max_regens {
                                        let solution = &mut total_solutions[index];

                                        let missing_energy = higher_cost - solution.orbs.energy;
                                        let missing_energy_fragments =
                                            (missing_energy * 2.0).ceil().max(0.0);
                                        let energy_increase = missing_energy_fragments * 0.5;
                                        total_energy_increase += energy_increase;
                                        solution.orbs.energy += energy_increase - regenerate_cost; // granting fragments increases max energy as well
                                        solution.orbs.health =
                                            (solution.orbs.health + 30.0).min(max_health);

                                        let (health_increase, orbs) =
                                            health_fragments_solution(solution.orbs);
                                        let mut solution = TaggedSolution {
                                            orbs,
                                            ..solution.clone()
                                        };
                                        solution.inventory.skills.insert(Skill::Regenerate);
                                        solution.tags[Tag::UseRegenerate as usize] = Some(true);
                                        solution.inventory.health += health_increase;
                                        solution.inventory.energy += total_energy_increase; // the regen branch cannot be entered if we previously used Life Pact (this would not make sense), so we don't need to check health_payed_for_life_pact, it will be zero
                                        debug_assert_eq!(solution.health_payed_for_life_pact, 0.0);
                                        total_solutions.push(solution);
                                    }
                                }
                            }

                            // Reuse the current slot for the solution without Regenerate
                            let solution = &mut total_solutions[index];
                            solution.inventory.health += health_increase;
                            solution.orbs = orbs;
                        }
                    },
                );
            },
        );
    }

    fn needed_for_energy(
        &self,
        total_solutions: &mut Vec<TaggedSolution>,
        solutions: Vec<usize>,
        mut cost: f32,
        consuming: bool,
    ) {
        if self.settings.difficulty < Difficulty::Unsafe {
            cost *= 2.0
        }

        self.call_for_alternatives(
            total_solutions,
            solutions,
            Tag::UseOvercharge,
            0.5,
            1.0,
            |total_solutions, solutions, cost_mod| {
                let cost = cost * cost_mod;

                self.call_for_alternatives(
                    total_solutions,
                    solutions,
                    Tag::UseLifePact,
                    true,
                    false,
                    |total_solutions, solutions, use_life_pact| {
                        // We are accepting some redundancies here if the player doesn't actually need Life Pact
                        if use_life_pact {
                            let difficulty_mod = if self.settings.hard { 2.0 } else { 1.0 };
                            self.call_for_alternatives(
                                total_solutions,
                                solutions,
                                Tag::UseResilience,
                                0.9 * difficulty_mod,
                                difficulty_mod,
                                |total_solutions, mut solutions, defense_mod| {
                                    let f =
                                        |total_solutions: &mut Vec<TaggedSolution>,
                                         solutions: Vec<usize>,
                                         use_regenerate| {
                                            for index in solutions {
                                                let solution = &mut total_solutions[index];
                                                let mut new_index = None; // If we need to duplicate the solution, we will store the index here
                                                                          // This flag includes the information that we are allowed and may want to heal
                                                if use_regenerate {
                                                    debug_assert_eq!(
                                                        solution.health_payed_for_life_pact,
                                                        0.0
                                                    );
                                                    let max_health = solution
                                                        .inventory
                                                        .max_health(self.settings.difficulty);
                                                    let max_heal =
                                                        max_health - solution.orbs.health; // This is only correct because this solution didn't use Life Pact yet. Otherwise, we would have to grant energy fragments until we undid all the health payed for life pact

                                                    let game_thinks_regenerate_cost =
                                                        Skill::Regenerate.energy_cost();
                                                    let mut regenerate_cost =
                                                        game_thinks_regenerate_cost * cost_mod;
                                                    if self.settings.difficulty
                                                        < Difficulty::Unsafe
                                                    {
                                                        regenerate_cost *= 2.0;
                                                    }
                                                    let higher_cost = regenerate_cost
                                                        .max(game_thinks_regenerate_cost);

                                                    debug_assert!(
                                                        max_heal > regenerate_cost * 10.0
                                                    );
                                                    // Life Pact cannot use Health to pay for Regenerate, so before trying to regenerate we need to grant sufficient energy so we
                                                    // This is a bit absurd if we already payed a lot of health for life pact...
                                                    // It sounds very unlikely for regenerate to be worth it if we have to grant 10 energy fragments
                                                    // just to be able to regenerate at all because we used life pact so much
                                                    // But right now I can't think of a proof that regenerates are guaranteed to be redundant in any case here, so...
                                                    let grant_energy = |solution: &mut TaggedSolution, total_cost: f32, game_thinks_total_cost: f32| {
                                                        let higher_cost = total_cost.max(game_thinks_total_cost);
                                                        let missing_energy = higher_cost - solution.orbs.energy;
                                                        let missing_energy_fragments = (missing_energy * 2.0).ceil().max(0.0);
                                                        let energy_increase = missing_energy_fragments * 0.5;
                                                        solution.orbs.energy += energy_increase - total_cost;  // granting fragments increases max energy as well
                                                        solution.inventory.energy += energy_increase;
                                                    };

                                                    let regens = max_heal / 30.0;
                                                    let max_optimal_regens = regens.floor();

                                                    let total_cost =
                                                        regenerate_cost * max_optimal_regens;
                                                    let game_thinks_total_cost =
                                                        total_cost - regenerate_cost + higher_cost; // On the final regenerate we have to make sure the the game is happy with our amount of resources
                                                    grant_energy(
                                                        solution,
                                                        total_cost,
                                                        game_thinks_total_cost,
                                                    );
                                                    solution.orbs.health +=
                                                        30.0 * max_optimal_regens; // This cannot exceed max health because we are doing optimal heals

                                                    let max_heal =
                                                        max_health - solution.orbs.health;
                                                    if max_heal > regenerate_cost * 10.0 {
                                                        // oh no we found a suboptimal heal that we may want to do
                                                        let old_solution = solution.clone();
                                                        debug_assert!(
                                                            solution.orbs.health
                                                                > max_health - 30.0
                                                        );

                                                        grant_energy(
                                                            solution,
                                                            regenerate_cost,
                                                            higher_cost,
                                                        );
                                                        solution.orbs.health = max_health; // We are doing a suboptimal heal

                                                        new_index = Some(total_solutions.len());
                                                        total_solutions.push(old_solution);
                                                    }
                                                }

                                                for index in iter::once(index).chain(new_index) {
                                                    let mut solution = &mut total_solutions[index];

                                                    let max_energy = solution
                                                        .inventory
                                                        .max_energy(self.settings.difficulty);
                                                    let mut missing_energy =
                                                        (cost - solution.orbs.energy).max(0.0);
                                                    let mut health_cost = missing_energy * 10.0;

                                                    let health_fragment_solution = |solution: &mut TaggedSolution, health_cost: f32| {
                                                        let true_health_cost = health_cost * defense_mod;
                                                        let higher_amount = health_cost.max(true_health_cost);
                                                        let missing_health = higher_amount - solution.orbs.health;
                                                        let missing_health_fragments = (missing_health * 0.2).ceil().max(0.0);
                                                        let mut health_increase = missing_health_fragments * 5.0;
                                                        if missing_health >= health_increase { health_increase += 5.0 }
                                                        solution.orbs.health += health_increase;  // granting fragments increases max health as well
                                                        solution.orbs.health -= true_health_cost;
                                                        solution.health_payed_for_life_pact += true_health_cost;
                                                        if consuming { solution.orbs.energy = 0.0; }
                                                        else { solution.orbs.energy = (solution.orbs.energy + true_health_cost * 0.1).min(max_energy) }

                                                        solution.inventory.health += health_increase as usize;
                                                    };

                                                    if solution
                                                        .do_not_generate_all_life_pact_variants
                                                    {
                                                        // This flag indicates we only need to generate the health fragment solution
                                                        // In greater detail, it indicates a previous requirement already generated a full set of health/energy fragment combinations
                                                        // Since the branch for when this flag is false grants one energy at a time, it is guaranteed that on the
                                                        // previous requirement, an alternate solution with one more energy fragment has already been generated
                                                        // This is true all the way through the solution chain up to the solution with the most energy fragments,
                                                        // which will not have this tag and will be the only one that generates all fragment combinations again
                                                        // This approach greatly reduces redundancies on chained energy requirements
                                                        health_fragment_solution(
                                                            solution,
                                                            health_cost,
                                                        );
                                                        continue;
                                                    }

                                                    debug_assert_eq!(
                                                        solution.health_payed_for_life_pact,
                                                        0.0
                                                    );
                                                    loop {
                                                        if missing_energy <= 0.0 {
                                                            if consuming {
                                                                solution.orbs.energy -= cost
                                                            }
                                                            // This solution (the pure energy fragment solution) is the only one that won't have the do_not_generate_all_life_pact_variants flag set
                                                            // On future requirements, we will only generate the full set of health/energy fragment combinations based on this one.
                                                            break;
                                                        } else {
                                                            let mut solution = solution.clone();

                                                            health_fragment_solution(
                                                                &mut solution,
                                                                health_cost,
                                                            );
                                                            solution
                                                        .do_not_generate_all_life_pact_variants =
                                                        true; // This is a note to future requirements so we can generate the health/energy fragment combinations for chained requirements with less redundancies

                                                            total_solutions.push(solution);
                                                        }

                                                        solution = &mut total_solutions[index];
                                                        solution.inventory.energy += 0.5;
                                                        solution.orbs.energy += 0.5; // As asserted above, this solution has never used life pact. Otherwise, we would have to perform additional calculations to determine how much we would increase the health instead of the energy
                                                        missing_energy -= 0.5;
                                                        health_cost -= 5.0;
                                                        // The logic below could be optimized away based on assumptions around the do_not_generate_all_life_pact_variants flag, keeping it in case I forgot something
                                                        // // If a previous solution used Life Pact, then granting an energy fragment here may not actually increase our energy
                                                        // // Instead the previous requirement will consume this energy and we have a little more health in return
                                                        // // This matters because of the game's incorrect affordability calculations
                                                        // // For example, on two consecutive Grenade throws, we may choose to solve the first Grenade throw using
                                                        // // Grenade, Life Pact, 2 Health Fragment, Energy Fragment, Resilience; keeping 5.5 Health and 0 Energy afterwards, so seemingly missing 4.5 energy for the next grenade
                                                        // // If we then attempt to solve the second Grenade throw with just one additional Energy Fragment, that solution would be incorrect.
                                                        // // After granting the additional Energy Fragment, the first Grenade will consume Energy instead of consuming health as originally assumed.
                                                        // // On zero energy and one orb of health the game then refuses to cast the second Grenade despite the Resilience
                                                        // let increased_health_instead_of_energy = solution.health_payed_for_life_pact.min(health_per_energy_fragment);
                                                        // let energy_refill = (0.5 - increased_health_instead_of_energy * 0.1 / defense_mod).max(0.0);
                                                        // solution.orbs.energy += energy_refill;  // granting fragments increases max energy as well
                                                        // solution.orbs.health += increased_health_instead_of_energy;
                                                        // solution.health_payed_for_life_pact -= increased_health_instead_of_energy;
                                                        // missing_energy -= energy_refill;
                                                        // health_cost -= 10.0 * energy_refill;
                                                    }
                                                }
                                            }
                                        };

                                    // If we intend to use Life Pact, we may want to Regenerate before attempting to pay the cost
                                    // In most cases, as long as we can heal the full 30 health, we will always want to use Regenerate here before we run out of energy and can't anymore
                                    // Only on logical difficulties where regen costs 2 energy and hard in-game difficulty, the health is less valuable than the energy
                                    // In cases where we can still heal with a value plus, but not the full 30 health, we cannot decide which option is preferrable
                                    // we may take damage from spikes on a later requirement and then want to heal after that, or we may require the energy to even progress
                                    // So when taking a suboptimal heal we have to split this solution into two
                                    // The need to heal in advance is also why we can't use needed_for_health here
                                    let regen_is_worth_it = !(self.settings.hard
                                        && self.settings.difficulty < Difficulty::Unsafe);
                                    let never_regen = if regen_is_worth_it {
                                        // TODO this is a little eager? Many of these solutions will have full health, if they can't use Regenerate usefully they will be redundant
                                        // If there is a sort of filter added to what should be considered for regenerate use should it get the health payed for life pact check as well?
                                        let partition_index =
                                            partition::partition_index(&mut solutions, |index| {
                                                // TODO use partition_in_place once stabilized
                                                let solution = &total_solutions[*index];
                                                // We only consider using regenerate if this solution hasn't used life pact yet
                                                // This is because life pact cannot pay for regenerate with health
                                                // So in order to regenerate, we would have to grant sufficient energy fragments until we wouldn't have had to pay any health for life pact
                                                // However, even among the solutions tagged to use life pact, there will always be one that solved purely through energy fragments and didn't pay any health yet
                                                // If we granted sufficient energy to use regenerate here we would have to grant as much energy as that one solution
                                                // We can eliminate redundancies here by only considering regenerate for the one solution that hasn't payed life pact yet
                                                if solution.health_payed_for_life_pact > 0.0 {
                                                    return false;
                                                }

                                                // TODO we do this calculation again later... should we store it somewhere?
                                                let max_health = solution
                                                    .inventory
                                                    .max_health(self.settings.difficulty);
                                                // We don't have to subtract health_payed_for_life_pact because we already checked whether it's > 0 above
                                                let max_heal = max_health - solution.orbs.health;

                                                let mut regenerate_cost =
                                                    Skill::Regenerate.energy_cost() * cost_mod;
                                                if self.settings.difficulty < Difficulty::Unsafe {
                                                    regenerate_cost *= 2.0;
                                                }

                                                max_heal > regenerate_cost * 10.0
                                            });
                                        let never_regen = solutions.split_off(partition_index);
                                        self.call_for_alternatives(
                                            total_solutions,
                                            solutions,
                                            Tag::UseRegenerate,
                                            true,
                                            false,
                                            f,
                                        );
                                        never_regen
                                    } else {
                                        solutions
                                    };
                                    f(total_solutions, never_regen, false);
                                },
                            );
                        } else {
                            for index in solutions {
                                let solution = &mut total_solutions[index];
                                let missing_energy = cost - solution.orbs.energy;
                                let missing_energy_fragments =
                                    (missing_energy * 2.0).ceil().max(0.0);
                                let energy_increase = missing_energy_fragments * 0.5;
                                solution.orbs.energy += energy_increase; // granting fragments increases max energy as well
                                if consuming {
                                    solution.orbs.energy -= cost;
                                }
                                solution.inventory.energy += energy_increase; // this branch cannot be entered if we previously used Life Pact, so we don't need to check health_payed_for_life_pact, it will be zero
                                debug_assert_eq!(solution.health_payed_for_life_pact, 0.0);
                            }
                        }
                    },
                );
            },
        );
    }

    fn needed_for_energy_skill(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        skill: Skill,
        amount: f32,
        consuming: bool,
    ) {
        require_skill(solutions, skill);
        self.needed_for_energy(
            solutions,
            all(solutions),
            skill.energy_cost() * amount,
            consuming,
        ); // TODO because of the game miscalculating use costs, it's slightly incorrect to just multiply the cost here
    }
    fn needed_to_destroy<const TARGET_IS_WALL: bool>(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        health: f32,
    ) {
        alternate_solutions(
            solutions,
            self.progression_weapons::<TARGET_IS_WALL>(),
            |solutions, weapon| self.needed_to_destroy_with(solutions, weapon, 1.0, health),
        );
    }
    fn needed_to_destroy_with(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        weapon: Skill,
        cost_mod: f32,
        health: f32,
    ) {
        let cost = self.kill_cost(weapon, health, false) * cost_mod;
        require_skill(solutions, weapon);
        self.needed_for_energy(solutions, all(solutions), cost, true);
    }
    fn kill_cost(&self, weapon: Skill, health: f32, flying_target: bool) -> f32 {
        // TODO damage buff progressions
        let damage = weapon.damage(self.settings.difficulty >= Difficulty::Unsafe)
            * self.damage_mod(flying_target, matches!(weapon, Skill::Bow)) // TODO is it correct to use self here??
            + weapon.burn_damage(); // We are only allowed to add the player's damage mod here because we're not factoring damage upgrades into the progressions yet
        weapon.energy_cost() * (health / damage).ceil()
    }

    fn needed_for_combat(&self, solutions: &mut Vec<TaggedSolution>, enemies: &[(Enemy, u8)]) {
        let mut aerial = false;
        let mut dangerous = false;
        let mut ranged = false;
        let mut melee = self.settings.difficulty >= Difficulty::Unsafe;
        let mut shielded = false;
        let mut bash = false;
        let mut burrow = false;

        for (enemy, _) in enemies {
            shielded |= enemy.shielded();
            burrow |= matches!(enemy, Enemy::Sandworm);
            if self.settings.difficulty < Difficulty::Unsafe {
                bash |= matches!(enemy, Enemy::Bat);
                aerial |= enemy.aerial();
                dangerous |= enemy.dangerous();
                if enemy.ranged() {
                    ranged = true
                } else {
                    melee = true
                }
            }
        }

        // Skip unneccesary iterations over weapons that are redundant anyway
        let weapons = if melee {
            self.progression_weapons::<false>() // TODO considering the solutions inventory isn't trivial because currently we perform this operation once for all solutions, but it might be worth trying anyway
        } else {
            smallvec![Skill::Sword]
        };
        let ranged_weapons = if ranged {
            self.ranged_progression_weapons()
        } else {
            smallvec![Skill::Spear]
        };
        let shield_weapons = if shielded {
            self.shield_progression_weapons()
        } else {
            smallvec![Skill::Spear]
        };
        let use_burrow: SmallVec<[_; 2]> = if burrow {
            if self.settings.difficulty < Difficulty::Unsafe
                || self.inventory.skills.contains(&Skill::Burrow)
            {
                smallvec![true]
            } else {
                smallvec![true, false]
            }
        } else {
            smallvec![false]
        };

        // Filter combinations of weapons for redundancies
        let weapons_len = weapons.len();
        let mut weapon_combinations =
            Vec::with_capacity(weapons_len * ranged_weapons.len() * shield_weapons.len());
        for ranged_weapon in ranged_weapons {
            for &shield_weapon in &shield_weapons {
                // TODO I think this is meant to filter overlap between ranged and shield weapons as well
                let weapon_position = weapons
                    .iter()
                    .position(|&weapon| weapon == ranged_weapon || weapon == shield_weapon)
                    .map_or(weapons_len, |index| (index + 1).min(weapons_len));
                for weapon in &weapons[0..weapon_position] {
                    for burrow in &use_burrow {
                        weapon_combinations.push((*weapon, ranged_weapon, shield_weapon, *burrow));
                    }
                }
            }
        }

        alternate_solutions(
            solutions,
            weapon_combinations,
            |solutions, (weapon, ranged_weapon, shield_weapon, burrow)| {
                let mut cost = 0.0;

                for (enemy, amount) in enemies {
                    let amount = f32::from(*amount);
                    match enemy {
                        Enemy::EnergyRefill => {
                            self.needed_for_energy(solutions, all(solutions), cost, true);
                            solutions.iter_mut().for_each(|solution| {
                                solution.inventory.recharge(
                                    &mut solution.orbs,
                                    amount,
                                    self.settings.difficulty,
                                )
                            });
                            cost = 0.0;
                            continue;
                        }
                        Enemy::Sandworm if burrow => continue,
                        _ => {}
                    }

                    let mut health = enemy.health();

                    if enemy.shielded() {
                        cost += shield_weapon.energy_cost() * amount;
                        health = (health - shield_weapon.burn_damage()).max(0.0);
                    } else if enemy.armored() && self.settings.difficulty < Difficulty::Unsafe {
                        health *= 2.0
                    }; // No enemy is shielded and armored

                    let used_weapon =
                        if enemy.ranged() && self.settings.difficulty < Difficulty::Unsafe {
                            ranged_weapon
                        } else {
                            weapon
                        };

                    cost += self.kill_cost(used_weapon, health, enemy.flying()) * amount;
                }

                self.needed_for_energy(solutions, all(solutions), cost, true);
                if melee {
                    require_skill(solutions, weapon);
                }
                if ranged {
                    require_skill(solutions, ranged_weapon);
                }
                if shielded {
                    require_skill(solutions, shield_weapon);
                }
                if burrow {
                    require_skill(solutions, Skill::Burrow);
                }
            },
        );

        if self.settings.difficulty < Difficulty::Unsafe {
            if bash {
                require_skill(solutions, Skill::Bash);
            }
            if aerial && (!bash || self.settings.difficulty < Difficulty::Gorlek) {
                let mut ranged_skills = vec![Skill::DoubleJump, Skill::Launch];
                if self.settings.difficulty >= Difficulty::Gorlek {
                    ranged_skills.push(Skill::Bash);
                }
                require_any_of(self, solutions, ranged_skills);
            } else if dangerous && !bash {
                let evasion_skills = [Skill::DoubleJump, Skill::Dash, Skill::Bash, Skill::Launch];
                require_any_of(self, solutions, evasion_skills);
            }
        }
    }

    fn check_slot_limits(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        slots: usize,
        world_slots: usize,
        initial_item_count: usize,
    ) {
        solutions.retain(|solution| {
            let spirit_light_slots = item_count_from_spirit_light_amount(
                solution.inventory.spirit_light - self.inventory.spirit_light,
            );
            solution.inventory.world_item_count() + spirit_light_slots <= initial_item_count + slots
                && spirit_light_slots <= world_slots
        });
    }

    fn call_for_alternatives<T, F>(
        &self,
        total_solutions: &mut Vec<TaggedSolution>,
        solutions: Vec<usize>,
        tag: Tag,
        value_when_using: T,
        value_when_not_using: T,
        f: F,
    ) where
        F: Fn(&mut Vec<TaggedSolution>, Vec<usize>, T),
    {
        if self.settings.difficulty >= tag.difficulty() {
            if tag.has(&self.inventory) {
                f(total_solutions, solutions, value_when_using);
            } else {
                let (using, not_using) = duplicate_solutions(total_solutions, solutions, tag);
                f(total_solutions, using, value_when_using);
                f(total_solutions, not_using, value_when_not_using);
            }
        } else {
            f(total_solutions, solutions, value_when_not_using);
        }
    }
}

/// We tag some solutions to avoid redundancies
///
/// For instance, in a chain of Damage requirements which add solutions with or without Resilience, when naively combining all the individual solutions,
/// the amount of solutions will increase exponentially with the amount of chained damage requirements, only for almost everything to be filtered out again
///
/// To avoid this, Damage requirements may tag their solutions as having used or denied to use Resilience, which future requirements in the chain will pick up
#[derive(Debug, Clone)]
struct TaggedSolution {
    inventory: Inventory, // TODO would the code improve if this was a player? We'd only be copying extras pointers, shouldn't hurt
    orbs: Orbs,
    tags: [Option<bool>; 4], // TODO this could be mem::variant_count once stabilized
    health_payed_for_life_pact: f32, // need to track this because it influences what happens when we give extra energy fragments (see needed_for_energy)
    do_not_generate_all_life_pact_variants: bool, // purely for optimization, we use this marker to skip generating some redundant solutions
}
impl TaggedSolution {
    fn new(inventory: Inventory, orbs: Orbs) -> Self {
        Self {
            inventory,
            orbs,
            tags: <[Option<bool>; 4]>::default(),
            health_payed_for_life_pact: 0.0,
            do_not_generate_all_life_pact_variants: false,
        }
    }
}
#[allow(clippy::enum_variant_names)]
#[repr(usize)]
#[derive(Clone, Copy)]
enum Tag {
    UseResilience,
    UseOvercharge,
    UseLifePact,
    UseRegenerate,
}
impl Tag {
    // TODO this would feel more intuitive on player
    fn has(self, inventory: &Inventory) -> bool {
        match self {
            Tag::UseResilience => inventory.shards.contains(&Shard::Resilience),
            Tag::UseOvercharge => inventory.shards.contains(&Shard::Overcharge),
            Tag::UseLifePact => inventory.shards.contains(&Shard::LifePact),
            Tag::UseRegenerate => inventory.skills.contains(&Skill::Regenerate),
        }
    }
    fn grant(self, inventory: &mut Inventory) -> bool {
        match self {
            Tag::UseResilience => inventory.shards.insert(Shard::Resilience),
            Tag::UseOvercharge => inventory.shards.insert(Shard::Overcharge),
            Tag::UseLifePact => inventory.shards.insert(Shard::LifePact),
            Tag::UseRegenerate => inventory.skills.insert(Skill::Regenerate),
        }
    }
    fn difficulty(self) -> Difficulty {
        match self {
            Tag::UseRegenerate => Difficulty::Moki,
            Tag::UseResilience => Difficulty::Gorlek,
            Tag::UseOvercharge | Tag::UseLifePact => Difficulty::Unsafe,
        }
    }
}

fn duplicate_solutions(
    total_solutions: &mut Vec<TaggedSolution>,
    solutions: Vec<usize>,
    tag: Tag,
) -> (Vec<usize>, Vec<usize>) {
    // This is the core of our redundancy reduction
    let len = total_solutions.len();
    let (using, mut not_using) = solutions.into_iter().partition::<Vec<_>, _>(|index| {
        total_solutions[*index].tags[tag as usize].unwrap_or_else(|| {
            // If this tag hasn't been applied yet, we have to duplicate this solution to start tracking both variants and it shouldn't cause redundancies
            let old = &mut total_solutions[*index];
            let mut new = old.clone();
            tag.grant(&mut old.inventory);
            old.tags[tag as usize] = Some(true);
            new.tags[tag as usize] = Some(false);
            total_solutions.push(new);
            true
        })
    });
    not_using.extend(len..total_solutions.len()); // These are the ones we actually duplicated
    (using, not_using)
}

fn all(solutions: &[TaggedSolution]) -> Vec<usize> {
    (0..solutions.len()).collect()
}

// TODO it might be a lot of work but anything using this function could probably be optimized for less redundancies
fn alternate_solutions<T, I, F>(solutions: &mut Vec<TaggedSolution>, alternatives: I, f: F)
where
    I: IntoIterator<Item = T>,
    F: Fn(&mut Vec<TaggedSolution>, T),
{
    *solutions = alternatives
        .into_iter()
        .flat_map(|alternative| {
            let mut new_solutions = solutions.clone();
            f(&mut new_solutions, alternative);
            new_solutions
        })
        .collect();
}

fn require<F: FnMut(&mut TaggedSolution)>(solutions: &mut [TaggedSolution], f: F) {
    solutions.iter_mut().for_each(f);
}
fn require_spirit_light(solutions: &mut [TaggedSolution], amount: usize) {
    require(solutions, |solution| {
        solution.inventory.spirit_light = usize::max(solution.inventory.spirit_light, amount);
    })
}
fn require_gorlek_ore(solutions: &mut [TaggedSolution], amount: usize) {
    require(solutions, |solution| {
        solution.inventory.gorlek_ore = usize::max(solution.inventory.gorlek_ore, amount);
    })
}
fn require_keystones(solutions: &mut [TaggedSolution], amount: usize) {
    require(solutions, |solution| {
        solution.inventory.keystones = usize::max(solution.inventory.keystones, amount);
    })
}
fn require_skill(solutions: &mut [TaggedSolution], skill: Skill) {
    require(solutions, |solution| {
        solution.inventory.skills.insert(skill);
    });
}
fn require_shard(solutions: &mut [TaggedSolution], shard: Shard) {
    require(solutions, |solution| {
        solution.inventory.shards.insert(shard);
    });
}
fn require_teleporter(solutions: &mut [TaggedSolution], teleporter: Teleporter) {
    require(solutions, |solution| {
        solution.inventory.teleporters.insert(teleporter);
    });
}
fn require_clean_water(solutions: &mut [TaggedSolution]) {
    require(solutions, |solution| solution.inventory.clean_water = true);
}
// TODO delete
fn require_any_of<I: IntoIterator<Item = Skill> + AsRef<[Skill]>>(
    player: &Player,
    solutions: &mut Vec<TaggedSolution>,
    skills: I,
) {
    if !skills
        .as_ref()
        .iter()
        .any(|skill| player.inventory.skills.contains(skill))
    {
        alternate_solutions(solutions, skills, |solutions, skill| {
            require_skill(solutions, skill)
        });
    }
}

pub(crate) fn filter_redundancies(solutions: &mut Vec<Inventory>) {
    solutions.sort_unstable_by_key(Inventory::item_count); // start with the small solutions to eliminate many redundancies quickly
    let mut len = solutions.len();

    for index in 1.. {
        if index >= len {
            break;
        }
        let solution = solutions[index - 1].clone();
        // Filtering manually so we can skip going through parts of the solutions that we're already done with
        let mut deleted = 0;
        for other_index in index..len {
            if solutions[other_index].contains(&solution) {
                deleted += 1;
            } else if deleted > 0 {
                solutions.swap(other_index, other_index - deleted);
            }
        }
        len -= deleted;
    }
    solutions.truncate(len);
}
