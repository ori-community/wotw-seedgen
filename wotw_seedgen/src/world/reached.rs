use std::{
    fmt::{self, Display},
    hash::Hash,
    mem,
    ops::{ControlFlow, Deref},
};

use super::World;
use crate::{
    logical_difficulty::LogicalDifficulty,
    orbs::{self, format_orb_variants, OrbVariants},
    world::{graph_ref::EqIgnore, GraphRef, Missing},
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
    Skill, UberIdentifier,
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
    /// All reached nodes and if they are anchors, the best orbs they have been reached with
    best_orbs: FxHashMap<usize, OrbVariants>,
    /// [`TP_ANCHOR`] has been reached
    tp_reached: bool,
    fails: ReachStateFails<'graph>,
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

// TODO were these capacities good?
// best_orbs: FxHashMap::with_capacity_and_hasher(graph.nodes.len(), FxBuildHasher),
// tp_reached: false,
// uber_state_fails: FxHashMap::with_capacity_and_hasher(80, FxBuildHasher),
// logical_state_fails: FxHashMap::with_capacity_and_hasher(5, FxBuildHasher),
// orb_fail: false,

impl<'graph> ReachState<'graph> {
    fn clear(&mut self) {
        self.best_orbs.clear();
        self.tp_reached = false;
        self.fails.clear();
    }

    fn orb_fail(&self) -> bool {
        !(self.fails.health.is_empty() && self.fails.energy.is_empty())
    }
}

impl<'graph> ReachStateFails<'graph> {
    fn clear(&mut self) {
        self.uber_state.clear();
        self.logical_state.clear();
        self.health.clear();
        self.energy.clear();
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
        world: &World<'graph, '_>,
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
                write!(f, " -> {}", self.graph.nodes[connection.to].identifier())?
            }
        }

        self.connection.requirement.fmt(f)
    }
}

impl Display for ConnectionRequirement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(full) => write!(f, " -> {}", full.0),
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
            orb_variants = format_orb_variants(&self.orb_variants.0)
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
            Some(orb_variants) => orb_variants,
        }
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

        if let Some(fails) = self.reach.state.fails.uber_state.remove(&uber_identifier) {
            trace!("removed {uber_identifier} from UberState fails");
            for fail in fails {
                self.progress(fail, events);
            }
        }

        if !was_updating_reach {
            if self.reach.state.orb_fail()
                && self.settings.difficulty.may_increase_orbs(uber_identifier)
            {
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

    fn progress(&mut self, connection_index: ConnectionIndex<'graph>, events: &[Event]) {
        let orb_variants = self.get_connection_orbs(&connection_index).clone();

        match &connection_index.connection {
            ConnectionOrRefill::Refill(_) => {
                let node_index = connection_index.node_index(self.graph);
                self.traverse(node_index, orb_variants, events)
            }
            ConnectionOrRefill::Connection(connection) => {
                self.traverse_connection(connection.0, orb_variants, connection_index, events)
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
        anchor: &'graph Anchor,
        node_index: usize,
        mut orb_variants: OrbVariants,
        events: &[Event],
    ) {
        self.use_refills(anchor, &mut orb_variants);

        self.reach
            .state
            .best_orbs
            .insert(node_index, orb_variants.clone());

        self.attempt_teleport(anchor, events);

        for connection in &anchor.connections {
            self.traverse_connection(
                connection,
                orb_variants.clone(),
                ConnectionIndex::connection(anchor, connection),
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

        if let Some(fails) = self.reach.state.fails.logical_state.remove(&index) {
            for fail in fails {
                self.progress(fail, events);
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
        connection: &'graph Connection,
        mut orb_variants: OrbVariants,
        connection_index: ConnectionIndex<'graph>,
        events: &[Event],
    ) {
        let ControlFlow::Continue(revisit) = self.should_visit(connection, &mut orb_variants)
        else {
            return;
        };

        trace!(
            "attempting connection {}",
            connection_index.display(self.graph)
        );

        if let Some(mut target_orbs) = self.attempt_requirement(orb_variants, connection_index) {
            if revisit && !self.should_still_visit(connection, &mut target_orbs) {
                return;
            }

            self.traverse(connection.to, target_orbs, events)
        }
    }

    // TODO slightly incorrect I think, best_orbs is post-refill so it's not a fair comparison with the pre-refill orbs
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
            previous_orbs = format_orb_variants(previous),
            orbs = format_orb_variants(orb_variants),
        );

        true
    }

    fn attempt_requirement(
        &mut self,
        mut orb_variants: OrbVariants,
        mut connection: ConnectionIndex<'graph>,
    ) -> Option<OrbVariants> {
        match connection.is_met(self, &mut orb_variants) {
            ControlFlow::Continue(()) => Some(orb_variants),
            ControlFlow::Break(missing) => {
                trace!("missing {missing}");
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
                self.add_energy_or_better_weapon_fail::<true>(connection)
            }
            Missing::EnergyOrBetterEnemyWeapon(_) => {
                self.add_energy_or_better_weapon_fail::<false>(connection)
            }
            Missing::EnergyOrBurrowOrBetterEnemyWeapon(_) => {
                add_fail_to(
                    &mut self.reach.state.fails.uber_state,
                    Skill::BURROW_ID,
                    connection.clone(),
                );
                self.add_energy_or_better_weapon_fail::<false>(connection)
            }
            Missing::Any(options) => {
                for missing in options {
                    self.add_fail(missing, connection.clone());
                }
            }
            Missing::Or(ors, orb_variants) => {
                for (missing, requirement) in ors {
                    let connection = ConnectionIndex {
                        requirement: ConnectionRequirement::Partial(ConnectionRequirementPartial {
                            requirement,
                            orb_variants: orb_variants.clone(),
                        }),
                        ..connection.clone()
                    };

                    self.add_fail(missing, connection);
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

    fn add_energy_or_better_weapon_fail<const TARGET_IS_WALL: bool>(
        &mut self,
        connection: ConnectionIndex<'graph>,
    ) {
        self.reach.state.fails.energy.insert(connection.clone());
        self.add_any_skill_fail(
            connection,
            // TODO avoidable collect
            self.better_weapons::<TARGET_IS_WALL>().collect::<Vec<_>>(),
        );
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
    ) -> impl Iterator<Item = Skill> + '_ {
        let mut lowest_cost = Skill::Spear.energy_cost();
        let mut highest_dpe = Skill::Sentry.damage_per_energy(false);

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
}

fn add_fail_to<K: Eq + Hash, V: Eq + Hash>(map: &mut FxHashMap<K, FxHashSet<V>>, key: K, value: V) {
    map.entry(key).or_default().insert(value);
}
