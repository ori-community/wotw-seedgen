mod legacy;
#[cfg(test)]
mod tests;

use std::{
    cmp::Ordering,
    fmt::{self, Display},
    iter::Rev,
    mem,
    ops::{ControlFlow, Deref},
    slice, vec,
};

use itertools::Itertools;
use log::{debug, log_enabled, trace, Level::Debug};
use ordered_float::OrderedFloat;

use crate::logic_language::output::{
    Anchor, Connection, Entrance, Graph, Node, OrbChangeKind, Refill, Requirement,
};

const LAST_NON_ORB_CHANGING: isize = Requirement::Keystone(1).discriminant_value();
const FIRST_ORB_CHANGING: isize = Requirement::Damage(OrderedFloat(1.)).discriminant_value();

// This implementation relies on the order of `Requirement` discriminants.
// For details check the comment above `Requirement`.
const _: () = {
    assert!(
        LAST_NON_ORB_CHANGING + 1 == FIRST_ORB_CHANGING,
        "FIRST_ORB_CHANGING does not follow LAST_NON_ORB_CHANGING"
    )
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
            identifier,
            position: _,
            entrance,
            can_spawn: _,
            teleport_restriction,
            refills,
            connections,
        } = self;

        trace!("optimizing {identifier}");

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
        if std::env::var("NO_OPTIMIZE").is_ok() {
            return;
        }

        if std::env::var("LEGACY").is_ok() {
            self.legacy_optimize();
            return;
        }

        match self {
            // TODO I don't think going straight for the ors should be worth it
            // ideally, for instance in combat requirements we can't optimize
            // the interaction between forced Regenerate and the region
            // requirements like this
            Self::And(ands) => {
                ands.optimize();
                *self = Self::and(ands.drain(..));
            }
            Self::Or(_) => self.optimize_or(),
            _ => {}
        }

        // TODO maintain order instead?
        // self.reorder();
    }
}

impl Requirement {
    fn optimize_or(&mut self) {
        let before = log_enabled!(Debug).then(|| {
            let before = self.to_string();

            trace!("optimizing {before}");

            before
        });

        self.disjunctive_normal_form();

        trace!("disjunctive normal form: -> {self}");

        self.normalize_order();

        trace!("normalized order: -> {self}");

        self.dedup();
        self.filter_redundant_ors();

        trace!("filtered redundant ors: -> {self}");

        self.heuristically_apply_associativity();

        if let Some(before) = &before {
            debug!("optimized {before} -> {self}");
        }
    }

    /// Transforms the `Requirement` into [disjunctive normal form](https://en.wikipedia.org/wiki/Disjunctive_normal_form).
    ///
    /// This is not a good final form to use because the conjunctions will have many shared requirements but it's a useful base to apply optimizations to.
    ///
    /// Note that disjunctions and conjunctions may be flattened into single requirements where possible.
    fn disjunctive_normal_form(&mut self) {
        match self {
            Self::And(ands) => {
                let mut disjunction = vec![vec![]];

                for mut and in ands.drain(..) {
                    and.disjunctive_normal_form();

                    match and {
                        Self::Or(ors) => {
                            disjunction = disjunction
                                .drain(..)
                                .cartesian_product(ors)
                                .map(|(mut a, b)| {
                                    a.push(b);
                                    a
                                })
                                .collect();
                        }
                        literal => {
                            for conjunction in disjunction.iter_mut() {
                                conjunction.push(literal.clone());
                            }
                        }
                    }
                }

                *self = Requirement::or(disjunction.into_iter().map(|conjunction| {
                    Requirement::and(conjunction.into_iter().chain(ands.iter().cloned()))
                }));
            }
            Self::Or(ors) => {
                for index in 0..ors.len() {
                    let or = &mut ors[index];

                    or.disjunctive_normal_form();

                    if let Self::Or(nested_ors) = or {
                        let mut nested_ors = mem::take(nested_ors);
                        *or = nested_ors.pop().unwrap();
                        ors.append(&mut nested_ors);
                    }
                }
            }
            _ => {}
        }
    }

    fn is_disjunctive_normal_form(&self) -> bool {
        fn contains_no_nesting(ands: &[Requirement]) -> bool {
            !ands
                .iter()
                .any(|and| matches!(and, Requirement::And(_) | Requirement::Or(_)))
        }

        match self {
            Self::And(ands) => contains_no_nesting(ands),
            Self::Or(ors) => ors.iter().all(|or| match or {
                Self::And(ands) => contains_no_nesting(ands),
                Self::Or(_) => false,
                _ => true,
            }),
            _ => true,
        }
    }

    /// Normalizes the order of contained requirements.
    ///
    /// Less costly requirements will be ordered earlier where possible to allow faster shortcuts.
    ///
    /// # Prerequisites
    ///
    /// - [`Requirement::disjunctive_normal_form`]
    fn normalize_order(&mut self) {
        debug_assert!(
            self.is_disjunctive_normal_form(),
            "called `normalize_order` without `disjunctive_normal_form` on {self}"
        );

        if let Self::Or(ors) = self {
            for or in ors.iter_mut() {
                if let Self::And(ands) = or {
                    ands.sort_by(cmp_ands);
                }
            }

            ors.sort_by(cmp_ors);
        }
    }

