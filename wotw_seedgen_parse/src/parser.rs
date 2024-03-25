use crate::{Error, ErrorKind, Tokenize};
use std::{
    fmt::{self, Debug},
    ops::Range,
    slice::SliceIndex,
    str::FromStr,
};

/// Enables the [`bool`] [`Ast`] implementation for your `Token`
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseBoolToken};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[token("true", |_| true)]
///     #[token("false", |_| false)]
///     Boolean(bool),
///     #[regex(r".")]
///     Symbol,
///     Error,
///     Eof,
/// }
///
/// impl ParseBoolToken for Token {
///     fn bool(&self) -> Option<bool> {
///         match self {
///             Token::Boolean(value) => Some(*value),
///             _ => None,
///         }
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(parse_ast::<_, bool>("true", tokenizer).into_result(), Ok(true));
/// assert_eq!(parse_ast::<_, bool>("false", tokenizer).into_result(), Ok(false));
/// assert!(parse_ast::<_, bool>("tralse", tokenizer).into_result().is_err());
/// ```
///
/// [`Ast`]: crate::Ast
pub trait ParseBoolToken {
    /// Parse this token as a [`bool`]
    ///
    /// Should return `Some(bool)` if successful and `None` if this is not a boolean token
    fn bool(&self) -> Option<bool>;
}
/// Enables integer [`Ast`] implementations for your `Token`
///
/// The only required method is [`ParseIntToken::is_int`]. By default, if it returns `true` the token content
/// will be parsed using the [`FromStr`] implementation of the desired integer type.
///
/// If your integers cannot be parsed using Rust's [`FromStr`] implementations, you may override the
/// provided methods [`ParseIntToken::parse_u8`] through [`ParseIntToken::parse_i128`].
///
/// Alternatively, you could define your own integer Newtypes and implement [`Ast`] on those.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseIntToken};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"-?\d+")]
///     Number,
///     #[regex(r".", priority = 0)]
///     Symbol,
///     Error,
///     Eof,
/// }
///
/// impl ParseIntToken for Token {
///     fn is_int(&self) -> bool {
///         matches!(self, Token::Number)
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(parse_ast::<_, i32>("-1000000", tokenizer).into_result(), Ok(-1000000));
/// assert_eq!(parse_ast::<_, u128>(&u128::MAX.to_string(), tokenizer).into_result(), Ok(u128::MAX));
/// assert!(parse_ast::<_, u8>("963", tokenizer).into_result().is_err());
/// ```
///
/// [`Ast`]: crate::Ast
pub trait ParseIntToken {
    /// Check whether this token may represent an integer
    fn is_int(&self) -> bool;

    /// Attempt to parse `slice` into [`u8`]
    #[inline]
    fn parse_u8(slice: &str) -> std::result::Result<u8, String> {
        u8::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`u16`]
    #[inline]
    fn parse_u16(slice: &str) -> std::result::Result<u16, String> {
        u16::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`u32`]
    #[inline]
    fn parse_u32(slice: &str) -> std::result::Result<u32, String> {
        u32::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`u64`]
    #[inline]
    fn parse_u64(slice: &str) -> std::result::Result<u64, String> {
        u64::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`u128`]
    #[inline]
    fn parse_u128(slice: &str) -> std::result::Result<u128, String> {
        u128::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`i8`]
    #[inline]
    fn parse_i8(slice: &str) -> std::result::Result<i8, String> {
        i8::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`i16`]
    #[inline]
    fn parse_i16(slice: &str) -> std::result::Result<i16, String> {
        i16::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`i32`]
    #[inline]
    fn parse_i32(slice: &str) -> std::result::Result<i32, String> {
        i32::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`i64`]
    #[inline]
    fn parse_i64(slice: &str) -> std::result::Result<i64, String> {
        i64::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`i128`]
    #[inline]
    fn parse_i128(slice: &str) -> std::result::Result<i128, String> {
        i128::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`usize`]
    #[inline]
    fn parse_usize(slice: &str) -> std::result::Result<usize, String> {
        usize::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`isize`]
    #[inline]
    fn parse_isize(slice: &str) -> std::result::Result<isize, String> {
        isize::from_str(slice).map_err(|err| err.to_string())
    }
}
/// Enables float [`Ast`] implementations for your `Token`
///
/// The only required method is [`ParseFloatToken::is_float`]. By default, if it returns `true` the token content
/// will be parsed using the [`FromStr`] implementation of the desired float type.
///
/// If your floats cannot be parsed using Rust's [`FromStr`] implementations, you may override the
/// provided methods [`ParseFloatToken::parse_f32`] and [`ParseFloatToken::parse_f64`].
///
/// Alternatively, you could define your own float Newtypes and implement [`Ast`] on those.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseFloatToken};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"-?\d+", priority = 2)]
///     Integer,
///     #[regex(r"-?\d+(\.\d*)?(e\d+)?")]
///     Float,
///     #[regex(r".", priority = 0)]
///     Symbol,
///     Error,
///     Eof,
/// }
///
/// impl ParseFloatToken for Token {
///     fn is_float(&self) -> bool {
///         matches!(self, Token::Integer | Token::Float)
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(parse_ast::<_, f32>("-3.14", tokenizer).into_result(), Ok(-3.14));
/// assert_eq!(parse_ast::<_, f64>("2e3", tokenizer).into_result(), Ok(2000.));
/// assert!(parse_ast::<_, f32>("2,5", tokenizer).trailing.is_err());
/// ```
///
/// [`Ast`]: crate::Ast
pub trait ParseFloatToken {
    /// Check whether this token may represent a float
    fn is_float(&self) -> bool;

