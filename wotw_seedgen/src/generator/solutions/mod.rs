//! # Progression notes
//!
//! ## Abandoned progressions
//!
//! In v4, progression will always get you to a new anchor, even if it required solving multiple requirements.
//! Currently, a progression only solves a single unsolved requirement. This may unlock nothing and the
//! next progression may continue elsewhere, which is a major reason the seed quality has become worse.
//! However, the same quality problem has already existed before in cases where reaching a new anchor
//! does not unlock any new locations.
//!
//! One idea could be to "lock on" a certain connection and ensure consecutive progressions continue
//! on this same and even sequential connections until new locations are reached. BTW rename pickup -> location?
//! Implementing this would mean we can keep looking at single requirements as progressions and
//! simultaneously improve seed quality in the case of anchor progressions with no locations attached.
//!
//! ## Shards
//!
//! Then, there's the whole complexity around shards. Something like Life Pact has yet to be represented.
//! Maybe we can reasonably use a similar idea of splitting into smaller progressions. Life Pact could have
//! a weighted chance to be picked as orb progression, and with Life Pact available health modifying items
//! could start to be picked from the item pool. We just have to make sure that we remain commited to picking
//! items from the pool relevant to this progression until we get somewhere. A general problem will be that
//! we can't necessarily predict whether the item pool even supports our planned form of progression,
//! so we might need some rollback, but we already have snapshots for that. We're losing the neat lineup
//! of progressions we can reason about upfront, but that probably doesn't hurt seed quality?
//!
//! ## Launch Fragments
//!
//! Finally, there's Launch Fragments. v4 didn't really have a solution for this, but it did flush UberState
//! items from the pool in case of a progression dead end. We can't really do that anymore because now all
//! logically relevant items are "UberState items". Maybe we could flush all items setting custom states.
//! But ideally seedgen would figure out the relation of launch fragments to launch and then present
//! launch fragments as an option when trying to obtain launch as progress. That's a notable shift in
//! how seeds are shaped though, since in v4 launch fragments never being forced progression contributes
//! to their function as almost entirely removing launch for players doing starved routing while
//! keeping it for collection happy players.

// TODO Adding the specific requirement to `ConnectionIndex` does remove a lot of dead paths, but it also creates a lot more
// unique ConnectionIndexes that don't get swallowed in the hashsets. We might need to figure out some other things before
// being able to really look into it.
// Maybe storing the missing instance would be more fruitful?

#[cfg(test)]
mod tests;
mod weight;

pub(crate) use weight::Cost;

use log::trace;
use smallvec::SmallVec;

use std::{
    cmp::Ordering,
    collections::hash_map::Entry,
    env,
    fmt::{self, Display},
    iter, mem,
    ops::ControlFlow,
    sync::LazyLock,
};

use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use wotw_seedgen_data::{
    logic_language::output::{Connection, Graph},
    seed_language::{
        output::{AsConstant, CommandVoid, ContainedWrites, Event},
        simulate::{Simulate, Simulation, Snapshot},
    },
    Skill, UberIdentifier,
};

use crate::{
    item_pool::ItemPool,
    logical_difficulty::LogicalDifficulty,
    world::{ConnectionIndex, ConnectionOrRefill, Missing, ReachStateFails},
    World,
};

/// Maximum radius of connections to solve before aborting a solution.
///
/// Lower values reduce search time but also reduce progression quality.
/// We're currently forced to default to 0 which is pretty bad but also how v4 behaved.
///
/// Average `cargo bench solutions` times as of 2026-03-21:
/// - `MAX_SEARCH_RADIUS=0`: 2.131 ms
/// - `MAX_SEARCH_RADIUS=1`: 4.177 ms
/// - `MAX_SEARCH_RADIUS=2`: 7.052 ms
/// - `MAX_SEARCH_RADIUS=u8::MAX`: 7.097 ms
///
/// Average `cargo bench generation/gorlek rspawn` times:
/// - `MAX_SEARCH_RADIUS=0`: 279.99 ms
/// - `MAX_SEARCH_RADIUS=1`: Too long to measure :orithump:
static MAX_SEARCH_RADIUS: LazyLock<u8> = LazyLock::new(|| {
    env::var("SOLUTION_MAX_SEARCH_RADIUS")
        .ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0)
});

pub type SolutionItems = SmallVec<[usize; 8]>;

#[derive(Debug, Clone)]
pub struct Solution {
    pub items: SolutionItems,
    pub spirit_light: i32,
    pub new_reached: usize,
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        // TODO can this be applied elsewhere?
        // spirit_light can be ignored because finished solutions are non-redundant
        // and solutions with the same items, but different spirit lights would be redundant.
        // new_reached can be ignored because it depends on the other two values.
        let eq = self.items == other.items;

