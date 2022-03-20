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

impl<T: FromStr> V<T> {
    /// Accept `$PARAM()` syntax, or otherwise try to parse the value
    pub fn try_wrap(s: &str) -> Result<V<T>, T::Err> {
        Self::parse_param(s).map_or_else(|| T::from_str(s).map(V::Literal), Ok)
    }
    /// Accept `$PARAM()` syntax, or return [`None`]
    pub fn parse_param(s: &str) -> Option<V<T>> {
        s.strip_prefix("$PARAM(").and_then(|parameter|
            parameter.find(')').map(|index|
                V::Parameter(parameter[..index].to_string())))
    }
}

impl<'a, T: FromStr + From<&'a str>> V<T> {
    /// Accept `$PARAM()` syntax, or otherwise parse the value
    pub fn wrap(s: &'a str) -> V<T> {
        Self::parse_param(s).map_or_else(
            || V::Literal(T::from(s)),
            |nonliteral| nonliteral
        )
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
