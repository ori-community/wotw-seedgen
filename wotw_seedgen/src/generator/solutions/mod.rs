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

// TODO there seems to be some infinite loop condition on the unoptimized shapes of requirements like MarshSpawn.BurrowArena.

#[cfg(test)]
mod tests;
mod weight;

pub(crate) use weight::{solution_weights, Cost};

use arrayvec::ArrayVec;
use indexmap::IndexMap;
use log::{log_enabled, trace, warn, Level::Trace};
use ordered_float::{Float, OrderedFloat};
use smallvec::SmallVec;

use std::{
    cmp::Ordering,
    collections::{hash_map::Entry, VecDeque},
    env,
    fmt::{self, Display},
    mem,
    ops::{ControlFlow, Sub},
    sync::LazyLock,
};

use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use wotw_seedgen_data::{
    logic_language::output::{Connection, Graph, Requirement},
    seed_language::{
        output::{CommandVoid, CommonWriteCommand, ContainedWrites, Event, UberStateWrite},
        simulate::{Simulate, Simulation, Snapshot},
    },
    Difficulty, Shard, Skill, UberIdentifier,
};

use crate::{
    item_pool::ItemPool,
    logical_difficulty::LogicalDifficulty,
    orbs::OrbVariants,
    world::{
        ConnectionIndex, ConnectionOrRefill, ConnectionRequirement, ConnectionRequirementPartial,
        EqIgnore, GraphRef, Missing, ReachStateFails,
    },
    World,
};