        if cfg!(debug_assertions) {
            assert!(
                eq == (eq
                    && (self.spirit_light == other.spirit_light)
                    && (self.new_reached == other.new_reached))
            );
        }

        eq
    }
}

impl Eq for Solution {}

impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        // see notes on PartialEq impl
        let ord = self.items.cmp(&other.items);

        if cfg!(debug_assertions) {
            assert!(
                ord == ord
                    .then(self.spirit_light.cmp(&other.spirit_light))
                    .then(self.new_reached.cmp(&other.new_reached))
            )
        }

        ord
    }
}

impl Solution {
    fn new(solution: PartialSolution, new_reached: usize) -> Self {
        Self {
            items: solution.used_items,
            spirit_light: solution.spirit_light,
            new_reached,
        }
    }
}

pub trait SolutionLike<'graph> {
    fn items(&self) -> &SolutionItems;

    fn spirit_light(&self) -> i32;

    fn connection(&self) -> Option<&ConnectionIndex<'graph>>;

    fn used_slots(&self) -> usize {
        self.used_spirit_light_slots() + self.items().len()
    }

    fn used_spirit_light_slots(&self) -> usize {
        // Conservative guess
        (self.spirit_light() + 49) as usize / 50
    }

    fn is_empty(&self) -> bool {
        self.items().is_empty() && self.spirit_light() == 0
    }

    fn is_redundant_with<'o, S: SolutionLike<'o>>(&self, other: &S) -> bool {
        other.items().iter().all(|item| self.items().contains(item))
            && other.spirit_light() <= self.spirit_light()
    }

    fn display<'pool, 'solution>(
        &'solution self,
        item_pool: &'pool ItemPool,
        graph: Option<&'graph Graph>,
    ) -> DisplaySolution<'graph, 'pool, 'solution> {
        DisplaySolution::new(self, item_pool, graph)
    }
}

impl<'graph> SolutionLike<'graph> for Solution {
    fn items(&self) -> &SolutionItems {
        &self.items
    }

    fn spirit_light(&self) -> i32 {
        self.spirit_light
    }

    fn connection(&self) -> Option<&ConnectionIndex<'graph>> {
        None
    }
}

pub struct DisplaySolution<'graph, 'pool, 'solution> {
    connection: Option<(&'solution ConnectionIndex<'graph>, &'graph Graph)>,
    items: &'solution SolutionItems,
    spirit_light: i32,
    item_pool: &'pool ItemPool,
}

impl<'graph, 'pool, 'solution> DisplaySolution<'graph, 'pool, 'solution> {
    fn new<S: SolutionLike<'graph> + ?Sized>(
        solution: &'solution S,
        item_pool: &'pool ItemPool,
        graph: Option<&'graph Graph>,
    ) -> Self {
        Self {
            connection: solution.connection().zip(graph),
            items: solution.items(),
            spirit_light: solution.spirit_light(),
            item_pool,
        }
    }
}

impl Display for DisplaySolution<'_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((connection, graph)) = &self.connection {
            write!(f, "{} / ", connection.display(graph),)?;
        }

        if self.spirit_light > 0 {
            write!(f, "{} Spirit Light", self.spirit_light)?;

            if !self.items.is_empty() {
                ", ".fmt(f)?;
            }
        } else if self.items.is_empty() {
            return "empty inventory".fmt(f);
        }

        let mut item_counts = FxHashMap::<&CommandVoid, u32>::with_capacity_and_hasher(
            self.items.len(),
            FxBuildHasher,
        );

        for item in self.items {
            *item_counts.entry(&self.item_pool[*item]).or_insert(0) += 1;
        }

        item_counts
            .into_iter()
            .format_with(", ", |(item, count), f| {
                if count > 1 {
                    f(&format_args!("{count} "))?;
                }

                f(&item.log_display())
            })
            .fmt(f)
    }
}

