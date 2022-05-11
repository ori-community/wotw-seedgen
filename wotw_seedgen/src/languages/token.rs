use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, wotw_seedgen_derive::Display)]
pub(crate) enum TokenKind {
    /// `\n`
    Newline,
    /// General whitespace characters, not including newlines
    Whitespace,
    /// Additional whitespace at the start of a line
    Indent,
    /// Reduced whitespace at the start of a line
    /// 
    /// `matching` is false if the new amount of spaces doesn't line up with any prior indent
    Dedent { matching: bool },
    /// `// Helpful explanation`, `/// My Header`
    Comment { kind: CommentKind },
    /// `1`, `-999`, `345.67`
    Number,
    /// `header_core`, `opher`, `int`
    Identifier,
    /// `"Greetings Hello"`
    String { terminated: bool },
    /// `|`
    Separator,
    /// `=`
    Eq,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `:`
    Colon,
    /// `!`
    Bang,
    /// `$`
    Dollar,
    /// `(`
    OpenParen,
    /// `)`
    CloseParen,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `#`
    Pound,
    /// `x` after a number, for instance in `2x`
    X,
    /// End of File
    /// 
    /// This is never directly returned by the tokenizer, but can be useful in later processing
    Eof,
    /// Tokens not used in the language, e.g. `@`
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CommentKind {
    /// `// Common Comment`
    Note,
    /// `/// My Header`
    HeaderDoc,
    /// `//// Amount to add to the pool`
    ConfigDoc,
}

#[derive(Clone)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub range: Range<usize>,
}
