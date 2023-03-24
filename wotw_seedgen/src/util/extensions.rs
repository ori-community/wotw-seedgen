use std::{iter::FusedIterator, ops::Range};

pub(crate) trait StrExtension {
    /// Returns an iterator over the lines of a string, as [`Range`]s to index into the string
    ///
    /// Unlike the `lines` method on [`str`], this will include trailing newlines
    fn line_ranges(&self) -> LineRanges;
}
impl StrExtension for str {
    fn line_ranges(&self) -> LineRanges {
        LineRanges::from(self)
    }
}
impl StrExtension for String {
    fn line_ranges(&self) -> LineRanges {
        LineRanges::from(&self[..])
    }
}

pub(crate) struct LineRanges<'a> {
    source: &'a str,
    next_index: Option<usize>,
}
impl<'a> From<&'a str> for LineRanges<'a> {
    fn from(source: &'a str) -> Self {
        let next_index = if source.is_empty() { None } else { Some(0) };
        LineRanges { source, next_index }
    }
}
impl<'a> Iterator for LineRanges<'a> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_index.map(|next_index| {
            let start_index = next_index;

            self.next_index = self
                .source
                .get(next_index..)
                .and_then(|remaining| remaining.find('\n'))
                .map(|index| next_index + index + 1);

            let end_index = self.next_index.unwrap_or(self.source.len());
            start_index..end_index
        })
    }
}
impl<'a> FusedIterator for LineRanges<'a> {}