    fn is_normalized_order(&self) -> bool {
        fn is_normalized_order_by<F>(reqs: &[Requirement], mut f: F) -> bool
        where
            F: FnMut(&Requirement, &Requirement) -> Ordering,
        {
            reqs.iter().all(Requirement::is_normalized_order)
                && reqs.is_sorted_by(|a, b| matches!(f(a, b), Ordering::Less | Ordering::Equal))
        }

        match self {
            Self::And(ands) => is_normalized_order_by(ands, cmp_ands),
            Self::Or(ors) => is_normalized_order_by(ors, cmp_ors),
            _ => true,
        }
    }

    /// Reproduces [`std::intrinsics::discriminant_value`].
    const fn discriminant_value(&self) -> isize {
        match self {
            Self::Free => 0,
            Self::Impossible => 1,
            Self::NormalGameDifficulty => 2,
            Self::Difficulty(_) => 3,
            Self::Trick(_) => 4,
            Self::State(_) => 5,
            Self::Water => 6,
            Self::Skill(_) => 7,
            Self::Shard(_) => 8,
            Self::Teleporter(_) => 9,
            Self::SpiritLight(_) => 10,
            Self::GorlekOre(_) => 11,
            Self::Keystone(_) => 12,
            Self::Damage(_) => 13,
            Self::Danger(_) => 14,
            Self::NonConsumingEnergySkill(_) => 15,
            Self::EnergySkill(..) => 16,
            Self::ShurikenBreak(_) => 17,
            Self::SentryBreak(_) => 18,
            Self::Extern(_) => 19,
            Self::BreakWall(_) => 20,
            Self::Boss(_) => 21,
            Self::Combat(_) => 22,
            Self::And(_) => 23,
            Self::Or(_) => 24,
        }
    }

    // TODO can use the sorted property now?
    /// Deduplicates identical requirements in ands.
    ///
    /// These can occur sometimes when ors and ands are nested in a way that creates duplicate requirements after normalizing.
    ///
    /// Only truly dedups everything after [`Requirement::disjunctive_normal_form`].
    fn dedup(&mut self) {
        match self {
            Self::And(ands) => {
                // TODO experiment here and in similar cases between iteration, BTrees and Hashes

                let mut index = 0;

                while index < ands.len() - 1 {
                    let (head, tail) = ands[index..].split_first().unwrap();

                    if !head.changes_orbs() && tail.contains(head) {
                        if let ControlFlow::Break(single) = remove_and(ands, index) {
                            *self = single;
                            return;
                        }
                    } else {
                        index += 1;
                    }
                }
            }
            Self::Or(ors) => {
                for or in ors {
                    or.dedup();
                }
            }
            _ => {}
        }
    }

    /// Filters directly redundant or branches.
    ///
    /// Only truly filters all redundancies after [`Requirement::disjunctive_normal_form`].
    fn filter_redundant_ors(&mut self) {
        let Self::Or(ors) = self else {
            return;
        };

        // TODO make something reasonable instead? oriLol

        let mut index = 0;
        'outer: while index < ors.len() - 1 {
            let (head, tail) = ors[index..].split_first_mut().unwrap();
            let mut tail_len = tail.len();

            let mut tail_index = 0;
            while tail_index < tail_len {
                let tail_or = &tail[tail_index];

                match head.logical_cmp(tail_or) {
                    Some(Ordering::Less | Ordering::Equal) => {
                        trace!("{tail_or} is redundant with {head}");

                        tail_len -= 1;
                        tail.swap(tail_index, tail_len);
                    }
                    Some(Ordering::Greater) => {
                        trace!("{head} is redundant with {tail_or}");

                        tail_len -= 1;
                        let tail_or_index = index + 1 + tail_index;
                        let new_len = index + 1 + tail_len;

                        ors.swap(index, tail_or_index);
                        ors.swap(tail_or_index, new_len);
                        ors.truncate(new_len);

                        continue 'outer;
                    }
                    None => tail_index += 1,
                }

                // if tail_or.is_redundant_with(head) {
                //     tail_len -= 1;
                //     tail.swap(tail_index, tail_len);
                // } else if head.is_redundant_with(tail_or) {
                //     tail_len -= 1;
                //     let tail_or_index = index + 1 + tail_index;
                //     let new_len = index + 1 + tail_len;

                //     ors.swap(index, tail_or_index);
                //     ors.swap(tail_or_index, new_len);
                //     ors.truncate(new_len);

                //     continue 'outer;
                // } else {
                //     tail_index += 1;
                // }
            }

            index += 1;
            ors.truncate(index + tail_len);
        }

        if ors.len() == 1 {
            *self = ors.pop().unwrap();
        } else {
            // TODO preserve instead?
            self.normalize_order();
        }

