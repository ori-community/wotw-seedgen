use std::{
    hash::{Hash, Hasher},
    mem,
    ops::Deref,
};

#[derive(Debug)]
pub struct GraphRef<'graph, T>(pub &'graph T);

impl<T> GraphRef<'_, T> {
    pub fn index<S>(&self, source: &[S]) -> usize {
        const {
            assert!(mem::size_of::<T>() == mem::size_of::<S>());
        }

        ((self.address() - source.as_ptr() as isize) / mem::size_of::<T>() as isize) as usize
    }

    fn address(&self) -> isize {
        self.0 as *const T as isize
    }
}

impl<T> Clone for GraphRef<'_, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<T> Copy for GraphRef<'_, T> {}

impl<T> PartialEq for GraphRef<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl<T> Eq for GraphRef<'_, T> {}

impl<T> Hash for GraphRef<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state);
    }
}

impl<'graph, T> Deref for GraphRef<'graph, T> {
    type Target = T;

    fn deref(&self) -> &'graph Self::Target {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EqIgnore<T>(pub T);

impl<T> PartialEq for EqIgnore<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for EqIgnore<T> {}

impl<T> Hash for EqIgnore<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}
