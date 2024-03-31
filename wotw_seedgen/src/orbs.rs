use std::ops::{Add, AddAssign, Sub, SubAssign};

use smallvec::{smallvec, SmallVec, ToSmallVec};

pub type OrbVariants = SmallVec<[Orbs; 3]>;

/// A representation of a player's health and energy
///
/// Commonly used as [`OrbVariants`] to represent multiple possibilities of what the logical player can have
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Orbs {
    pub health: f32,
    pub energy: f32,
}
impl Add for Orbs {
    type Output = Orbs;
    fn add(self, other: Orbs) -> Orbs {
        Orbs {
            health: self.health + other.health,
            energy: self.energy + other.energy,
        }
    }
}
impl AddAssign for Orbs {
    fn add_assign(&mut self, other: Orbs) {
        *self = *self + other;
    }
}
impl Sub for Orbs {
    type Output = Orbs;
    fn sub(self, other: Orbs) -> Orbs {
        Orbs {
            health: self.health - other.health,
            energy: self.energy - other.energy,
        }
    }
}
impl SubAssign for Orbs {
    fn sub_assign(&mut self, other: Orbs) {
        *self = *self - other;
    }
}

/// For two lists of [`Orbs`] representing alternative possible options, returns a list of [`Orbs`] that contains the options of both, but filtered for any redundancies
///
/// # Examples
///
/// ```
/// # use wotw_seedgen::orbs::{either, Orbs, OrbVariants};
/// # use smallvec::smallvec;
/// #
/// let a = vec![Orbs { health: 0.0, energy: 2.0 }];
/// let b = vec![Orbs { health: 30.0, energy: 0.0 }];
/// let either_orbs: OrbVariants = smallvec![Orbs { health: 0.0, energy: 2.0 }, Orbs { health: 30.0, energy: 0.0 }];
/// assert_eq!(either(&a, &b), either_orbs);
///
/// let a = vec![Orbs { health: 10.0, energy: 3.0 }, Orbs { health: 20.0, energy: 0.0 }];
/// let b = vec![Orbs { health: 30.0, energy: 0.0 }];
/// let either_orbs: OrbVariants = smallvec![Orbs { health: 10.0, energy: 3.0 }, Orbs { health: 30.0, energy: 0.0 }];
/// assert_eq!(either(&a, &b), either_orbs);
///
/// let a = vec![Orbs { health: 30.0, energy: 1.0 }, Orbs { health: 10.0, energy: 3.0 }];
/// let b = vec![Orbs { health: 30.0, energy: 3.0 }];
/// let either_orbs: OrbVariants = smallvec![Orbs { health: 30.0, energy: 3.0 }];
/// assert_eq!(either(&a, &b), either_orbs);
///
/// let a = vec![Orbs { health: 0.0, energy: 2.0 }];
/// let b = vec![];
/// let either_orbs: OrbVariants = smallvec![Orbs::default()];
/// assert_eq!(either(&a, &b), either_orbs);
///
/// let a = vec![Orbs { health: 20.0, energy: 0.0 }, Orbs { health: 10.0, energy: 2.0 }];
/// let b = vec![Orbs { health: 15.0, energy: 1.0 }];
/// let either_orbs: OrbVariants = smallvec![Orbs { health: 20.0, energy: 0.0 }, Orbs { health: 10.0, energy: 2.0 }, Orbs { health: 15.0, energy: 1.0 }];
/// assert_eq!(either(&a, &b), either_orbs);
/// ```
// TODO optimization idea take ownership
#[must_use]
pub fn either(a: &[Orbs], b: &[Orbs]) -> OrbVariants {
    if b.is_empty() || a.is_empty() {
        // TODO whyever are there these is_empty checks?
        smallvec![Orbs::default()]
    } else {
        let mut sum: OrbVariants = a.to_smallvec();
        for b in b {
            if !sum
                .iter()
                .any(|a| a.energy >= b.energy && a.health >= b.health)
            {
                sum.retain(|a| a.energy > b.energy || a.health > b.health);
                sum.push(*b);
            }
        }
        sum
    }
}
/// For a lists of [`Orbs`] representing alternative possible options and one additional option, returns a list of [`Orbs`] that contains the options of both, filtered for any redundancies
///
/// This is an optimization over [`orbs::either`](either) for only one additional option, see [`orbs::either`](either) for further documentation
#[must_use]
pub fn either_single(a: &[Orbs], b: Orbs) -> OrbVariants {
    if a.is_empty() {
        smallvec![Orbs::default()]
    } else {
        let mut sum: OrbVariants = a.to_smallvec();
        if !sum
            .iter()
            .any(|a| a.energy >= b.energy && a.health >= b.health)
        {
            sum.retain(|a| a.energy > b.energy || a.health > b.health);
            sum.push(b);
        }
        sum
    }
}
/// For two lists of [`Orbs`] representing alternative possible options, returns all possible sums, filtered for any redundancies
///
/// # Examples
///
/// ```
/// # use wotw_seedgen::orbs::{both, Orbs, OrbVariants};
/// # use smallvec::smallvec;
/// #
/// let a = vec![Orbs { health: 0.0, energy: 2.0 }];
/// let b = vec![Orbs { health: 30.0, energy: 0.0 }];
/// let both_orbs: OrbVariants = smallvec![Orbs { health: 30.0, energy: 2.0 }];
/// assert_eq!(both(&a, &b), both_orbs);
///
/// let a = vec![Orbs { health: 10.0, energy: 3.0 }, Orbs { health: 20.0, energy: 0.0 }];
/// let b = vec![Orbs { health: 30.0, energy: 0.0 }];
/// let both_orbs: OrbVariants = smallvec![Orbs { health: 40.0, energy: 3.0 }, Orbs { health: 50.0, energy: 0.0 }];
/// assert_eq!(both(&a, &b), both_orbs);
///
/// let a = vec![Orbs { health: 100.0, energy: 30.0 }, Orbs { health: 200.0, energy: 10.0 }];
/// let b = vec![Orbs { health: 0.0, energy: -10.0 }, Orbs { health: -50.0, energy: -3.0 }];
/// let both_orbs: OrbVariants = smallvec![
///     Orbs { health: 100.0, energy: 20.0 },
///     Orbs { health: 50.0, energy: 27.0 },
///     Orbs { health: 200.0, energy: 0.0 },
///     Orbs { health: 150.0, energy: 7.0 },
/// ];
/// assert_eq!(both(&a, &b), both_orbs);
///
/// let a = vec![Orbs { health: 0.0, energy: 2.0 }];
/// let b = vec![];
/// let both_orbs: OrbVariants = smallvec![Orbs { health: 0.0, energy: 2.0 }];
/// assert_eq!(both(&a, &b), both_orbs);
/// ```
#[must_use]
pub fn both(a: &[Orbs], b: &[Orbs]) -> OrbVariants {
    if b.is_empty() {
        a.to_smallvec()
    } else if a.is_empty() {
        b.to_smallvec()
    } else {
        let mut product = SmallVec::<[Orbs; 3]>::with_capacity(a.len());
        for a_ in a {
            for b_ in b {
                let orbs = *a_ + *b_;
                if !product.contains(&orbs) {
                    product.push(orbs);
                }
            }
        }
        product
            .iter()
            .filter(|orbs| {
                !product.iter().any(|other| {
                    other.energy > orbs.energy && other.health >= orbs.health
                        || other.energy >= orbs.energy && other.health > orbs.health
                })
            })
            .copied()
            .collect()
    }
}
/// For a lists of [`Orbs`] representing alternative possible options and one additional option, returns all possible sums, filtered for any redundancies
///
/// This is an optimization over [`orbs::both`](both) with only one additional option, see [`orbs::both`](both) for further documentation
#[must_use]
pub fn both_single(a: &[Orbs], b: Orbs) -> OrbVariants {
    if a.is_empty() {
        smallvec![b]
    } else {
        let mut product = SmallVec::<[Orbs; 3]>::with_capacity(a.len());
        for a_ in a {
            let orbs = *a_ + b;
            if !product.contains(&orbs) {
                product.push(orbs);
            }
        }
        product
            .iter()
            .filter(|orbs| {
                !product.iter().any(|other| {
                    other.energy > orbs.energy && other.health >= orbs.health
                        || other.energy >= orbs.energy && other.health > orbs.health
                })
            })
            .copied()
            .collect()
    }
}
