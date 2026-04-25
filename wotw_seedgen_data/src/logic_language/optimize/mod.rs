#[cfg(test)]
mod tests;

use std::{cmp::Ordering, mem};

use itertools::Itertools;
use log::trace;

use crate::logic_language::output::{
    Anchor, Connection, Entrance, Graph, Node, Refill, Requirement,
};

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
            entrance,
            can_spawn: _,
            teleport_restriction,
            refills,
            connections,
        } = self;

        entrance.optimize();
        teleport_restriction.optimize();

        refills.optimize();
        refills.retain(|refill| !matches!(refill.requirement, Requirement::Impossible));
        refills.shrink_to_fit();

        connections.optimize();
        connections.retain(|connection| !matches!(connection.requirement, Requirement::Impossible));
        connections.shrink_to_fit();
    }
}

impl Optimize for Entrance {
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
    fn remove_common_factor<const FRONT: bool>(&mut self, other: &mut Self) -> CommonFactors {
        trace!("searching common factor for {self} and {other}");

        let factor = self.remove_common_factor_impl::<FRONT>(other);

        if factor.is_none() {
            trace!("no factor found in {self} and {other}");
        } else {
            if let Some(factor) = &factor.front {
                trace!("removed front factor {factor} from {self} and {other}");
            }
            if let Some(factor) = &factor.back {
                trace!("removed back factor {factor} from {self} and {other}");
            }
        }

        factor
    }

    #[must_use]
    fn remove_common_factor_impl<const FRONT: bool>(&mut self, other: &mut Self) -> CommonFactors {
        fn remove<const FRONT: bool>(a: &mut Requirement, b: &mut Requirement) -> CommonFactors {
            *a = Requirement::Free;
            CommonFactors::one::<FRONT>(mem::replace(b, Requirement::Free))
        }

        fn remove_amount<const FRONT: bool>(
            a_amount: f32,
            b_amount: f32,
            a: &mut Requirement,
            b: &mut Requirement,
            f: impl FnOnce(f32) -> Requirement,
        ) -> CommonFactors {
            match f32::total_cmp(&a_amount, &b_amount) {
                Ordering::Less => {
                    *b = f(b_amount - a_amount);
                    CommonFactors::one::<FRONT>(mem::replace(a, Requirement::Free))
                }
                Ordering::Equal => remove::<FRONT>(a, b),
                Ordering::Greater => {
                    *a = f(a_amount - b_amount);
                    CommonFactors::one::<FRONT>(mem::replace(b, Requirement::Free))
                }
            }
        }

        if self == other {
            return remove::<FRONT>(self, other);
        }

        match (&mut *self, &mut *other) {
            // could work with incremental difficulty but it would probably just create unfavourable requirement chains
            // similar for other non-consuming amount requirements like spirit light
            (Self::EnergySkill(a, a_amount), Self::EnergySkill(b, b_amount)) if a == b => {
                let skill = *a;
                remove_amount::<FRONT>(*a_amount, *b_amount, self, other, move |amount| {
                    Requirement::EnergySkill(skill, amount)
                })
            }
            // TODO currently this could change regenerate behaviour, but that's already a problem
            // and regenerate probably cannot be allowed between requirements anyway...
            (Self::Damage(a_amount), Self::Damage(b_amount)) => {
                remove_amount::<FRONT>(*a_amount, *b_amount, self, other, Self::Damage)
            }
            (Self::Danger(a_amount), Self::Danger(b_amount)) => {
                remove_amount::<FRONT>(*a_amount, *b_amount, self, other, Self::Danger)
            }
            // destroy requirements have a lot of gotchas and probably cannot be sanely factored
            (and @ Self::And(_), other) | (other, and @ Self::And(_)) => {
                let Self::And(ands) = &mut *and else {
                    unreachable!()
                };

                let mut common_factors = CommonFactors::NONE;

                let mut chunks = ReorderableChunks::new();
                let first_chunk = chunks.next(ands).unwrap();

                trace!(
                    "searching common factor in first chunk {}",
                    first_chunk.iter().format(" & ")
                );

                pull_common_factors_in_chunk::<true>(&mut common_factors, first_chunk, other);

                if !chunks.is_finished() {
                    let last_chunk = ReorderableChunksRev::new(ands).next(ands).unwrap();

                    trace!(
                        "searching common factor in last chunk {}",
                        last_chunk.iter().format(" & ")
                    );

                    pull_common_factors_in_chunk::<false>(&mut common_factors, last_chunk, other);
                }

                if common_factors.is_some() {
                    *and = Requirement::and(ands.drain(..));
                }

                common_factors
            }
            (or @ Self::Or(_), other) | (other, or @ Self::Or(_)) => {
                let Self::Or(ors) = &mut *or else {
                    unreachable!()
                };

                other.remove_common_factor_from_ors(ors);

                pull_common_factors(or);

                CommonFactors::NONE
            }
            _ => CommonFactors::NONE,
        }
    }

