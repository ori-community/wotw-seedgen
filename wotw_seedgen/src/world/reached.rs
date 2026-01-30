use std::{
    fmt::{self, Display},
    hash::Hash,
    mem,
    ops::{ControlFlow, Deref},
};

use super::World;
use crate::{
    logical_difficulty::LogicalDifficulty,
    orbs::{self, OrbVariants},
    world::is_met::Missing,
};
use itertools::Itertools;
use log::trace;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use smallvec::smallvec;
use wotw_seedgen_data::{
    assets::{LocDataEntry, StateDataEntry},
    logic_language::output::{Anchor, Connection, Graph, Node, Refill, RefillValue, Requirement},
    seed_language::{
        output::Event,
        simulate::{CloneSnapshot, Simulation, Snapshot},
    },
    Shard, Skill, UberIdentifier,
};

pub const TP_ANCHOR: &str = "Teleporters";

// TODO figuring out how to update existing best_orbs with orb changes is NOT reasonable. Abort this idea.
#[derive(Debug)]
pub struct Reach {
    state: CloneSnapshot<ReachState>,
    logic_state_map: LogicStateMap,
}

impl Reach {
    pub fn new(graph: &Graph) -> Self {
        Self {
            state: CloneSnapshot::default(),
            logic_state_map: LogicStateMap::new(graph),
        }
    }
}

impl Snapshot for Reach {
    fn snapshot(&mut self) {
        self.state.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.state.restore_snapshot();
    }
}

#[derive(Debug, Clone, Default)]
struct ReachState {
    /// All reached nodes and if they are anchors, the best orbs they have been reached with
    best_orbs: FxHashMap<usize, OrbVariants>,
    /// [`TP_ANCHOR`] has been reached
    tp_reached: bool,
    /// All [`ConnectionIndex`] which failed to solve and might be solved by advancing the [`UberIdentifier`]
    uber_state_fails: FxHashMap<UberIdentifier, FxHashSet<ConnectionIndex>>,
    /// All [`ConnectionIndex`] which failed to solve and might be solved by reaching the logical state
    logical_state_fails: FxHashMap<usize, FxHashSet<ConnectionIndex>>,
    /// Some connections failed to solve and might require more health.
    /// Resuming progress along those connections would be very hard because of refill logic,
    /// So we just reset the entire Reach when progressing orbs.
    health_fails: FxHashSet<ConnectionIndex>,
    /// Same as `health_fail`, but for energy.
    energy_fails: FxHashSet<ConnectionIndex>,
}

// TODO were these capacities good?
// best_orbs: FxHashMap::with_capacity_and_hasher(graph.nodes.len(), FxBuildHasher),
// tp_reached: false,
// uber_state_fails: FxHashMap::with_capacity_and_hasher(80, FxBuildHasher),
// logical_state_fails: FxHashMap::with_capacity_and_hasher(5, FxBuildHasher),
// orb_fail: false,

impl ReachState {
    fn clear(&mut self) {
        self.best_orbs.clear();
        self.tp_reached = false;
        self.uber_state_fails.clear();
        self.logical_state_fails.clear();
        self.health_fails.clear();
        self.energy_fails.clear();
    }

    fn add_fail(&mut self, missing: Missing, connection: ConnectionIndex) {
        match missing {
            Missing::Impossible => {}
            // TODO optimize by using the missing integer value and skipping reach attempts?
            Missing::Boolean(uber_identifier) => {
                add_fail_to(&mut self.uber_state_fails, uber_identifier, connection);
            }
            Missing::Integer(uber_identifier, _) => {
                add_fail_to(&mut self.uber_state_fails, uber_identifier, connection);
            }
            Missing::LogicalState(index) => {
                add_fail_to(&mut self.logical_state_fails, index, connection);
            }
            Missing::Health => {
                self.health_fails.insert(connection);
            }
            Missing::Energy => {
                self.energy_fails.insert(connection);
            }
            Missing::Any(options) => {
                for missing in options {
                    self.add_fail(missing, connection);
                }
            }
        }
    }

    fn orb_fail(&self) -> bool {
        !(self.health_fails.is_empty() && self.energy_fails.is_empty())
    }
}

