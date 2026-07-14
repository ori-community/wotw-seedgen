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