impl<'graph> World<'graph, '_> {
    pub fn find_solutions(
        &mut self,
        item_pool: &ItemPool,
        events: &[Event],
        slots: usize,
        spirit_light_slots: usize,
        search_radius: Option<u8>,
    ) -> Vec<Solution> {
        let fails = self.fails();
        let initial_solutions = fails
            .uber_state
            .values()
            .flatten()
            .chain(&fails.health)
            .chain(&fails.energy)
            .map(|fail| PartialSolution::new(fail.clone(), item_pool, search_radius))
            .collect::<Vec<_>>();

        let mut context = SolutionContext::new(self, events, item_pool, slots, spirit_light_slots);

        // First going through all untouched solutions is not always, but generally faster.
        // I think it's because solving untouched solutions is cheaper so it's a big win
        // if an untouched solutions eliminates redundant touched solutions that will
        // never have to be resumed, which is relatively expensive.
        for solution in initial_solutions {
            context.solve_untouched(solution);
        }

        while let Some(solution) = context.solutions.pop() {
            context.solve_touched(solution);
        }

        context.finish()
    }

    fn simulate_solution<S: SolutionLike<'graph>>(
        &mut self,
        solution: &S,
        item_pool: &ItemPool,
        events: &[Event],
    ) {
        for item in solution.items() {
            item_pool[*item].simulate(self, events);
        }

        let spirit_light = solution.spirit_light();
        if spirit_light > 0 {
            self.add_spirit_light(spirit_light, events);
        }
    }
}

#[derive(Debug, Clone)]
struct PartialSolution<'graph> {
    /// The current connection this solution wants to solve
    connection: ConnectionIndex<'graph>,
    /// Indices into [`ItemPool`] for items used so far
    used_items: SolutionItems,
    /// Indices into [`ItemPool`] for items not used so far, needs to be kept in sync with `used_items`
    remaining_items: FxHashSet<usize>,
    /// Amount of spirit light used so far
    spirit_light: i32,
    // /// Clones of other branches that are solving the same connection,
    // /// can be used to filter redundancies even between unfinished solutions.
    // other_branches: Vec<MinimalSolution>,
    /// Connections that have already been branched into from a common ancestor,
    /// can be used to avoid entering redundant search paths.
    new_fails: ReachStateFails<'graph>,
    /// Commitments made on the assumption that other branches have commited to other possibilities
    /// and do not need to be entered again
    commitments: Commitments,
    /// Remaining radius of connections to solve before aborting
    search_radius: u8,
}

impl<'graph> PartialSolution<'graph> {
    fn new(
        connection: ConnectionIndex<'graph>,
        item_pool: &ItemPool,
        search_radius: Option<u8>,
    ) -> Self {
        Self {
            connection,
            used_items: SmallVec::new(),
            remaining_items: (0..item_pool.len()).collect(),
            spirit_light: 0,
            // other_branches: vec![],
            new_fails: ReachStateFails::default(),
            commitments: Commitments::default(),
            search_radius: search_radius.unwrap_or(*MAX_SEARCH_RADIUS),
        }
    }
}

impl<'graph> SolutionLike<'graph> for PartialSolution<'graph> {
    fn items(&self) -> &SolutionItems {
        &self.used_items
    }

    fn spirit_light(&self) -> i32 {
        self.spirit_light
    }

    fn connection(&self) -> Option<&ConnectionIndex<'graph>> {
        Some(&self.connection)
    }
}

// TODO bitflags?
#[derive(Debug, Clone, Default)]
struct Commitments {
    better_weapon: bool,
    damage_regenerate: bool,
    skip_regenerate_energy: bool,
}

// #[derive(Debug, Clone)]
// struct MinimalSolution {
//     items: SolutionItems,
//     spirit_light: i32,
// }

// impl SolutionLike for MinimalSolution {
//     fn items(&self) -> &SolutionItems {
//         &self.items
//     }

//     fn spirit_light(&self) -> i32 {
//         self.spirit_light
//     }

//     fn connection(&self) -> Option<ConnectionIndex> {
//         None
//     }
// }

// impl MinimalSolution {
//     fn new(solution: &PartialSolution) -> Self {
//         Self {
//             items: solution.used_items.clone(),
//             spirit_light: solution.spirit_light,
//         }
//     }
// }

struct SolutionContext<'world, 'graph, 'settings, 'events, 'pool> {
    world: &'world mut World<'graph, 'settings>,
    events: &'events [Event],
    item_pool: &'pool ItemPool,
    slots: usize,
    spirit_light_slots: usize,
    initial_pickup_count: usize,
    initial_fails: ReachStateFails<'graph>,
    solutions: Vec<PartialSolution<'graph>>,
    finished: Vec<Solution>,
    aborted: Vec<Solution>,
}

