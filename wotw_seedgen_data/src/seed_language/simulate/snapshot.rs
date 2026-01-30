use std::{
    mem,
    ops::{Deref, DerefMut},
};

pub trait Snapshot {
    // TODO return a struct to check it doesn't get dropped unrestored?
    fn snapshot(&mut self);

    fn restore_snapshot(&mut self);
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CloneSnapshot<T> {
    pub value: T,
    snapshot: T,
}

impl<T: Clone + Default> CloneSnapshot<T> {
    pub fn new(value: T) -> Self {
        Self::from(value)
    }
}

impl<T: Clone + Default> From<T> for CloneSnapshot<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            snapshot: T::default(),
        }
    }
}

impl<T> Deref for CloneSnapshot<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for CloneSnapshot<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Clone + Default> Snapshot for CloneSnapshot<T> {
    fn snapshot(&mut self) {
        self.snapshot = self.value.clone();
    }

    fn restore_snapshot(&mut self) {
        self.value = mem::take(&mut self.snapshot);
    }
}
