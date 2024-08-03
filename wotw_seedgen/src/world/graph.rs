use std::fmt;

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::smallvec;

use super::{player::Player, requirement::Requirement};
use crate::generator::NodeSummary;
use crate::uber_state::{UberIdentifier, UberStateTrigger};
use crate::util::{
    constants::TP_ANCHOR,
    orbs::{self, OrbVariants},
    NodeKind, Position, RefillValue, Zone,
};

#[derive(Debug)]
pub struct Refill {
    pub value: RefillValue,
    pub requirement: Requirement,
}

#[derive(Debug)]
pub struct Connection {
    pub to: usize,
    pub requirement: Requirement,
}

#[derive(Debug)]
pub struct Anchor {
    pub identifier: String,
    pub position: Option<Position>,
    pub can_spawn: bool,
    pub teleport_restriction: Requirement,
    pub index: usize,
    pub refills: Vec<Refill>,
    pub connections: Vec<Connection>,
}
#[derive(Debug)]
pub struct Pickup {
    pub identifier: String,
    pub position: Option<Position>,
    pub map_position: Option<Position>,
    pub zone: Zone,
    pub index: usize,
    pub trigger: UberStateTrigger,
}
#[derive(Debug)]
pub struct State {
    pub identifier: String,
    pub index: usize,
    pub trigger: Option<UberStateTrigger>,
}
#[derive(Debug)]
pub struct Quest {
    pub identifier: String,
    pub position: Option<Position>,
    pub map_position: Option<Position>,
    pub zone: Zone,
    pub index: usize,
    pub trigger: UberStateTrigger,
}

#[derive(Debug)]
pub enum Node {
    Anchor(Anchor),
    Pickup(Pickup),
    State(State),
    Quest(Quest),
}
impl Node {
    pub fn node_kind(&self) -> NodeKind {
        match self {
            Node::Anchor(_) => NodeKind::Anchor,
            Node::Pickup(_) => NodeKind::Pickup,
            Node::State(_) => NodeKind::State,
            Node::Quest(_) => NodeKind::Quest,
        }
    }
    pub fn identifier(&self) -> &str {
        match self {
            Node::Anchor(anchor) => &anchor.identifier[..],
            Node::Pickup(pickup) => &pickup.identifier[..],
            Node::State(state) => &state.identifier[..],
            Node::Quest(quest) => &quest.identifier[..],
        }
    }
    pub fn zone(&self) -> Option<Zone> {
        match self {
            Node::Pickup(pickup) => Some(pickup.zone),
            Node::Quest(quest) => Some(quest.zone),
            _ => None,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            Node::Anchor(anchor) => anchor.index,
            Node::Pickup(pickup) => pickup.index,
            Node::State(state) => state.index,
            Node::Quest(quest) => quest.index,
        }
    }
    pub fn trigger(&self) -> Option<&UberStateTrigger> {
        match self {
            Node::Anchor(_) => None,
            Node::Pickup(pickup) => Some(&pickup.trigger),
            Node::State(state) => state.trigger.as_ref(),
            Node::Quest(quest) => Some(&quest.trigger),
        }
    }
    pub fn position(&self) -> Option<&Position> {
        match self {
            Node::Anchor(anchor) => anchor.position.as_ref(),
            Node::Pickup(pickup) => pickup.position.as_ref(),
            Node::State(_) => None,
            Node::Quest(quest) => quest.position.as_ref(),
        }
    }
    pub fn map_position(&self) -> Option<&Position> {
        match self {
            Node::Anchor(anchor) => anchor.position.as_ref(),
            Node::Pickup(pickup) => pickup.map_position.as_ref(),
            Node::State(_) => None,
            Node::Quest(quest) => quest.map_position.as_ref(),
        }
    }
    pub fn can_place(&self) -> bool {
        matches!(self, Node::Pickup(_) | Node::Quest(_))
    }
    pub fn can_spawn(&self) -> bool {
        if let Node::Anchor(anchor) = self {
            anchor.position.is_some() && anchor.can_spawn
        } else {
            false
        }
    }
    pub(crate) fn summary(&self) -> NodeSummary {
        NodeSummary {
            identifier: self.identifier().to_string(),
            position: self.position().cloned(),
            zone: self.zone(),
        }
    }
}
impl fmt::Display for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.identifier())
    }
}