impl<'world, 'graph, 'settings, 'events, 'pool>
    SolutionContext<'world, 'graph, 'settings, 'events, 'pool>
{
    fn new(
        world: &'world mut World<'graph, 'settings>,
        events: &'events [Event],
        item_pool: &'pool ItemPool,
        slots: usize,
        spirit_light_slots: usize,
    ) -> Self {
        // TODO investigate perf impact, seems to be positive in moki and negative in gorlek - we do have the settings...
        // world.clean_fails();

        let initial_pickup_count = world.reached_pickup_count();
        let initial_fails = ReachStateFails {
            logical_state: FxHashMap::default(),
            ..world.fails().clone()
        };

        Self {
            world,
            events,
            item_pool,
            slots,
            spirit_light_slots,
            initial_pickup_count,
            initial_fails,
            solutions: vec![],
            finished: vec![],
            aborted: vec![],
        }
    }

    fn solve_untouched(&mut self, solution: PartialSolution<'graph>) {
        trace!("starting solve for {}", self.display_solution(&solution));

        self.world.snapshot();

        self.solve_until_progress(solution);

        self.world.restore_snapshot();
    }

    fn solve_touched(&mut self, solution: PartialSolution<'graph>) {
        trace!("resuming solve for {}", self.display_solution(&solution));

        self.world.snapshot();

        self.world
            .simulate_solution(&solution, self.item_pool, self.events);

        trace!(
            "resuming with {} and {}",
            self.world.inventory_display(),
            self.world
                .reached_nodes()
                .map(|node| node.identifier())
                .format(", ")
        );

        self.solve_until_progress(solution);

        self.world.restore_snapshot();
    }

    fn solve_until_progress(&mut self, mut solution: PartialSolution<'graph>) {
        loop {
            let flow = match solution.connection.connection {
                ConnectionOrRefill::Refill(_) => self.solve_requirement(solution),
                ConnectionOrRefill::Connection(connection) => {
                    self.solve_connection(connection.0, solution)
                }
            };

            match flow {
                ControlFlow::Continue(next_solution) => solution = next_solution,
                ControlFlow::Break(()) => break,
            }
        }
    }

    fn solve_connection(
        &mut self,
        connection: &'graph Connection,
        solution: PartialSolution<'graph>,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        trace!(
            "solving connection {solution}",
            solution = self.display_solution(&solution)
        );

        if self.world.has_reached(connection.to) {
            trace!("already reached");

            self.check_solution(solution)
        } else {
            // TODO sometimes the requirement is already solved which sounds wrong
            // if the node isn't reached. Is there a logic error in the reach expansion?
            self.solve_requirement(solution)
        }
    }

    fn solve_requirement(
        &mut self,
        solution: PartialSolution<'graph>,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        trace!(
            "solving {solution}",
            solution = self.display_solution(&solution)
        );

        // let requirement = solution.connection.requirement.0;
        let requirement = solution.connection.connection.requirement();
        let mut orb_variants = self.world.get_connection_orbs(&solution.connection).clone();
        match self.world.is_met(requirement, &mut orb_variants) {
            ControlFlow::Continue(()) => {
                trace!("already met");

                self.check_solution(solution)
            }
            ControlFlow::Break(missing) => self.solve(solution, missing, true),
        }
    }

    fn check_solution(
        &mut self,
        solution: PartialSolution<'graph>,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if solution.is_empty() {
            // stale progression, nothing to see here
            return ControlFlow::Break(());
        }

        let pickup_count = self.world.reached_pickup_count();
        if pickup_count > self.initial_pickup_count {
            self.finish_solution(solution, pickup_count - self.initial_pickup_count);

            ControlFlow::Break(())
        } else {
            self.continue_solution(solution)
        }
    }

    fn finish_solution(&mut self, solution: PartialSolution<'graph>, new_reached: usize) {
        let finished = Solution::new(solution, new_reached);

        trace!(
            "finished solution {finished}, {new_reached} reached",
            finished = self.display_solution(&finished),
        );

        self.solutions.retain(|solution| {
            !trace_is_redundant_with(solution, &finished, self.world.graph, self.item_pool)
        });

        self.aborted.retain(|other| {
            !trace_is_redundant_with(other, &finished, self.world.graph, self.item_pool)
        });

        self.finished.retain(|other| {
            !trace_is_redundant_with(other, &finished, self.world.graph, self.item_pool)
        });

        self.finished.push(finished);
    }

    fn abort_solution(&mut self, solution: PartialSolution<'graph>) {
        let aborted = Solution::new(solution, 0);

        trace!(
            "search limit reached, aborting solution {aborted}",
            aborted = self.display_solution(&aborted),
        );

        self.aborted.retain(|other| {
            !trace_is_redundant_with(other, &aborted, self.world.graph, self.item_pool)
        });

        self.aborted.push(aborted);
    }

    fn continue_solution(
        &mut self,
        mut solution: PartialSolution<'graph>,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if solution.search_radius == 0 {
            self.abort_solution(solution);
            return ControlFlow::Break(());
        }

        solution.search_radius -= 1;

        trace!("continuing solution {}", self.display_solution(&solution));

        // solution.other_branches.clear();

        let mut new_solutions =
            self.new_fails(&mut solution)
                .into_iter()
                .map(|connection| PartialSolution {
                    connection,
                    ..solution.clone()
                });

        let Some(next_solution) = new_solutions.next() else {
            // TODO this can happen for multiple reasons, for example when solving a state that does not immediately solve another connection, which is not ideal
            // It also happens when solving refills that don't immediately progress anything, which sounds unsolvable but maybe fine
            trace!("no progress, unable to continue solution");
            return ControlFlow::Break(());
        };

        for solution in new_solutions {
            self.pause_solution(solution);
        }

        ControlFlow::Continue(next_solution)
    }

    fn new_fails(
        &self,
        solution: &mut PartialSolution<'graph>,
    ) -> FxHashSet<ConnectionIndex<'graph>> {
        let mut new_fails = FxHashSet::default();

        let fails = self.world.fails();

        // TODO other fails?
        for (current_uber_identifier, current_connections) in &fails.uber_state {
            let current_connections_iter = current_connections.iter();

            let initial_filter = self
                .initial_fails
                .uber_state
                .get(current_uber_identifier)
                .map(|initial_connections| {
                    |connection: &&ConnectionIndex<'graph>| {
                        !initial_connections.contains(connection)
                    }
                });

            match solution
                .new_fails
                .uber_state
                .entry(*current_uber_identifier)
            {
                Entry::Occupied(mut occupied) => {
                    let solution_connections = occupied.get_mut();
                    let solution_filter = |connection: &&ConnectionIndex<'graph>| {
                        solution_connections.insert((*connection).clone())
                    };

                    match initial_filter {
                        None => new_fails
                            .extend(current_connections_iter.filter(solution_filter).cloned()),
                        Some(initial_filter) => new_fails.extend(
                            current_connections_iter
                                .filter(solution_filter)
                                .filter(initial_filter)
                                .cloned(),
                        ),
                    }
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(current_connections.clone());

                    match initial_filter {
                        None => new_fails.extend(current_connections_iter.cloned()),
                        Some(initial_filter) => new_fails
                            .extend(current_connections_iter.filter(initial_filter).cloned()),
                    }
                }
            }
        }

        new_fails.extend(
            fails
                .health
                .iter()
                .filter(|fail| {
                    !self.initial_fails.health.contains(fail)
                        && solution.new_fails.health.insert((*fail).clone())
                })
                .cloned(),
        );

        new_fails.extend(
            fails
                .energy
                .iter()
                .filter(|fail| {
                    !self.initial_fails.energy.contains(fail)
                        && solution.new_fails.energy.insert((*fail).clone())
                })
                .cloned(),
        );

        new_fails
    }

    fn pause_solution(&mut self, solution: PartialSolution<'graph>) {
        // TODO we shouldn't pause if the solution is already finished - but can we know if it's finished if it's not currently simulated?
        trace!("pausing solution {}", self.display_solution(&solution));

        self.solutions.push(solution);
    }

    fn solve(
        &mut self,
        solution: PartialSolution<'graph>,
        missing: Missing<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        trace!(
            "solving {missing} for {solution}",
            solution = self.display_solution(&solution)
        );

        match missing {
            Missing::Impossible => ControlFlow::Break(()),
            Missing::Boolean(uber_identifier) => {
                self.solve_boolean(solution, uber_identifier, simulate)
            }
            Missing::Integer(uber_identifier, amount) => {
                self.solve_integer(solution, uber_identifier, amount, simulate)
            }
            Missing::LogicalState(_) => ControlFlow::Break(()),
            Missing::Health => self.solve_health(solution, simulate),
            Missing::Energy => self.solve_energy(solution, simulate),
            Missing::WallWeapon => self.solve_weapon::<true>(solution, simulate),
            Missing::EnemyWeapon => self.solve_weapon::<false>(solution, simulate),
            Missing::EnergyOrBetterWallWeapon => {
                self.solve_energy_or_better_weapon::<true>(solution, simulate)
            }
            Missing::EnergyOrBetterEnemyWeapon => {
                self.solve_energy_or_better_weapon::<false>(solution, simulate)
            }
            Missing::Any(any) => self.solve_any(solution, any, simulate),
            Missing::Or(ors, _) => self.solve_any(solution, ors, simulate),
        }
    }

    fn solve_health(
        &mut self,
        mut solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if self.world.skill(Skill::Regenerate) {
            if solution.commitments.skip_regenerate_energy {
                self.solve_boolean_branches(solution, self.health_options(), simulate)
            } else {
                let mut health_solution = solution.clone();
                health_solution.commitments.skip_regenerate_energy = true;
                let health_flow =
                    self.solve_boolean_branches(health_solution, self.health_options(), simulate);

                let energy_flow = self.solve_boolean_branches(
                    solution,
                    self.energy_options(),
                    simulate && health_flow.is_break(),
                );

                match (health_flow, energy_flow) {
                    (
                        health_flow @ ControlFlow::Continue(_),
                        ControlFlow::Continue(energy_solution),
                    ) => {
                        self.pause_solution(energy_solution);
                        health_flow
                    }
                    (flow @ ControlFlow::Continue(_), ControlFlow::Break(()))
                    | (ControlFlow::Break(()), flow) => flow,
                }
            }
        // TODO adding these checks should scale better as complexity increases, but results were inconclusive for now
        } else if solution.commitments.damage_regenerate {
            self.solve_boolean_branches(solution, self.health_options(), simulate)
        } else {
            solution.commitments.damage_regenerate = true;

            self.solve_boolean_branches(
                solution,
                self.health_options().chain(Some(Skill::REGENERATE_ID)),
                simulate,
            )
        }
    }

    fn solve_energy(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_boolean_branches(solution, self.energy_options(), simulate)
    }

    fn health_options(&self) -> impl Iterator<Item = UberIdentifier> {
        iter::once(UberIdentifier::MAX_HEALTH)
        // .chain(self.resilience_option())
        // .chain(self.vitality_option())
    }

    // TODO maybe after some optimizations oriShy
    // fn resilience_option(&self) -> Option<UberIdentifier> {
    //     (self.world.settings.difficulty.resilience() && !self.world.shard(Shard::Resilience))
    //         .then_some(Shard::RESILIENCE_ID)
    // }

    // fn vitality_option(&self) -> Option<UberIdentifier> {
    //     (self.world.settings.difficulty.vitality() && !self.world.shard(Shard::Vitality))
    //         .then_some(Shard::VITALITY_ID)
    // }

    fn energy_options(&self) -> impl Iterator<Item = UberIdentifier> {
        iter::once(UberIdentifier::MAX_ENERGY)
        // .chain(self.energy_shard_option())
    }

    // fn energy_shard_option(&self) -> Option<UberIdentifier> {
    //     (self.world.settings.difficulty.energy_shard() && !self.world.shard(Shard::Energy))
    //         .then_some(Shard::ENERGY_ID)
    // }

    fn solve_weapon<const TARGET_IS_WALL: bool>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        debug_assert!(!solution.commitments.better_weapon);
        solution.commitments.better_weapon = true;

        let branches = self
            .world
            .settings
            .difficulty
            .weapons_iter::<TARGET_IS_WALL>()
            .map(Skill::uber_identifier);

        self.solve_boolean_branches(solution, branches, simulate)
    }

    fn solve_energy_or_better_weapon<const TARGET_IS_WALL: bool>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if mem::replace(&mut solution.commitments.better_weapon, true) {
            return self.solve_energy(solution, simulate);
        }

        let branches = self
            .world
            .better_weapons::<TARGET_IS_WALL>()
            .map(Skill::uber_identifier)
            .chain(self.energy_options())
            .collect::<Vec<_>>();

        self.solve_boolean_branches(solution, branches, simulate)
    }

    fn solve_boolean(
        &mut self,
        mut solution: PartialSolution<'graph>,
        uber_identifier: UberIdentifier,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.has_free_slot(&solution)?;

        let Some(index) = solution.remaining_items.iter().copied().find(|index| {
            // TODO only consider positive writes, this would break for instance when shuffling remove skills into the item pool
            self.item_pool[*index]
                .contained_write_identifiers()
                .contains(&uber_identifier)
        }) else {
            // TODO can we remember pointless paths that we know end in this branch?
            trace!("no items in the pool to solve");

            return ControlFlow::Break(());
        };

        self.add_item(&mut solution, index, simulate)?;

        trace!(
            "progressed {uber_identifier} for {solution}",
            solution = self.display_solution(&solution),
        );

        // if self
        //     .world
        //     .settings
        //     .difficulty
        //     .may_increase_orbs(uber_identifier)
        // {
        //     solution.connection.orb_reset();
        // }

        ControlFlow::Continue(solution)
    }

    fn solve_integer(
        &mut self,
        mut solution: PartialSolution<'graph>,
        uber_identifier: UberIdentifier,
        mut amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if uber_identifier == UberIdentifier::SPIRIT_LIGHT {
            return self.solve_spirit_light(solution, amount, simulate);
        }

        self.has_free_slot(&solution)?;

        let mut items = vec![];
        let mut slots = self.slots - solution.used_slots();

        for index in solution.remaining_items.iter().copied() {
            let mut item_helps = false;

            for write in self.item_pool[index]
                // TODO positive writes etc.
                .contained_writes()
                .filter(|write| write.uber_identifier == uber_identifier)
            {
                item_helps = true;

                match write.command.try_as_integer().unwrap().as_constant() {
                    None => amount = 0,
                    Some(item_amount) => amount -= item_amount,
                }

                if amount <= 0 {
                    break;
                }
            }

            if item_helps {
                items.push(index);

                if amount <= 0 {
                    break;
                }

                if slots > 1 {
                    slots -= 1;
                } else {
                    trace!("not enough slots to solve");

                    return ControlFlow::Break(());
                }
            }
        }

        if items.is_empty() {
            trace!("no items in the pool to solve");

            return ControlFlow::Break(());
        }

        for index in items {
            self.add_item(&mut solution, index, simulate)?;
        }

        trace!(
            "progressed {uber_identifier} for {solution}",
            solution = self.display_solution(&solution),
        );

        ControlFlow::Continue(solution)
    }

    fn has_free_slot(&self, solution: &PartialSolution) -> ControlFlow<()> {
        if self.slots > solution.used_slots() {
            ControlFlow::Continue(())
        } else {
            // TODO can we remember pointless paths that we know end in this branch?
            trace!("not enough slots to solve");
            ControlFlow::Break(())
        }
    }

    fn solve_spirit_light(
        &mut self,
        mut solution: PartialSolution<'graph>,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.add_spirit_light(&mut solution, amount, simulate)?;

        if self.spirit_light_slots >= solution.used_spirit_light_slots() {
            ControlFlow::Continue(solution)
        } else {
            trace!("not enough spirit light slots to solve");

            ControlFlow::Break(())
        }
    }

    fn solve_any(
        &mut self,
        solution: PartialSolution<'graph>,
        any: Vec<Missing<'graph>>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        // TODO make these unique earlier in the program logic?
        // here they have to be unique so identical branches don't eliminate eachother
        // although, if different missings resolve to the same item pool item, could an issue still arise?
        // TODO also optimize the graph more so we get fewer duplicates here to begin with
        fn unique_missing<'graph>(
            any: Vec<Missing<'graph>>,
            unique: &mut FxHashSet<Missing<'graph>>,
        ) {
            for missing in any {
                match missing {
                    Missing::Any(any) => unique_missing(any, unique),
                    single => {
                        unique.insert(single);
                    }
                }
            }
        }

        let mut unique = FxHashSet::with_capacity_and_hasher(any.len(), FxBuildHasher);
        unique_missing(any, &mut unique);

        fn solve_branch<'graph>(
            context: &mut SolutionContext<'_, 'graph, '_, '_, '_>,
            solution: PartialSolution<'graph>,
            missing: Missing<'graph>,
            simulate: bool,
        ) -> ControlFlow<(), PartialSolution<'graph>> {
            context.solve(solution.clone(), missing, simulate)
        }

        self.solve_branches(solution, unique, simulate, solve_branch)
    }

    // fn solve_or(
    //     &mut self,
    //     solution: PartialSolution<'graph>,
    //     // ors: Vec<(Missing<'graph>, GraphRef<'graph, Requirement>)>,
    //     ors: Vec<Missing<'graph>>,
    //     simulate: bool,
    // ) -> ControlFlow<(), PartialSolution<'graph>> {
    //     fn solve_branch<'graph>(
    //         context: &mut SolutionContext<'_, 'graph, '_, '_, '_>,
    //         solution: PartialSolution<'graph>,
    //         // (missing, _): (Missing<'graph>, GraphRef<'graph, Requirement>),
    //         missing: Missing<'graph>,
    //         simulate: bool,
    //     ) -> ControlFlow<(), PartialSolution<'graph>> {
    //         // TODO this does nothing right solution.connection.requirement = requirement;
    //         context.solve(solution.clone(), missing, simulate)
    //     }

    //     self.solve_branches(solution, ors, simulate, solve_branch)
    // }

    fn solve_branches<I, T, F>(
        &mut self,
        solution: PartialSolution<'graph>,
        branches: I,
        simulate: bool,
        mut solve: F,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        I: IntoIterator<Item = T>,
        F: FnMut(
            &mut SolutionContext<'_, 'graph, '_, '_, '_>,
            PartialSolution<'graph>,
            T,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
    {
        // let start = self.solutions.len();
        let mut branches = branches.into_iter();

        let Some(next_solution) = branches.find_map(|branch| {
            // if solve returns break, it hasn't simulated anything, so we can carry the simulation into the next one
            solve(self, solution.clone(), branch, simulate).continue_value()
        }) else {
            return ControlFlow::Break(());
        };

        for branch in branches {
            if let ControlFlow::Continue(solution) = solve(self, solution.clone(), branch, false) {
                self.pause_solution(solution);
            }
        }

        // let branch_range = start..self.solutions.len();

        // let mut other_branches = Vec::with_capacity(branch_range.len() + 1);
        // other_branches.extend(
        //     self.solutions[branch_range.clone()]
        //         .iter()
        //         .map(MinimalSolution::new),
        // );
        // other_branches.push(MinimalSolution::new(&next_solution));

        // for index in branch_range {
        //     let mut other_branches = other_branches.clone();
        //     other_branches.swap_remove(index - start);

        //     self.solutions[index].other_branches.extend(other_branches);
        // }
        // other_branches.pop();
        // next_solution.other_branches = other_branches;

        ControlFlow::Continue(next_solution)
    }

    fn solve_boolean_branches<I>(
        &mut self,
        solution: PartialSolution<'graph>,
        branches: I,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        I: IntoIterator<Item = UberIdentifier>,
    {
        fn solve_branch<'graph>(
            context: &mut SolutionContext<'_, 'graph, '_, '_, '_>,
            solution: PartialSolution<'graph>,
            uber_identifier: UberIdentifier,
            simulate: bool,
        ) -> ControlFlow<(), PartialSolution<'graph>> {
            context.solve_boolean(solution, uber_identifier, simulate)
        }

        self.solve_branches(solution, branches, simulate, solve_branch)
    }

    fn add_item(
        &mut self,
        solution: &mut PartialSolution,
        index: usize,
        simulate: bool,
    ) -> ControlFlow<()> {
        solution.used_items.push(index);

        self.check_redundancy(solution)?;

        solution.remaining_items.remove(&index);

        if simulate {
            self.item_pool[index].simulate(self.world, self.events);
        }

        ControlFlow::Continue(())
    }

    fn add_spirit_light(
        &mut self,
        solution: &mut PartialSolution,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<()> {
        solution.spirit_light += amount;

        self.check_redundancy(solution)?;

        if simulate {
            self.world.add_spirit_light(amount, self.events);
        }

        ControlFlow::Continue(())
    }

    fn check_redundancy(&self, solution: &PartialSolution) -> ControlFlow<()> {
        if self.finished.iter().any(|finished| {
            trace_is_redundant_with(solution, finished, self.world.graph, self.item_pool)
        })
        // TODO this still eliminates DoubleJump, Grapple when spawning on Feeding Grounds TP
        // || solution
        // .other_branches
        // .iter()
        // .any(|other| trace_is_redundant_with(solution, other, self.world.graph, self.item_pool))
        {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn finish(mut self) -> Vec<Solution> {
        self.finished.extend(self.aborted);

        if cfg!(debug_assertions) {
            for solution in &self.finished {
                self.world.snapshot();

                self.world
                    .simulate_solution(solution, self.item_pool, self.events);

                let new_reached = self.world.reached_pickup_count() - self.initial_pickup_count;

                self.world.restore_snapshot();

                assert_eq!(
                    new_reached, solution.new_reached,
                    "solution {solution} reported {solution_new_reached} reachables but actually reaches {new_reached}",
                    solution = solution.display(self.item_pool, Some(self.world.graph)),
                    solution_new_reached = solution.new_reached,
                );
            }
        }

        self.finished
    }

    fn display_solution<'context, 'solution, S: SolutionLike<'graph>>(
        &'context self,
        solution: &'solution S,
    ) -> DisplaySolution<'graph, 'pool, 'solution> {
        solution.display(self.item_pool, Some(self.world.graph))
    }
}

fn trace_is_redundant_with<'graph, S: SolutionLike<'graph>, O: SolutionLike<'graph>>(
    solution: &S,
    other: &O,
    graph: &'graph Graph,
    item_pool: &ItemPool,
) -> bool {
    let is_redundant = solution.is_redundant_with(other);

    if is_redundant {
        trace!(
            "{solution} is redundant with {other}",
            solution = solution.display(item_pool, Some(graph)),
            other = other.display(item_pool, Some(graph)),
        );
    }

    is_redundant
}
