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

use crate::{
    logic_language::output::{Anchor, Connection, Entrance, Graph, Node, Refill, Requirement},
    partial_then::partial_then_iters,
    PartialThen, Skill,
};

impl Graph {
    pub fn optimize(&mut self) {
        self.nodes.optimize();
        self.extern_requirements.optimize();
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
        if !matches!(self, Self::And(_) | Self::Or(_)) {
            return;
        }

        let before = log_enabled!(Debug).then(|| {
            let before = self.to_string();

            trace!("optimizing {before}");

            before
        });

        match self {
            // Not destructuring the top-level and loses some minor optimization potential,
            // but the difference is not measurable and this makes the optimization itself faster.
            Self::And(ands) => {
                let mut index = 0;
                while index < ands.len() {
                    let and = &mut ands[index];

                    and.optimize();

                    if let Self::And(nested) = and {
                        let nested = mem::take(nested);
                        let range = index..index + 1;
                        index += nested.len();
                        ands.splice(range, nested);
                    } else {
                        index += 1;
                    }
                }
            }
            Self::Or(_) => self.optimize_or(),
            _ => unreachable!(),
        }

        self.improve_order();

        trace!("improved order: -> {self}");

        if let Some(before) = &before {
            debug!("optimized {before} -> {self}");
        }
    }
}

macro_rules! debug_assert_shape {
    ($f:ident($($arg:expr),+): $($check: ident),+) => {
        {
            for arg in [$(&$arg),+] {
                $(
                    debug_assert!(
                        arg.$check(),
                        concat!(
                            "called `",
                            stringify!($f),
                            "` failing `",
                            stringify!($check),
                            "` on {}",
                        ),
                        arg
                    );
                )+
            }
        }
    };
}