        debug_assert!(
            self.is_normalized_order(),
            "broke normalized order within filter_redundant_ors for {self}"
        );
    }

    // // TODO can this be superseded by logical_cmp now?
    // /// Checks whether `self` is redundant with `other`, meaning that if `self` is met, `other` is always also met.
    // fn is_redundant_with(&self, other: &Self) -> bool {
    //     match (self, other) {
    //         (_, Self::Free) | (Self::Impossible, _) => true,
    //         (Self::Free, _) | (_, Self::Impossible) => false,
    //         (Self::Difficulty(a), Self::Difficulty(b)) => a >= b,
    //         (Self::EnergySkill(a, a_amount), Self::EnergySkill(b, b_amount)) if a == b => {
    //             a_amount >= b_amount
    //         }
    //         (Self::SpiritLight(a), Self::SpiritLight(b))
    //         | (Self::GorlekOre(a), Self::GorlekOre(b))
    //         | (Self::Keystone(a), Self::Keystone(b)) => a >= b,
    //         (Self::Damage(a), Self::Damage(b))
    //         | (Self::Danger(a), Self::Danger(b))
    //         | (Self::Boss(a), Self::Boss(b))
    //         | (Self::BreakWall(a), Self::BreakWall(b))
    //         | (Self::ShurikenBreak(a), Self::ShurikenBreak(b))
    //         | (Self::SentryBreak(a), Self::SentryBreak(b)) => a >= b,
    //         (Self::Combat(a), Self::Combat(b)) => {
    //             a.len() >= b.len()
    //                 && iter::zip(a, b)
    //                     .all(|((a, a_amount), (b, b_amount))| a == b && a_amount >= b_amount)
    //         }
    //         (Self::And(a), Self::And(b)) => {
    //             b.iter().all(|b| a.iter().any(|a| a.is_redundant_with(b)))
    //         }
    //         (Self::And(a), b) => a.iter().any(|a| a.is_redundant_with(b)),
    //         (_, Self::And(_)) => false,
    //         // TODO eq basically redoes the whole matching, maybe we should manually add all the other branches here?
    //         _ => self == other,
    //     }
    // }

    // TODO Not sure this was a bad idea
    // /// Compares two `Requirements` for redundancy
    // ///
    // /// - `None` means the requirements are unrelated
    // /// - `Some(Ordering::Less)` means `self` is strictly more easily met than `other`
    // /// - `Some(Ordering::Equal)` means `self` and `other` are always met together
    // /// - `Some(Ordering::Greater)` means `other` is strictly more easily met than `self`
    // fn redundancy_cmp(&self, other: &Self) -> Option<Ordering> {
    //     match (self, other) {
    //         (Self::Free, Self::Free) | (Self::Impossible, Self::Impossible) => {
    //             Some(Ordering::Equal)
    //         }
    //         (Self::Free, _) | (_, Self::Impossible) => Some(Ordering::Less),
    //         (_, Self::Free) | (Self::Impossible, _) => Some(Ordering::Greater),
    //         (Self::Difficulty(a), Self::Difficulty(b)) => Some(a.cmp(b)),
    //         (Self::EnergySkill(a, a_amount), Self::EnergySkill(b, b_amount)) if a == b => {
    //             Some(a_amount.total_cmp(b_amount))
    //         }
    //         (Self::SpiritLight(a), Self::SpiritLight(b))
    //         | (Self::GorlekOre(a), Self::GorlekOre(b))
    //         | (Self::Keystone(a), Self::Keystone(b)) => Some(a.cmp(b)),
    //         (Self::Damage(a), Self::Damage(b))
    //         | (Self::Danger(a), Self::Danger(b))
    //         | (Self::Boss(a), Self::Boss(b))
    //         | (Self::BreakWall(a), Self::BreakWall(b))
    //         | (Self::ShurikenBreak(a), Self::ShurikenBreak(b))
    //         | (Self::SentryBreak(a), Self::SentryBreak(b)) => Some(a.total_cmp(b)),
    //         (Self::Combat(a), Self::Combat(b)) => {
    //             let mut a_iter = a.iter();
    //             let mut b_iter = b.iter();

    //             let mut ordering = Ordering::Equal;

    //             // Note if readded: this is a broken use of zip!!
    //             for ((a_enemy, a_amount), (b_enemy, b_amount)) in
    //                 iter::zip(&mut a_iter, &mut b_iter)
    //             {
    //                 if a_enemy != b_enemy {
    //                     return None;
    //                 }

    //                 ordering = ordering.partial_then(a_amount.cmp(b_amount))?;
    //             }

    //             ordering.partial_then(a_iter.len().cmp(&b_iter.len()))
    //         }
    //         (Self::And(a), Self::And(b)) => {
    //             todo!()
    //         }
    //         (Self::And(a), b) => a.iter().try_fold(Ordering::Equal, |ordering, a_and| {
    //             ordering.partial_then(a_and.redundancy_cmp(b)?)
    //         }),
    //         (a, Self::And(b)) => b.iter().try_fold(Ordering::Equal, |ordering, b_and| {
    //             ordering.partial_then(a.redundancy_cmp(b_and)?)
    //         }),
    //         // TODO eq basically redoes the whole matching, maybe we should manually add all the other branches here?
    //         _ if self == other => Some(Ordering::Equal),
    //         _ => None,
    //     }
    // }

    // /// Reorders contained requirements in a way that might allow earlier shortcuts.
    // ///
    // /// Only truly applies all possible reorderings after [`Requirement::disjunctive_normal_form`].
    // fn reorder(&mut self) {
    //     match self {
    //         Self::And(reqs) | Self::Or(reqs) => {
    //             for req in reqs.iter_mut() {
    //                 req.reorder();
    //             }

    //             reqs.sort_by_key(Self::reorder_key);
    //         }
    //         _ => {}
    //     }
    // }

    // fn reorder_key(&self) -> ReorderKey {
    //     match self {
    //         Self::Free
    //         | Self::Impossible
    //         | Self::Difficulty(_)
    //         | Self::NormalGameDifficulty
    //         | Self::Trick(_)
    //         | Self::State(_) => ReorderKey::Unsolvable,
    //         Self::Skill(_)
    //         | Self::SpiritLight(_)
    //         | Self::GorlekOre(_)
    //         | Self::Keystone(_)
    //         | Self::Shard(_)
    //         | Self::Teleporter(_)
    //         | Self::Water => ReorderKey::DoesNotChangeOrbs,
    //         Self::Extern(_)
    //         | Self::EnergySkill(_, _)
    //         | Self::NonConsumingEnergySkill(_)
    //         | Self::Damage(_)
    //         | Self::Danger(_)
    //         | Self::Combat(_)
    //         | Self::Boss(_)
    //         | Self::BreakWall(_)
    //         | Self::ShurikenBreak(_)
    //         | Self::SentryBreak(_) => ReorderKey::ChangesOrbs,
    //         Self::And(requirements) | Self::Or(requirements) => {
    //             requirements.iter().map(Self::reorder_key).max().unwrap()
    //         }
    //     }
    // }

    /// Tries to create nested groups to avoid redundant checks across or branches.
    ///
    /// Assumes ordering from [`Requirement::reorder`].
    ///
    /// This basically undoes [`Requirement::disjunctive_normal_form`] but chooses new groups that seem appropriate after optimizations.
    fn heuristically_apply_associativity(&mut self) {
        let Self::Or(ors) = self else { return };

        // Ideal use of associativity can be difficult:
        //
        // If we choose to factor out Dash from (Dash & Glide) | (Dash & Damage=10) | (Glide & Damage=10),
        // we get (Dash & (Glide | Damage=10)) | (Glide & Damage=10). Now neither Glide or Damage=10 can be
        // factored out anymore, which is suboptimal because Damage=10 was the more expensive requirement
        // and would benefit more than the others from being factored out.
        //
        // Once we expand the example to (Dash & Glide) | (Dash & DoubleJump) | (Dash & Damage=10) | (Glide & Damage=10)
        // it starts becoming extremely ambiguous. This is why we follow a path of heuristics.

        factor_out_orb_changes(ors);

        loop {
            if !factor_out_non_orb_changes(ors) {
                break;
            }

            if !factor_out_orb_changes(ors) {
                break;
            }
        }

        // TODO this exists in many places and should be solvable without a separate Vec allocation?
        *self = Self::or(ors.drain(..))
    }

    fn remove_orb_changing_factor(&mut self, side: OrbChangingFactorSide) {
        match side {
            OrbChangingFactorSide::Front => self.remove_front_orb_changing_factor(),
            OrbChangingFactorSide::Back => self.remove_back_orb_changing_factor(),
        }
    }

    fn remove_front_orb_changing_factor(&mut self) {
        match self {
            Self::And(ands) => {
                let index = ands.iter().position(Requirement::changes_orbs).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            Self::Or(_) => unimplemented!(),
            _ => *self = Self::Free,
        }
    }

    fn remove_back_orb_changing_factor(&mut self) {
        match self {
            Self::And(ands) => {
                let index = ands.iter().rposition(Requirement::changes_orbs).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            Self::Or(_) => unimplemented!(),
            _ => *self = Self::Free,
        }
    }

    fn remove_non_orb_changing_factor(&mut self, factor: &Requirement) {
        match self {
            Self::And(ands) => {
                let index = ands.iter().position(|and| and == factor).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            Self::Or(_) => unimplemented!(),
            _ => *self = Self::Free,
        }
    }
}

fn binary_search_index_by<T, F>(slice: &[T], f: F) -> usize
where
    F: FnMut(&T) -> Ordering,
{
    match slice.binary_search_by(f) {
        Ok(index) => index,
        Err(index) => index,
    }
}

fn orb_changing_start(ands: &[Requirement]) -> usize {
    binary_search_index_by(ands, |and| {
        if and.discriminant_value() < FIRST_ORB_CHANGING {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    })
}

fn split_orb_changing(ands: &[Requirement]) -> (&[Requirement], &[Requirement]) {
    let orb_changing_start = orb_changing_start(ands);
    ands.split_at(orb_changing_start)
}

struct ReverseSortedAnds<'r> {
    orb_changing: vec::IntoIter<&'r Requirement>,
    non_orb_changing: Rev<slice::Iter<'r, Requirement>>,
}

impl<'r> ReverseSortedAnds<'r> {
    fn new(ands: &'r [Requirement]) -> Self {
        let (non_orb_changing, orb_changing) = split_orb_changing(ands);

        // the orb-changing ands could not be sorted, so we need to sort them now
        let mut orb_changing = orb_changing.iter().collect::<Vec<_>>();
        // sort is reversed
        orb_changing.sort_by(|a, b| cmp_orb_changing(b, a));

        Self {
            orb_changing: orb_changing.into_iter(),
            // the non-orb-changing ands are already assumed to be sorted
            non_orb_changing: non_orb_changing.iter().rev(),
        }
    }
}

impl<'r> Iterator for ReverseSortedAnds<'r> {
    type Item = &'r Requirement;

    fn next(&mut self) -> Option<Self::Item> {
        self.orb_changing
            .next()
            .or_else(|| self.non_orb_changing.next())
    }
}

impl ExactSizeIterator for ReverseSortedAnds<'_> {
    fn len(&self) -> usize {
        self.orb_changing.len() + self.non_orb_changing.len()
    }
}

fn max_and(ands: &[Requirement]) -> &Requirement {
    let orb_changing_start = orb_changing_start(ands);

    let orb_changing = &ands[orb_changing_start..];
    // the orb-changing ands could not be sorted, so we still need to find their maximum
    let orb_changing_max = orb_changing.iter().max_by(|a, b| cmp_orb_changing(a, b));

    orb_changing_max.unwrap_or_else(|| {
        // the non-orb-changing ands are already assumed to be sorted
        &ands[orb_changing_start - 1]
    })
}

fn cmp_payload(a: &Requirement, b: &Requirement) -> Ordering {
    match (a, b) {
        (Requirement::Difficulty(a), Requirement::Difficulty(b)) => a.cmp(b),
        (Requirement::Trick(a), Requirement::Trick(b)) => a.cmp(b),
        (Requirement::State(a), Requirement::State(b)) => a.cmp(b),
        (Requirement::Skill(a), Requirement::Skill(b)) => a.cmp(b),
        (Requirement::Shard(a), Requirement::Shard(b)) => a.cmp(b),
        (Requirement::Teleporter(a), Requirement::Teleporter(b)) => a.cmp(b),
        (Requirement::SpiritLight(a), Requirement::SpiritLight(b)) => a.cmp(b),
        (Requirement::GorlekOre(a), Requirement::GorlekOre(b)) => a.cmp(b),
        (Requirement::Keystone(a), Requirement::Keystone(b)) => a.cmp(b),
        (Requirement::Damage(a), Requirement::Damage(b)) => a.cmp(b),
        (Requirement::Danger(a), Requirement::Danger(b)) => a.cmp(b),
        (Requirement::NonConsumingEnergySkill(a), Requirement::NonConsumingEnergySkill(b)) => {
            a.cmp(b)
        }
        (
            Requirement::EnergySkill(a_skill, a_amount),
            Requirement::EnergySkill(b_skill, b_amount),
        ) => a_amount.cmp(b_amount).then(a_skill.cmp(b_skill)),
        (Requirement::ShurikenBreak(a), Requirement::ShurikenBreak(b)) => a.cmp(b),
        (Requirement::SentryBreak(a), Requirement::SentryBreak(b)) => a.cmp(b),
        (Requirement::BreakWall(a), Requirement::BreakWall(b)) => a.cmp(b),
        (Requirement::Boss(a), Requirement::Boss(b)) => a.cmp(b),
        (Requirement::Combat(a), Requirement::Combat(b)) => Ord::cmp(&a.len(), &b.len()),
        _ => Ordering::Equal,
    }
}

fn cmp_simple(a: &Requirement, b: &Requirement) -> Ordering {
    Ord::cmp(&a.discriminant_value(), &b.discriminant_value()).then_with(|| cmp_payload(a, b))
}

fn cmp_orb_changing_payload(a: &Requirement, b: &Requirement) -> Ordering {
    match (a, b) {
        (Requirement::Damage(a), Requirement::Damage(b)) => a.cmp(b),
        (Requirement::Danger(a), Requirement::Danger(b)) => a.cmp(b),
        (Requirement::NonConsumingEnergySkill(a), Requirement::NonConsumingEnergySkill(b)) => {
            a.cmp(b)
        }
        (
            Requirement::EnergySkill(a_skill, a_amount),
            Requirement::EnergySkill(b_skill, b_amount),
        ) => a_amount.cmp(b_amount).then(a_skill.cmp(b_skill)),
        (Requirement::ShurikenBreak(a), Requirement::ShurikenBreak(b)) => a.cmp(b),
        (Requirement::SentryBreak(a), Requirement::SentryBreak(b)) => a.cmp(b),
        (Requirement::BreakWall(a), Requirement::BreakWall(b)) => a.cmp(b),
        (Requirement::Boss(a), Requirement::Boss(b)) => a.cmp(b),
        (Requirement::Combat(a), Requirement::Combat(b)) => Ord::cmp(&a.len(), &b.len()),
        _ => Ordering::Equal,
    }
}

fn cmp_orb_changing(a: &Requirement, b: &Requirement) -> Ordering {
    Ord::cmp(&a.discriminant_value(), &b.discriminant_value())
        .then_with(|| cmp_orb_changing_payload(a, b))
}

fn cmp_ands(a: &Requirement, b: &Requirement) -> Ordering {
    // Assuming `disjunctive_normal_form`, a and b cannot have nested requirements.

    let a_discriminant = a.discriminant_value();
    let b_discriminant = b.discriminant_value();

    match (a_discriminant, b_discriminant) {
        // If EITHER requirement is non-orb-changing, we may reorder.
        (..=LAST_NON_ORB_CHANGING, _) | (_, ..=LAST_NON_ORB_CHANGING) => {
            // First we attempt discriminant-based ordering, which is set up in ascending complexity.
            // Identical discriminants are ordered by payload to ensure consistent ordering.
            Ord::cmp(&a_discriminant, &b_discriminant).then_with(|| cmp_payload(a, b))
        }
        // IF BOTH requirements are orb-changing, we may not reorder.
        (FIRST_ORB_CHANGING.., FIRST_ORB_CHANGING..) => Ordering::Equal,
    }
}

// static CMP_ORS_CHECKER: LazyLock<Mutex<CmpChecker>> =
//     LazyLock::new(|| Mutex::new(CmpChecker::default()));

// #[derive(Default)]
// struct CmpChecker {
//     inner: FxHashMap<Requirement, FxHashMap<Requirement, Ordering>>,
// }

// impl CmpChecker {
//     fn record(&mut self, a: &Requirement, b: &Requirement, ord: Ordering) {
//         for (a, b, ord) in [(a, b, ord), (b, a, ord.reverse())] {
//             match self.inner.entry(a.clone()).or_default().entry(b.clone()) {
//                 Entry::Occupied(occupied) => assert_eq!(*occupied.get(), ord),
//                 Entry::Vacant(vacant) => {
//                     vacant.insert(ord);
//                 }
//             }
//         }
//     }

//     fn clear(&mut self) {
//         self.inner.clear();
//     }

//     fn check(&self) {
//         eprintln!("checking...");

//         for (first, comparisons) in &self.inner {
//             for (second, ord) in comparisons {
//                 let Ordering::Less = ord else { continue };

//                 for (third, ord) in &self.inner[second] {
//                     let Ordering::Less = ord else { continue };

//                     eprintln!("checking {first} < {second} < {third}");

//                     let first_to_third = cmp_ors(first, third);

//                     assert_eq!(first_to_third, Ordering::Less, "{first} < {second} and {second} < {third}, but {first} {first_to_third:?} {third}!");
//                 }
//             }
//         }
//     }
// }

fn cmp_ors(a: &Requirement, b: &Requirement) -> Ordering {
    // Assuming `disjunctive_normal_form`, a and b are Ands most of the time, but occasionally single requirements.
    // We sort Ands by their most expensive requirement(s).

    let ord = match (a, b) {
        // TODO we lose knowledge in these And branches about which requirements were orb-changing
        // That information could be used for more optimized compare implementations
        (Requirement::And(a), Requirement::And(b)) => {
            let mut a_iter = ReverseSortedAnds::new(a);
            let mut b_iter = ReverseSortedAnds::new(b);

            // Cannot use zip because we need the remaining state after
            while let Some(a) = a_iter.next() {
                match b_iter.next() {
                    None => return Ordering::Greater,
                    Some(b) => match cmp_simple(a, b) {
                        Ordering::Equal => {}
                        non_equal => return non_equal,
                    },
                }
            }

            if b_iter.len() == 0 {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
        (Requirement::And(a), b) => {
            let a_max = max_and(a);
            match cmp_simple(a_max, b) {
                Ordering::Less => Ordering::Less,
                _ => Ordering::Greater,
            }
        }
        (a, Requirement::And(b)) => {
            let b_max = max_and(b);
            match cmp_simple(a, b_max) {
                Ordering::Greater => Ordering::Greater,
                _ => Ordering::Less,
            }
        }
        (a, b) => cmp_simple(a, b),
    };

    ord
}

/// Removes `index` from `ands`, unless only a single element would remain,
/// in which case `ControlFlow::Break` is returned with the single element.
/// If `ControlFlow::Break` is returned, the outcome of `ands` is unspecified.
fn remove_and(ands: &mut Vec<Requirement>, index: usize) -> ControlFlow<Requirement> {
    match (ands.len(), index) {
        (1 | 2, 0) => ControlFlow::Break(ands.pop().unwrap()),
        (2, 1) => ControlFlow::Break(ands.drain(..).next().unwrap()),
        _ => {
            ands.remove(index);

            ControlFlow::Continue(())
        }
    }
}

fn insert_by<F>(reqs: &mut Vec<Requirement>, insert: Requirement, mut f: F)
where
    F: FnMut(&Requirement, &Requirement) -> Ordering,
{
    let index = binary_search_index_by(reqs, |or| f(or, &insert));
    reqs.insert(index, insert);
}

fn insert_or(ors: &mut Vec<Requirement>, insert: Requirement) {
    insert_by(ors, insert, cmp_ors);
}

// /// Provides a key to reorder requirements chained with and.
// ///
// /// Not all requirements are commutative, so this may only be used with stable sorts!
// /// Additionally, only one shared key can be allowed for all non-commutative requirements to keep their relative order.
// /// Commutative requirements may have any amount of different keys.
// #[derive(PartialEq, Eq, PartialOrd, Ord)]
// enum ReorderKey {
//     /// Unsolvable requirements are commutative and shortcut everything.
//     Unsolvable,
//     /// Requirements that don't change orbs are commutative and usually create fewer branches.
//     DoesNotChangeOrbs,
//     /// Requirements that change orbs are not commutative.
//     /// They also create many branches, so we order them after all commutative requirements.
//     ChangesOrbs,
// }

// TODO These might be misusing the comparison functions - at this point the requirements are not in disjunctive normal form anymore

// fn insert_orb_changing_and(
//     mut requirement: Requirement,
//     insert: Requirement,
//     side: OrbChangingFactorSide,
// ) -> Requirement {
//     trace!("inserting {insert} into {requirement} at {side:?}");

//     match &mut requirement {
//         Requirement::And(ands) => {
//             let mut insert_index = ands
//                 .iter()
//                 .position(|and| matches!(and, Requirement::Or(_)))
//                 .unwrap();

//             debug_assert_eq!(
//                 insert_index,
//                 ands.iter()
//                     .rposition(|and| matches!(and, Requirement::Or(_)))
//                     .unwrap()
//             );

//             if matches!(side, OrbChangingFactorSide::Back) {
//                 insert_index += 1;
//             }

//             ands.insert(insert_index, insert);

//             requirement
//         }
//         _ => match side {
//             OrbChangingFactorSide::Front => Requirement::And(vec![insert, requirement]),
//             OrbChangingFactorSide::Back => Requirement::And(vec![requirement, insert]),
//         },
//     }
// }

fn factor_out_orb_changes(ors: &mut Vec<Requirement>) -> bool {
    let Some(mut orb_changing_factor) = choose_orb_changing_factor(ors) else {
        return false;
    };

    loop {
        let OrbChangingFactor { side, factor } = orb_changing_factor;

        let factor_requirement = factor.requirement.clone();
        trace!(
            "factoring out {factor_requirement} from ({ors}) at indices {indices}",
            ors = ors.iter().format(" | "),
            indices = factor.indices
        );

        let mut factored = Requirement::or(factor.indices.remove_from_ors(ors).map(|mut or| {
            or.remove_orb_changing_factor(side);
            or
        }));

        factored.heuristically_apply_associativity();

        factored = match side {
            OrbChangingFactorSide::Front => Requirement::and([factor_requirement, factored]),
            OrbChangingFactorSide::Back => Requirement::and([factored, factor_requirement]),
        };

        // factored = insert_orb_changing_and(factored, factor_requirement, side);

        insert_or(ors, factored);

        match choose_orb_changing_factor(ors) {
            None => return true,
            Some(next) => orb_changing_factor = next,
        }
    }
}

fn insert_non_orb_changing_and(mut requirement: Requirement, insert: Requirement) -> Requirement {
    match &mut requirement {
        Requirement::And(ands) => {
            insert_by(ands, insert, cmp_ands);
            requirement
        }
        _ => Requirement::And(vec![insert, requirement]),
    }
}

fn factor_out_non_orb_changes(ors: &mut Vec<Requirement>) -> bool {
    let Some(mut factor) = choose_non_orb_changing_factor(ors) else {
        return false;
    };

    loop {
        let factor_requirement = factor.requirement.clone();
        trace!(
            "factoring out {factor_requirement} from ({})",
            ors.iter().format(" | ")
        );

        let mut factored = Requirement::or(factor.indices.remove_from_ors(ors).map(|mut or| {
            or.remove_non_orb_changing_factor(&factor_requirement);
            or
        }));

        factored.heuristically_apply_associativity();

        factored = insert_non_orb_changing_and(factored, factor_requirement);

        insert_or(ors, factored);

        match choose_non_orb_changing_factor(ors) {
            None => return true,
            Some(next) => factor = next,
        }
    }
}

// TODO if we maintain the ordering rule that orb requirements are in the back then we could maybe do
// a binary search for an index and then return slices for the [non-] orb changing requirements
// But currently it gets broken when factoring out orb requirements to the front

// fn orb_changing_requirements(ands: &[Requirement]) -> &[Requirement] {
//     &ands[orb_changing_requirements_start(ands)..]
// }

// fn non_orb_changing_requirements(ands: &[Requirement]) -> &[Requirement] {
//     &ands[..orb_changing_requirements_start(ands)]
// }

// fn orb_changing_requirements_start(ands: &[Requirement]) -> usize {
//     // TODO is this better than binary search?
//     match ands.iter().rposition(|and| !and.changes_orbs()) {
//         None => 0,
//         Some(last_non_orb_changing) => last_non_orb_changing + 1,
//     }
// }

fn choose_orb_changing_factor(ors: &[Requirement]) -> Option<OrbChangingFactor<'_>> {
    // No information is carried over between iterations, this would be a bit complex.

    trace!("searching factor in ({})", ors.iter().format(" | "));

    let mut finder = OrbChangingFactorFinder::new(ors);

    for (index, or) in ors.iter().enumerate() {
        match or {
            Requirement::And(ands) => {
                let mut orb_changing_requirements = ands.iter().filter(|and| and.changes_orbs());

                if let Some(front) = orb_changing_requirements.next() {
                    let back = orb_changing_requirements.next_back().unwrap_or(front);

                    finder.add(front, back, index);
                }
            }
            // Direct nested Or should not be present in optimized form
            literal => {
                if literal.changes_orbs() {
                    finder.add(literal, literal, index);
                }
            }
        }
    }

    finder.into_max()
}

fn choose_non_orb_changing_factor(ors: &[Requirement]) -> Option<Factor<'_>> {
    let mut occurences = FactorOccurences::new(ors);

    for (index, or) in ors.iter().enumerate() {
        match or {
            Requirement::And(ands) => {
                for requirement in ands.iter().filter(|and| !and.changes_orbs()) {
                    // TODO can we run into nested requirements here again?
                    occurences.add(requirement, index);
                }
            }
            Requirement::Or(_) => {
                // TODO do something? maybe if all branches share the same thing?
            }
            literal => {
                if !literal.changes_orbs() {
                    occurences.add(literal, index);
                }
            }
        }
    }

    occurences.into_max()
}

struct FactorOccurences<'r> {
    inner: Vec<(&'r Requirement, Vec<usize>)>,
}

impl<'r> FactorOccurences<'r> {
    fn new(ors: &[Requirement]) -> Self {
        Self {
            inner: Vec::with_capacity(ors.len()),
        }
    }

    fn get_indices(&mut self, requirement: &'r Requirement) -> Option<&mut Vec<usize>> {
        self.inner
            .iter_mut()
            .find(|(r, _)| *r == requirement)
            .map(|(_, indices)| indices)
    }

    fn push_new(&mut self, requirement: &'r Requirement, index: usize) {
        self.inner.push((requirement, vec![index]));
    }

    fn add(&mut self, requirement: &'r Requirement, index: usize) {
        match self.get_indices(requirement) {
            None => self.push_new(requirement, index),
            Some(indices) => indices.push(index),
        }
    }

    fn into_max(self) -> Option<Factor<'r>> {
        // trace!(
        //     "found candidates [{}]",
        //     self.inner
        //         .iter()
        //         .format_with(", ", |(requirement, indices), f| f(&format_args!(
        //             "{requirement} ({indices:?})"
        //         )))
        // );

        self.inner
            .into_iter()
            .max_by_key(|(_, indices)| indices.len())
            .and_then(|(requirement, indices)| Factor::new(requirement, indices))
    }
}

