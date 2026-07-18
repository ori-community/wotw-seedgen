use std::cmp::Ordering;

pub trait PartialThen: Sized {
    fn partial_then(self, other: Self) -> Option<Self>;
}

impl PartialThen for Ordering {
    fn partial_then(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Ordering::Equal, Ordering::Equal) => Some(Ordering::Equal),
            (Ordering::Less | Ordering::Equal, Ordering::Less | Ordering::Equal) => {
                Some(Ordering::Less)
            }
            (Ordering::Greater | Ordering::Equal, Ordering::Greater | Ordering::Equal) => {
                Some(Ordering::Greater)
            }
            (Ordering::Less, Ordering::Greater) | (Ordering::Greater, Ordering::Less) => None,
        }
    }
}

pub fn partial_then_iters<A, B, F>(a: A, b: B, mut f: F) -> Option<Ordering>
where
    A: IntoIterator,
    B: IntoIterator,
    F: FnMut(A::Item, B::Item) -> Option<Ordering>,
{
    let mut a_iter = a.into_iter();
    let mut b_iter = b.into_iter();

    let mut ordering = Ordering::Equal;

    // Cannot use zip because we need to check the remaining state after
    while let Some(a) = a_iter.next() {
        match b_iter.next() {
            None => return ordering.partial_then(Ordering::Greater),
            Some(b) => ordering = ordering.partial_then(f(a, b)?)?,
        }
    }

    match b_iter.next() {
        None => Some(ordering),
        Some(_) => ordering.partial_then(Ordering::Less),
    }
}
