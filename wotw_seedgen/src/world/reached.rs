use std::{
    fmt::{self, Display},
    hash::Hash,
    ops::{ControlFlow, Deref},
};

use super::World;
use crate::{
    logical_difficulty::{LogicalDifficulty, SHIELD_WEAPONS},
    orb_variants,
    orbs::OrbVariants,
    perf_data::PerfData,
    world::{is_met::MissingWeaponKind, GraphRef, Missing, ReachUpdateState},
};
use arrayvec::ArrayVec;
use itertools::Itertools;
use log::{trace, warn};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use wotw_seedgen_data::{
    assets::{LocDataEntry, StateDataEntry},
    logic_language::output::{Anchor, Connection, Graph, Node, Refill, RefillValue, Requirement},
    seed_language::{
        output::CommandsOutput,
        simulate::{CloneSnapshot, Simulation, Snapshot},
    },
    EqIgnore, Skill, UberIdentifier,
};

pub const TP_ANCHOR: &str = "Teleporters";

// TODO figuring out how to update existing best_orbs with orb changes is NOT reasonable. Abort this idea.
#[derive(Debug)]
pub struct Reach<'graph> {
    state: CloneSnapshot<ReachState<'graph>>,
    logic_state_map: LogicStateMap,
}

impl<'graph> Reach<'graph> {
    pub fn new(graph: &'graph Graph) -> Self {
        Self {
            state: CloneSnapshot::default(),
            logic_state_map: LogicStateMap::new(graph),
        }
    }
}

impl Snapshot for Reach<'_> {
    fn snapshot(&mut self) {
        self.state.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.state.restore_snapshot();
    }
}

#[derive(Debug, Clone, Default)]
struct ReachState<'graph> {
    // TODO try Vec? Maybe while also putting anchors to the front?
    /// All reached nodes and if they are anchors, the best orbs they have been reached with
    best_orbs: FxHashMap<usize, BestOrbs>,
    /// [`TP_ANCHOR`] has been reached
    tp_reached: bool,
    fails: ReachStateFails<'graph>,
}

// TODO are we hurting ourselves by increasing this type size?
#[derive(Debug, Clone)]
struct BestOrbs {
    pre_refills: OrbVariants,
    post_refills: OrbVariants,
    do_not_clear: bool,
}

impl BestOrbs {
    fn new(pre_refills: OrbVariants) -> Self {
        Self {
            pre_refills,
            post_refills: orb_variants![],
            do_not_clear: false,
        }
    }

    fn placeholder() -> Self {
        Self::new(orb_variants![])
    }

    fn do_not_clear(&mut self) {
        self.do_not_clear = true;
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ReachStateFails<'graph> {
    /// All [`ConnectionIndex`] which failed to solve and might be solved by advancing the [`UberIdentifier`]
    pub uber_state: FxHashMap<UberIdentifier, FxHashSet<ConnectionIndex<'graph>>>,
    /// All [`ConnectionIndex`] which failed to solve and might be solved by reaching the logical state
    pub logical_state: FxHashMap<usize, FxHashSet<ConnectionIndex<'graph>>>,
    /// Some connections failed to solve and might require more health.
    /// Resuming progress along those connections would be very hard because of refill logic,
    /// So we just reset the entire Reach when progressing orbs.
    pub health: FxHashSet<ConnectionIndex<'graph>>,
    /// Same as `health_fail`, but for energy.
    pub energy: FxHashSet<ConnectionIndex<'graph>>,
}

impl ReachState<'_> {
    fn clear(&mut self) {
        self.best_orbs.retain(|_, best_orbs| best_orbs.do_not_clear);
        self.tp_reached = false;
        self.fails.clear();
    }
}

impl<'graph> ReachStateFails<'graph> {
    fn is_empty(&self) -> bool {
        self.uber_state.is_empty()
            && self.logical_state.is_empty()
            && self.health.is_empty()
            && self.energy.is_empty()
    }