struct Factor<'r> {
    requirement: &'r Requirement,
    indices: FactorIndices,
}

impl<'r> Factor<'r> {
    fn new(requirement: &'r Requirement, indices: Vec<usize>) -> Option<Self> {
        (indices.len() > 1).then_some(Self {
            requirement,
            indices: FactorIndices::new(indices),
        })
    }
}

struct FactorIndices {
    inner: Vec<usize>,
}

impl FactorIndices {
    fn new(indices: Vec<usize>) -> Self {
        Self { inner: indices }
    }

    fn remove_from_ors<'r>(self, ors: &'r mut Vec<Requirement>) -> RemoveFactorFromOrs<'r> {
        RemoveFactorFromOrs::new(self.inner, ors)
    }
}

impl Deref for FactorIndices {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for FactorIndices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.iter().format(", ").fmt(f)
    }
}

struct RemoveFactorFromOrs<'r> {
    indices: vec::IntoIter<usize>,
    ors: &'r mut Vec<Requirement>,
}

impl<'r> RemoveFactorFromOrs<'r> {
    fn new(indices: Vec<usize>, ors: &'r mut Vec<Requirement>) -> Self {
        Self {
            indices: indices.into_iter(),
            ors,
        }
    }
}

impl Iterator for RemoveFactorFromOrs<'_> {
    type Item = Requirement;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.next()?;
        Some(mem::replace(&mut self.ors[index], Requirement::Impossible))
    }
}

