use std::{mem, ops::ControlFlow};

use super::World;
use crate::{
    logical_difficulty,
    orbs::{self, OrbVariants, Orbs},
    world::is_met::Missing,
};
use itertools::Itertools;
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::smallvec;
use wotw_seedgen_assets::{LocDataEntry, StateDataEntry};
use wotw_seedgen_data::{Shard, Skill, UberIdentifier};
use wotw_seedgen_logic_language::output::{
    Anchor, Connection, Graph, Node, RefillValue, Requirement,
};
use wotw_seedgen_seed_language::output::{CommandBoolean, Event};

pub const TP_ANCHOR: &str = "Teleporters";

pub(crate) const ALL_CONNECTIONS: usize = usize::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Progression {
    pub node_index: usize,
    pub connection_index: usize,
}

// TODO figuring out how to update existing best_orbs with orb changes is NOT reasonable. Abort this idea.
#[derive(Debug, Clone)]
pub struct Reach {
    best_orbs: FxHashMap<usize, OrbVariants>,
    tp_reached: bool,
    pub(crate) uber_state_progressions: FxHashMap<UberIdentifier, FxHashSet<Progression>>,
    pub(crate) logical_state_progressions: FxHashMap<usize, FxHashSet<Progression>>,
    pub(crate) orb_progression: bool,
    pub(super) logic_state_map: FxHashMap<UberIdentifier, Vec<usize>>,
}

impl Reach {
    pub fn new(graph: &Graph) -> Self {
        let mut reach = Self {
            best_orbs: Default::default(),
            tp_reached: false,
            uber_state_progressions: Default::default(),
            logical_state_progressions: Default::default(),
            orb_progression: false,
            logic_state_map: Default::default(),
        };

        for (index, node) in graph.nodes.iter().enumerate() {
            if let Some(uber_identifier) = node.uber_identifier() {
                reach
                    .logic_state_map
                    .entry(uber_identifier)
                    .or_default()
                    .push(index);
            }
        }

        reach
    }

    fn clear(&mut self) {
        self.best_orbs.clear();
        self.tp_reached = false;
        self.uber_state_progressions.clear();
        self.logical_state_progressions.clear();
        self.orb_progression = false;
    }

    fn add_progression(&mut self, missing: Missing, progression: Progression) {
        match missing {
            Missing::Impossible => {}
            Missing::UberState(uber_identifier) => {
                self.add_uber_identifier_progression(uber_identifier, progression)
            }
            Missing::LogicalState(index) => self.add_logical_state_progression(index, progression),
            Missing::Orbs => {
                self.orb_progression = true;
            }
            Missing::Any(options) => {
                for missing in options {
                    self.add_progression(missing, progression.clone())
                }
            }
        }
    }

    fn add_uber_identifier_progression(
        &mut self,
        uber_identifier: UberIdentifier,
        progression: Progression,
    ) {
        self.uber_state_progressions
            .entry(uber_identifier)
            .or_default()
            .insert(progression);
    }

    fn add_logical_state_progression(&mut self, index: usize, progression: Progression) {
        self.logical_state_progressions
            .entry(index)
            .or_default()
            .insert(progression);
    }
}