fn add_fail_to<K: Eq + Hash, V: Eq + Hash>(map: &mut FxHashMap<K, FxHashSet<V>>, key: K, value: V) {
    map.entry(key).or_default().insert(value);
}

/// All the logic states which might be solved by advancing the [`UberIdentifier`]
#[derive(Debug)]
struct LogicStateMap {
    inner: FxHashMap<UberIdentifier, Vec<usize>>,
}

impl LogicStateMap {
    fn new(graph: &Graph) -> Self {
        let mut inner =
            FxHashMap::<_, Vec<_>>::with_capacity_and_hasher(graph.nodes.len(), FxBuildHasher);

        for (index, node) in graph.nodes.iter().enumerate() {
            if let Some(uber_identifier) = node.uber_identifier() {
                inner.entry(uber_identifier).or_default().push(index);
            }
        }

        inner.shrink_to_fit();
        Self { inner }
    }
}

impl Deref for LogicStateMap {
    type Target = FxHashMap<UberIdentifier, Vec<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A connection inside the [`Graph`] which may allow further progress in a different state.
///
/// You can use [`ConnectionIndex::index_graph`] to find the referenced connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ConnectionIndex {
    /// Index into [`Graph`] `nodes`.
    /// Should always point to an [`Anchor`].
    pub node_index: usize,
    /// Index into [`Anchor`] `connections`, or `refills` if `is_refill`.
    pub connection_index: usize,
    /// Whether `connection_index` references a refill instead of a connection.
    pub is_refill: bool,
}

impl ConnectionIndex {
    pub(crate) fn connection(node_index: usize, connection_index: usize) -> Self {
        Self {
            node_index,
            connection_index,
            is_refill: false,
        }
    }

    pub(crate) fn refill(node_index: usize, refill_index: usize) -> Self {
        Self {
            node_index,
            connection_index: refill_index,
            is_refill: true,
        }
    }

    pub(crate) fn index_graph<'g>(self, graph: &'g Graph) -> ConnectionRef<'g> {
        let anchor = graph.nodes[self.node_index].expect_anchor();

        let connection = if self.is_refill {
            ConnectionRefValue::Refill(&anchor.refills[self.connection_index])
        } else {
            ConnectionRefValue::Connection(&anchor.connections[self.connection_index])
        };

        ConnectionRef { anchor, connection }
    }

    pub(crate) fn display<'g>(self, graph: &'g Graph) -> ConnectionIndexDisplay<'g> {
        ConnectionIndexDisplay {
            connection: self,
            graph,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ConnectionRef<'g> {
    pub anchor: &'g Anchor,
    pub connection: ConnectionRefValue<'g>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ConnectionRefValue<'g> {
    Refill(&'g Refill),
    Connection(&'g Connection),
}

pub(crate) struct ConnectionIndexDisplay<'g> {
    connection: ConnectionIndex,
    graph: &'g Graph,
}

impl Display for ConnectionIndexDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.connection
            .index_graph(&self.graph)
            .display(&self.graph)
            .fmt(f)
    }
}

impl<'g> ConnectionRef<'g> {
    pub(crate) fn display(self, graph: &'g Graph) -> ConnectionRefDisplay<'g> {
        ConnectionRefDisplay {
            connection: self,
            graph,
        }
    }
}

pub(crate) struct ConnectionRefDisplay<'g> {
    connection: ConnectionRef<'g>,
    graph: &'g Graph,
}

impl Display for ConnectionRefDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.connection.anchor.identifier.fmt(f)?;

        match self.connection.connection {
            ConnectionRefValue::Refill(refill) => write!(f, " -> {}", refill.value),
            ConnectionRefValue::Connection(connection) => {
                write!(f, " -> {}", self.graph.nodes[connection.to].identifier())
            }
        }
    }
}

