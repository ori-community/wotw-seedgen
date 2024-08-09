use super::World;
use crate::orbs::{self, OrbVariants};
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use wotw_seedgen_logic_language::output::{Connection, Node, RefillValue, Requirement};

pub const TP_ANCHOR: &str = "Teleporters";

#[derive(Debug, Default)]
pub struct ReachedLocations<'graph> {
    pub reached: Vec<&'graph Node>,
    pub progressions: Vec<(&'graph Requirement, OrbVariants)>,
}

impl<'graph, 'settings> World<'graph, 'settings> {
    pub(super) fn reach_recursion(
        &mut self,
        current_node_index: usize,
        mut best_orbs: OrbVariants,
        context: &mut ReachContext<'graph>,
    ) {
        context
            .best_orbs
            .insert(current_node_index, best_orbs.clone());
        let current_node = &self.graph.nodes[current_node_index];
        match current_node {
            Node::Anchor(anchor) => {
                let max_orbs = self.player.max_orbs();
                if best_orbs
                    .get(0)
                    .map_or(true, |first_orbs| first_orbs != &max_orbs)
                {
                    for refill in &anchor.refills {
                        let mut refill_orbs = self.player.is_met(
                            &refill.requirement,
                            &self.logic_states,
                            best_orbs.clone(),
                        );
                        if !refill_orbs.is_empty() {
                            if matches!(refill.value, RefillValue::Full) {
                                // shortcut
                                best_orbs = smallvec![max_orbs];
                                break;
                            }
                            self.player.refill(refill.value, &mut refill_orbs);
                            best_orbs = orbs::either(&best_orbs, &refill_orbs);
                        }
                    }
                }

                for connection in &anchor.connections {
                    if context.best_orbs.contains_key(&connection.to) {
                        // TODO loop with improved orbs?
                        continue;
                    }
                    let target_orbs = self.player.is_met(
                        &connection.requirement,
                        &self.logic_states,
                        best_orbs.clone(),
                    );
                    if target_orbs.is_empty() {
                        let mut states = vec![];
                        contained_states(&connection.requirement, &mut states);

                        if states.is_empty() {
                            if context.progression_check {
                                context
                                    .reached_locations
                                    .progressions
                                    .push((&connection.requirement, best_orbs.clone()));
                            }
                        } else {
                            for state in states {
                                context
                                    .state_progressions
                                    .entry(state)
                                    .or_default()
                                    .push((current_node_index, connection));
                            }
                        }
                    } else {
                        self.reach_recursion(connection.to, target_orbs, context);
                    }
                }
            }
            Node::Pickup(_) | Node::State(_) | Node::LogicalState(_) => {
                // TODO simulate uberState change? If that is implemented, it might affect the lookahead logic when doing placements though
                self.logic_states.insert(current_node_index);
                context.reached_locations.reached.push(current_node);
                self.follow_state_progressions(current_node_index, context);
            }
        }
    }
    pub(super) fn reached_by_teleporter(&mut self, context: &mut ReachContext<'graph>) {
        if context
            .best_orbs
            .iter()
            .any(|(index, orb_variants)| match &self.graph.nodes[*index] {
                Node::Anchor(anchor) => !self
                    .player
                    .is_met(
                        &anchor.teleport_restriction,
                        &self.logic_states,
                        orb_variants.clone(),
                    )
                    .is_empty(),
                _ => false,
            })
        {
            if let Some(tp_anchor) = self
                .graph
                .nodes
                .iter()
                .position(|node| node.identifier() == TP_ANCHOR)
            {
                if !context.best_orbs.contains_key(&tp_anchor) {
                    self.reach_recursion(tp_anchor, smallvec![self.player.max_orbs()], context);
                }
            }
        }
    }
    fn follow_state_progressions(&mut self, index: usize, context: &mut ReachContext<'graph>) {
        if let Some(connections) = context.state_progressions.get(&index) {
            for (from, connection) in connections.clone() {
                if context.best_orbs.contains_key(&connection.to) {
                    // TODO loop with improved orbs?
                    continue;
                }
                let target_orbs = self.player.is_met(
                    &connection.requirement,
                    &self.logic_states,
                    context.best_orbs[&from].clone(),
                );
                if !target_orbs.is_empty() {
                    self.reach_recursion(connection.to, target_orbs, context);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct ReachContext<'graph> {
    pub progression_check: bool,
    state_progressions: FxHashMap<usize, Vec<(usize, &'graph Connection)>>,
    best_orbs: FxHashMap<usize, OrbVariants>,
    pub reached_locations: ReachedLocations<'graph>,
}

// TODO this optimization existed previously for contained_states, is it relevant?
//     /// Checks whether this [`Requirement`] is possible to meet with the given settings
//     pub(crate) fn is_possible_for(&self, settings: &WorldSettings) -> bool {
//         match self {
//             Requirement::Impossible => false,
//             Requirement::Difficulty(difficulty) => settings.difficulty >= *difficulty,
//             Requirement::NormalGameDifficulty => !settings.hard,
//             Requirement::Trick(trick) => settings.tricks.contains(trick),
//             Requirement::And(nested) => nested
//                 .iter()
//                 .all(|requirement| requirement.is_possible_for(settings)),
//             Requirement::Or(nested) => nested
//                 .iter()
//                 .any(|requirement| requirement.is_possible_for(settings)),
//             _ => true,
//         }
//     }
fn contained_states(requirement: &Requirement, states: &mut Vec<usize>) {
    match requirement {
        Requirement::State(index) => states.push(*index),
        Requirement::And(nested) | Requirement::Or(nested) => {
            for requirement in nested {
                contained_states(requirement, states);
            }
        }
        _ => {}
    }
}
