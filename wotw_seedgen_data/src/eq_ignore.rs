use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
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

impl<T> Deref for EqIgnore<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for EqIgnore<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for EqIgnore<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