/// Maximum radius of connections to solve before aborting a solution.
///
/// Lower values reduce search time but also reduce progression quality.
/// We currently default to 0 which is pretty bad but also how v4 behaved.
///
/// `cargo bench solutions` times as of 2026-03-21:
/// - `MAX_SEARCH_RADIUS=0`: 2.131 ms
/// - `MAX_SEARCH_RADIUS=1`: 4.177 ms
/// - `MAX_SEARCH_RADIUS=2`: 7.052 ms
/// - `MAX_SEARCH_RADIUS=u8::MAX`: 7.097 ms
///
/// `cargo bench generation/unsafe` times as of 2026-04-10:
/// - `MAX_SEARCH_RADIUS=0`: 549.56 ms
/// - `MAX_SEARCH_RADIUS=1`: 868.59 ms
/// - `MAX_SEARCH_RADIUS=2`: 1.4614 s
static MAX_SEARCH_RADIUS: LazyLock<u8> = LazyLock::new(|| {
    env::var("SOLUTION_MAX_SEARCH_RADIUS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
});

/// Maximum number of progression items in a solution
///
/// Lower values reduce search time but also ignore some complex progressions
///
/// `cargo bench generation/unsafe` times as of 2026-04-10:
/// - `SOLUTION_MAX_ITEMS=5`: 230.25 ms
/// - `SOLUTION_MAX_ITEMS=7`: 556.95 ms
/// - `SOLUTION_MAX_ITEMS=10`: 933.20 ms
/// - `SOLUTION_MAX_ITEMS=usize::MAX`: 1.2723 s
pub static SOLUTION_MAX_ITEMS: LazyLock<usize> = LazyLock::new(|| {
    env::var("SOLUTION_MAX_ITEMS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
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
            write!(f, "{} / ", connection.display(graph))?;
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
        // This really slows down default settings for some reason? But unsafe is much more critical...
        let capped_slots = usize::min(slots, *SOLUTION_MAX_ITEMS);
        let mut solutions = self.find_solutions_no_max_items(
            item_pool,
            events,
            capped_slots,
            spirit_light_slots,
            search_radius,
        );

        if solutions.is_empty() && slots > capped_slots {
            warn!("no solutions found, retrying with uncapped solution size");

            solutions = self.find_solutions_no_max_items(
                item_pool,
                events,
                slots,
                spirit_light_slots,
                search_radius,
            );
        }

        solutions
    }

    pub fn find_solutions_no_max_items(
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
            .collect::<FxHashSet<_>>()
            .into_iter()
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

        // Touched solutions are a queue so that branching paths predictably execute
        // one variant before the other, allowing us to prioritize paths that are
        // likely to eliminate redundancies earlier.
        while let Some(solution) = context.solutions.pop_front() {
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
    // skip_energy_branches: bool,
    better_weapon: bool,
    burrow_as_weapon: bool,
    energy_shard: bool,
    vitality: bool,
    resilience: bool,
    overcharge: bool,
    life_pact: bool,
}

impl Commitments {
    fn commit_better_weapon(&mut self) -> bool {
        mem::replace(&mut self.better_weapon, true)
    }

    fn commit_burrow_as_weapon(&mut self) -> bool {
        mem::replace(&mut self.burrow_as_weapon, true)
    }

    fn commit_energy_shard(&mut self) -> bool {
        mem::replace(&mut self.energy_shard, true)
    }

    fn commit_vitality(&mut self) -> bool {
        mem::replace(&mut self.vitality, true)
    }

    fn commit_resilience(&mut self) -> bool {
        mem::replace(&mut self.resilience, true)
    }

    fn commit_overcharge(&mut self) -> bool {
        mem::replace(&mut self.overcharge, true)
    }

    fn commit_life_pact(&mut self) -> bool {
        mem::replace(&mut self.life_pact, true)
    }
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
    solutions: VecDeque<PartialSolution<'graph>>,
    finished: Vec<Solution>,
    aborted: Vec<Solution>,
    perf_counters: IndexMap<usize, u32, FxBuildHasher>,
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
            solutions: VecDeque::new(),
            finished: vec![],
            aborted: vec![],
            perf_counters: IndexMap::default(),
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
                .format(", "),
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
        mut solution: PartialSolution<'graph>,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if log_enabled!(target: "perf_counters", Trace) {
            if let ConnectionOrRefill::Connection(connection) = solution.connection.connection {
                *self.perf_counters.entry(connection.to).or_default() += 1;
            }
        }

        let mut orb_variants = self.world.get_connection_orbs(&solution.connection).clone();
        match solution.connection.is_met(self.world, &mut orb_variants) {
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

        self.solutions.push_back(solution);
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
            Missing::Health(amount) => self.solve_health(solution, *amount.ceil() as i32, simulate),
            Missing::Energy(amount) => self.solve_energy::<true>(solution, amount, simulate),
            Missing::WallWeapon => self.solve_weapon::<true>(solution, simulate),
            Missing::EnemyWeapon => self.solve_weapon::<false>(solution, simulate),
            Missing::EnergyOrBetterWallWeapon(amount) => {
                self.solve_energy_or_better_weapon::<true>(solution, amount, simulate)
            }
            Missing::EnergyOrBetterEnemyWeapon(amount) => {
                self.solve_energy_or_better_weapon::<false>(solution, amount, simulate)
            }
            Missing::EnergyOrBurrowOrBetterEnemyWeapon(amount) => {
                self.solve_energy_or_burrow_or_better_enemy_weapon(solution, amount, simulate)
            }
            Missing::Any(any) => self.solve_any(solution, any, simulate),
            Missing::Or(ors, orb_variants) => self.solve_or(solution, ors, orb_variants, simulate),
        }
    }

    fn solve_health(
        &mut self,
        mut solution: PartialSolution<'graph>,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        solution.connection.orb_reset();

        self.solve_shard_branches(
            solution,
            simulate,
            Shard::Vitality,
            Commitments::commit_vitality,
            Difficulty::vitality,
            |this, solution, simulate| {
                this.solve_shard_branches(
                    solution,
                    simulate,
                    Shard::Resilience,
                    // TODO we could optimize some branches if we do the other branch first and use information
                    // from that somehow. For example, if solve_max_health only needs one health-boosting item,
                    // then we know a resilience branch will always be redundant.
                    Commitments::commit_resilience,
                    Difficulty::resilience,
                    |this, solution, simulate| this.solve_max_health(solution, amount, simulate),
                )
            },
        )
    }

    fn solve_max_health(
        &mut self,
        solution: PartialSolution<'graph>,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_amount(
            solution,
            UberIdentifier::MAX_HEALTH,
            amount,
            // health cannot drop to zero
            |amount| amount < 0,
            convert_integer_write,
            simulate,
        )
    }

    fn solve_energy<const LIFE_PACT: bool>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        amount: OrderedFloat<f32>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        solution.connection.orb_reset();

        self.solve_shard_branches(
            solution,
            simulate,
            Shard::Energy,
            Commitments::commit_energy_shard,
            Difficulty::energy_shard,
            |this, solution, simulate| {
                this.solve_shard_branches(
                    solution,
                    simulate,
                    Shard::Overcharge,
                    Commitments::commit_overcharge,
                    Difficulty::overcharge,
                    |this, mut solution, simulate| {
                        if !LIFE_PACT || !this.world.settings.difficulty.life_pact() {
                            this.solve_max_energy(solution, amount, simulate)
                        } else if this.world.shard(Shard::LifePact) {
                            // is_met communicates if health is missing due to life pact, so we're already branching into those variants.
                            // But the amount returned is the full missing amount, for a complete set of solutions we need to include
                            // mixes of health and energy. We can solev this in a non-overlapping way by letting the health branch
                            // solve its communicated max amount and doing one step of energy here in the energy branch.
                            // This will iteratively create a tree where energy is solved stepwise and health directly jumps to the leaves,
                            // which is ideal. We can use solve_boolean for a single step since the UberState type doesn't actually matter.
                            this.solve_boolean(solution, UberIdentifier::MAX_ENERGY, simulate)
                            // TODO remove if not needed
                            // this.solve_orb_tree(
                            //     solution,
                            //     simulate,
                            //     |this, solution, simulate| this.solve_health(solution, 1, simulate),
                            //     |this, solution, simulate| {
                            //         this.solve_max_energy(solution, amount, simulate)
                            //     },
                            // )
                        } else if solution.commitments.commit_life_pact() {
                            this.solve_max_energy(solution, amount, simulate)
                        } else {
                            this.solve_simple_branch(
                                solution,
                                simulate,
                                |this, solution, simulate| {
                                    this.solve_max_energy(solution, amount, simulate)
                                },
                                Self::solve_life_pact,
                            )
                        }
                    },
                )
            },
        )
    }

    fn solve_max_energy(
        &mut self,
        solution: PartialSolution<'graph>,
        amount: OrderedFloat<f32>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_float(solution, UberIdentifier::MAX_ENERGY, amount, simulate)
    }

    fn solve_life_pact(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_boolean(solution, Shard::LIFE_PACT_ID, simulate)
    }

    fn solve_shard_branches<C, D, F>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        simulate: bool,
        shard: Shard,
        commitment: C,
        difficulty: D,
        next: F,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        C: FnOnce(&mut Commitments) -> bool,
        D: FnOnce(Difficulty) -> bool,
        F: FnOnce(
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
            PartialSolution<'graph>,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
    {
        if commitment(&mut solution.commitments)
            || self.world.shard(shard)
            || !difficulty(self.world.settings.difficulty)
        {
            next(self, solution, simulate)
        } else {
            self.solve_simple_branch(solution, simulate, next, move |this, solution, simulate| {
                this.solve_shard(solution, simulate, shard)
            })
        }
    }

    fn solve_shard(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
        shard: Shard,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_boolean(solution, shard.uber_identifier(), simulate)
    }

    // fn solve_orb_tree<H, E>(
    //     &mut self,
    //     solution: PartialSolution<'graph>,
    //     simulate: bool,
    //     solve_health: H,
    //     solve_energy: E,
    // ) -> ControlFlow<(), PartialSolution<'graph>>
    // where
    //     H: FnOnce(
    //         &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
    //         PartialSolution<'graph>,
    //         bool,
    //     ) -> ControlFlow<(), PartialSolution<'graph>>,
    //     E: FnOnce(
    //         &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
    //         PartialSolution<'graph>,
    //         bool,
    //     ) -> ControlFlow<(), PartialSolution<'graph>>,
    // {
    //     if solution.commitments.skip_energy_branches {
    //         solve_health(self, solution, simulate)
    //     } else {
    //         self.solve_simple_branch(
    //             solution,
    //             simulate,
    //             |this, mut solution, simulate| {
    //                 // Unlike other commitments, only one branch should commit in this case to keep building a tree
    //                 solution.commitments.skip_energy_branches = true;
    //                 solve_health(this, solution, simulate)
    //             },
    //             solve_energy,
    //         )
    //     }
    // }

    fn solve_simple_branch<L, R>(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
        left: L,
        right: R,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        L: FnOnce(
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
            PartialSolution<'graph>,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
        R: FnOnce(
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
            PartialSolution<'graph>,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
    {
        let left_flow = left(self, solution.clone(), simulate);
        let right_flow = right(self, solution, simulate && left_flow.is_break());

        match (left_flow, right_flow) {
            (left_flow @ ControlFlow::Continue(_), ControlFlow::Continue(right_solution)) => {
                self.pause_solution(right_solution);
                left_flow
            }
            (flow @ ControlFlow::Continue(_), ControlFlow::Break(()))
            | (ControlFlow::Break(()), flow) => flow,
        }
    }

    fn solve_committing_branch<C, L, R>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        simulate: bool,
        commitment: C,
        always: L,
        once: R,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        C: FnOnce(&mut Commitments) -> bool,
        L: FnOnce(
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
            PartialSolution<'graph>,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
        R: FnOnce(
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
            PartialSolution<'graph>,
            bool,
        ) -> ControlFlow<(), PartialSolution<'graph>>,
    {
        if commitment(&mut solution.commitments) {
            always(self, solution, simulate)
        } else {
            self.solve_simple_branch(solution, simulate, always, once)
        }
    }

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

        self.solve_branches(solution, branches, simulate, Self::solve_boolean)
    }

    fn solve_energy_or_burrow_or_better_enemy_weapon(
        &mut self,
        solution: PartialSolution<'graph>,
        amount: OrderedFloat<f32>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_committing_branch(
            solution,
            simulate,
            Commitments::commit_burrow_as_weapon,
            |this, solution, simulate| {
                this.solve_energy_or_better_weapon::<false>(solution, amount, simulate)
            },
            Self::solve_burrow,
        )
    }

    fn solve_burrow(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_boolean(solution, Skill::BURROW_ID, simulate)
    }

    fn solve_energy_or_better_weapon<const TARGET_IS_WALL: bool>(
        &mut self,
        solution: PartialSolution<'graph>,
        amount: OrderedFloat<f32>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        self.solve_committing_branch(
            solution,
            simulate,
            Commitments::commit_better_weapon,
            |this, solution, simulate| this.solve_energy::<true>(solution, amount, simulate),
            Self::solve_better_weapon::<TARGET_IS_WALL>,
        )
    }

    fn solve_better_weapon<const TARGET_IS_WALL: bool>(
        &mut self,
        solution: PartialSolution<'graph>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        // TODO on some branches energyless weapons might be guaranteed redundant.
        // For example, if we are in an overcharge branch and there is a non-overcharge
        // alternative, then that other path is going to find the better energyless
        // weapon solutions. But I don't think we currently store enough info for that.
        let weapons = self
            .world
            .better_weapons::<TARGET_IS_WALL>()
            .map(Skill::uber_identifier)
            .collect::<ArrayVec<_, 9>>();

        self.solve_branches(solution, weapons, simulate, Self::solve_boolean)
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
            trace!("no items in the pool to solve {uber_identifier}");

            return ControlFlow::Break(());
        };

        if self
            .world
            .settings
            .difficulty
            .may_increase_orbs(uber_identifier)
        {
            solution.connection.orb_reset();
        }

        self.add_item(&mut solution, index, simulate)?;

        trace!(
            "progressed {uber_identifier} for {solution}",
            solution = self.display_solution(&solution),
        );

        ControlFlow::Continue(solution)
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

    fn solve_integer(
        &mut self,
        solution: PartialSolution<'graph>,
        uber_identifier: UberIdentifier,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        if uber_identifier == UberIdentifier::SPIRIT_LIGHT {
            self.solve_spirit_light(solution, amount, simulate)
        } else {
            self.solve_amount(
                solution,
                uber_identifier,
                amount,
                |amount| amount <= 0,
                convert_integer_write,
                simulate,
            )
        }
    }

    fn solve_float(
        &mut self,
        solution: PartialSolution<'graph>,
        uber_identifier: UberIdentifier,
        amount: OrderedFloat<f32>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        fn convert(write: &UberStateWrite) -> Option<OrderedFloat<f32>> {
            match CommonWriteCommand::from_write(write) {
                Some(CommonWriteCommand::AddFloat(amount)) => Some(amount),
                _ => None,
            }
        }

        self.solve_amount(
            solution,
            uber_identifier,
            amount,
            |amount| amount <= (0.).into(),
            convert,
            simulate,
        )
    }

    fn solve_amount<A, FA, FW>(
        &mut self,
        mut solution: PartialSolution<'graph>,
        uber_identifier: UberIdentifier,
        mut amount: A,
        mut amount_finished: FA,
        mut amount_from_write: FW,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>>
    where
        A: Copy + PartialOrd + Sub<Output = A> + Display,
        FA: FnMut(A) -> bool,
        FW: FnMut(&UberStateWrite<'pool>) -> Option<A>,
    {
        self.has_free_slot(&solution)?;

        let mut items = vec![];
        let mut slots = self.slots - solution.used_slots();

        'outer: for index in solution.remaining_items.iter().copied() {
            let mut item_helps = false;

            for write in self.item_pool[index]
                // TODO positive writes etc.
                .contained_writes()
                .filter(|write| write.uber_identifier == uber_identifier)
            {
                item_helps = true;

                match amount_from_write(&write) {
                    None => {
                        trace!("unable to read into {}, solving stepwise", write.command);

                        items.push(index);
                        break 'outer;
                    }
                    Some(item_amount) => amount = amount - item_amount,
                }

                if amount_finished(amount) {
                    break;
                }
            }

            if item_helps {
                items.push(index);

                if amount_finished(amount) {
                    break;
                }

                if slots > 1 {
                    slots -= 1;
                } else {
                    trace!("not enough slots to solve {uber_identifier}*{amount}");

                    return ControlFlow::Break(());
                }
            }
        }

        if items.is_empty() {
            trace!("no items in the pool to solve {uber_identifier}*{amount}");

            return ControlFlow::Break(());
        }

        self.add_items(&mut solution, items, simulate)?;

        trace!(
            "progressed {uber_identifier} for {solution}",
            solution = self.display_solution(&solution),
        );

        ControlFlow::Continue(solution)
    }

    fn add_items(
        &mut self,
        solution: &mut PartialSolution,
        items: Vec<usize>,
        simulate: bool,
    ) -> ControlFlow<()> {
        solution.used_items.extend(items.iter().copied());

        self.check_redundancy(&solution)?;

        for index in &items {
            solution.remaining_items.remove(index);
        }

        if simulate {
            // TODO maybe queue up changes to have fewer reach refreshes?
            for index in items {
                self.item_pool[index].simulate(self.world, self.events);
            }
        }

        ControlFlow::Continue(())
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
        // TODO is this irrelevant now?
        // // TODO make these unique earlier in the program logic?
        // // here they have to be unique so identical branches don't eliminate eachother
        // // although, if different missings resolve to the same item pool item, could an issue still arise?
        // // TODO also optimize the graph more so we get fewer duplicates here to begin with
        // fn unique_missing<'graph>(
        //     any: Vec<Missing<'graph>>,
        //     unique: &mut IndexSet<Missing<'graph>, FxBuildHasher>,
        // ) {
        //     for missing in any {
        //         match missing {
        //             Missing::Any(any) => unique_missing(any, unique),
        //             single => {
        //                 unique.insert(single);
        //             }
        //         }
        //     }
        // }

        // let mut unique = IndexSet::with_capacity_and_hasher(any.len(), FxBuildHasher);
        // unique_missing(any, &mut unique);

        self.solve_branches(solution, any, simulate, Self::solve)
    }

    fn solve_or(
        &mut self,
        mut solution: PartialSolution<'graph>,
        ors: Vec<(Missing<'graph>, GraphRef<'graph, Requirement>)>,
        orb_variants: EqIgnore<OrbVariants>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution<'graph>> {
        fn solve_branch<'graph>(
            context: &mut SolutionContext<'_, 'graph, '_, '_, '_>,
            mut solution: PartialSolution<'graph>,
            (missing, requirement): (Missing<'graph>, GraphRef<'graph, Requirement>),
            simulate: bool,
        ) -> ControlFlow<(), PartialSolution<'graph>> {
            let ConnectionRequirement::Partial(partial) = &mut solution.connection.requirement
            else {
                unreachable!()
            };

            partial.requirement = requirement;
            context.solve(solution, missing, simulate)
        }

        solution.connection.requirement =
            ConnectionRequirement::Partial(ConnectionRequirementPartial {
                requirement: GraphRef(&Requirement::Impossible),
                orb_variants,
            });

        self.solve_branches(solution, ors, simulate, solve_branch)
    }

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
            &mut SolutionContext<'world, 'graph, 'settings, 'events, 'pool>,
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

        if log_enabled!(target: "perf_counters", Trace) {
            self.perf_counters.sort_unstable_by(|_, a, _, b| b.cmp(a));

            trace!(
                target: "perf_counters",
                "Solution perf counters:\n{}",
                self.perf_counters
                    .iter()
                    .format_with("\n", |(index, count), f| f(&format_args!(
                        "{count:04} -> {node}",
                        node = self.world.graph.nodes[*index].identifier()
                    )))
            );
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

fn convert_integer_write(write: &UberStateWrite) -> Option<i32> {
    match CommonWriteCommand::from_write(write) {
        Some(CommonWriteCommand::AddInteger(amount)) => Some(amount),
        _ => None,
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
