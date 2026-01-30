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

#[cfg(test)]
mod tests;
mod weight;

pub(crate) use weight::Cost;

use log::trace;
use smallvec::SmallVec;

use std::{
    collections::hash_map::Entry,
    fmt::{self, Display},
    iter,
    ops::ControlFlow,
};

use itertools::Itertools;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use wotw_seedgen_data::{
    logic_language::output::{Connection, Graph, Requirement},
    seed_language::{
        output::{AsConstant, CommandVoid, ContainedWrites, Event},
        simulate::{Simulate, Simulation, Snapshot},
    },
    Skill, UberIdentifier,
};

use crate::{
    item_pool::ItemPool,
    orbs::OrbVariants,
    world::{ConnectionIndex, ConnectionRefValue, Missing},
    World,
};

pub type SolutionItems = SmallVec<[usize; 8]>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    pub items: SolutionItems,
    pub spirit_light: i32,
    pub new_reached: usize,
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

pub trait SolutionLike {
    fn items(&self) -> &SolutionItems;

    fn spirit_light(&self) -> i32;

    fn connection(&self) -> Option<ConnectionIndex>;

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

    fn is_redundant_with<S: SolutionLike>(&self, other: &S) -> bool {
        other.items().iter().all(|item| self.items().contains(item))
            && other.spirit_light() <= self.spirit_light()
    }

    fn display<'graph, 'pool, 'solution>(
        &'solution self,
        item_pool: &'pool ItemPool,
        graph: Option<&'graph Graph>,
    ) -> DisplaySolution<'graph, 'pool, 'solution> {
        DisplaySolution::new(self, item_pool, graph)
    }
}

impl SolutionLike for Solution {
    fn items(&self) -> &SolutionItems {
        &self.items
    }

    fn spirit_light(&self) -> i32 {
        self.spirit_light
    }

    fn connection(&self) -> Option<ConnectionIndex> {
        None
    }
}

pub struct DisplaySolution<'graph, 'pool, 'solution> {
    connection: Option<(ConnectionIndex, &'graph Graph)>,
    items: &'solution SolutionItems,
    spirit_light: i32,
    item_pool: &'pool ItemPool,
}

