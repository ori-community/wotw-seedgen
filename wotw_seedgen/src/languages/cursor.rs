use std::{ops::Range, str::Chars};

/// Peekable iterator over a char sequence.
///
/// Next characters can be peeked via `first` method,
/// and position can be shifted forward via `bump` method.
pub(super) struct Cursor<'a> {
    initial_len: usize,
    offset: usize,
    chars: Chars<'a>,
}

const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub(super) fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            initial_len: input.len(),
            offset: 0,
            chars: input.chars(),
        }
    }

    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub(super) fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }
    /// Peeks the second next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub(super) fn second(&self) -> char {
        self.chars.clone().nth(1).unwrap_or(EOF_CHAR)
    }

    /// Checks if there is nothing more to consume.
    pub(super) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns range of consumed symbols since the last time calling this function
    pub(super) fn reset_consumed_range(&mut self) -> Range<usize> {
        let range = self.consumed_range();
        self.offset = range.end;
        range
    }
    /// Returns range of consumed symbols since the last time calling `reset_consumed_range`
    pub(super) fn consumed_range(&mut self) -> Range<usize> {
        self.offset..self.initial_len - self.chars.as_str().len()
    }
    /// Returns count of consumed symbols since the last time calling `reset_consumed_range`
    pub(super) fn consumed_count(&mut self) -> usize {
        self.consumed_range().len()
    }

    /// Moves to the next character.
    pub(super) fn bump(&mut self) -> Option<char> {
        self.chars.next()
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    pub(super) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }
}
