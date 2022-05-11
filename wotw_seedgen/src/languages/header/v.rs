use std::fmt;
use std::str::FromStr;

use rustc_hash::FxHashMap;

/// Resolve a value to its literal counterpart
pub trait VResolve<T> {
    /// Apply parameters if needed and try to parse the resulting value
    fn resolve(self, parameters: &FxHashMap<String, String>) -> Result<T, String>;
}

/// Value in a header file that can be either a literal or a referenced value to be applied at generation time
#[derive(Debug, Clone)]
pub enum V<T: FromStr> {
    Literal(T),
    Parameter(String),
}
impl<T: FromStr + fmt::Display> fmt::Display for V<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(t) => t.fmt(f),
            Self::Parameter(identifier) => write!(f, "(configuration value {identifier})"),
        }
    }
}
impl<T: FromStr> VResolve<T> for V<T> {
    fn resolve(self, parameters: &FxHashMap<String, String>) -> Result<T, String> {
        match self {
            V::Literal(t) => Ok(t),
            V::Parameter(identifier) => parameters.get(&identifier)
                .ok_or_else(|| format!("Unknown parameter {identifier}"))
                .and_then(|value| T::from_str(value).map_err(|_| format!("Invalid value {value} for parameter {identifier}")))
        }
    }
}

macro_rules! vdisplay {
    (
        $vty:ty,
        impl $($($(::)?std::)?fmt::)?Display for $ty:ty { $impl:item }
    ) => {
        impl ::std::fmt::Display for $ty { $impl }
        impl ::std::fmt::Display for $vty { $impl }
    };
}
pub(crate) use vdisplay;

/// [`String`] with possibly contained [`V`]s
#[derive(Debug, Clone)]
pub struct VString(pub String);
impl VResolve<String> for VString {
    fn resolve(mut self, parameters: &FxHashMap<String, String>) -> Result<String, String> {
        while let Some(range) = self.0.find("$PARAM(")
            .and_then(|start| self.0[start + 7..].find(')').map(|end| start..start + end + 8))
        {
            let inner_range = range.start + 7..range.end - 1;
            let identifier = &self.0[inner_range.clone()];
            let value = parameters.get(identifier).ok_or_else(|| format!("Unknown parameter {identifier}"))?;
            self.0.replace_range(range, value);
        }

        Ok(self.0)
    }
}
impl fmt::Display for VString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// Blanket implementations for wrapper types
impl<O, I: VResolve<O>> VResolve<Option<O>> for Option<I> {
    fn resolve(self, parameters: &FxHashMap<String, String>) -> Result<Option<O>, String> {
        self.map_or(Ok(None), |i| i.resolve(parameters).map(Some))
    }
}
impl<O, I: VResolve<O>> VResolve<Box<O>> for Box<I> {
    fn resolve(self, parameters: &FxHashMap<String, String>) -> Result<Box<O>, String> {
        (*self).resolve(parameters).map(Box::new)
    }
}
