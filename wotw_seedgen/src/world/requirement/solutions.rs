use super::*;

use std::iter;

use rustc_hash::FxHashSet;
use smallvec::smallvec;

use crate::inventory::Inventory;
use crate::item::Item;
use crate::util::orbs::{OrbVariants, Orbs};
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

impl Requirement {
    /// Returns a set of [`Inventory`]s that would solve this [`Requirement`] if they were to be granted to the [`Player`]
    ///
    /// The solutions are checked to fit into the given amount of item slots. The returned `Vec` may be empty if no solution works with the available slots.
    ///
    /// The returned options are not guaranteed to be exhaustive across all the possible ways to solve the requirement.
    pub fn solutions(
        &self,
        player: &Player,
        states: &FxHashSet<usize>,
        orb_variants: OrbVariants,
        slots: usize,
        world_slots: usize,
    ) -> Vec<Inventory> {
        let (player_item_count, player_spirit_light) =
            item_count_and_spirit_light(&player.inventory);

        // We only need to clone this in debug builds where we want to display it later when some checks fail
        #[cfg(debug_assertions)]
        let cloned_if_debug = orb_variants.clone();
        #[cfg(not(debug_assertions))]
        let cloned_if_debug = orb_variants;
        let mut solutions = cloned_if_debug.into_iter().flat_map(|orbs| {
            let mut solutions = vec![TaggedSolution::new(player.inventory.clone(), orbs)];
            self.find_solutions(&mut solutions, player, states, slots as u32, world_slots as u32, player_item_count, player_spirit_light);

            solutions.into_iter().map(|solution| {
                #[cfg(debug_assertions)] assert!(solution.orbs.health >= 0.0 && solution.orbs.energy >= 0.0, "Negative health or energy after creating solution!\n\nRequirement: {self:?}\nSolution: {solution:?}\nPlayer inventory: {}\nOrb Variants: {orb_variants:?}", player.inventory);

                let mut inventory = solution.inventory;
                for (item, amount) in &player.inventory.items {
                    inventory.remove(item, *amount);
                }
                inventory
            })
        }).collect::<Vec<_>>();

        filter_redundancies(&mut solutions);

        #[cfg(debug_assertions)]
        self.check_solutions(&solutions, player, states, orb_variants);

        solutions
    }

    fn find_solutions(
        &self,
        solutions: &mut Vec<TaggedSolution>,
        player: &Player,
        states: &FxHashSet<usize>,
        slots: u32,
        world_slots: u32,
        player_item_count: u32,
        player_spirit_light: u32,
    ) {
        match self {
            Requirement::Free => {}
            Requirement::Impossible
            | Requirement::Difficulty(_)
            | Requirement::NormalGameDifficulty
            | Requirement::Trick(_) => solutions.clear(),
            Requirement::Skill(skill) => require(solutions, Item::Skill(*skill)),
            Requirement::EnergySkill(skill, amount) => {
                needed_for_energy_skill(solutions, player, *skill, *amount, true)
            }
            Requirement::NonConsumingEnergySkill(skill) => {
                needed_for_energy_skill(solutions, player, *skill, 1.0, false)
            }
            Requirement::SpiritLight(amount) => {
                require_missing(solutions, Item::SpiritLight(1), *amount)
            }
            Requirement::Resource(resource, amount) => {
                require_missing(solutions, Item::Resource(*resource), *amount)
            }
            Requirement::Shard(shard) => require(solutions, Item::Shard(*shard)),
            Requirement::Teleporter(teleporter) => {
                require(solutions, Item::Teleporter(*teleporter))
            }
            Requirement::Water => require(solutions, Item::Water),
            Requirement::State(state) => {
                if !states.contains(state) {
                    solutions.clear()
                }
            }
            Requirement::Damage(amount) => {
                needed_for_health(solutions, all(solutions), *amount, player, true)
            }
            Requirement::Danger(amount) => {
                needed_for_health(solutions, all(solutions), *amount, player, false)
            }
            Requirement::BreakWall(health) => needed_to_destroy::<true>(solutions, player, *health),
            Requirement::Boss(health) => needed_to_destroy::<false>(solutions, player, *health),
            Requirement::ShurikenBreak(health) => needed_to_destroy_with(
                solutions,
                Skill::Shuriken,
                if player.settings.difficulty >= Difficulty::Unsafe {
                    2.0
                } else {
                    3.0
                },
                player,
                *health,
            ),
            Requirement::SentryBreak(health) => {
                needed_to_destroy_with(solutions, Skill::Sentry, 6.25, player, *health)
            }
            Requirement::Combat(enemies) => needed_for_combat(solutions, player, enemies),
            Requirement::And(ands) => ands.iter().for_each(|and| {
                and.find_solutions(
                    solutions,
                    player,
                    states,
                    slots,
                    world_slots,
                    player_item_count,
                    player_spirit_light,
                )
            }),
            Requirement::Or(ors) => alternate_solutions(solutions, ors, |solutions, or| {
                or.find_solutions(
                    solutions,
                    player,
                    states,
                    slots,
                    world_slots,
                    player_item_count,
                    player_spirit_light,
                )
            }),
        }
        check_slot_limits(
            solutions,
            slots,
            world_slots,
            player_item_count,
            player_spirit_light,
        );
    }