impl Requirement {
    fn optimize_or(&mut self) {
        self.disjunctive_normal_form();

        trace!("disjunctive normal form: -> {self}");

        self.normalize_order();

        trace!("normalized order: -> {self}");

        self.dedup_literals();
        self.filter_redundant_conjunctions();

        trace!("filtered redundant ors: -> {self}");

        self.heuristically_apply_associativity();

        trace!("applied associativity: -> {self}");
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
                        Self::And(ands) => {
                            for conjunction in disjunction.iter_mut() {
                                conjunction.append(&mut ands.clone());
                            }
                        }
                        Self::Or(ors) => {
                            disjunction = disjunction
                                .into_iter()
                                .cartesian_product(ors)
                                .map(|(mut a, b)| {
                                    match b {
                                        Self::And(mut ands) => a.append(&mut ands),
                                        literal => a.push(literal),
                                    }

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

                *self = Self::or(
                    disjunction
                        .into_iter()
                        .map(|conjunction| Self::and_from(conjunction)),
                );
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
        match self {
            Self::And(ands) => ands.iter().all(Self::is_literal),
            Self::Or(ors) => ors.iter().all(|or| match or {
                Self::And(ands) => ands.iter().all(Self::is_literal),
                Self::Or(_) => false,
                _ => true,
            }),
            _ => true,
        }
    }

    fn is_conjunction(&self) -> bool {
        match self {
            Self::And(ands) => ands.iter().all(Self::is_literal),
            Self::Or(_) => false,
            _ => true,
        }
    }

    const fn is_literal(&self) -> bool {
        !matches!(self, Self::And(_) | Self::Or(_))
    }

    /// Normalizes the order of contained requirements.
    ///
    /// Less costly requirements will be ordered earlier where possible to allow faster shortcuts.
    ///
    /// # Prerequisites
    ///
    /// - [`Requirement::disjunctive_normal_form`] (is preserved)
    fn normalize_order(&mut self) {
        debug_assert_shape!(normalize_order(self): is_disjunctive_normal_form);

        // This is never an and because we only run the optimization pipeline on ors
        // and at this point it won't be transformed to that degree yet
        debug_assert!(
            !matches!(self, Self::And(_)),
            "normalize_order does not currently account for top-level ands"
        );

        if let Self::Or(conjunctions) = self {
            for conjunction in conjunctions.iter_mut() {
                if let Self::And(literals) = conjunction {
                    literals.sort_by(Self::and_order_cmp_literals);
                }
            }

            conjunctions.sort_by(Self::or_order_cmp_conjunctions);
        }
    }

    fn and_order_cmp_literals(&self, other: &Self) -> Ordering {
        debug_assert_shape!(and_order_cmp_literals(self, other): is_literal);

        let a_discriminant = self.discriminant_value();
        let b_discriminant = other.discriminant_value();

        match (a_discriminant, b_discriminant) {
            // If EITHER requirement is non-orb-changing, we may reorder.
            (..=Self::LAST_NON_ORB_CHANGING, _) | (_, ..=Self::LAST_NON_ORB_CHANGING) => {
                // First we attempt discriminant-based ordering, which is set up in ascending complexity.
                // Identical discriminants are ordered by payload to ensure consistent ordering.
                Ord::cmp(&a_discriminant, &b_discriminant)
                    .then_with(|| self.order_cmp_non_orb_changing_literal_payload(other))
            }
            // IF BOTH requirements are orb-changing, we may not reorder.
            (Self::FIRST_ORB_CHANGING.., Self::FIRST_ORB_CHANGING..) => Ordering::Equal,
        }
    }

    fn order_cmp_ors_by<'a, S, I, C, M, L>(
        &'a self,
        other: &'a Self,
        mut rev_sort: S,
        mut cmp_and: C,
        mut max: M,
        mut cmp_literal: L,
    ) -> Ordering
    where
        S: FnMut(&'a [Requirement]) -> I,
        I: IntoIterator<Item = &'a Requirement>,
        I::IntoIter: ExactSizeIterator,
        C: FnMut(&Requirement, &Requirement) -> Ordering,
        M: FnMut(&[Requirement]) -> &Requirement,
        L: FnMut(&Requirement, &Requirement) -> Ordering,
    {
        debug_assert_shape!(order_cmp_ors_by(self, other): is_not_or);

        // We sort Ands by their most expensive requirement(s).

        match (self, other) {
            (Self::And(a), Self::And(b)) => cmp_iter_by(rev_sort(a), rev_sort(b), cmp_and),
            (Self::And(a), b) => match cmp_and(max(a), b) {
                Ordering::Less => Ordering::Less,
                _ => Ordering::Greater,
            },
            (a, Self::And(b)) => match cmp_and(a, max(b)) {
                Ordering::Greater => Ordering::Greater,
                _ => Ordering::Less,
            },
            (a, b) => cmp_literal(a, b),
        }
    }

    fn or_order_cmp_conjunctions(&self, other: &Self) -> Ordering {
        debug_assert_shape!(or_order_cmp_conjunctions(self, other): is_conjunction);

        self.order_cmp_ors_by(
            other,
            ReverseSorted::literals,
            Self::or_order_cmp_literals,
            max_literal,
            Self::or_order_cmp_literals,
        )
    }

    fn or_order_cmp_literals(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.discriminant_value(), &other.discriminant_value())
            .then_with(|| self.or_order_cmp_literal_payload(other))
    }

    fn or_order_cmp_literal_payload(&self, other: &Self) -> Ordering {
        self.order_cmp_non_orb_changing_literal_payload(other)
            .then_with(|| self.order_cmp_orb_changing_literal_payload(other))
    }

    fn order_cmp_orb_changing_literal(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.discriminant_value(), &other.discriminant_value())
            .then_with(|| self.order_cmp_orb_changing_literal_payload(other))
    }

    fn order_cmp_orb_changing_literal_payload(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Danger(a), Self::Danger(b))
            | (Self::Damage(a), Self::Damage(b))
            | (Self::ShurikenBreak(a), Self::ShurikenBreak(b))
            | (Self::SentryBreak(a), Self::SentryBreak(b))
            | (Self::BreakWall(a), Self::BreakWall(b))
            | (Self::Boss(a), Self::Boss(b)) => a.total_cmp(b),
            (Self::NonConsumingEnergySkill(a), Self::NonConsumingEnergySkill(b)) => a.cmp(b),
            (Self::EnergySkill(a_skill, a_amount), Self::EnergySkill(b_skill, b_amount)) => {
                a_amount.total_cmp(b_amount).then(a_skill.cmp(b_skill))
            }
            (Self::Extern(a), Self::Extern(b)) => a.cmp(b),
            (Self::Combat(a), Self::Combat(b)) => Ord::cmp(&a.len(), &b.len()).then_with(|| {
                a.iter()
                    .map(|(_, amount)| *amount)
                    .cmp(b.iter().map(|(_, amount)| *amount))
            }),
            _ => Ordering::Equal,
        }
    }

    fn order_cmp_non_orb_changing_literal(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.discriminant_value(), &other.discriminant_value())
            .then_with(|| self.order_cmp_non_orb_changing_literal_payload(other))
    }

    fn order_cmp_non_orb_changing_literal_payload(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Difficulty(a), Self::Difficulty(b)) => a.cmp(b),
            (Self::Trick(a), Self::Trick(b)) => a.cmp(b),
            (Self::State(a), Self::State(b))
            | (Self::SpiritLight(a), Self::SpiritLight(b))
            | (Self::GorlekOre(a), Self::GorlekOre(b))
            | (Self::Keystone(a), Self::Keystone(b)) => a.cmp(b),
            (Self::Skill(a), Self::Skill(b)) => a.cmp(b),
            (Self::Shard(a), Self::Shard(b)) => a.cmp(b),
            (Self::Teleporter(a), Self::Teleporter(b)) => a.cmp(b),
            _ => Ordering::Equal,
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
            Self::And(ands) => is_normalized_order_by(ands, Self::and_order_cmp_literals),
            Self::Or(ors) => is_normalized_order_by(ors, Self::or_order_cmp_conjunctions),
            _ => true,
        }
    }

    /// [`Vec::dedup`] for `Requirement`, making sure not to dedup orb-changing literals.
    ///
    /// # Prerequisites
    ///
    /// - [`Requirement::disjunctive_normal_form`] (is preserved)
    /// - [`Requirement::normalize_order`] **(not preserved since the length of conjunctions may change)**
    fn dedup_literals(&mut self) {
        debug_assert_shape!(dedup_literals(self): is_disjunctive_normal_form, is_normalized_order);

        #[inline]
        fn dedup_impl(literals: &mut Vec<Requirement>) {
            // Adapted from SmallVec::dedup

            let mut write: usize = 1;

            for read in 1..literals.len() {
                let literal_read = &literals[read];

                if literal_read.changes_orbs() {
                    literals.drain(write..read);
                    return;
                }

                if literal_read == &literals[write - 1] {
                    trace!(
                        "removing duplicate {literal_read} from {}",
                        literals.iter().format(" & ")
                    );
                } else {
                    literals.swap(read, write);
                    write += 1;
                }
            }

            literals.truncate(write);
        }

        match self {
            Self::And(literals) => {
                dedup_impl(literals);

                // Yes, this happens sometimes oriLol
                if literals.len() == 1 {
                    *self = literals.pop().unwrap();
                }
            }
            Self::Or(conjunctions) => {
                for conjunction in conjunctions.iter_mut() {
                    conjunction.dedup_literals();
                }

                // Redundant conjunctions will be caught by `filter_redundant_conjunctions`
            }
            _ => {}
        }
    }

    /// Filters redundant or branches.
    ///
    /// # Prerequisites
    ///
    /// - [`Requirement::disjunctive_normal_form`] (is preserved)
    fn filter_redundant_conjunctions(&mut self) {
        debug_assert_shape!(filter_redundant_conjunctions(self): is_disjunctive_normal_form);

        let Self::Or(conjunctions) = self else {
            return;
        };

        'outer: for index in 0..conjunctions.len() {
            let (head, tail) = conjunctions[index..].split_first_mut().unwrap();

            if matches!(head, Self::Impossible) {
                continue;
            }

            for tail_item in tail
                .iter_mut()
                .filter(|tail_item| !matches!(tail_item, Self::Impossible))
            {
                match head.logical_cmp_conjunctions(tail_item) {
                    Some(Ordering::Less) => {
                        trace!("{tail_item} is redundant with {head}");

                        *tail_item = Self::Impossible;
                    }
                    Some(Ordering::Equal | Ordering::Greater) => {
                        trace!("{head} is redundant with {tail_item}");

                        *head = Self::Impossible;

                        continue 'outer;
                    }
                    None => {}
                }
            }
        }

        conjunctions.retain(|conjunction| !matches!(conjunction, Self::Impossible));

        if conjunctions.len() == 1 {
            *self = conjunctions.pop().unwrap();
        }
    }

    /// Specialized variant of [`Requirement::logical_cmp`] with the knowledge
    /// that the compared requirements are conjunctions and in normalized order.
    fn logical_cmp_conjunctions(&self, other: &Self) -> Option<Ordering> {
        debug_assert_shape!(logical_cmp_conjunctions(self, other): is_conjunction, is_non_trivial);

        match (self, other) {
            (Self::And(a), Self::And(b)) => {
                fn is_fully_contained(a: &[Requirement], b: &[Requirement]) -> bool {
                    let mut b_iter = b.iter();
                    a.iter().all(|a| b_iter.any(|b| a.logical_le_literal(b)))
                }

                let mut index = 0;
                let mut ordering = Ordering::Equal;

                let a_len = a.len();
                while index < a_len {
                    let a_literal = &a[index];

                    match b.get(index) {
                        None => return ordering.partial_then(Ordering::Greater),
                        Some(b_literal) => match a_literal.logical_cmp_literals(b_literal) {
                            None => {
                                // The current requirements aren't directly comparable, but there might still be an ordering
                                // if for example a = (x & y & z) and b = (x & z).

                                if matches!(ordering, Ordering::Less | Ordering::Equal)
                                    && is_fully_contained(&a[index..], &b[index..])
                                {
                                    return Some(Ordering::Less);
                                }
                                if matches!(ordering, Ordering::Equal | Ordering::Greater)
                                    && is_fully_contained(&b[index..], &a[index..])
                                {
                                    return Some(Ordering::Greater);
                                }
                                return None;
                            }
                            Some(other) => ordering = ordering.partial_then(other)?,
                        },
                    }

                    index += 1;
                }

                if index == b.len() {
                    Some(ordering)
                } else {
                    ordering.partial_then(Ordering::Less)
                }
            }
            (Self::And(many), one) => many
                .iter()
                .any(|many_item| one.logical_le_literal(many_item))
                .then_some(Ordering::Greater),
            (one, Self::And(many)) => many
                .iter()
                .any(|many_item| one.logical_le_literal(many_item))
                .then_some(Ordering::Less),
            _ => self.logical_cmp_literals(other),
        }
    }

    /// Specialized variant of [`Requirement::logical_cmp`] with the knowledge that the compared requirements are literals.
    fn logical_cmp_literals(&self, other: &Self) -> Option<Ordering> {
        debug_assert_shape!(logical_cmp_literals(self, other): is_literal, is_non_trivial);

        match (self, other) {
            (Self::Difficulty(a), Self::Difficulty(b)) => Some(a.cmp(b)),
            (Self::Trick(a), Self::Trick(b)) => (a == b).then_some(Ordering::Equal),
            (Self::State(a), Self::State(b)) | (Self::Extern(a), Self::Extern(b)) => {
                (a == b).then_some(Ordering::Equal)
            }
            (Self::NonConsumingEnergySkill(a), Self::NonConsumingEnergySkill(b))
            | (Self::Skill(a), Self::Skill(b)) => (a == b).then_some(Ordering::Equal),
            (Self::Shard(a), Self::Shard(b)) => (a == b).then_some(Ordering::Equal),
            (Self::Teleporter(a), Self::Teleporter(b)) => (a == b).then_some(Ordering::Equal),
            (Self::SpiritLight(a), Self::SpiritLight(b))
            | (Self::GorlekOre(a), Self::GorlekOre(b))
            | (Self::Keystone(a), Self::Keystone(b)) => Some(a.cmp(b)),
            (Self::EnergySkill(a, a_amount), Self::EnergySkill(b, b_amount)) => {
                (a == b).then_some(a_amount.total_cmp(b_amount))
            }
            (Self::Danger(a), Self::Danger(b))
            | (Self::Damage(a), Self::Damage(b))
            | (Self::ShurikenBreak(a), Self::ShurikenBreak(b))
            | (Self::SentryBreak(a), Self::SentryBreak(b))
            | (Self::BreakWall(a), Self::BreakWall(b))
            | (Self::Boss(a), Self::Boss(b)) => Some(a.total_cmp(b)),
            (Self::Combat(a), Self::Combat(b)) => {
                partial_then_iters(a, b, |(a_enemy, a_amount), (b_enemy, b_amount)| {
                    (a_enemy == b_enemy).then(|| a_amount.cmp(b_amount))
                })
            }
            // Non-consuming Danger can't be more expensive than Damage.
            (Self::Danger(a), Self::Damage(b)) => (a <= b).then_some(Ordering::Less),
            (Self::Damage(a), Self::Danger(b)) => (a >= b).then_some(Ordering::Greater),
            // Non-energy-consuming skill use can't be more expensive than energy-consuming skill use.
            (Self::Skill(a), Self::NonConsumingEnergySkill(b) | Self::EnergySkill(b, _))
            | (Self::NonConsumingEnergySkill(a), Self::EnergySkill(b, _)) => {
                (a == b).then_some(Ordering::Less)
            }
            (Self::EnergySkill(a, _), Self::NonConsumingEnergySkill(b) | Self::Skill(b))
            | (Self::NonConsumingEnergySkill(a), Self::Skill(b)) => {
                (a == b).then_some(Ordering::Greater)
            }
            // Having weapons or using them once can't be more expensive than breaking something.
            (
                Self::Skill(Skill::Shuriken) | Self::EnergySkill(Skill::Shuriken, ..=1.),
                Self::ShurikenBreak(_),
            )
            | (
                Self::Skill(Skill::Sentry) | Self::EnergySkill(Skill::Sentry, ..=1.),
                Self::SentryBreak(_),
            )
            | (
                Self::Skill(Skill::Sword | Skill::Hammer)
                | Self::EnergySkill(
                    // TODO add Sentry once allowed
                    Skill::Grenade | Skill::Spear | Skill::Bow | Skill::Shuriken | Skill::Blaze,
                    ..=1.,
                ),
                Self::BreakWall(_) | Self::Boss(_),
            )
            | (Self::EnergySkill(Skill::Flash, ..=1.), Self::Boss(_)) => Some(Ordering::Less),
            (
                Self::ShurikenBreak(_),
                Self::Skill(Skill::Shuriken) | Self::EnergySkill(Skill::Shuriken, ..=1.),
            )
            | (
                Self::SentryBreak(_),
                Self::Skill(Skill::Sentry) | Self::EnergySkill(Skill::Sentry, ..=1.),
            )
            | (
                Self::BreakWall(_) | Self::Boss(_),
                Self::Skill(Skill::Sword | Skill::Hammer)
                | Self::EnergySkill(
                    // TODO add Sentry once allowed
                    Skill::Grenade | Skill::Spear | Skill::Bow | Skill::Shuriken | Skill::Blaze,
                    ..=1.,
                ),
            )
            | (Self::Boss(_), Self::EnergySkill(Skill::Flash, ..=1.)) => Some(Ordering::Greater),
            // Comparing Combat to skill use seems difficult with how much depends on the enemies and difficulty...
            _ => {
                // catch for unit variants
                (self.discriminant_value() == other.discriminant_value()).then_some(Ordering::Equal)
            }
        }
    }

    fn logical_le_literal(&self, other: &Self) -> bool {
        matches!(
            self.logical_cmp_literals(other),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    const fn is_non_trivial(&self) -> bool {
        !matches!(self, Self::Free | Self::Impossible)
    }

    /// Tries to create nested groups to avoid redundant checks across or branches.
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

        if ors.len() == 1 {
            *self = ors.pop().unwrap();
        }
    }

    fn factor_priority(&self) -> Option<OrbChangingFactorPriority> {
        match self {
            Self::Danger(_) | Self::Damage(_) => Some(OrbChangingFactorPriority::Health),
            Self::NonConsumingEnergySkill(_)
            | Self::EnergySkill(_, _)
            | Self::ShurikenBreak(_)
            | Self::SentryBreak(_)
            | Self::Extern(_) => Some(OrbChangingFactorPriority::Energy),
            Self::BreakWall(_) | Self::Boss(_) | Self::Combat(_) => {
                Some(OrbChangingFactorPriority::WeaponChoice)
            }
            Self::And(requirements) | Self::Or(requirements) => {
                requirements.iter().filter_map(Self::factor_priority).max()
            }
            _ => None,
        }
    }

    fn remove_orb_changing_factor(&mut self, side: OrbChangingFactorSide) {
        match side {
            OrbChangingFactorSide::Front => self.remove_front_orb_changing_factor(),
            OrbChangingFactorSide::Back => self.remove_back_orb_changing_factor(),
        }
    }

    fn remove_front_orb_changing_factor(&mut self) {
        debug_assert_shape!(remove_front_orb_changing_factor(self): is_not_or);

        match self {
            Self::And(ands) => {
                let index = ands.iter().position(Self::changes_orbs).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            _ => *self = Self::Free,
        }
    }

    fn remove_back_orb_changing_factor(&mut self) {
        debug_assert_shape!(remove_back_orb_changing_factor(self): is_not_or);

        match self {
            Self::And(ands) => {
                let index = ands.iter().rposition(Self::changes_orbs).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            _ => *self = Self::Free,
        }
    }

    fn remove_non_orb_changing_factor(&mut self, factor: &Requirement) {
        debug_assert_shape!(remove_non_orb_changing_factor(self): is_not_or);

        match self {
            Self::And(ands) => {
                let index = ands.iter().position(|and| and == factor).unwrap();

                if let ControlFlow::Break(single) = remove_and(ands, index) {
                    *self = single;
                }
            }
            _ => *self = Self::Free,
        }
    }

    /// Improves the order of contained requirements.
    ///
    /// Less costly requirements will be ordered earlier where possible to allow faster shortcuts.
    ///
    /// Like [`Requirement::normalize_order`], but with no prerequisites, weaker guarantees about the outcome and slower performance.
    fn improve_order(&mut self) {
        match self {
            Self::And(ands) => {
                for and in ands.iter_mut() {
                    and.improve_order();
                }

                ands.sort_by(Self::and_order_cmp);
            }
            Self::Or(ors) => {
                for or in ors.iter_mut() {
                    or.improve_order();
                }

                ors.sort_by(Self::or_order_cmp);
            }
            _ => {}
        }
    }

    fn and_order_cmp(&self, other: &Self) -> Ordering {
        match (self.changes_orbs(), other.changes_orbs()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => self.and_order_cmp_non_orb_changing(other),
        }
    }

    fn order_cmp_ands_by<C, L>(&self, other: &Self, mut cmp_ors: C, mut cmp_literals: L) -> Ordering
    where
        C: FnMut(&Requirement, &Requirement) -> Ordering,
        L: FnMut(&Requirement, &Requirement) -> Ordering,
    {
        debug_assert_shape!(and_order_cmp_by(self, other): is_not_and);

        // We sort Ors by their most expensive requirement(s).

        match (self, other) {
            (Self::Or(a), Self::Or(b)) => {
                cmp_iter_by(trivial_rev_sort(a), trivial_rev_sort(b), cmp_ors)
            }
            (Self::Or(a), b) => match cmp_ors(trivial_max(a), b) {
                Ordering::Less => Ordering::Less,
                _ => Ordering::Greater,
            },
            (a, Self::Or(b)) => match cmp_ors(a, trivial_max(b)) {
                Ordering::Greater => Ordering::Greater,
                _ => Ordering::Less,
            },
            (a, b) => cmp_literals(a, b),
        }
    }

    fn and_order_cmp_non_orb_changing(&self, other: &Self) -> Ordering {
        debug_assert_shape!(and_order_cmp_non_orb_changing(self, other): is_not_orb_changing);

        self.order_cmp_ands_by(
            other,
            Self::or_order_cmp,
            Self::order_cmp_non_orb_changing_literal,
        )
    }

    fn or_order_cmp(&self, other: &Self) -> Ordering {
        self.order_cmp_ors_by(
            other,
            ReverseSorted::ands,
            Self::or_order_cmp_ands,
            max_and,
            Self::or_order_cmp_literals,
        )
    }

    fn or_order_cmp_ands(&self, other: &Self) -> Ordering {
        self.order_cmp_ands_by(other, Self::or_order_cmp, Self::or_order_cmp_literals)
    }

    fn or_order_cmp_orb_changing_ands(&self, other: &Self) -> Ordering {
        debug_assert_shape!(or_order_cmp_orb_changing_ands(self, other): changes_orbs);

        self.order_cmp_ands_by(
            other,
            Self::or_order_cmp,
            Self::order_cmp_orb_changing_literal,
        )
    }

    fn changes_orbs_literal(&self) -> bool {
        debug_assert_shape!(changes_orbs_literal(self): is_literal);

        self.discriminant_value() >= Requirement::FIRST_ORB_CHANGING
    }

    fn is_not_and(&self) -> bool {
        !matches!(self, Self::And(_))
    }

    fn is_not_or(&self) -> bool {
        !matches!(self, Self::Or(_))
    }

    fn is_not_orb_changing(&self) -> bool {
        !self.changes_orbs()
    }
}

/// rev_sort to be used when the slice is already sorted
fn trivial_rev_sort(ands: &[Requirement]) -> Rev<slice::Iter<'_, Requirement>> {
    ands.iter().rev()
}

/// max to be used when the slice is already sorted
fn trivial_max(ands: &[Requirement]) -> &Requirement {
    ands.last().unwrap()
}

fn cmp_iter_by<'a, A, B, F>(a: A, b: B, mut f: F) -> Ordering
where
    A: IntoIterator<Item = &'a Requirement>,
    B: IntoIterator<Item = &'a Requirement>,
    B::IntoIter: ExactSizeIterator,
    F: FnMut(&'a Requirement, &'a Requirement) -> Ordering,
{
    let mut a_iter = a.into_iter();
    let mut b_iter = b.into_iter();

    // Cannot use zip because we need the remaining state after
    while let Some(a_item) = a_iter.next() {
        match b_iter.next() {
            None => return Ordering::Greater,
            Some(b_item) => match f(a_item, b_item) {
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

/// Temporary reverse-sorted view into ands that could not be fully sorted because of orb-changing requirements
struct ReverseSorted<'r> {
    orb_changing: vec::IntoIter<&'r Requirement>,
    non_orb_changing: Rev<slice::Iter<'r, Requirement>>,
}

impl<'r> ReverseSorted<'r> {
    fn new<F, C>(ands: &'r [Requirement], changes_orbs: F, mut cmp_orb_changing: C) -> Self
    where
        F: FnMut(&Requirement) -> bool,
        C: FnMut(&Requirement, &Requirement) -> Ordering,
    {
        let orb_changing_start = orb_changing_start_by(ands, changes_orbs);
        let (non_orb_changing, orb_changing) = ands.split_at(orb_changing_start);

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

    fn literals(literals: &'r [Requirement]) -> Self {
        Self::new(
            literals,
            Requirement::changes_orbs_literal,
            Requirement::order_cmp_orb_changing_literal,
        )
    }

    fn ands(ands: &'r [Requirement]) -> Self {
        Self::new(
            ands,
            Requirement::changes_orbs,
            Requirement::or_order_cmp_orb_changing_ands,
        )
    }
}

impl<'r> Iterator for ReverseSorted<'r> {
    type Item = &'r Requirement;

    fn next(&mut self) -> Option<Self::Item> {
        self.orb_changing
            .next()
            .or_else(|| self.non_orb_changing.next())
    }
}

impl ExactSizeIterator for ReverseSorted<'_> {
    fn len(&self) -> usize {
        self.orb_changing.len() + self.non_orb_changing.len()
    }
}

fn max_by<F, C>(ands: &[Requirement], changes_orbs: F, mut cmp_orb_changing: C) -> &Requirement
where
    F: FnMut(&Requirement) -> bool,
    C: FnMut(&Requirement, &Requirement) -> Ordering,
{
    let orb_changing_start = orb_changing_start_by(ands, changes_orbs);

    let orb_changing = &ands[orb_changing_start..];
    // the orb-changing ands could not be sorted, so we still need to find their maximum
    let orb_changing_max = orb_changing.iter().max_by(|a, b| cmp_orb_changing(a, b));

    orb_changing_max.unwrap_or_else(|| {
        // the non-orb-changing ands are already assumed to be sorted
        &ands[orb_changing_start - 1]
    })
}

fn max_literal(literals: &[Requirement]) -> &Requirement {
    max_by(
        literals,
        Requirement::changes_orbs_literal,
        Requirement::order_cmp_orb_changing_literal,
    )
}

fn max_and(ands: &[Requirement]) -> &Requirement {
    max_by(
        ands,
        Requirement::changes_orbs,
        Requirement::or_order_cmp_orb_changing_ands,
    )
}

fn orb_changing_start_by<F>(literals: &[Requirement], mut changes_orbs: F) -> usize
where
    F: FnMut(&Requirement) -> bool,
{
    binary_search_index_by(literals, |and| {
        if changes_orbs(and) {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    })
}

fn binary_search_index_by<T, F>(slice: &[T], mut f: F) -> usize
where
    T: Display,
    F: FnMut(&T) -> Ordering,
{
    debug_assert!(
        slice.is_sorted_by(|a, b| f(a) <= f(b)),
        "attempted binary search on unsorted slice [{}]",
        slice.iter().format(", ")
    );

    match slice.binary_search_by(f) {
        Ok(index) => index,
        Err(index) => index,
    }
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

        ors.push(factored);

        match choose_orb_changing_factor(ors) {
            None => return true,
            Some(next) => orb_changing_factor = next,
        }
    }
}

fn insert_non_orb_changing_and(mut requirement: Requirement, insert: Requirement) -> Requirement {
    match &mut requirement {
        Requirement::And(ands) => {
            ands.push(insert);
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

        ors.push(factored);

        match choose_non_orb_changing_factor(ors) {
            None => return true,
            Some(next) => factor = next,
        }
    }
}

fn choose_orb_changing_factor(ors: &[Requirement]) -> Option<OrbChangingFactor<'_>> {
    trace!("searching factor in ({})", ors.iter().format(" | "));

    let mut finder = OrbChangingFactorFinder::new(ors);

    for (index, or) in ors.iter().enumerate() {
        // Direct nested Or should not be present in optimized form
        debug_assert_shape!(choose_orb_changing_factor(or): is_not_or);

        match or {
            Requirement::And(ands) => {
                let mut orb_changing_requirements = ands.iter().filter(|and| and.changes_orbs());

                if let Some(front) = orb_changing_requirements.next() {
                    let back = orb_changing_requirements.next_back().unwrap_or(front);

                    finder.add(front, back, index);
                }
            }
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
        // Direct nested Or should not be present in optimized form
        debug_assert_shape!(choose_non_orb_changing_factor(or): is_not_or);

        match or {
            Requirement::And(ands) => {
                for requirement in ands.iter().filter(|and| !and.changes_orbs()) {
                    occurences.add(requirement, index);
                }
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
    min_relevant_priority: OrbChangingFactorPriority,
    front: OrbChangingFactorFinderSide<'r>,
    back: OrbChangingFactorFinderSide<'r>,
}

impl<'r> OrbChangingFactorFinder<'r> {
    fn new(ors: &[Requirement]) -> Self {
        Self {
            min_relevant_priority: OrbChangingFactorPriority::default(),
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
enum OrbChangingFactorPriority {
    // TODO there seems to be a curious effect where kii is slightly faster if we don't factor out energy at all! What's that about?
    #[default]
    Energy,
    Health,
    WeaponChoice,
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
        min_relevant_priority: OrbChangingFactorPriority,
    ) -> Option<OrbChangingFactorPriority> {
        let requirement_priority = requirement.factor_priority().unwrap();

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