    /// Attempt to parse `slice` into [`f32`]
    #[inline]
    fn parse_f32(slice: &str) -> std::result::Result<f32, String> {
        f32::from_str(slice).map_err(|err| err.to_string())
    }
    /// Attempt to parse `slice` into [`f64`]
    #[inline]
    fn parse_f64(slice: &str) -> std::result::Result<f64, String> {
        f64::from_str(slice).map_err(|err| err.to_string())
    }
}
/// Enables string [`Ast`] implementations for your `Token`
///
/// The only required method is [`ParseStringToken::is_string`]. By default, if it returns `true` the token content
/// will be parsed by stripping the first and last byte from the slice (since usually these will be quotes).
/// If your strings handle delimiters differently, you may override the provided method [`ParseStringToken::parse_str`].
///
/// Alternatively, you could define your own string Newtypes and implement [`Ast`] on those.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseStringToken};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r#""[^"]*""#)]
///     String,
///     #[regex(r".", priority = 0)]
///     Symbol,
///     Error,
///     Eof,
/// }
///
/// impl ParseStringToken for Token {
///     fn is_string(&self) -> bool {
///         matches!(self, Token::String)
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(parse_ast::<_, &str>("\"hi\"", tokenizer).into_result(), Ok("hi"));
/// assert_eq!(parse_ast::<_, String>("\"oh you.\"", tokenizer).into_result(), Ok("oh you.".to_string()));
/// assert!(parse_ast::<_, &str>("no", tokenizer).into_result().is_err());
/// ```
///
/// [`Ast`]: crate::Ast
pub trait ParseStringToken {
    /// Check whether this token may represent a string
    fn is_string(&self) -> bool;

    /// Attempt to parse `slice` into [`&str`](str)
    #[inline]
    fn parse_str(slice: &str) -> std::result::Result<&str, String> {
        Ok(&slice[1..slice.len() - 1])
    }
}
/// Enables parsing identifiers for your `Token`
///
/// This will be used by the [`Ast`] implementation of [`Identifier`], but also any derived [`Ast`] implementations of unit `struct`s or unit `enum` variants;
///
/// The only required method is [`ParseIdentToken::is_ident`]. By default, if it returns `true` the token content will be considered your identifier.
/// If your identifiers have to be parsed further, you may override the provided method [`ParseIdentToken::parse_ident`].
///
/// Alternatively, you could define your own identifier type and implement [`Ast`] on it.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Identifier, LogosTokenizer, parse_ast, ParseIdentToken};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"\w+")]
///     Identifier,
///     #[regex(r".", priority = 0)]
///     Symbol,
///     Error,
///     Eof,
/// }
///
/// impl ParseIdentToken for Token {
///     fn is_ident(&self) -> bool {
///         matches!(self, Token::Identifier)
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(parse_ast::<_, Identifier>("thisisanidentifier", tokenizer).into_result(), Ok(Identifier("thisisanidentifier")));
/// assert!(parse_ast::<_, Identifier>("\"thisisnoidentifier\"", tokenizer).into_result().is_err());
/// ```
///
/// [`Ast`]: crate::Ast
/// [`Identifier`]: crate::Identifier
pub trait ParseIdentToken {
    /// Check whether this token may represent an identifier
    fn is_ident(&self) -> bool;

    /// Attempt to parse `slice` as an identifier into [`&str`](str)
    #[inline]
    fn parse_ident(slice: &str) -> std::result::Result<&str, String> {
        Ok(slice)
    }
}