pub type Reached<'a> = Vec<&'a Node>;
pub type Progressions<'a> = Vec<(&'a Requirement, OrbVariants)>;

#[derive(Debug)]
struct ReachContext<'a, 'b, 'c> {
    player: &'b Player<'c>,
    progression_check: bool,
    states: FxHashSet<usize>,
    state_progressions: FxHashMap<usize, Vec<(usize, &'a Connection)>>,
    world_state: FxHashMap<usize, OrbVariants>,
    reached: Vec<&'a Node>,
    progressions: Vec<(&'a Requirement, OrbVariants)>,
}
impl<'b, 'c> ReachContext<'_, 'b, 'c> {
    fn new(player: &'b Player<'c>, progression_check: bool, states: FxHashSet<usize>) -> Self {
        ReachContext {
            player,
            progression_check,
            states,
            state_progressions: Default::default(),
            world_state: Default::default(),
            reached: Default::default(),
            progressions: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub spawn_pickup_node: Node,
}
impl Graph {
    pub fn new(nodes: Vec<Node>) -> Graph {
        let spawn_pickup_node = Node::Pickup(Pickup {
            identifier: String::from("Spawn"),
            zone: Zone::Spawn,
            index: usize::MAX,
            trigger: UberStateTrigger {
                identifier: UberIdentifier::spawn(),
                condition: None,
            },
            position: None,
            map_position: None,
        });
        Graph {
            nodes,
            spawn_pickup_node,
        }
    }

    fn follow_state_progressions<'a>(
        &'a self,
        index: usize,
        context: &mut ReachContext<'a, '_, '_>,
    ) {
        if let Some(connections) = context.state_progressions.get(&index) {
            for (from, connection) in connections.clone() {
                if context.world_state.contains_key(&connection.to) {
                    // TODO loop with improved orbs?
                    continue;
                }
                let target_orbs = connection.requirement.is_met(
                    context.player,
                    &context.states,
                    context.world_state[&from].clone(),
                );
                if !target_orbs.is_empty() {
                    self.reach_recursion(&self.nodes[connection.to], target_orbs, context);
                }
            }
        }
    }

    fn reach_recursion<'a>(
        &'a self,
        entry: &'a Node,
        mut best_orbs: OrbVariants,
        context: &mut ReachContext<'a, '_, '_>,
    ) {
        context.world_state.insert(entry.index(), best_orbs.clone());
        match entry {
            Node::Anchor(anchor) => {
                let max_orbs = context.player.max_orbs();
                if best_orbs
                    .first()
                    .map_or(true, |first_orbs| first_orbs != &max_orbs)
                {
                    for refill in &anchor.refills {
                        let mut refill_orbs = refill.requirement.is_met(
                            context.player,
                            &context.states,
                            best_orbs.clone(),
                        );
                        if !refill_orbs.is_empty() {
                            if matches!(refill.value, RefillValue::Full) {
                                // shortcut
                                best_orbs = smallvec![max_orbs];
                                break;
                            }
                            context.player.refill(refill.value, &mut refill_orbs);
                            best_orbs = orbs::either(&best_orbs, &refill_orbs);
                        }
                    }
                }

                for connection in &anchor.connections {
                    if context.world_state.contains_key(&connection.to) {
                        // TODO loop with improved orbs?
                        continue;
                    }
                    let target_orbs = connection.requirement.is_met(
                        context.player,
                        &context.states,
                        best_orbs.clone(),
                    );
                    if target_orbs.is_empty() {
                        let states = connection
                            .requirement
                            .contained_requirements(context.player.settings)
                            .filter_map(|requirement| match requirement {
                                Requirement::State(state) if !context.states.contains(state) => {
                                    Some(*state)
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>();

                        if states.is_empty() {
                            if context.progression_check {
                                context
                                    .progressions
                                    .push((&connection.requirement, best_orbs.clone()));
                            }
                        } else {
                            for state in states {
                                context
                                    .state_progressions
                                    .entry(state)
                                    .or_default()
                                    .push((anchor.index, connection));
                            }
                        }
                    } else {
                        self.reach_recursion(&self.nodes[connection.to], target_orbs, context);
                    }
                }
            }
            Node::Pickup(_) => context.reached.push(entry),
            Node::State(state) => {
                context.states.insert(state.index);
                context.reached.push(entry);
                self.follow_state_progressions(state.index, context);
            }
            Node::Quest(quest) => {
                context.states.insert(quest.index);
                context.reached.push(entry);
                self.follow_state_progressions(quest.index, context);
            }
        }
    }
    fn reached_by_teleporter<'a>(&'a self, context: &mut ReachContext<'a, '_, '_>) {
        if context
            .world_state
            .iter()
            .any(|(index, orb_variants)| match &self.nodes[*index] {
                Node::Anchor(anchor) => !anchor
                    .teleport_restriction
                    .is_met(context.player, &context.states, orb_variants.clone())
                    .is_empty(),
                _ => false,
            })
        {
            if let Some(tp_anchor) = self
                .nodes
                .iter()
                .find(|&node| node.identifier() == TP_ANCHOR)
            {
                if !context.world_state.contains_key(&tp_anchor.index()) {
                    self.reach_recursion(tp_anchor, smallvec![context.player.max_orbs()], context);
                }
            }
        }
    }

    fn collect_extra_states(
        &self,
        extra_states: &FxHashMap<UberIdentifier, f32>,
        sets: &[usize],
    ) -> FxHashSet<usize> {
        let mut states = FxHashSet::default();

        for (trigger, index) in self
            .nodes
            .iter()
            .filter(|node| matches!(node, Node::State(_) | Node::Quest(_)))
            .filter_map(|node| node.trigger().map(|trigger| (trigger, node.index())))
        {
            if let Some(value) = extra_states.get(&trigger.identifier) {
                if trigger.check_value(*value) {
                    states.insert(index);
                }
            }
        }

        states.reserve(sets.len());
        for set in sets {
            states.insert(*set);
        }

        states
    }

    #[inline]
    pub fn find_spawn(&self, spawn: &str) -> Result<&Node, String> {
        let entry = self
            .nodes
            .iter()
            .find(|&node| node.identifier() == spawn)
            .ok_or_else(|| format!("Spawn {} not found", spawn))?;
        if !matches!(entry, Node::Anchor(_)) {
            return Err(format!(
                "Spawn has to be an anchor, {} is a {:?}",
                spawn,
                entry.node_kind()
            ));
        }
        Ok(entry)
    }

    pub fn reached_locations<'a>(
        &'a self,
        player: &Player,
        spawn: &'a Node,
        extra_states: &FxHashMap<UberIdentifier, f32>,
        sets: &[usize],
    ) -> Reached<'a> {
        let mut context =
            ReachContext::new(player, false, self.collect_extra_states(extra_states, sets));

        self.reach_recursion(spawn, smallvec![player.max_orbs()], &mut context);
        self.reached_by_teleporter(&mut context);

        context.reached
    }
    pub fn reached_and_progressions<'a>(
        &'a self,
        player: &Player,
        spawn: &'a Node,
        extra_states: &FxHashMap<UberIdentifier, f32>,
        sets: &[usize],
    ) -> (Reached<'a>, Progressions<'a>) {
        let mut context =
            ReachContext::new(player, true, self.collect_extra_states(extra_states, sets));

        self.reach_recursion(spawn, smallvec![player.max_orbs()], &mut context);
        self.reached_by_teleporter(&mut context);

        // add progressions containing states that were never met
        for (_, state_progressions) in context.state_progressions {
            for (from, connection) in state_progressions {
                if !context.world_state.contains_key(&connection.to) {
                    context
                        .progressions
                        .push((&connection.requirement, context.world_state[&from].clone()));
                }
            }
        }

        (context.reached, context.progressions)
    }
}