impl<'graph, 'pool, 'solution> DisplaySolution<'graph, 'pool, 'solution> {
    fn new<S: SolutionLike + ?Sized>(
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
        if let Some((connection, graph)) = self.connection {
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

impl World<'_, '_> {
    pub fn find_solutions(
        &mut self,
        item_pool: &ItemPool,
        events: &[Event],
        slots: usize,
        spirit_light_slots: usize,
    ) -> Vec<Solution> {
        let initial_solutions = self
            .uber_state_fails()
            .values()
            .flatten()
            .chain(self.health_fails())
            .chain(self.energy_fails())
            .map(|fail| PartialSolution::new(*fail, item_pool))
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

    fn simulate_solution(
        &mut self,
        solution: &PartialSolution,
        item_pool: &ItemPool,
        events: &[Event],
    ) {
        for item in &solution.used_items {
            item_pool[*item].simulate(self, events);
        }

        if solution.spirit_light > 0 {
            self.add_spirit_light(solution.spirit_light, events);
        }
    }
}

#[derive(Debug, Clone)]
struct PartialSolution {
    /// The current connection this solution wants to solve
    connection: ConnectionIndex,
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
    new_fails: FxHashMap<UberIdentifier, FxHashSet<ConnectionIndex>>,
}

impl PartialSolution {
    fn new(connection: ConnectionIndex, item_pool: &ItemPool) -> Self {
        Self {
            connection,
            used_items: SmallVec::new(),
            remaining_items: (0..item_pool.len()).collect(),
            spirit_light: 0,
            // other_branches: vec![],
            new_fails: FxHashMap::default(),
        }
    }
}

impl SolutionLike for PartialSolution {
    fn items(&self) -> &SolutionItems {
        &self.used_items
    }

    fn spirit_light(&self) -> i32 {
        self.spirit_light
    }

    fn connection(&self) -> Option<ConnectionIndex> {
        Some(self.connection)
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
    initial_fails: FxHashMap<UberIdentifier, FxHashSet<ConnectionIndex>>,
    solutions: Vec<PartialSolution>,
    finished: Vec<Solution>,
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
        let initial_fails = world.uber_state_fails().clone();

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
        }
    }

    fn solve_untouched(&mut self, solution: PartialSolution) {
        trace!("starting solve for {}", self.display_solution(&solution));

        self.world.snapshot();

        self.solve_until_progress(solution);

        self.world.restore_snapshot();
    }

    fn solve_touched(&mut self, solution: PartialSolution) {
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

    fn solve_until_progress(&mut self, mut solution: PartialSolution) {
        loop {
            let (connection_ref, orb_variants) = self.world.get_connection(solution.connection);

            let flow = match connection_ref.connection {
                ConnectionRefValue::Refill(refill) => {
                    self.solve_requirement(&refill.requirement, solution, orb_variants)
                }
                ConnectionRefValue::Connection(connection) => {
                    self.solve_connection(connection, solution, orb_variants)
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
        connection: &Connection,
        solution: PartialSolution,
        orb_variants: OrbVariants,
    ) -> ControlFlow<(), PartialSolution> {
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
            self.solve_requirement(&connection.requirement, solution, orb_variants)
        }
    }

    fn solve_requirement(
        &mut self,
        requirement: &Requirement,
        solution: PartialSolution,
        mut orb_variants: OrbVariants,
    ) -> ControlFlow<(), PartialSolution> {
        trace!(
            "solving {requirement} for {solution}",
            solution = self.display_solution(&solution)
        );

        // TODO one source of inefficiency is that we don't commit to specific ORs in the requirement
        // maybe we could store an index for that
        match self.world.is_met(requirement, &mut orb_variants) {
            ControlFlow::Continue(()) => {
                trace!("already met");

                self.check_solution(solution)
            }
            ControlFlow::Break(missing) => self.solve(solution, missing, true),
        }
    }

    fn check_solution(&mut self, solution: PartialSolution) -> ControlFlow<(), PartialSolution> {
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

    fn finish_solution(&mut self, solution: PartialSolution, new_reached: usize) {
        let finished = Solution::new(solution, new_reached);

        trace!(
            "finished solution {finished}, {new_reached} reached",
            finished = self.display_solution(&finished),
        );

        self.solutions.retain(|solution| {
            !trace_is_redundant_with(solution, &finished, self.world.graph, self.item_pool)
        });

        self.finished.retain(|other| {
            !trace_is_redundant_with(other, &finished, self.world.graph, self.item_pool)
        });

        self.finished.push(finished);
    }

    fn continue_solution(
        &mut self,
        mut solution: PartialSolution,
    ) -> ControlFlow<(), PartialSolution> {
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

    fn new_fails(&self, solution: &mut PartialSolution) -> FxHashSet<ConnectionIndex> {
        let mut new_fails = FxHashSet::default();

        for (current_uber_identifier, current_connections) in self.world.uber_state_fails() {
            let current_connections_iter = current_connections.iter().copied();

            let initial_filter =
                self.initial_fails
                    .get(current_uber_identifier)
                    .map(|initial_connections| {
                        |connection: &ConnectionIndex| !initial_connections.contains(connection)
                    });

            match solution.new_fails.entry(*current_uber_identifier) {
                Entry::Occupied(mut occupied) => {
                    let solution_connections = occupied.get_mut();
                    let solution_filter =
                        |connection: &ConnectionIndex| solution_connections.insert(*connection);

                    match initial_filter {
                        None => new_fails.extend(current_connections_iter.filter(solution_filter)),
                        Some(initial_filter) => new_fails.extend(
                            current_connections_iter
                                .filter(solution_filter)
                                .filter(initial_filter),
                        ),
                    }
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(current_connections.clone());

                    match initial_filter {
                        None => new_fails.extend(current_connections_iter),
                        Some(initial_filter) => {
                            new_fails.extend(current_connections_iter.filter(initial_filter))
                        }
                    }
                }
            }
        }

        new_fails
    }

    fn pause_solution(&mut self, solution: PartialSolution) {
        // TODO we shouldn't pause if the solution is already finished
        trace!("pausing solution {}", self.display_solution(&solution));

        self.solutions.push(solution);
    }

    fn solve(
        &mut self,
        solution: PartialSolution,
        missing: Missing,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
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
            Missing::Any(any) => self.solve_any(solution, any, simulate),
        }
    }

    fn solve_health(
        &mut self,
        solution: PartialSolution,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
        if self.world.skill(Skill::Regenerate) {
            self.solve_boolean_branches(
                solution,
                self.health_options().chain(self.energy_options()),
                simulate,
            )
        } else {
            self.solve_boolean_branches(
                solution,
                iter::once(Skill::REGENERATE_ID).chain(self.health_options()),
                simulate,
            )
        }
    }

    fn solve_energy(
        &mut self,
        solution: PartialSolution,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
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

    fn solve_boolean(
        &mut self,
        mut solution: PartialSolution,
        uber_identifier: UberIdentifier,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
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

        trace!("solved {}", self.display_solution(&solution));

        ControlFlow::Continue(solution)
    }

    fn solve_integer(
        &mut self,
        mut solution: PartialSolution,
        uber_identifier: UberIdentifier,
        mut amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
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

        trace!("solved {}", self.display_solution(&solution));

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
        mut solution: PartialSolution,
        amount: i32,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
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
        solution: PartialSolution,
        any: Vec<Missing>,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution> {
        // TODO make these unique earlier in the program logic?
        // here they have to be unique so identical branches don't eliminate eachother
        // although, if different missings resolve to the same item pool item, could an issue still arise?
        // TODO also optimize the graph more so we get fewer duplicates here to begin with
        fn unique_missing(any: Vec<Missing>, unique: &mut FxHashSet<Missing>) {
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

        self.solve_branches(
            solution,
            unique,
            simulate,
            |context, solution, missing, simulate| {
                context.solve(solution.clone(), missing, simulate)
            },
        )
    }

    fn solve_branches<I, T, F>(
        &mut self,
        solution: PartialSolution,
        branches: I,
        simulate: bool,
        mut solve: F,
    ) -> ControlFlow<(), PartialSolution>
    where
        I: IntoIterator<Item = T>,
        F: FnMut(
            &mut SolutionContext,
            PartialSolution,
            T,
            bool,
        ) -> ControlFlow<(), PartialSolution>,
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
        solution: PartialSolution,
        branches: I,
        simulate: bool,
    ) -> ControlFlow<(), PartialSolution>
    where
        I: IntoIterator<Item = UberIdentifier>,
    {
        fn solve_branch(
            context: &mut SolutionContext,
            solution: PartialSolution,
            uber_identifier: UberIdentifier,
            simulate: bool,
        ) -> ControlFlow<(), PartialSolution> {
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

    fn finish(self) -> Vec<Solution> {
        self.finished
    }

    fn display_solution<'context, 'solution, S: SolutionLike>(
        &'context self,
        solution: &'solution S,
    ) -> DisplaySolution<'graph, 'pool, 'solution> {
        solution.display(self.item_pool, Some(self.world.graph))
    }
}

fn trace_is_redundant_with<S: SolutionLike, O: SolutionLike>(
    solution: &S,
    other: &O,
    graph: &Graph,
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
