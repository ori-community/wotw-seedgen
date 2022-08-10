use std::fmt::{self, Display};
use std::slice::SliceIndex;
use std::iter::Peekable;
use std::error::Error;
use std::ops::{Range, Deref, DerefMut};

use super::{Token, TokenKind};

use crate::util::extensions::StrExtension;

pub(crate) struct Parser<'a, TokenStream: Iterator<Item = Token>> {
    input: &'a str,
    tokens: Peekable<TokenStream>,
    current_token: Token,
    eof_token: Token,
}

impl<'a, TokenStream: Iterator<Item = Token>> Parser<'a, TokenStream> {
    /// Returns a new [`Parser`] for the input string
    pub(crate) fn new(input: &'a str, tokens: TokenStream) -> Self {
        let mut tokens = tokens.peekable();
        let len = input.len();
        let eof_token = Token { kind: TokenKind::Eof, range: len..len };
        let current_token = tokens.next().unwrap_or_else(|| eof_token.clone());
        Self { input, tokens, current_token, eof_token }
    }

    /// Expects the current [`Token`] to match the [`TokenKind`], returns it and steps to the next [`Token`]
    pub(crate) fn eat(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.next_token();
        if token.kind == kind {
            Ok(token)
        } else {
            Err(self.error(format!("Expected {kind}"), token.range))
        }
    }
    /// Convenience function to call [`Parser::eat`] and add a [`Suggestion`] to the potential [`ParseError`]
    pub(crate) fn eat_or_suggest(&mut self, kind: TokenKind, suggestion: impl Display) -> Result<Token, ParseError> {
        let token = self.next_token();
        if token.kind == kind {
            Ok(token)
        } else {
            Err(self.error(format!("Expected {kind}"), token.range).with_suggestion(suggestion))
        }
    }
    /// If the current [`Token`] matches the [`TokenKind`], steps to the next [`Token`]
    pub(crate) fn skip(&mut self, kind: TokenKind) {
        if self.current_token.kind == kind {
            self.next_token();
        }
    }
    /// Skips [`Token`]s as long as they fulfill a condition
    pub(crate) fn skip_while(&mut self, mut condition: impl FnMut(TokenKind) -> bool) {
        while condition(self.current_token.kind) {
            let token = self.next_token();
            if token.kind == TokenKind::Eof { break }
        }
    }

    /// Returns the current [`Token`], or an end-of-file [`Token`] if no [`Token`]s are left and steps to the next [`Token`]
    pub(crate) fn next_token(&mut self) -> Token {
        let next = self.tokens.next().unwrap_or_else(|| self.eof_token.clone());
        std::mem::replace(&mut self.current_token, next)
    }
    /// Returns a reference to the current [`Token`], or an end-of-file [`Token`] if no [`Token`]s are left
    pub(crate) fn current_token(&self) -> &Token {
        &self.current_token
    }
    /// Returns the next [`Token`] without committing to step forwards
    pub(crate) fn peek_token(&mut self) -> &Token {
        self.tokens.peek().unwrap_or(&self.eof_token)
    }

    /// Returns the string corresponding to a [`Token`]
    pub(crate) fn read_token(&self, token: &Token) -> &'a str {
        &self.input[token.range.clone()]
    }
    /// Returns the string corresponding to the index
    pub(crate) fn read<I: SliceIndex<str>>(&self, index: I) -> &I::Output {
        &self.input[index]
    }

    /// Returns the remaining portion of the input string
    pub(crate) fn remaining(&self) -> &str {
        let start = self.current_token.range.start;
        &self.input[start..]
    }

    /// Checks that the remaining portion of the input string is empty and returns a [`ParseError`] otherwise
    pub(crate) fn expect_end(&self) -> Result<(), ParseError> {
        let remaining = self.remaining();
        if remaining.is_empty() {
            Ok(())
        } else {
            Err(self.error(format!("Input left after parsing: \"{remaining}\""), self.current_token.range.clone()))
        }
    }

    /// Returns a [`ParseError`] with the given message and error range
    pub(crate) fn error(&self, message: impl AsRef<str>, range: Range<usize>) -> ParseError {
        ParseError::new(message, self.input, range)
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    source: String,
    pub range: Range<usize>,
    pub suggestion: Option<String>,
}
impl ParseError {
    pub(crate) fn new(message: impl AsRef<str>, source: impl AsRef<str>, range: Range<usize>) -> Self {
        let message = message.as_ref().to_string();
        let source = source.as_ref().to_string();
        Self { message, source, range, suggestion: None }
    }

