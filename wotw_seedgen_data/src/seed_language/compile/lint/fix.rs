use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Fix {
    pub kind: FixKind,
    pub span: Range<usize>,
    pub content: String,
}

impl Fix {
    pub fn resulting_offset(&self) -> isize {
        self.content.len() as isize - self.span.len() as isize
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FixKind {
    PreferredName,
}
