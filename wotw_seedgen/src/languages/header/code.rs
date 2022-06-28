use std::fmt::{self, Display};

type DisplayFn<'a, T> = fn(&'a T, &mut fmt::Formatter) -> fmt::Result;

pub struct CodeDisplay<'a, T>(&'a T, DisplayFn<'a, T>);
impl<'a, T> CodeDisplay<'a, T> {
    pub(crate) fn new(s: &'a T, f: DisplayFn<'a, T>) -> Self { Self(s, f) }
}
impl<T> Display for CodeDisplay<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       self.1(self.0, f)
    }
}