    fn clear(&mut self) {
        self.uber_state.clear();
        self.logical_state.clear();
        self.health.clear();
        self.energy.clear();
    }

    #[cfg(test)]
    pub fn display<'fails>(
        &'fails self,
        graph: &'graph Graph,
    ) -> ReachStateFailsDisplay<'fails, 'graph> {
        ReachStateFailsDisplay { fails: self, graph }
    }
}

#[cfg(test)]
pub struct ReachStateFailsDisplay<'fails, 'graph> {
    fails: &'fails ReachStateFails<'graph>,
    graph: &'graph Graph,
}

#[cfg(test)]
impl Display for ReachStateFailsDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_connections<'fails, 'graph>(
            connections: &'fails FxHashSet<ConnectionIndex<'graph>>,
            graph: &'graph Graph,
        ) -> impl Display + use<'fails, 'graph> {
            connections
                .iter()
                .map(|connection| connection.display(graph))
                .format(", ")
        }

        self.fails
            .uber_state
            .iter()
            .format_with(", ", |(uber_identifier, connections), f| {
                f(&format_args!(
                    "{uber_identifier} for [{connections}]",
                    connections = format_connections(connections, self.graph),
                ))
            })
            .fmt(f)?;

        let mut comma = !self.fails.uber_state.is_empty();
        if comma {
            write!(f, ", ")?;
        }

        self.fails
            .logical_state
            .iter()
            .format_with(", ", |(state, connections), f| {
                f(&format_args!(
                    "{{{state}}} for [{connections}]",
                    connections = format_connections(connections, self.graph),
                ))
            })
            .fmt(f)?;

        comma |= !self.fails.logical_state.is_empty();
        if comma {
            write!(f, ", ")?;
        }

        if !self.fails.health.is_empty() {
            write!(
                f,
                "Health for [{connections}]",
                connections = format_connections(&self.fails.health, self.graph),
            )?;
        }

        if !self.fails.energy.is_empty() {
            write!(
                f,
                "Energy for [{connections}]",
                connections = format_connections(&self.fails.energy, self.graph),
            )?;
        }

        Ok(())
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConnectionIndex<'graph> {
    pub anchor: GraphRef<'graph, Anchor>,
    pub connection: ConnectionOrRefill<'graph>,
    pub requirement: ConnectionRequirement<'graph>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ConnectionOrRefill<'graph> {
    Refill(GraphRef<'graph, Refill>),
    Connection(GraphRef<'graph, Connection>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConnectionRequirement<'graph> {
    Full(GraphRef<'graph, Requirement>),
    Partial(ConnectionRequirementPartial<'graph>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConnectionRequirementPartial<'graph> {
    pub requirement: GraphRef<'graph, Requirement>,
    pub orb_variants: EqIgnore<OrbVariants>,
}

impl<'graph> ConnectionIndex<'graph> {
    pub(crate) fn connection(anchor: &'graph Anchor, connection: &'graph Connection) -> Self {
        Self {
            anchor: GraphRef(anchor),
            connection: ConnectionOrRefill::Connection(GraphRef(connection)),
            requirement: ConnectionRequirement::Full(GraphRef(&connection.requirement)),
        }
    }

    pub(crate) fn refill(anchor: &'graph Anchor, refill: &'graph Refill) -> Self {
        Self {
            anchor: GraphRef(anchor),
            connection: ConnectionOrRefill::Refill(GraphRef(refill)),
            requirement: ConnectionRequirement::Full(GraphRef(&refill.requirement)),
        }
    }

    pub(crate) fn node_index(&self, graph: &'graph Graph) -> usize {
        self.anchor.index(&graph.nodes)
    }

    pub(crate) fn orb_reset(&mut self) {
        self.requirement = ConnectionRequirement::Full(GraphRef(self.connection.requirement()));
    }

    pub(crate) fn is_met(
        &mut self,
        world: &World<'graph, '_, '_, '_>,
        orb_variants: &mut OrbVariants,
    ) -> ControlFlow<Missing<'graph>> {
        if let ConnectionRequirement::Partial(partial) = &self.requirement {
            world.is_met(partial.requirement.0, &mut partial.orb_variants.0.clone())?;

            self.requirement = ConnectionRequirement::Full(GraphRef(self.connection.requirement()));
        }

        let ConnectionRequirement::Full(requirement) = self.requirement else {
            unreachable!()
        };

        world.is_met(requirement.0, orb_variants)
    }

    pub(crate) fn display<'index>(
        &'index self,
        graph: &'graph Graph,
    ) -> ConnectionIndexDisplay<'index, 'graph> {
        ConnectionIndexDisplay {
            connection: self,
            graph,
        }
    }
}

pub(crate) struct ConnectionIndexDisplay<'index, 'graph> {
    connection: &'index ConnectionIndex<'graph>,
    graph: &'graph Graph,
}

impl Display for ConnectionIndexDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.connection.anchor.identifier.fmt(f)?;

        match self.connection.connection {
            ConnectionOrRefill::Refill(refill) => write!(f, " -> {}", refill.value)?,
            ConnectionOrRefill::Connection(connection) => {
                write!(f, " -> {}", self.graph.nodes[connection.to].identifier())?;
            }
        }

        self.connection.requirement.fmt(f)
    }
}

impl Display for ConnectionRequirement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(_) => Ok(()),
            Self::Partial(partial) => partial.fmt(f),
        }
    }
}

