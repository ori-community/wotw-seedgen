use std::{
    cmp::Ordering,
    fmt::{self, Display},
    ops::{Add, AddAssign, Deref, DerefMut, Sub, SubAssign},
};

use itertools::Itertools;
use smallvec::SmallVec;
use wotw_seedgen_data::PartialThen;

/// A representation of a player's life and energy
///
/// Commonly used as [`OrbVariants`] to represent multiple possibilities of what the logical player can have
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Orbs {
    pub life: f32,
    pub energy: f32,
}

impl Orbs {
    pub const fn new(life: f32, energy: f32) -> Self {
        Self { life, energy }
    }
}

impl Add for Orbs {
    type Output = Orbs;

    fn add(self, other: Orbs) -> Orbs {
        Orbs {
            life: self.life + other.life,
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
            life: self.life - other.life,
            energy: self.energy - other.energy,
        }
    }
}

impl SubAssign for Orbs {
    fn sub_assign(&mut self, other: Orbs) {
        *self = *self - other;
    }
}

impl PartialOrd for Orbs {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.life
            .total_cmp(&other.life)
            .partial_then(self.energy.total_cmp(&other.energy))
    }
}

impl Display for Orbs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO why did I put life left and energy right in this codebase?
        write!(f, "({}/{})", self.life, self.energy)
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct OrbVariants {
    inner: OrbVariantsInner,
}

type OrbVariantsInner = SmallVec<[Orbs; 3]>;

impl OrbVariants {
    pub fn new(inner: OrbVariantsInner) -> Self {
        Self { inner }
    }

    /// For two `OrbVariants`, returns `OrbVariants` that contain the best options among both, filtered for redundancies.
    ///
    /// The existing `OrbVariants` are expected to be internally non-redundant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use wotw_seedgen::orbs::{orb_variants, Orbs, OrbVariants};
    /// #
    /// let a = orb_variants![Orbs { life: 0.0, energy: 2.0 }];
    /// let b = orb_variants![Orbs { life: 30.0, energy: 0.0 }];
    /// let combined_orbs = orb_variants![Orbs { life: 0.0, energy: 2.0 }, Orbs { life: 30.0, energy: 0.0 }];
    /// assert_eq!(OrbVariants::alternatives(a, b), combined_orbs);
    ///
    /// let a = orb_variants![Orbs { life: 10.0, energy: 3.0 }, Orbs { life: 20.0, energy: 0.0 }];
    /// let b = orb_variants![Orbs { life: 30.0, energy: 0.0 }];
    /// let combined_orbs = orb_variants![Orbs { life: 10.0, energy: 3.0 }, Orbs { life: 30.0, energy: 0.0 }];
    /// assert_eq!(OrbVariants::alternatives(a, b), combined_orbs);
    ///
    /// let a = orb_variants![Orbs { life: 30.0, energy: 1.0 }, Orbs { life: 10.0, energy: 3.0 }];
    /// let b = orb_variants![Orbs { life: 30.0, energy: 3.0 }];
    /// let combined_orbs = orb_variants![Orbs { life: 30.0, energy: 3.0 }];
    /// assert_eq!(OrbVariants::alternatives(a, b), combined_orbs);
    ///
    /// let a = orb_variants![Orbs { life: 0.0, energy: 2.0 }];
    /// let b = orb_variants![];
    /// let combined_orbs = orb_variants![Orbs { life: 0.0, energy: 2.0 }];
    /// assert_eq!(OrbVariants::alternatives(a, b), combined_orbs);
    ///
    /// let a = orb_variants![Orbs { life: 20.0, energy: 0.0 }, Orbs { life: 10.0, energy: 2.0 }];
    /// let b = orb_variants![Orbs { life: 15.0, energy: 1.0 }];
    /// let combined_orbs = orb_variants![Orbs { life: 20.0, energy: 0.0 }, Orbs { life: 10.0, energy: 2.0 }, Orbs { life: 15.0, energy: 1.0 }];
    /// assert_eq!(OrbVariants::alternatives(a, b), combined_orbs);
    /// ```
    pub fn alternatives(mut a: Self, b: Self) -> OrbVariants {
        a.insert_alternatives(b);
        a
    }

    /// Inserts alternative `OrbVariants` in place. See [`OrbVariants::alternatives`] for more details.
    pub fn insert_alternatives(&mut self, mut b: OrbVariants) {
        b.retain(|b| self.check_alternative(*b));

        self.extend(b);
    }

    pub fn insert_single_alternative(&mut self, b: Orbs) -> bool {
        let keep = self.check_alternative(b);

        if keep {
            self.push(b);
        }

        keep
    }

    fn check_alternative(&mut self, b: Orbs) -> bool {
        let mut keep = true;

        self.retain(|a| match (*a).partial_cmp(&b) {
            None => true,
            Some(Ordering::Less) => false,
            Some(Ordering::Equal | Ordering::Greater) => {
                keep = false;
                true
            }
        });

        keep
    }
}

impl Deref for OrbVariants {
    type Target = OrbVariantsInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for OrbVariants {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for OrbVariants {
    type Item = <OrbVariantsInner as IntoIterator>::Item;

    type IntoIter = <OrbVariantsInner as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a OrbVariants {
    type Item = <&'a [Orbs] as IntoIterator>::Item;

    type IntoIter = <&'a [Orbs] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut OrbVariants {
    type Item = <&'a mut [Orbs] as IntoIterator>::Item;

    type IntoIter = <&'a mut [Orbs] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl Display for OrbVariants {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.iter().format(" / ").fmt(f)
    }
}

#[macro_export]
macro_rules! orb_variants {
    ($($t:tt)*) => {
        $crate::orbs::OrbVariants::new(smallvec::smallvec![$($t)*])
    };
}
pub use orb_variants;