    fn remove_common_factor_from_ors(&mut self, ors: &mut [Requirement]) {
        for or in ors {
            let factor = self.remove_common_factor::<true>(or);
            factor.apply(self, or);
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

#[derive(PartialEq)]
struct CommonFactors {
    front: Option<Requirement>,
    back: Option<Requirement>,
}

impl CommonFactors {
    const NONE: Self = Self {
        front: None,
        back: None,
    };

    const fn front(factor: Requirement) -> Self {
        Self {
            front: Some(factor),
            back: None,
        }
    }

    const fn back(factor: Requirement) -> Self {
        Self {
            front: None,
            back: Some(factor),
        }
    }

    const fn one<const FRONT: bool>(factor: Requirement) -> Self {
        if FRONT {
            Self::front(factor)
        } else {
            Self::back(factor)
        }
    }

    const fn is_none(&self) -> bool {
        matches!(self, &Self::NONE)
    }

    const fn is_some(&self) -> bool {
        !matches!(self, &Self::NONE)
    }

    fn merge(&mut self, other: CommonFactors) {
        fn merge_factor(a: Option<Requirement>, b: Option<Requirement>) -> Option<Requirement> {
            match (a, b) {
                (None, None) => None,
                (factor @ Some(_), None) | (None, factor @ Some(_)) => factor,
                (Some(a), Some(b)) => Some(Requirement::and([a, b])),
            }
        }

        self.front = merge_factor(mem::take(&mut self.front), other.front);
        self.back = merge_factor(mem::take(&mut self.back), other.back);
    }

    fn apply(self, a: &mut Requirement, b: &mut Requirement) {
        if self.is_none() {
            return;
        }

        let a_taken = mem::replace(a, Requirement::Impossible);
        let b_taken = mem::replace(b, Requirement::Impossible);

        *a = match (self.front, self.back) {
            (None, None) => unreachable!(),
            (Some(front), None) => Requirement::and([front, Requirement::or([a_taken, b_taken])]),
            (None, Some(back)) => Requirement::and([Requirement::or([a_taken, b_taken]), back]),
            (Some(front), Some(back)) => {
                Requirement::and([front, Requirement::or([a_taken, b_taken]), back])
            }
        };
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

fn pull_common_factors_in_chunk<const FRONT: bool>(
    common_factors: &mut CommonFactors,
    chunk: &mut [Requirement],
    other: &mut Requirement,
) {
    for and in chunk {
        // Our logic breaks down in this nesting and there shouldn't be anything left
        // to optimize in an or anyway, since this or was entered in recursion before
        // and if it had common factors, they would live in the surrounding and by now
        if matches!(and, Requirement::Or(_)) {
            continue;
        }

        let factor = and.remove_common_factor::<FRONT>(other);

        if factor.is_some() {
            common_factors.merge(factor);

            if matches!(other, Requirement::Free) {
                break;
            }
        }
    }
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
struct ReorderableChunks {
    last_start: usize,
    state: ReorderableChunksState,
}

enum ReorderableChunksState {
    NotStarted,
    InProgress,
    Finished,
}

impl ReorderableChunks {
    const fn new() -> Self {
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

        let end = orb_changers.nth(1).unwrap_or_else(|| {
            self.state = ReorderableChunksState::Finished;
            ands.len()
        });

        Some(&mut ands[start..end])
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, ReorderableChunksState::Finished)
    }
}

/// Like [`ReorderableChunks`], but going in reverse
struct ReorderableChunksRev {
    last_end: usize,
    state: ReorderableChunksState,
}

impl ReorderableChunksRev {
    const fn new(ands: &[Requirement]) -> Self {
        Self {
            last_end: ands.len(),
            state: ReorderableChunksState::NotStarted,
        }
    }

    fn next<'a>(&mut self, ands: &'a mut [Requirement]) -> Option<&'a mut [Requirement]> {
        let mut orb_changers = ands[..self.last_end]
            .iter()
            .enumerate()
            .filter(|(_, req)| req.changes_orbs())
            .map(move |(i, _)| i);

        let end = match self.state {
            ReorderableChunksState::NotStarted => {
                self.state = ReorderableChunksState::InProgress;
                ands.len()
            }
            ReorderableChunksState::InProgress => orb_changers.next_back().unwrap(),
            ReorderableChunksState::Finished => return None,
        };
        self.last_end = end;

        let start = match orb_changers.nth_back(1) {
            None => {
                self.state = ReorderableChunksState::Finished;
                0
            }
            Some(start) => start + 1,
        };

        Some(&mut ands[start..end])
    }
}
