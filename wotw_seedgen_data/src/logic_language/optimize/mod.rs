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
            Requirement::And(ands) => {
                ands.optimize();
                *self = Requirement::and(ands.drain(..));
            }
            Requirement::Or(ors) => {
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
                let mut may_reorder = true;

                for and in &mut *ands {
                    if !and.is_commutative() {
                        if !may_reorder {
                            break;
                        }

                        may_reorder = false;
                    }

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

    fn is_commutative(&self) -> bool {
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
            | Self::State(_) => true,
            Self::EnergySkill(_, _)
            | Self::NonConsumingEnergySkill(_)
            | Self::Damage(_)
            | Self::Danger(_)
            | Self::Combat(_)
            | Self::Boss(_)
            | Self::BreakWall(_)
            | Self::ShurikenBreak(_)
            | Self::SentryBreak(_) => false,
            Self::And(requirements) | Self::Or(requirements) => {
                requirements.iter().all(Requirement::is_commutative)
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