impl<'graph> World<'graph, '_> {
    #[inline]
    pub fn reached_indices(&self) -> impl Iterator<Item = usize> + use<'_> {
        self.reach.state.best_orbs.keys().copied()
    }

    #[inline]
    pub fn reached_nodes<'s>(&'s self) -> impl Iterator<Item = &'graph Node> + use<'s, 'graph> {
        self.reached_indices().map(|index| &self.graph.nodes[index])
    }

    #[inline]
    pub fn reached_pickups<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'graph LocDataEntry> + use<'s, 'graph> {
        self.reached_nodes().filter_map(Node::try_as_pickup_ref)
    }

    #[inline]
    pub fn reached_pickup_count(&self) -> usize {
        self.reached_pickups().count()
    }

    #[inline]
    pub fn has_reached(&self, index: usize) -> bool {
        self.reach.state.best_orbs.contains_key(&index)
    }

    pub fn traverse_spawn(&mut self, events: &[Event]) {
        self.check_all_states();

        let orb_variants = smallvec![self.max_orbs()];
        self.traverse(self.spawn, orb_variants, events);

        self.attempt_spawn_teleport(events);
    }

    // TODO other fails?
    pub(crate) fn uber_state_fails(
        &self,
    ) -> &FxHashMap<UberIdentifier, FxHashSet<ConnectionIndex>> {
        &self.reach.state.uber_state_fails
    }

    pub(crate) fn health_fails(&self) -> &FxHashSet<ConnectionIndex> {
        &self.reach.state.health_fails
    }

    pub(crate) fn energy_fails(&self) -> &FxHashSet<ConnectionIndex> {
        &self.reach.state.energy_fails
    }

    pub(crate) fn get_connection(
        &self,
        connection: ConnectionIndex,
    ) -> (ConnectionRef<'graph>, OrbVariants) {
        // TODO this still fails sometimes...
        assert!(
            self.reach.state.best_orbs.contains_key(&connection.node_index),
            "Failed to get connection!\nBacktrace: {}\nInventory: {}\nReached: {}\nTried connection: {}",
            std::backtrace::Backtrace::force_capture(),
            self.inventory_display(),
            self.reached_nodes()
                .filter_map(Node::try_as_anchor_ref)
                .map(|anchor| &anchor.identifier)
                .format(", "),
            connection.display(self.graph)
        );

        (
            connection.index_graph(self.graph),
            self.reach.state.best_orbs[&connection.node_index].clone(),
        )
    }

    // /// Clean any stale fails
    // pub(crate) fn clean_fails(&mut self) {
    //     // TODO other fails?
    //     for connections in self.reach.state.value.uber_state_fails.values_mut() {
    //         connections.retain(
    //             |connection| match connection.index_graph(self.graph).connection {
    //                 ConnectionRefValue::Refill(_) => true,
    //                 ConnectionRefValue::Connection(connection) => !self
    //                     .reach
    //                     .state
    //                     .value
    //                     .best_orbs
    //                     .contains_key(&connection.to),
    //             },
    //         );
    //     }
    // }

    fn attempt_spawn_teleport(&mut self, events: &[Event]) {
        let reached_anchors = self
            .reach
            .state
            .best_orbs
            .keys()
            .filter_map(|node_index| self.graph.nodes[*node_index].try_as_anchor_ref())
            .collect::<Vec<_>>();

        for anchor in reached_anchors {
            self.attempt_teleport(anchor, events)
        }
    }

    fn attempt_teleport(&mut self, anchor: &Anchor, events: &[Event]) {
        if self.reach.state.tp_reached {
            return;
        }

        let mut orb_variants = smallvec![self.max_orbs()];

        if self
            .is_met(&anchor.teleport_restriction, &mut orb_variants)
            .is_continue()
        {
            if let Ok(tp_index) = self.graph.find_node(TP_ANCHOR) {
                self.reach.state.tp_reached = true;
                self.traverse(tp_index, orb_variants, events);
            }
        }
    }

    pub(super) fn update_reached(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        trace!(
            "updating reach for {uber_identifier} with {inventory}",
            inventory = self.inventory_display(),
        );

        let was_updating_reach = mem::replace(&mut self.updating_reach, true);

        self.check_states_for(uber_identifier);

        if let Some(fails) = self.reach.state.uber_state_fails.remove(&uber_identifier) {
            trace!("removed {uber_identifier} from UberState fails");
            for fail in fails {
                self.progress(fail, events);
            }
        }

        if !was_updating_reach {
            if self.reach.state.orb_fail() && self.may_increase_orbs(uber_identifier) {
                self.reach.state.clear();

                self.traverse_spawn(events);
            }

            self.updating_reach = false;
        }
    }

    fn check_all_states(&mut self) {
        let logic_states = self
            .reach
            .logic_state_map
            .values()
            .flat_map(|logic_states| self.filter_unreached_states(logic_states))
            .collect::<Vec<_>>();

        self.check_states(logic_states);
    }

    fn check_states_for(&mut self, uber_identifier: UberIdentifier) {
        if let Some(logic_states) = self.reach.logic_state_map.get(&uber_identifier) {
            let logic_states = self.filter_unreached_states(logic_states);

            if !logic_states.is_empty() {
                trace!(
                    "checking states for {uber_identifier}: {}",
                    logic_states
                        .iter()
                        .format_with(", ", |index, f| f(&self.graph.nodes[*index].identifier()))
                );

                self.check_states(logic_states);
            }
        }
    }

    fn filter_unreached_states(&self, logic_states: &[usize]) -> Vec<usize> {
        logic_states
            .iter()
            .filter(|index| !self.has_reached(**index))
            .copied()
            .collect()
    }

    fn check_states(&mut self, logic_states: Vec<usize>) {
        for index in logic_states {
            let node = &self.graph.nodes[index];
            let uber_identifier = node.uber_identifier().unwrap();

            let met = if uber_identifier.is_door() {
                self.door_condition_met(uber_identifier, node.value().unwrap())
            } else {
                self.loc_data_condition_met(uber_identifier, node.value())
            };

            if met {
                self.reach.state.best_orbs.insert(index, smallvec![]);
            }
        }
    }

    fn may_increase_orbs(&self, uber_identifier: UberIdentifier) -> bool {
        match uber_identifier {
            UberIdentifier::MAX_HEALTH | UberIdentifier::MAX_ENERGY | Skill::REGENERATE_ID => true,
            Shard::RESILIENCE_ID => self.settings.difficulty.resilience(),
            Shard::VITALITY_ID => self.settings.difficulty.vitality(),
            Shard::ENERGY_ID => self.settings.difficulty.energy_shard(),
            Shard::OVERCHARGE_ID => self.settings.difficulty.overcharge(),
            Shard::LIFE_PACT_ID => self.settings.difficulty.life_pact(),
            Shard::OVERFLOW_ID => self.settings.difficulty.overflow(),
            Shard::CATALYST_ID => self.settings.difficulty.catalyst(),
            _ => false,
        }
    }

    fn progress(&mut self, connection_index: ConnectionIndex, events: &[Event]) {
        let (connection_ref, orb_variants) = self.get_connection(connection_index);

        match connection_ref.connection {
            ConnectionRefValue::Refill(_) => {
                self.traverse(connection_index.node_index, orb_variants, events)
            }
            ConnectionRefValue::Connection(connection) => {
                self.traverse_connection(connection, orb_variants, connection_index, events)
            }
        }
    }

    fn traverse(&mut self, node_index: usize, orb_variants: OrbVariants, events: &[Event]) {
        let node = &self.graph.nodes[node_index];

        trace!(
            "{identifier} reached with {best_orbs}",
            identifier = node.identifier(),
            best_orbs = orb_variants.iter().format(" or "),
        );

        match node {
            Node::Anchor(anchor) => self.traverse_anchor(anchor, node_index, orb_variants, events),
            Node::Pickup(LocDataEntry {
                uber_identifier,
                value,
                ..
            })
            | Node::State(StateDataEntry {
                uber_identifier,
                value,
                ..
            }) => self.traverse_state(node_index, *uber_identifier, *value, events),
            Node::LogicalState(_) => self.traverse_logical_state(node_index, events),
        }
    }

    fn traverse_anchor(
        &mut self,
        anchor: &Anchor,
        node_index: usize,
        mut orb_variants: OrbVariants,
        events: &[Event],
    ) {
        self.use_refills(anchor, &mut orb_variants, node_index);

        self.reach
            .state
            .best_orbs
            .insert(node_index, orb_variants.clone());

        self.attempt_teleport(anchor, events);

        for (connection_index, connection) in anchor.connections.iter().enumerate() {
            self.traverse_connection(
                connection,
                orb_variants.clone(),
                ConnectionIndex::connection(node_index, connection_index),
                events,
            )
        }
    }

    fn traverse_state(
        &mut self,
        index: usize,
        uber_identifier: UberIdentifier,
        value: Option<i32>,
        events: &[Event],
    ) {
        self.reach.state.best_orbs.insert(index, smallvec![]);

        match value {
            None => self.store_boolean(uber_identifier, true, events),
            Some(value) => {
                // logical states are incremental
                if self.fetch_integer(uber_identifier) < value {
                    self.store_integer(uber_identifier, value, events);
                }
            }
        }
    }

    fn traverse_logical_state(&mut self, index: usize, events: &[Event]) {
        debug_assert!(self.graph.nodes[index].is_logical_state());

        self.reach.state.best_orbs.insert(index, smallvec![]);

        if let Some(fails) = self.reach.state.logical_state_fails.remove(&index) {
            for fail in fails {
                self.progress(fail, events);
            }
        }
    }

    fn use_refills(&mut self, anchor: &Anchor, orb_variants: &mut OrbVariants, node_index: usize) {
        let max_orbs = self.max_orbs();
        if orb_variants[0] == max_orbs {
            return;
        }

        for (refill_index, refill) in anchor.refills.iter().enumerate() {
            if let Some(mut refill_orbs) = self.attempt_requirement(
                &refill.requirement,
                orb_variants.clone(),
                ConnectionIndex::refill(node_index, refill_index),
            ) {
                if matches!(refill.value, RefillValue::Full) {
                    // shortcut
                    *orb_variants = smallvec![max_orbs];
                    return;
                }

                self.refill(refill.value, &mut refill_orbs);
                *orb_variants = orbs::either(orb_variants, &refill_orbs);
            }
        }
    }

    fn traverse_connection(
        &mut self,
        connection: &Connection,
        mut orb_variants: OrbVariants,
        connection_index: ConnectionIndex,
        events: &[Event],
    ) {
        let ControlFlow::Continue(revisit) = self.should_visit(connection, &mut orb_variants)
        else {
            return;
        };

        trace!(
            "{identifier} -> {to_identifier} attempting connection",
            identifier = self.graph.nodes[connection_index.node_index].identifier(),
            to_identifier = self.graph.nodes[connection.to].identifier(),
        );

        if let Some(mut target_orbs) =
            self.attempt_requirement(&connection.requirement, orb_variants, connection_index)
        {
            if revisit && !self.should_still_visit(connection, &mut target_orbs) {
                return;
            }

            self.traverse(connection.to, target_orbs, events)
        }
    }

    fn should_visit(
        &self,
        connection: &Connection,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<(), bool> {
        match self.reach.state.best_orbs.get(&connection.to) {
            None => ControlFlow::Continue(false),
            Some(previous) => {
                if !self.graph.nodes[connection.to].is_anchor() {
                    return ControlFlow::Break(());
                }

                orb_variants
                    .retain(|orbs| previous.iter().any(|previous_orbs| previous_orbs < orbs));

                if orb_variants.is_empty() {
                    return ControlFlow::Break(());
                }

                ControlFlow::Continue(true)
            }
        }
    }

    fn should_still_visit(&self, connection: &Connection, orb_variants: &mut OrbVariants) -> bool {
        let previous = &self.reach.state.best_orbs[&connection.to];

        orb_variants.retain(|orbs| previous.iter().any(|previous_orbs| previous_orbs < orbs));

        if orb_variants.is_empty() {
            return false;
        }

        trace!(
            "revisiting {to_identifier} to improve previous orbs {previous_orbs} with {orbs}",
            to_identifier = self.graph.nodes[connection.to].identifier(),
            previous_orbs = previous.iter().format(" / "),
            orbs = orb_variants.iter().format(" / "),
        );

        true
    }

    fn attempt_requirement(
        &mut self,
        requirement: &Requirement,
        mut orb_variants: OrbVariants,
        connection: ConnectionIndex,
    ) -> Option<OrbVariants> {
        match self.is_met(requirement, &mut orb_variants) {
            ControlFlow::Continue(()) => Some(orb_variants),
            ControlFlow::Break(missing) => {
                trace!("missing {missing}");
                self.reach.state.add_fail(missing, connection);
                None
            }
        }
    }
}