    fn check_solutions(
        &self,
        solutions: &[Inventory],
        player: &Player,
        states: &FxHashSet<usize>,
        orb_variants: OrbVariants,
    ) {
        for solution in solutions {
            let mut solution_player = player.clone();
            let solution_extra_orbs = solution.max_orbs(player.settings.difficulty);
            solution_player.inventory.merge(solution.clone());
            let solution_orb_variants = orb_variants
                .iter()
                .map(|orbs| *orbs + solution_extra_orbs)
                .collect();
            let is_met = !self
                .is_met(&solution_player, states, solution_orb_variants)
                .is_empty();

            assert!(is_met, "Solution doesn't solve requirement!\nRequirement: {self:?}\nSolution: {}\nPlayer inventory: {}\nOrb Variants: {orb_variants:?}", solution, player.inventory);
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
    inventory: Inventory,
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
#[repr(usize)]
#[derive(Clone, Copy)]
enum Tag {
    UseResilience,
    UseOvercharge,
    UseLifePact,
    UseRegenerate,
}
impl Tag {
    fn item(self) -> Item {
        match self {
            Tag::UseResilience => Item::Shard(Shard::Resilience),
            Tag::UseOvercharge => Item::Shard(Shard::Overcharge),
            Tag::UseLifePact => Item::Shard(Shard::LifePact),
            Tag::UseRegenerate => Item::Skill(Skill::Regenerate),
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
            old.inventory.grant(tag.item(), 1);
            old.tags[tag as usize] = Some(true);
            new.tags[tag as usize] = Some(false);
            total_solutions.push(new);
            true
        })
    });
    not_using.extend(len..total_solutions.len()); // These are the ones we actually duplicated
    (using, not_using)
}

fn call_for_alternatives<T, F>(
    total_solutions: &mut Vec<TaggedSolution>,
    solutions: Vec<usize>,
    player: &Player,
    tag: Tag,
    value_when_using: T,
    value_when_not_using: T,
    f: F,
) where
    F: Fn(&mut Vec<TaggedSolution>, Vec<usize>, T),
{
    if player.settings.difficulty >= tag.difficulty() {
        if player.inventory.has_any(&tag.item()) {
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

fn require(solutions: &mut [TaggedSolution], item: Item) {
    solutions
        .iter_mut()
        .for_each(|solution| solution.inventory.grant(item.clone(), 1));
}
fn require_missing(solutions: &mut [TaggedSolution], item: Item, amount: u32) {
    solutions.iter_mut().for_each(|solution| {
        let has = solution.inventory.get(&item);
        solution
            .inventory
            .grant(item.clone(), amount.saturating_sub(has));
    });
}
fn require_any_of<I: IntoIterator<Item = Item> + AsRef<[Item]>>(
    solutions: &mut Vec<TaggedSolution>,
    items: I,
    player: &Player,
) {
    if !items
        .as_ref()
        .iter()
        .any(|item| player.inventory.has_any(item))
    {
        alternate_solutions(solutions, items, |solutions, item| require(solutions, item));
    }
}

fn needed_for_health(
    total_solutions: &mut Vec<TaggedSolution>,
    solutions: Vec<usize>,
    mut amount: f32,
    player: &Player,
    consuming: bool,
) {
    let game_thinks_regenerate_cost = Skill::Regenerate.energy_cost();
    let mut regenerate_cost = game_thinks_regenerate_cost;
    if player.settings.difficulty < Difficulty::Unsafe {
        regenerate_cost *= 2.0;
    }

    if player.settings.hard {
        amount *= 2.0
    }

    call_for_alternatives(
        total_solutions,
        solutions,
        player,
        Tag::UseResilience,
        0.9,
        1.0,
        |total_solutions, solutions, defense_mod| {
            let amount = amount * defense_mod;

            let health_fragments_solution = |mut orbs: Orbs| {
                let missing_health = amount - orbs.health;
                let mut missing_health_fragments = (missing_health * 0.2).ceil().max(0.0);
                if missing_health - missing_health_fragments * 5.0 >= 0.0 {
                    missing_health_fragments += 1.0
                }
                orbs.health += missing_health_fragments * 5.0; // granting fragments increases max health as well
                if consuming {
                    orbs.health -= amount
                }
                (missing_health_fragments as u32, orbs)
            };

            call_for_alternatives(
                total_solutions,
                solutions,
                player,
                Tag::UseOvercharge,
                0.5,
                1.0,
                |total_solutions, solutions, cost_mod| {
                    // We are accepting some redundancies here if the player doesn't actually need to heal
                    let regenerate_cost = regenerate_cost * cost_mod;

                    for index in solutions {
                        let solution = &total_solutions[index];
                        let max_health = solution.inventory.max_health(player.settings.difficulty);

                        let (missing_health_fragments, orbs) =
                            health_fragments_solution(solution.orbs);

                        let higher_cost = regenerate_cost.max(game_thinks_regenerate_cost);
                        // If this solution used Life Pact before, that implies it used up all of its energy and Regenerate prohibits using health to pay for it
                        // If we tried to grant extra energy fragments to pay for the Regenerate, that extra energy would just get eaten up by the earlier cost that Life Pact payed for
                        // We can assume that some other solution contains a variant where enough energy was granted not to need Life Pact, we'll just put the Regenerate solutions into that other solution
                        let can_regen = solution.orbs.energy >= higher_cost
                            || solution.health_payed_for_life_pact == 0.0;
                        if can_regen {
                            // We don't use the tag here because in this context Regenerate never has a strict advantage, we'd never to cover all cases anyway
                            let max_regens =
                                ((max_health - solution.orbs.health) * (1.0 / 30.0)).ceil() as u32;
                            if max_regens > 0 {
                                let mut total_missing_energy_fragments = 0;
                                for _ in 1..=max_regens {
                                    let solution = &mut total_solutions[index];

                                    let missing_energy = higher_cost - solution.orbs.energy;
                                    let missing_energy_fragments =
                                        (missing_energy * 2.0).ceil().max(0.0);
                                    total_missing_energy_fragments +=
                                        missing_energy_fragments as u32;
                                    solution.orbs.energy +=
                                        missing_energy_fragments * 0.5 - regenerate_cost; // granting fragments increases max energy as well
                                    solution.orbs.health =
                                        (solution.orbs.health + 30.0).min(max_health);

                                    let (missing_health_fragments, orbs) =
                                        health_fragments_solution(solution.orbs);
                                    let mut solution = TaggedSolution {
                                        orbs,
                                        ..solution.clone()
                                    };
                                    solution.inventory.grant(Item::Skill(Skill::Regenerate), 1);
                                    solution.tags[Tag::UseRegenerate as usize] = Some(true);
                                    solution.inventory.grant(
                                        Item::Resource(Resource::HealthFragment),
                                        missing_health_fragments,
                                    );
                                    solution.inventory.grant(
                                        Item::Resource(Resource::EnergyFragment),
                                        total_missing_energy_fragments,
                                    ); // the regen branch cannot be entered if we previously used Life Pact (this would not make sense), so we don't need to check health_payed_for_life_pact, it will be zero
                                    debug_assert_eq!(solution.health_payed_for_life_pact, 0.0);
                                    total_solutions.push(solution);
                                }
                            }
                        }

                        // Reuse the current slot for the solution without Regenerate
                        let solution = &mut total_solutions[index];
                        solution.inventory.grant(
                            Item::Resource(Resource::HealthFragment),
                            missing_health_fragments,
                        );
                        solution.orbs = orbs;
                    }
                },
            );
        },
    );
}

fn needed_for_energy(
    total_solutions: &mut Vec<TaggedSolution>,
    solutions: Vec<usize>,
    mut cost: f32,
    player: &Player,
    consuming: bool,
) {
    if player.settings.difficulty < Difficulty::Unsafe {
        cost *= 2.0
    }

    call_for_alternatives(
        total_solutions,
        solutions,
        player,
        Tag::UseOvercharge,
        0.5,
        1.0,
        |total_solutions, solutions, cost_mod| {
            let cost = cost * cost_mod;

            call_for_alternatives(
                total_solutions,
                solutions,
                player,
                Tag::UseLifePact,
                true,
                false,
                |total_solutions, solutions, use_life_pact| {
                    // We are accepting some redundancies here if the player doesn't actually need Life Pact
                    if use_life_pact {
                        let difficulty_mod = if player.settings.hard { 2.0 } else { 1.0 };
                        call_for_alternatives(
                            total_solutions,
                            solutions,
                            player,
                            Tag::UseResilience,
                            0.9 * difficulty_mod,
                            difficulty_mod,
                            |total_solutions, mut solutions, defense_mod| {
                                let f = |total_solutions: &mut Vec<TaggedSolution>,
                                         solutions: Vec<usize>,
                                         use_regenerate| {
                                    for index in solutions {
                                        let mut solution = &mut total_solutions[index];
                                        let mut new_index = None; // If we need to duplicate the solution, we will store the index here
                                                                  // This flag includes the information that we are allowed and may want to heal
                                        if use_regenerate {
                                            debug_assert_eq!(
                                                solution.health_payed_for_life_pact,
                                                0.0
                                            );
                                            let max_health = solution
                                                .inventory
                                                .max_health(player.settings.difficulty);
                                            let max_heal = max_health - solution.orbs.health; // This is only correct because this solution didn't use Life Pact yet. Otherwise, we would have to grant energy fragments until we undid all the health payed for life pact

                                            let game_thinks_regenerate_cost =
                                                Skill::Regenerate.energy_cost();
                                            let mut regenerate_cost =
                                                game_thinks_regenerate_cost * cost_mod;
                                            if player.settings.difficulty < Difficulty::Unsafe {
                                                regenerate_cost *= 2.0;
                                            }
                                            let higher_cost =
                                                regenerate_cost.max(game_thinks_regenerate_cost);

                                            debug_assert!(max_heal > regenerate_cost * 10.0);
                                            // Life Pact cannot use Health to pay for Regenerate, so before trying to regenerate we need to grant sufficient energy so we
                                            // This is a bit absurd if we already payed a lot of health for life pact...
                                            // It sounds very unlikely for regenerate to be worth it if we have to grant 10 energy fragments
                                            // just to be able to regenerate at all because we used life pact so much
                                            // But right now I can't think of a proof that regenerates are guaranteed to be redundant in any case here, so...
                                            let grant_energy = |solution: &mut TaggedSolution, total_cost: f32, game_thinks_total_cost: f32| {
                                    let higher_cost = total_cost.max(game_thinks_total_cost);
                                    let missing_energy = higher_cost - solution.orbs.energy;
                                    let missing_energy_fragments = (missing_energy * 2.0).ceil().max(0.0);
                                    solution.orbs.energy += missing_energy_fragments * 0.5 - total_cost;  // granting fragments increases max energy as well
                                    solution.inventory.grant(Item::Resource(Resource::EnergyFragment), missing_energy_fragments as u32);
                                };

                                            let regens = max_heal / 30.0;
                                            let max_optimal_regens = regens.floor();

                                            let total_cost = regenerate_cost * max_optimal_regens;
                                            let game_thinks_total_cost =
                                                total_cost - regenerate_cost + higher_cost; // On the final regenerate we have to make sure the the game is happy with our amount of resources
                                            grant_energy(
                                                solution,
                                                total_cost,
                                                game_thinks_total_cost,
                                            );
                                            solution.orbs.health += 30.0 * max_optimal_regens; // This cannot exceed max health because we are doing optimal heals

                                            let max_heal = max_health - solution.orbs.health;
                                            if max_heal > regenerate_cost * 10.0 {
                                                // oh no we found a suboptimal heal that we may want to do
                                                let old_solution = solution.clone();
                                                debug_assert!(
                                                    solution.orbs.health > max_health - 30.0
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
                                                .max_energy(player.settings.difficulty);
                                            let mut missing_energy = cost - solution.orbs.energy;
                                            let mut health_cost = missing_energy * 10.0;

                                            let health_fragment_solution = |solution: &mut TaggedSolution, health_cost: f32| {
                                    let true_health_cost = health_cost * defense_mod;
                                    let higher_amount = health_cost.max(true_health_cost);
                                    let missing_health = higher_amount - solution.orbs.health;
                                    let mut missing_health_fragments = (missing_health * 0.2).ceil().max(0.0);
                                    if missing_health - missing_health_fragments * 5.0 >= 0.0 { missing_health_fragments += 1.0 }
                                    solution.orbs.health += missing_health_fragments * 5.0;  // granting fragments increases max health as well
                                    solution.orbs.health -= true_health_cost;
                                    solution.health_payed_for_life_pact += true_health_cost;
                                    if consuming { solution.orbs.energy = 0.0; }
                                    else { solution.orbs.energy = (solution.orbs.energy + true_health_cost).min(max_energy) }

                                    solution.inventory.grant(Item::Resource(Resource::HealthFragment), missing_health_fragments as u32);
                                };

                                            if solution.do_not_generate_all_life_pact_variants {
                                                // This flag indicates we only need to generate the health fragment solution
                                                // In greater detail, it indicates a previous requirement already generated a full set of health/energy fragment combinations
                                                // Since the branch for when this flag is false grants one energy at a time, it is guaranteed that on the
                                                // previous requirement, an alternate solution with one more energy fragment has already been generated
                                                // This is true all the way through the solution chain up to the solution with the most energy fragments,
                                                // which will not have this tag and will be the only one that generates all fragment combinations again
                                                // This approach greatly reduces redundancies on chained energy requirements
                                                health_fragment_solution(solution, health_cost);
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
                                                solution.inventory.grant(
                                                    Item::Resource(Resource::EnergyFragment),
                                                    1,
                                                );
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
                                let regen_is_worth_it = !(player.settings.hard
                                    && player.settings.difficulty < Difficulty::Unsafe);
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
                                                .max_health(player.settings.difficulty);
                                            let max_heal = max_health
                                                - solution.orbs.health
                                                - solution.health_payed_for_life_pact; // In order to regenerate, we will have to grant energy fragments until we undid all the health payed for life pact

                                            let mut regenerate_cost =
                                                Skill::Regenerate.energy_cost() * cost_mod;
                                            if player.settings.difficulty < Difficulty::Unsafe {
                                                regenerate_cost *= 2.0;
                                            }

                                            max_heal > regenerate_cost * 10.0
                                        });
                                    let never_regen = solutions.split_off(partition_index);
                                    call_for_alternatives(
                                        total_solutions,
                                        solutions,
                                        player,
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
                            let missing_energy_fragments = (missing_energy * 2.0).ceil().max(0.0);
                            solution.orbs.energy += missing_energy_fragments * 0.5; // granting fragments increases max energy as well
                            if consuming {
                                solution.orbs.energy -= cost;
                            }
                            solution.inventory.grant(
                                Item::Resource(Resource::EnergyFragment),
                                missing_energy_fragments as u32,
                            ); // this branch cannot be entered if we previously used Life Pact, so we don't need to check health_payed_for_life_pact, it will be zero
                            debug_assert_eq!(solution.health_payed_for_life_pact, 0.0);
                        }
                    }
                },
            );
        },
    );
}

fn needed_for_energy_skill(
    solutions: &mut Vec<TaggedSolution>,
    player: &Player,
    skill: Skill,
    amount: f32,
    consuming: bool,
) {
    require(solutions, Item::Skill(skill));
    needed_for_energy(
        solutions,
        all(solutions),
        skill.energy_cost() * amount,
        player,
        consuming,
    ); // TODO because of the game miscalculating use costs, it's slightly incorrect to just multiply the cost here
}
fn needed_to_destroy<const TARGET_IS_WALL: bool>(
    solutions: &mut Vec<TaggedSolution>,
    player: &Player,
    health: f32,
) {
    alternate_solutions(
        solutions,
        player.progression_weapons::<TARGET_IS_WALL>(),
        |solutions, weapon| needed_to_destroy_with(solutions, weapon, 1.0, player, health),
    );
}
fn needed_to_destroy_with(
    solutions: &mut Vec<TaggedSolution>,
    weapon: Skill,
    cost_mod: f32,
    player: &Player,
    health: f32,
) {
    let cost = destroy_cost(weapon, player, health, false) * cost_mod;
    require(solutions, Item::Skill(weapon));
    needed_for_energy(solutions, all(solutions), cost, player, true);
}
fn destroy_cost(weapon: Skill, player: &Player, health: f32, flying_target: bool) -> f32 {
    // TODO damage buff progressions
    let damage = weapon.damage(player.settings)
        * player.damage_mod(flying_target, matches!(weapon, Skill::Bow))
        + weapon.burn_damage(); // We are only allowed to add the player's damage mod here because we're not factoring damage upgrades into the progressions yet
    weapon.energy_cost() * (health / damage).ceil()
}

fn needed_for_combat(
    solutions: &mut Vec<TaggedSolution>,
    player: &Player,
    enemies: &[(Enemy, u8)],
) {
    let mut aerial = false;
    let mut dangerous = false;
    let mut ranged = false;
    let mut melee = player.settings.difficulty >= Difficulty::Unsafe;
    let mut shielded = false;
    let mut bash = false;
    let mut burrow = false;

    for (enemy, _) in enemies {
        shielded |= enemy.shielded();
        burrow |= matches!(enemy, Enemy::Sandworm);
        if player.settings.difficulty < Difficulty::Unsafe {
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
        player.progression_weapons::<false>() // TODO considering the solutions inventory isn't trivial because currently we perform this operation once for all solutions, but it might be worth trying anyway
    } else {
        smallvec![Skill::Sword]
    };
    let ranged_weapons = if ranged {
        player.ranged_progression_weapons()
    } else {
        smallvec![Skill::Spear]
    };
    let shield_weapons = if shielded {
        player.shield_progression_weapons()
    } else {
        smallvec![Skill::Spear]
    };
    let use_burrow: SmallVec<[_; 2]> = if burrow {
        if player.settings.difficulty < Difficulty::Unsafe
            || player.inventory.has_any(&Item::Skill(Skill::Burrow))
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
                        needed_for_energy(solutions, all(solutions), cost, player, true);
                        solutions.iter_mut().for_each(|solution| {
                            solution.inventory.recharge(
                                &mut solution.orbs,
                                amount,
                                player.settings.difficulty,
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
                } else if enemy.armored() && player.settings.difficulty < Difficulty::Unsafe {
                    health *= 2.0
                }; // No enemy is shielded and armored

                let used_weapon =
                    if enemy.ranged() && player.settings.difficulty < Difficulty::Unsafe {
                        ranged_weapon
                    } else {
                        weapon
                    };

                cost += destroy_cost(used_weapon, player, health, enemy.flying()) * amount;
            }

            needed_for_energy(solutions, all(solutions), cost, player, true);
            if melee {
                require(solutions, Item::Skill(weapon));
            }
            if ranged {
                require(solutions, Item::Skill(ranged_weapon));
            }
            if shielded {
                require(solutions, Item::Skill(shield_weapon));
            }
            if burrow {
                require(solutions, Item::Skill(Skill::Burrow));
            }
        },
    );

    if player.settings.difficulty < Difficulty::Unsafe {
        if bash {
            require(solutions, Item::Skill(Skill::Bash));
        }
        if aerial && (!bash || player.settings.difficulty < Difficulty::Gorlek) {
            let mut ranged_skills =
                vec![Item::Skill(Skill::DoubleJump), Item::Skill(Skill::Launch)];
            if player.settings.difficulty >= Difficulty::Gorlek {
                ranged_skills.push(Item::Skill(Skill::Bash));
            }
            require_any_of(solutions, ranged_skills, player);
        } else if dangerous && !bash {
            let evasion_skills = [
                Item::Skill(Skill::DoubleJump),
                Item::Skill(Skill::Dash),
                Item::Skill(Skill::Bash),
                Item::Skill(Skill::Launch),
            ];
            require_any_of(solutions, evasion_skills, player);
        }
    }
}

// We don't use Inventory::item_count here because we need to be accurate about the Spirit Light and item_count does rounding
fn item_count_and_spirit_light(inventory: &Inventory) -> (u32, u32) {
    let mut spirit_light = 0;
    let item_count = inventory
        .items
        .iter()
        .filter_map(|(item, amount)| match item {
            Item::SpiritLight(stacked_amount) => {
                spirit_light += *amount * *stacked_amount;
                None
            }
            _ => Some(*amount),
        })
        .sum::<u32>();
    (item_count, spirit_light)
}
fn check_slot_limits(
    solutions: &mut Vec<TaggedSolution>,
    slots: u32,
    world_slots: u32,
    player_item_count: u32,
    player_spirit_light: u32,
) {
    solutions.retain(|solution| {
        let (item_count, spirit_light) = item_count_and_spirit_light(&solution.inventory);
        let spirit_light_slots = (spirit_light - player_spirit_light + 39) / 40;
        item_count - player_item_count + spirit_light_slots <= slots
            && spirit_light_slots <= world_slots // Spirit light is the only world item currently
    });
}

pub(crate) fn filter_redundancies(solutions: &mut Vec<Inventory>) {
    // log::trace!("unfiltered: {}", solutions.len());
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
    // log::trace!("filtered: {}", solutions.len());
}
