#[cfg(test)]
mod tests;

use std::{cmp::Ordering, mem};

use itertools::Itertools;
use log::trace;

use crate::logic_language::output::{Anchor, Connection, Door, Graph, Node, Refill, Requirement};

impl Graph {
    pub fn optimize(&mut self) {
        self.nodes.optimize();
    }
}

pub trait Optimize {
    fn optimize(&mut self);
}

impl<T: Optimize> Optimize for Option<T> {
    fn optimize(&mut self) {
        if let Some(t) = self {
            t.optimize();
        }
    }
}

impl<T: Optimize> Optimize for Vec<T> {
    fn optimize(&mut self) {
        for t in self {
            t.optimize();
        }
    }
}

impl Optimize for Node {
    fn optimize(&mut self) {
        if let Node::Anchor(anchor) = self {
            anchor.optimize();
        }
    }
}

impl Optimize for Anchor {
    fn optimize(&mut self) {
        let Self {
            identifier: _,
            position: _,
            door,
            can_spawn: _,
            teleport_restriction,
            refills,
            connections,
        } = self;

        door.optimize();
        teleport_restriction.optimize();

        refills.optimize();
        refills.retain(|refill| !matches!(refill.requirement, Requirement::Impossible));
        refills.shrink_to_fit();

        connections.optimize();
        connections.retain(|connection| !matches!(connection.requirement, Requirement::Impossible));
        connections.shrink_to_fit();
    }
}

impl Optimize for Door {
    fn optimize(&mut self) {
        self.requirement.optimize();
    }
}

impl Optimize for Refill {
    fn optimize(&mut self) {
        self.requirement.optimize();
    }
}

impl Optimize for Connection {
    fn optimize(&mut self) {
        self.requirement.optimize();
    }
}

impl Optimize for Requirement {
    fn optimize(&mut self) {
        match self {
            Self::And(ands) => {
                ands.optimize();

                *self = Self::and(ands.drain(..));

                if let Self::And(ands) = self {
                    reorder_ands(ands);
                }
            }
            Self::Or(ors) => {
                ors.optimize();
                pull_common_factors(self);
            }
            _ => {}
        }
    }
}

impl Requirement {
    #[must_use]
    fn remove_common_factor(&mut self, other: &mut Self) -> Option<Self> {
        trace!("searching common factor for {self} and {other}");

        let factor = self.remove_common_factor_impl(other);

        match &factor {
            None => trace!("no factor found in {self} and {other}"),
            Some(factor) => trace!("removed factor {factor} from {self} and {other}"),
        }

        factor
    }

    #[must_use]
    fn remove_common_factor_impl(&mut self, other: &mut Self) -> Option<Self> {
        fn remove(a: &mut Requirement, b: &mut Requirement) -> Option<Requirement> {
            *a = Requirement::Free;
            Some(mem::replace(b, Requirement::Free))
        }

        fn remove_amount<F>(
            a_amount: f32,
            b_amount: f32,
            a: &mut Requirement,
            b: &mut Requirement,
            f: F,
        ) -> Option<Requirement>
        where
            F: FnOnce(f32) -> Requirement,
        {
            match f32::total_cmp(&a_amount, &b_amount) {
                Ordering::Less => {
                    *b = f(b_amount - a_amount);
                    Some(mem::replace(a, Requirement::Free))
                }
                Ordering::Equal => remove(a, b),
                Ordering::Greater => {
                    *a = f(a_amount - b_amount);
                    Some(mem::replace(b, Requirement::Free))
                }
            }
        }

        if self == other {
            return remove(self, other);
        }

        match (&mut *self, &mut *other) {
            // could work with incremental difficulty but it would probably just create unfavourable requirement chains
            // similar for other non-consuming amount requirements like spirit light
            (Self::EnergySkill(a, a_amount), Self::EnergySkill(b, b_amount)) if a == b => {
                let skill = *a;
                remove_amount(*a_amount, *b_amount, self, other, move |amount| {
                    Requirement::EnergySkill(skill, amount)
                })
            }
            // TODO currently this could change regenerate behaviour, but that's already a problem
            // and regenerate probably cannot be allowed between requirements anyway...
            (Self::Damage(a_amount), Self::Damage(b_amount)) => {
                remove_amount(*a_amount, *b_amount, self, other, Self::Damage)
            }
            (Self::Danger(a_amount), Self::Danger(b_amount)) => {
                remove_amount(*a_amount, *b_amount, self, other, Self::Danger)
            }
            // destroy requirements have a lot of gotchas and probably cannot be sanely factored
            (and @ Self::And(_), other) | (other, and @ Self::And(_)) => {
                let Self::And(ands) = &mut *and else {
                    unreachable!()
                };

                let mut common_factors = vec![];

                for and in ReorderableChunks::new().next(ands).unwrap() {
                    // Our logic breaks down in this nesting and there shouldn't be anything left
                    // to optimize in an or anyway, since this or was entered in recursion before
                    // and if it had common factors, they would live in the surrounding and by now
                    if matches!(and, Self::Or(_)) {
                        continue;
                    }

                    if let Some(factor) = and.remove_common_factor(other) {
                        common_factors.push(factor);

                        if matches!(other, Requirement::Free) {
                            break;
                        }
                    }
                }

                if common_factors.is_empty() {
                    None
                } else {
                    *and = Requirement::and(ands.drain(..));
                    Some(Requirement::and(common_factors))
                }
            }
            (or @ Self::Or(_), other) | (other, or @ Self::Or(_)) => {
                let Self::Or(ors) = &mut *or else {
                    unreachable!()
                };

                other.remove_common_factor_from_ors(ors);

                pull_common_factors(or);

                None
            }
            _ => None,
        }
    }