impl Drop for RemoveFactorFromOrs<'_> {
    fn drop(&mut self) {
        self.for_each(drop);
        self.ors.retain(|or| !matches!(or, Requirement::Impossible));
    }
}

struct OrbChangingFactor<'r> {
    side: OrbChangingFactorSide,
    factor: Factor<'r>,
}

impl<'r> OrbChangingFactor<'r> {
    fn front(factor: Factor<'r>) -> Self {
        Self {
            side: OrbChangingFactorSide::Front,
            factor,
        }
    }

    fn back(factor: Factor<'r>) -> Self {
        Self {
            side: OrbChangingFactorSide::Back,
            factor,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OrbChangingFactorSide {
    Front,
    Back,
}

struct OrbChangingFactorFinder<'r> {
    min_relevant_priority: OrbChangeKind,
    front: OrbChangingFactorFinderSide<'r>,
    back: OrbChangingFactorFinderSide<'r>,
}

impl<'r> OrbChangingFactorFinder<'r> {
    fn new(ors: &[Requirement]) -> Self {
        Self {
            min_relevant_priority: OrbChangeKind::Health,
            front: OrbChangingFactorFinderSide::new(ors),
            back: OrbChangingFactorFinderSide::new(ors),
        }
    }

    fn add(&mut self, front: &'r Requirement, back: &'r Requirement, index: usize) {
        if let Some(new_priority) = self.front.add(front, index, self.min_relevant_priority) {
            self.min_relevant_priority = new_priority;
            self.back.occurences.inner.clear();
        }

        if let Some(new_priority) = self.back.add(back, index, self.min_relevant_priority) {
            self.min_relevant_priority = new_priority;
            self.front.occurences.inner.clear();
        }
    }

    fn into_max(self) -> Option<OrbChangingFactor<'r>> {
        match (self.front.into_max(), self.back.into_max()) {
            (None, None) => None,
            (Some(front), None) => Some(OrbChangingFactor::front(front)),
            (None, Some(back)) => Some(OrbChangingFactor::back(back)),
            (Some(front), Some(back)) => {
                if front.indices.len() > back.indices.len() {
                    Some(OrbChangingFactor::front(front))
                } else {
                    Some(OrbChangingFactor::back(back))
                }
            }
        }
    }
}

struct OrbChangingFactorFinderSide<'r> {
    occurences: FactorOccurences<'r>,
}

impl<'r> OrbChangingFactorFinderSide<'r> {
    fn new(ors: &[Requirement]) -> Self {
        Self {
            occurences: FactorOccurences::new(ors),
        }
    }

    #[must_use]
    fn add(
        &mut self,
        requirement: &'r Requirement,
        index: usize,
        min_relevant_priority: OrbChangeKind,
    ) -> Option<OrbChangeKind> {
        let requirement_priority = requirement.orb_change_kind().unwrap();

        if requirement_priority < min_relevant_priority {
            return None;
        }

        match self.occurences.get_indices(requirement) {
            None => self.occurences.push_new(requirement, index),
            Some(indices) => {
                if requirement_priority > min_relevant_priority {
                    // since we have a factor candidate in this priority now, previous priorities become irrelevant

                    let mut indices = mem::take(indices);
                    indices.push(index);

                    self.occurences.inner.clear();
                    self.occurences.inner.push((requirement, indices));

                    return Some(requirement_priority);
                } else {
                    indices.push(index);
                }
            }
        }

        None
    }

    fn into_max(self) -> Option<Factor<'r>> {
        self.occurences.into_max()
    }
}