impl World<'_, '_> {
    #[inline]
    pub fn reached_indices(&self) -> impl Iterator<Item = usize> + use<'_> {
        self.reach.best_orbs.keys().copied()
    }

    #[inline]
    pub fn reached_nodes(&self) -> impl Iterator<Item = &Node> {
        self.reached_indices().map(|index| &self.graph.nodes[index])
    }

    #[inline]
    pub fn reached_pickup_count(&self) -> usize {
        self.reached_nodes().filter(|node| node.is_pickup()).count()
    }

    #[inline]
    pub fn has_reached(&self, index: usize) -> bool {
        self.reach.best_orbs.contains_key(&index)
    }

    pub fn traverse_spawn(&mut self, events: &[Event]) {
        self.check_all_states(events);

        let orb_variants = smallvec![self.max_orbs()];
        self.traverse(self.spawn, orb_variants, events);

        self.attempt_spawn_teleport(events);
    }

    fn attempt_spawn_teleport(&mut self, events: &[Event]) {
        let reached_anchors = self
            .reach
            .best_orbs
            .keys()
            .filter_map(|node_index| self.graph.nodes[*node_index].try_as_anchor_ref())
            .collect::<Vec<_>>();

        for anchor in reached_anchors {
            self.attempt_teleport(anchor, events)
        }
    }

    fn attempt_teleport(&mut self, anchor: &Anchor, events: &[Event]) {
        if self.reach.tp_reached {
            return;
        }

        let mut orb_variants = smallvec![self.max_orbs()];

        if self
            .is_met(&anchor.teleport_restriction, &mut orb_variants)
            .is_continue()
        {
            if let Ok(tp_index) = self.graph.find_node(TP_ANCHOR) {
                self.reach.tp_reached = true;
                self.traverse(tp_index, orb_variants, events);
            }
        }
    }

    pub(super) fn update_reached(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        let was_updating_reach = mem::replace(&mut self.updating_reach, true);

        self.check_states_for(uber_identifier, events);

        if let Some(progressions) = self.reach.uber_state_progressions.remove(&uber_identifier) {
            for progression in progressions {
                self.progress(progression, events);
            }
        }

        if !was_updating_reach {
            if self.reach.orb_progression && self.may_increase_orbs(uber_identifier) {
                self.reach.clear();

                self.traverse_spawn(events);
            }

            self.updating_reach = false;
        }
    }

    pub(super) fn mark_reached(
        &mut self,
        index: usize,
        orb_variants: OrbVariants,
        events: &[Event],
    ) {
        self.reach.best_orbs.insert(index, orb_variants);

        if let Some(progressions) = self.reach.logical_state_progressions.remove(&index) {
            for progression in progressions {
                self.progress(progression, events);
            }
        }
    }

    fn check_all_states(&mut self, events: &[Event]) {
        let logic_states = self
            .reach
            .logic_state_map
            .values()
            .flat_map(|logic_states| self.filter_unreached_states(logic_states))
            .collect::<Vec<_>>();

        self.check_states(logic_states, events);
    }

    fn check_states_for(&mut self, uber_identifier: UberIdentifier, events: &[Event]) {
        if let Some(logic_states) = self.reach.logic_state_map.get(&uber_identifier) {
            let logic_states = self.filter_unreached_states(logic_states);
            self.check_states(logic_states, events);
        }
    }

    fn filter_unreached_states(&self, logic_states: &[usize]) -> Vec<usize> {
        logic_states
            .iter()
            .filter(|index| !self.has_reached(**index))
            .copied()
            .collect()
    }

    fn check_states(&mut self, logic_states: Vec<usize>, events: &[Event]) {
        for index in logic_states {
            let node = &self.graph.nodes[index];
            let uber_identifier = node.uber_identifier().unwrap();
            // TODO less hardcoded solution?
            let node_condition_f = if uber_identifier.is_door() {
                CommandBoolean::door_condition
            } else {
                CommandBoolean::loc_data_condition
            };
            let condition = node_condition_f(uber_identifier, node.value());

            // node conditions don't change UberStates
            if self.simulate(&condition, &[]) {
                self.mark_reached(index, smallvec![Orbs::default()], events);
            }
        }
    }

    fn may_increase_orbs(&self, uber_identifier: UberIdentifier) -> bool {
        match uber_identifier {
            UberIdentifier::MAX_HEALTH | UberIdentifier::MAX_ENERGY | Skill::REGENERATE_ID => true,
            Shard::RESILIENCE_ID => self.settings.difficulty >= logical_difficulty::RESILIENCE,
            Shard::VITALITY_ID => self.settings.difficulty >= logical_difficulty::VITALITY,
            Shard::ENERGY_ID => self.settings.difficulty >= logical_difficulty::ENERGY_SHARD,
            Shard::OVERCHARGE_ID => self.settings.difficulty >= logical_difficulty::OVERCHARGE,
            Shard::LIFE_PACT_ID => self.settings.difficulty >= logical_difficulty::LIFE_PACT,
            Shard::OVERFLOW_ID => self.settings.difficulty >= logical_difficulty::OVERFLOW,
            Shard::CATALYST_ID => self.settings.difficulty >= logical_difficulty::CATALYST,
            _ => false,
        }
    }

    fn progress(&mut self, progression: Progression, events: &[Event]) {
        let anchor = self.graph.nodes[progression.node_index].expect_anchor();
        let orb_variants = self.reach.best_orbs[&progression.node_index].clone();

        if progression.connection_index == ALL_CONNECTIONS {
            self.traverse(progression.node_index, orb_variants, events);
        } else {
            let connection = &anchor.connections[progression.connection_index];
            self.traverse_connection(connection, orb_variants, progression, events);
        }
    }

    fn traverse(&mut self, node_index: usize, mut orb_variants: OrbVariants, events: &[Event]) {
        let node = &self.graph.nodes[node_index];

        trace!(
            "[{identifier}] reached with {best_orbs}",
            identifier = node.identifier(),
            best_orbs = orb_variants.iter().format(" or "),
        );

        self.mark_reached(node_index, orb_variants.clone(), events);

        match node {
            Node::Anchor(anchor) => {
                let progression = Progression {
                    node_index,
                    connection_index: ALL_CONNECTIONS,
                };
                self.use_refills(anchor, &mut orb_variants, progression);

                self.attempt_teleport(anchor, events);

                for (connection_index, connection) in anchor.connections.iter().enumerate() {
                    let progression = Progression {
                        node_index,
                        connection_index,
                    };

                    self.traverse_connection(connection, orb_variants.clone(), progression, events)
                }
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
            }) => match value {
                None => self.set_boolean(*uber_identifier, true, events),
                Some(value) => {
                    // logical states are incremental
                    if self.uber_states.get(*uber_identifier).as_integer() < *value {
                        self.set_integer(*uber_identifier, *value, events);
                    }
                }
            },
            Node::LogicalState(_) => {}
        }
    }

    fn use_refills(
        &mut self,
        anchor: &Anchor,
        orb_variants: &mut OrbVariants,
        progression: Progression,
    ) {
        let max_orbs = self.max_orbs();
        if orb_variants[0] == max_orbs {
            return;
        }

        for refill in &anchor.refills {
            if let Some(mut refill_orbs) = self.attempt_requirement(
                &refill.requirement,
                orb_variants.clone(),
                progression.clone(),
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
        orb_variants: OrbVariants,
        progression: Progression,
        events: &[Event],
    ) {
        if self.reach.best_orbs.contains_key(&connection.to) {
            // TODO loop with improved orbs?
            return;
        }

        trace!(
            "[{identifier}] -> [{to_identifier}] attempting connection",
            identifier = self.graph.nodes[progression.node_index].identifier(),
            to_identifier = self.graph.nodes[connection.to].identifier(),
        );

        if let Some(target_orbs) =
            self.attempt_requirement(&connection.requirement, orb_variants, progression)
        {
            self.traverse(connection.to, target_orbs, events)
        }
    }

    fn attempt_requirement(
        &mut self,
        requirement: &Requirement,
        mut orb_variants: OrbVariants,
        progression: Progression,
    ) -> Option<OrbVariants> {
        match self.is_met(requirement, &mut orb_variants) {
            ControlFlow::Continue(()) => Some(orb_variants),
            ControlFlow::Break(missing) => {
                trace!("missing {missing:?}");
                self.reach.add_progression(missing, progression);
                None
            }
        }
    }
}