impl Display for ConnectionRequirementPartial<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            " -> {requirement} [{orb_variants}]",
            requirement = self.requirement.0,
            orb_variants = self.orb_variants,
        )
    }
}

impl<'graph> ConnectionOrRefill<'graph> {
    pub(crate) fn requirement(&self) -> &'graph Requirement {
        match self {
            Self::Refill(refill) => &refill.0.requirement,
            Self::Connection(connection) => &connection.0.requirement,
        }
    }
}

impl<'graph> World<'graph, '_, '_, '_> {
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

    pub fn reached_ks_cost(&self) -> usize {
        let mut keystone_cost = 0;

        for state in self.reached_nodes().filter_map(Node::try_as_state_ref) {
            match state.identifier.as_str() {
                // TODO derive from logic? Also maybe don't recompute all the time?
                "MarshSpawn.KeystoneDoor" => keystone_cost += 2,
                "HowlsDen.KeystoneDoor" => keystone_cost += 2,
                "MarshPastOpher.EyestoneDoor" => keystone_cost += 2,
                "MidnightBurrows.KeystoneDoor" => keystone_cost += 4,
                "WoodsEntry.KeystoneDoor" => keystone_cost += 2,
                "WoodsMain.KeystoneDoor" => keystone_cost += 4,
                "LowerReach.KeystoneDoor" => keystone_cost += 4,
                "UpperReach.KeystoneDoor" => keystone_cost += 4,
                "UpperDepths.EntryKeystoneDoor" => keystone_cost += 2,
                "UpperDepths.CentralKeystoneDoor" => keystone_cost += 2,
                "UpperPools.KeystoneDoor" => keystone_cost += 4,
                "UpperWastes.KeystoneDoor" => keystone_cost += 2,
                _ => {}
            }
        }

        keystone_cost
    }

    #[inline]
    pub fn has_reached(&self, index: usize) -> bool {
        self.reach.state.best_orbs.contains_key(&index)
    }

    pub fn traverse_spawn(&mut self, output: &CommandsOutput) {
        self.check_all_states();

        let orb_variants = orb_variants![self.max_orbs()];
        self.traverse(self.spawn, orb_variants, output);

        self.attempt_spawn_teleport(output);
    }

    pub(crate) fn set_logical_state(&mut self, identifier: &str) {
        match self.graph.find_node(identifier) {
            Ok(index) => {
                if self.graph.nodes[index].is_anchor() {
                    warn!(logger: self.log_capture, "Attempted to set anchor \"{identifier}\" as logical state");
                } else {
                    let mut best_orbs = BestOrbs::placeholder();
                    best_orbs.do_not_clear();
                    self.reach.state.best_orbs.insert(index, best_orbs);
                }
            }
            Err(err) => warn!(logger: self.log_capture, "Cannot set logical state: {err}"),
        }
    }

    pub(crate) fn fails(&self) -> &ReachStateFails<'graph> {
        &self.reach.state.fails
    }

    pub(crate) fn get_connection_orbs<'orbs>(
        &'orbs self,
        connection_index: &'orbs ConnectionIndex<'graph>,
    ) -> &'orbs OrbVariants {
        let node_index = connection_index.node_index(self.graph);

        match self.reach.state.best_orbs.get(&node_index) {
            None => panic!(
                "Failed to get connection!\nInventory: {}\nReached: {}\nTried connection: {}",
                self.inventory_display(),
                self.reached_nodes()
                    .filter_map(Node::try_as_anchor_ref)
                    .map(|anchor| &anchor.identifier)
                    .format(", "),
                connection_index.display(self.graph)
            ),
            Some(best_orbs) => {
                debug_assert!(
                    !best_orbs.post_refills.is_empty(),
                    "encountered empty post-refill orbs in connection {}",
                    connection_index.display(self.graph)
                );

                &best_orbs.post_refills
            }
        }
    }

    fn attempt_spawn_teleport(&mut self, output: &CommandsOutput) {
        let reached_anchors = self
            .reach
            .state
            .best_orbs
            .keys()
            .filter_map(|node_index| self.graph.nodes[*node_index].try_as_anchor_ref())
            .collect::<Vec<_>>();

        for anchor in reached_anchors {
            self.attempt_teleport(anchor, output);
        }
    }

    fn attempt_teleport(&mut self, anchor: &'graph Anchor, output: &CommandsOutput) {
        if self.reach.state.tp_reached {
            return;
        }

        let mut orb_variants = orb_variants![self.max_orbs()];

        if self
            .is_met(&anchor.teleport_restriction, &mut orb_variants)
            .is_continue()
        {
            if let Ok(tp_index) = self.graph.find_node(TP_ANCHOR) {
                self.reach.state.tp_reached = true;
                self.traverse(tp_index, orb_variants, output);
            }
        }
    }

    pub(super) fn update_reached(
        &mut self,
        uber_identifier: UberIdentifier,
        output: &CommandsOutput,
    ) {
        trace!(
            logger: self.log_capture,
            "updating reach for {uber_identifier} with {inventory}",
            inventory = self.inventory_display(),
        );

        let was_idle = matches!(self.reach_update_state, ReachUpdateState::Idle);
        if was_idle {
            self.reach_update_state = ReachUpdateState::Updating;
        }

        self.check_states_for(uber_identifier);

        if let Some(fails) = self.reach.state.fails.uber_state.remove(&uber_identifier) {
            trace!(logger: self.log_capture, "removed {uber_identifier} from UberState fails");
            for fail in fails {
                self.progress(fail, output);
            }
        }

        if self.settings.difficulty.may_increase_orbs(uber_identifier)
            && !self.reach.state.fails.is_empty()
        {
            self.reach_update_state = ReachUpdateState::PendingOrbReset;
        }

        if was_idle {
            if matches!(self.reach_update_state, ReachUpdateState::PendingOrbReset) {
                trace!(logger: self.log_capture, "resetting reach after orb change");

                self.reach.state.clear();

                self.traverse_spawn(output);
            }

            self.reach_update_state = ReachUpdateState::Idle;
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
                    logger: self.log_capture,
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

            let met = if uber_identifier.is_entrance() {
                self.entrance_condition_met(uber_identifier, node.value().unwrap())
            } else {
                self.loc_data_condition_met(uber_identifier, node.value())
            };

            if met {
                self.reach
                    .state
                    .best_orbs
                    .insert(index, BestOrbs::placeholder());
            }
        }
    }

    fn progress(&mut self, connection_index: ConnectionIndex<'graph>, output: &CommandsOutput) {
        let orb_variants = self.get_connection_orbs(&connection_index).clone();

        match &connection_index.connection {
            ConnectionOrRefill::Refill(_) => {
                let node_index = connection_index.node_index(self.graph);
                self.traverse(node_index, orb_variants, output);
            }
            ConnectionOrRefill::Connection(connection) => {
                self.traverse_connection(connection.0, orb_variants, connection_index, output);
            }
        }
    }

    fn traverse(&mut self, node_index: usize, orb_variants: OrbVariants, output: &CommandsOutput) {
        let node = &self.graph.nodes[node_index];

        trace!(
            logger: self.log_capture,
            "{identifier} reached with {orb_variants}",
            identifier = node.identifier(),
        );

        match node {
            Node::Anchor(anchor) => self.traverse_anchor(anchor, node_index, orb_variants, output),
            Node::Pickup(LocDataEntry {
                uber_identifier,
                value,
                ..
            })
            | Node::State(StateDataEntry {
                uber_identifier,
                value,
                ..
            }) => self.traverse_state(node_index, *uber_identifier, *value, output),
            Node::LogicalState(_) => self.traverse_logical_state(node_index, output),
        }
    }

    fn traverse_anchor(
        &mut self,
        anchor: &'graph Anchor,
        node_index: usize,
        mut orb_variants: OrbVariants,
        output: &CommandsOutput,
    ) {
        let mut best_orbs = BestOrbs::new(orb_variants.clone());

        self.use_refills(anchor, &mut orb_variants);

        best_orbs.post_refills = orb_variants.clone();
        self.reach.state.best_orbs.insert(node_index, best_orbs);

        self.attempt_teleport(anchor, output);

        for connection in &anchor.connections {
            self.traverse_connection(
                connection,
                orb_variants.clone(),
                ConnectionIndex::connection(anchor, connection),
                output,
            );
        }
    }

    fn traverse_state(
        &mut self,
        index: usize,
        uber_identifier: UberIdentifier,
        value: Option<i32>,
        output: &CommandsOutput,
    ) {
        self.reach
            .state
            .best_orbs
            .insert(index, BestOrbs::placeholder());

        match value {
            None => self.store_boolean(uber_identifier, true, output),
            Some(value) => {
                // logical states are incremental
                if self.fetch_integer(uber_identifier) < value {
                    self.store_integer(uber_identifier, value, output);
                }
            }
        }
    }

    fn traverse_logical_state(&mut self, index: usize, output: &CommandsOutput) {
        debug_assert!(self.graph.nodes[index].is_logical_state());

        self.reach
            .state
            .best_orbs
            .insert(index, BestOrbs::placeholder());

        if let Some(fails) = self.reach.state.fails.logical_state.remove(&index) {
            for fail in fails {
                self.progress(fail, output);
            }
        }
    }

    fn use_refills(&mut self, anchor: &'graph Anchor, orb_variants: &mut OrbVariants) {
        let max_orbs = self.max_orbs();
        if orb_variants[0] == max_orbs {
            return;
        }

        for refill in &anchor.refills {
            let connection_index = ConnectionIndex::refill(anchor, refill);

            if let Some(mut refill_orbs) =
                self.attempt_requirement(orb_variants.clone(), connection_index)
            {
                if matches!(refill.value, RefillValue::Full) {
                    // shortcut
                    *orb_variants = orb_variants![max_orbs];
                    return;
                }

                self.refill(refill.value, &mut refill_orbs);
                orb_variants.insert_alternative(refill_orbs);
            }
        }
    }

    fn traverse_connection(
        &mut self,
        connection: &'graph Connection,
        mut orb_variants: OrbVariants,
        connection_index: ConnectionIndex<'graph>,
        output: &CommandsOutput,
    ) {
        let ControlFlow::Continue(revisit) = self.should_visit(connection, &mut orb_variants)
        else {
            return;
        };

        trace!(
            logger: self.log_capture,
            "attempting connection {}",
            connection_index.display(self.graph)
        );

        let record = self.perf_data.and_then(PerfData::reached_start);

        let target_orbs = self.attempt_requirement(orb_variants, connection_index.clone());

        if let Some(record) = record {
            self.perf_data
                .unwrap()
                .reached_finish(record, connection_index.clone());
        }

        if let Some(mut target_orbs) = target_orbs {
            if revisit {
                let previous_orbs = &self.reach.state.best_orbs[&connection.to].pre_refills;
                let new_orbs = OrbVariants::alternatives(previous_orbs.clone(), target_orbs);

                let display = fmt::from_fn(|f| {
                    write!(
                        f,
                        "previous visit's orbs {previous_orbs} through {connection} with {new_orbs}",
                        connection = connection_index.display(self.graph),
                    )
                });

                if new_orbs.iter().any(|new_orbs| {
                    !previous_orbs
                        .iter()
                        .any(|previous_orbs| previous_orbs >= new_orbs)
                }) {
                    trace!(logger: self.log_capture, "improving {display}");

                    target_orbs = new_orbs;
                } else {
                    trace!(logger: self.log_capture, "cannot improve {display}");

                    return;
                }
            }

            self.traverse(connection.to, target_orbs, output);
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

                let pre_refills = &previous.pre_refills;
                orb_variants.retain(|orbs| {
                    !pre_refills
                        .iter()
                        .any(|previous_orbs| previous_orbs >= orbs)
                });

                if orb_variants.is_empty() {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(true)
                }
            }
        }
    }

    fn attempt_requirement(
        &mut self,
        mut orb_variants: OrbVariants,
        mut connection: ConnectionIndex<'graph>,
    ) -> Option<OrbVariants> {
        match connection.is_met(self, &mut orb_variants) {
            ControlFlow::Continue(()) => Some(orb_variants),
            ControlFlow::Break(missing) => {
                trace!(logger: self.log_capture, "missing {missing}");
                self.add_fail(missing, connection);
                None
            }
        }
    }

    fn add_fail(&mut self, missing: Missing<'graph>, connection: ConnectionIndex<'graph>) {
        match missing {
            Missing::Impossible => {}
            // TODO optimize by using the missing integer value and skipping reach attempts?
            Missing::Boolean(uber_identifier) | Missing::Integer(uber_identifier, _) => {
                add_fail_to(
                    &mut self.reach.state.fails.uber_state,
                    uber_identifier,
                    connection,
                );
            }
            Missing::LogicalState(index) => {
                add_fail_to(&mut self.reach.state.fails.logical_state, index, connection);
            }
            Missing::Health(_) => {
                self.reach.state.fails.health.insert(connection);
            }
            Missing::Energy(_) => {
                self.reach.state.fails.energy.insert(connection);
            }
            Missing::WallWeapon => self.add_weapon_fail::<true>(connection),
            Missing::EnemyWeapon => self.add_weapon_fail::<false>(connection),
            Missing::EnergyOrBetterWallWeapon(_) => {
                self.add_energy_or_better_wall_weapon_fail(connection);
            }
            Missing::EnergyOrBetterCombatWeapon {
                amount: _,
                burrow_reduces_cost,
                weapon,
            } => self.add_energy_or_better_combat_weapon_fail(
                connection,
                burrow_reduces_cost,
                weapon,
            ),
            Missing::Any(options) => {
                for missing in options {
                    self.add_fail(missing, connection.clone());
                }
            }
            Missing::Or(ors, _) => {
                for (missing, _) in ors {
                    self.add_fail(missing, connection.clone());
                }
            }
        }
    }

    fn add_weapon_fail<const TARGET_IS_WALL: bool>(&mut self, connection: ConnectionIndex<'graph>) {
        self.add_any_skill_fail(
            connection,
            self.settings.difficulty.weapons_iter::<TARGET_IS_WALL>(),
        );
    }

    fn add_energy_or_better_wall_weapon_fail(&mut self, connection: ConnectionIndex<'graph>) {
        self.reach.state.fails.energy.insert(connection.clone());
        self.add_any_skill_fail(
            connection,
            // TODO avoidable collect
            self.better_weapons::<true>().collect::<Vec<_>>(),
        );
    }

    fn add_energy_or_better_combat_weapon_fail(
        &mut self,
        connection: ConnectionIndex<'graph>,
        burrow_reduces_cost: bool,
        weapon: MissingWeaponKind,
    ) {
        self.reach.state.fails.energy.insert(connection.clone());

        if burrow_reduces_cost {
            add_fail_to(
                &mut self.reach.state.fails.uber_state,
                Skill::BURROW_ID,
                connection.clone(),
            );
        }

        let skills = match weapon {
            MissingWeaponKind::Any => self.better_weapons::<false>().collect(),
            MissingWeaponKind::Ranged => self.better_ranged_weapons().collect(),
            MissingWeaponKind::Shield => self.better_shield_weapons().collect(),
            MissingWeaponKind::RangedOrShield => self.better_ranged_or_shield_weapons(),
        };

        self.add_any_skill_fail(connection, skills);
    }

    fn add_any_skill_fail<I>(&mut self, connection: ConnectionIndex<'graph>, skills: I)
    where
        I: IntoIterator<Item = Skill>,
    {
        // TODO avoidable collect
        for weapon in skills {
            add_fail_to(
                &mut self.reach.state.fails.uber_state,
                weapon.uber_identifier(),
                connection.clone(),
            );
        }
    }

    pub(crate) fn better_weapons<const TARGET_IS_WALL: bool>(
        &self,
    ) -> impl Iterator<Item = Skill> + use<'_, 'graph, TARGET_IS_WALL> {
        self.better_weapons_from(
            Skill::Spear.energy_cost(),
            Skill::Sentry.damage_per_energy(false),
            self.settings.difficulty.weapons::<TARGET_IS_WALL>(),
        )
    }

    pub(crate) fn better_ranged_weapons(&self) -> impl Iterator<Item = Skill> + use<'_, 'graph> {
        self.better_weapons_from(
            Skill::Spear.energy_cost(),
            Skill::Sentry.damage_per_energy(false),
            self.settings.difficulty.ranged_weapons(),
        )
    }

    pub(crate) fn better_shield_weapons(&self) -> impl Iterator<Item = Skill> + use<'_, 'graph> {
        self.better_weapons_from(
            Skill::Spear.energy_cost(),
            Skill::Sentry.damage_per_energy(false),
            &SHIELD_WEAPONS,
        )
    }

    // cap 9 allows storing this in the same type as better_weapons and is only one higher than necessary
    pub(crate) fn better_ranged_or_shield_weapons(&self) -> ArrayVec<Skill, 9> {
        let mut weapons = self.better_ranged_weapons().collect::<ArrayVec<_, _>>();

        for weapon in self.better_shield_weapons() {
            if !weapons.contains(&weapon) {
                weapons.push(weapon);
            }
        }

        weapons
    }

    pub(crate) fn better_weapons_from<'a>(
        &'a self,
        mut lowest_cost: f32,
        mut highest_dpe: f32,
        weapons: &'a [Skill],
    ) -> impl Iterator<Item = Skill> + use<'a, 'graph> {
        for owned in self.owned_weapons_from(weapons) {
            let cost = owned.energy_cost();
            lowest_cost = lowest_cost.min(cost);
            highest_dpe = highest_dpe
                .max(owned.total_damage(self.settings.difficulty.charge_grenade()) / cost);
        }

        weapons.iter().copied().filter(move |weapon| {
            weapon.energy_cost() < lowest_cost
                || weapon.damage_per_energy(self.settings.difficulty.charge_grenade()) > highest_dpe
        })
    }
}

fn add_fail_to<K: Eq + Hash, V: Eq + Hash>(map: &mut FxHashMap<K, FxHashSet<V>>, key: K, value: V) {
    map.entry(key).or_default().insert(value);
}