    fn remove_common_factor_from_ors(&mut self, ors: &mut [Requirement]) {
        for or in ors {
            if let Some(factor) = self.remove_common_factor(or) {
                let a_taken = mem::replace(self, Requirement::Impossible);
                let b_taken = mem::replace(or, Requirement::Impossible);
                *self = Requirement::and([factor, Requirement::or([a_taken, b_taken])])
            }
        }
    }

    fn changes_orbs(&self) -> bool {
        match self {
            Self::Free
            | Self::Impossible
            | Self::Difficulty(_)
            | Self::NormalGameDifficulty
            | Self::Trick(_)
            | Self::Skill(_)
            | Self::SpiritLight(_)
            | Self::GorlekOre(_)
            | Self::Keystone(_)
            | Self::Shard(_)
            | Self::Teleporter(_)
            | Self::Water
            | Self::State(_) => false,
            Self::EnergySkill(_, _)
            | Self::NonConsumingEnergySkill(_)
            | Self::Damage(_)
            | Self::Danger(_)
            | Self::Combat(_)
            | Self::Boss(_)
            | Self::BreakWall(_)
            | Self::ShurikenBreak(_)
            | Self::SentryBreak(_) => true,
            Self::And(requirements) | Self::Or(requirements) => {
                requirements.iter().any(Requirement::changes_orbs)
            }
        }
    }
}

fn pull_common_factors(requirement: &mut Requirement) {
    let Requirement::Or(ors) = requirement else {
        panic!("pull_common_factors should only be applied to or requirements");
    };

    trace!("searching common factors in {}", ors.iter().format(" | "));

    for index in 0..ors.len() - 1 {
        let (a, rest) = ors[index..].split_first_mut().unwrap();

        a.remove_common_factor_from_ors(rest);
    }

    *requirement = Requirement::or(ors.drain(..));

    trace!("transformed common factors into {requirement}");
}

fn reorder_ands(ands: &mut [Requirement]) {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum ReorderKey {
        /// Unsolvable requirements shortcut both reach checks and solutions
        Unsolvable,
        /// Simple, cheap to check requirements might be good to place earlier
        Simple,
        /// Requirements that change orbs can create lots of branches
        ChangesOrbs,
        /// Destroy requirements change orbs and create even more branches for better weapons
        Destroy,
    }

    fn reorder_key(requirement: &Requirement) -> ReorderKey {
        match requirement {
            Requirement::Free
            | Requirement::Impossible
            | Requirement::Difficulty(_)
            | Requirement::NormalGameDifficulty
            | Requirement::Trick(_)
            | Requirement::State(_) => ReorderKey::Unsolvable,
            Requirement::Skill(_)
            | Requirement::SpiritLight(_)
            | Requirement::GorlekOre(_)
            | Requirement::Keystone(_)
            | Requirement::Shard(_)
            | Requirement::Teleporter(_)
            | Requirement::Water => ReorderKey::Simple,
            Requirement::EnergySkill(_, _)
            | Requirement::NonConsumingEnergySkill(_)
            | Requirement::Damage(_)
            | Requirement::Danger(_) => ReorderKey::ChangesOrbs,
            Requirement::Combat(_)
            | Requirement::Boss(_)
            | Requirement::BreakWall(_)
            | Requirement::ShurikenBreak(_)
            | Requirement::SentryBreak(_) => ReorderKey::Destroy,
            Requirement::And(requirements) | Requirement::Or(requirements) => {
                requirements.iter().map(reorder_key).max().unwrap()
            }
        }
    }

    trace!("reordering {}", ands.iter().format(" & "));

    let mut chunks = ReorderableChunks::new();
    while let Some(chunk) = chunks.next(ands) {
        trace!("reordering chunk {}", chunk.iter().format(" & "));

        chunk.sort_unstable_by_key(reorder_key);
    }

    trace!("reordered to {}", ands.iter().format(" & "));
}

/// Inside an and requirement, we can reorder any continuous chunks of requirements
/// that don't contain more than one requirement which interacts with orbs.
/// A chain of ands with multiple such requirements is not generally reorderable
/// because the orb costs of the requirements interact in nontrivial ways.
///
/// We cannot implement this as a proper Iterator with safe rust
/// because the returned chunks overlap.
#[derive(Debug)]
struct ReorderableChunks {
    last_start: usize,
    state: ReorderableChunksState,
}

#[derive(Debug)]
enum ReorderableChunksState {
    NotStarted,
    InProgress,
    Finished,
}

impl ReorderableChunks {
    fn new() -> Self {
        Self {
            last_start: 0,
            state: ReorderableChunksState::NotStarted,
        }
    }

    fn next<'a>(&mut self, ands: &'a mut [Requirement]) -> Option<&'a mut [Requirement]> {
        let offset = self.last_start;
        let mut orb_changers = ands[offset..]
            .iter()
            .enumerate()
            .filter(|(_, req)| req.changes_orbs())
            .map(move |(i, _)| offset + i);

        let start = match self.state {
            ReorderableChunksState::NotStarted => {
                self.state = ReorderableChunksState::InProgress;
                0
            }
            ReorderableChunksState::InProgress => {
                let last_start_new_position = orb_changers.next().unwrap();
                last_start_new_position + 1
            }
            ReorderableChunksState::Finished => return None,
        };
        self.last_start = start;

        let end = orb_changers.nth(2).unwrap_or_else(|| {
            self.state = ReorderableChunksState::Finished;
            ands.len()
        });

        Some(&mut ands[start..end])
    }
}