    /// Adds the completion to this [`ParseError`]
    pub(crate) fn with_suggestion(mut self, suggestion: impl Display) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    /// Returns a multiline visual representation of this [`ParseError`]
    /// 
    /// # Panics
    /// 
    /// May panic if the [`ParseError`] is out of bounds of its source string
    pub fn verbose_display(&self) -> String {
        if self.source.is_empty() { return format!("{}\n(input was empty)", self.message) }

        let source_view = self.source.line_ranges()
            .enumerate()
            .filter(|(_, line_range)| line_range.end >= self.range.start && line_range.start < self.range.end)
            .map(|(line_number, line_range)| {
                let line_number = format!("line {}", line_number + 1);
                let indent = " ".repeat(line_number.len());
                let err_range = line_range.start.max(self.range.start)..line_range.end.min(self.range.end);
                let err_offset = " ".repeat(err_range.start - line_range.start);
                let err_underline = "^".repeat(err_range.len());
                let line = &self.source[line_range];
                let newline = if line.ends_with('\n') { "" } else { "\n" };
                format!("\n\
                    {line_number}: {line}{newline}\
                    {indent}  {err_offset}{err_underline}\
                ")
            }).collect::<String>();

        assert!(!source_view.is_empty(), "Error range out of bounds");

        let message = &self.message;
        format!("{message}{source_view}")
    }
}
impl Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.message)
    }
}
impl Error for ParseError {}

#[derive(Debug, Default, Clone)]
pub struct ParseErrorCollection {
    errors: Vec<ParseError>,
}
impl From<Vec<ParseError>> for ParseErrorCollection {
    fn from(errors: Vec<ParseError>) -> ParseErrorCollection {
        ParseErrorCollection { errors }
    }
}
impl Deref for ParseErrorCollection {
    type Target = Vec<ParseError>;
    fn deref(&self) -> &Self::Target {
        &self.errors
    }
}
impl DerefMut for ParseErrorCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.errors
    }
}
impl Display for ParseErrorCollection {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let error_amount = self.errors.len();
        // This type can only be returned by calls to the library and guarantees to contain at least one error
        let first_error = &self.errors[0];
        write!(fmt, "{error_amount} errors\nFirst error: {first_error}")
    }
}
impl ParseErrorCollection {
    pub fn verbose_display(&self) -> String {
        self.errors.iter().map(ParseError::verbose_display).collect::<Vec<_>>().join("\n")
    }
}

macro_rules! invalid {
    ($token:ident, $parser:expr, $expected:path) => {
        |err| $parser.error(format!("Invalid {}: {}", $expected, err), $token.range).with_suggestion($expected)
    };
}
pub(super) use invalid;

macro_rules! parse_token {
    ($token_kind:path, $parser:expr, $expected:path) => {
        {
            $parser.eat_or_suggest($token_kind, $expected)
            .and_then(|token| {
                let string = $parser.read_token(&token);
                string.parse().map_err($crate::languages::parser::invalid!(token, $parser, $expected))
            })
        }
    };
}
pub(super) use parse_token;
macro_rules! parse_number {
    ($parser:expr, $expected:path) => {
        $crate::languages::parser::parse_token!($crate::languages::TokenKind::Number, $parser, $expected)
    };
}
pub(super) use parse_number;
macro_rules! parse_value {
    ($parser:expr, $expected:path, $prior_expected:path) => {
        {
            $parser.eat_or_suggest($crate::languages::TokenKind::Eq, $prior_expected)
            .and_then(|_| $crate::languages::parser::parse_number!($parser, $expected))
        }
    };
}
pub(super) use parse_value;
macro_rules! parse_ident {
    ($parser:expr, $expected:path) => {
        $crate::languages::parser::parse_token!($crate::languages::TokenKind::Identifier, $parser, $expected)
    };
}
pub(super) use parse_ident;

macro_rules! read_token {
    ($token_kind:path, $parser:expr, $expected:path) => {
        {
            $parser.eat_or_suggest($token_kind, $expected)
            .map(|token| $parser.read_token(&token))
        }
    };
}
pub(super) use read_token;
macro_rules! read_ident {
    ($parser:expr, $expected:path) => {
        $crate::languages::parser::read_token!($crate::languages::TokenKind::Identifier, $parser, $expected)
    };
}
pub(super) use read_ident;