/// The underlying parser you will have access to in [`Ast`] implementations
///
// TODO update documentation
/// `Token` is a type you should provide. You can implement tokenization however you want by using [`Parser::with_tokens`], although there is a convenience function [`Parser::new`]
/// if you decide to use the [`ParseToken`] trait.
///
/// Note that parsers cannot be cloned, you should use [`Parser::jump`] for any backtracking or lookahead needs
///
/// [`Ast`]: crate::Ast
pub struct Parser<'source, T: Tokenize> {
    source: &'source str,
    tokens: Vec<(T::Token, Range<usize>)>,
    position: usize,
    eof: (T::Token, Range<usize>),
}
impl<'source, T: Tokenize> Parser<'source, T> {
    /// Tokenizes `source` and constructs a new [`Parser`] to traverse the resulting tokens
    pub fn new(source: &'source str, tokenizer: T) -> Self {
        let output = tokenizer.tokenize(source);
        let eof_span = source.len()..source.len();
        Self {
            source,
            tokens: output.tokens,
            position: 0,
            eof: (output.eof_token, eof_span),
        }
    }

    /// Source that is being parsed
    #[inline]
    pub fn source(&self) -> &'source str {
        self.source
    }

    /// The current position, or the current token index
    ///
    /// You can use this later to [`Parser::jump`] back to this token or check nearby tokens with [`Parser::token_at`]
    #[inline]
    pub fn position(&self) -> usize {
        self.position
    }
    /// Moves the parser to the next token
    pub fn step(&mut self) {
        self.position += 1;
    }
    /// Jumps to any token
    ///
    /// You can use this to backtrack if you called [`Parser::position`] earlier
    #[inline]
    pub fn jump(&mut self, position: usize) {
        self.position = position
    }
    /// The `position` representing the end of the parser
    ///
    /// This is also the amount of tokens received from tokenizing the entire source.
    /// Note that [`Parser::position`] can overstep this if you keep calling [`Parser::step`] past the end
    #[inline]
    pub fn end(&self) -> usize {
        self.tokens.len()
    }
    /// Checks whether there are `Token`s left to parse
    ///
    // TODO update documentation
    /// This may differ from `parser.current().is_err()` if the [`ParseToken`] implementation can return [`Error`]s at the tokenization stage.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.position >= self.tokens.len()
    }

    // TODO check if this doc link behaves
    /// A reference to any token. You can obtain the `position` at any point by calling [`Parser::position`]
    ///
    /// If `position` is beyond the last Token, the Tokenizer's [`EOF_TOKEN`](Tokenizer::EOF_TOKEN) will be returned
    #[inline]
    pub fn token_at(&self, position: usize) -> &(T::Token, Range<usize>) {
        self.tokens.get(position).unwrap_or(&self.eof)
    }
    /// A reference to the current `Token` that should be parsed
    ///
    /// The Token is [`Err`] if either:
    /// - the parser is finished, then an [`Error`] with [`ErrorKind::UnexpectedEnd`] will be returned
    // TODO update documentation
    /// - the [`ParseToken`] implementation returned an [`Error`] at the tokenization stage
    #[inline]
    pub fn current(&self) -> &(T::Token, Range<usize>) {
        self.token_at(self.position)
    }

    /// Slice of the source
    #[inline]
    pub fn slice<I: SliceIndex<str>>(&self, span: I) -> &'source I::Output {
        &self.source[span]
    }
    /// Slice of the current `Token`
    #[inline]
    pub fn current_slice(&self) -> &'source str {
        &self.source[self.current().1.clone()]
    }
    /// Remaining input to be parsed after the current `Token`
    #[inline]
    pub fn remainder(&self) -> &'source str {
        &self.source[self.current().1.end..]
    }

    /// Searches for the next token that matches a predicate
    ///
    /// This is commonly useful in recovery strategies. You can then [`Parser::jump`] to the token using the returned position.
    /// Tokenization errors will always be skipped without invoking the predicate.
    pub fn find<F>(&self, mut f: F) -> usize
    where
        F: FnMut(&T::Token, &Range<usize>) -> bool,
    {
        // TODO I just learned about iter::position and now I want to check on all my uses of enumerate
        match self.tokens[self.position..]
            .iter()
            .position(|(token, span)| f(token, span))
        {
            None => self.tokens.len(),
            Some(index) => self.position + index,
        }
    }

    // TODO do these still make sense?
    /// Construct a new [`Error`] at the current span
    #[inline]
    pub fn error(&self, kind: ErrorKind) -> Error {
        Error::new(kind, self.current().1.clone())
    }
    /// Construct a new [`Error`] with a custom message at the current span
    #[inline]
    pub fn custom_error(&self, message: String) -> Error {
        Error::custom(message, self.current().1.clone())
    }
}

impl<'source, T> Debug for Parser<'source, T>
where
    T: Tokenize,
    T::Token: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let span = self.current().1.clone();
        let more_before = if span.start > 0 { "..." } else { "" };
        let more_after = if span.end < self.source.len() {
            "..."
        } else {
            ""
        };
        let source = format!("{}{}{}", more_before, &self.source[span], more_after,);

        f.debug_struct("Parser")
            .field("source", &source)
            .field("tokens", &self.tokens)
            .field("position", &self.position)
            .field("eof", &self.eof)
            .finish()
    }
}
