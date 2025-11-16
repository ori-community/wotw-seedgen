use crate::{Ast, ErrorKind, ErrorMode, ParseIdentToken, Parser, Tokenize};
use std::fmt::{self, Display};

/// [`Ast`] node parsing an identifier
///
/// [`Identifier`] implements [`Ast`] if your `Token` implements [`ParseIdentToken`].
/// If [`ParseIdentToken::is_ident`] returns `true`, the token's slice will be stored as-is inside this type.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{parse_ast, Identifier, LogosTokenizer, ParseIdentToken};
///
/// #[derive(Clone, Copy, Logos)]
/// #[logos(skip r"\s+")]
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
/// assert_eq!(
///     parse_ast("   OriIsAGoodGame   ", tokenizer).parsed,
///     Some(Identifier("OriIsAGoodGame"))
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Identifier<'source>(pub &'source str);
impl<'source, T> Ast<'source, T> for Identifier<'source>
where
    T: Tokenize,
    T::Token: ParseIdentToken,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let (token, span) = parser.current();

        if token.is_ident() {
            let slice = parser.slice(span.clone());

            parser.step();

            Some(Self(slice))
        } else {
            E::none(|| parser.error(ErrorKind::ExpectedToken("identifier".to_string())))
        }
    }
}

impl Display for Identifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// [`Ast`] node parsing a specific [`char`]
///
/// The implementation will not check the kind of `Token`, but it will only succeed if the `Token` contains *only* the character
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, ParseIntToken, Symbol};
///
/// #[derive(Clone, Copy, Logos)]
/// #[logos(skip r"\s+")]
/// enum Token {
///     #[regex(r"[A-Za-z_]\w*")]
///     Identifier,
///     #[regex(r"\d+")]
///     Number,
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
/// #[derive(Debug, PartialEq, Ast)]
/// struct HugsAmount {
///     amount: u128,
///     x: Symbol<'x'>,
///     hugs: HugsPlease,
/// }
/// #[derive(Debug, PartialEq, Ast)]
/// struct HugsPlease;
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("2x HugsPlease", tokenizer).parsed,
///     Some(HugsAmount {
///         amount: 2,
///         x: Symbol,
///         hugs: HugsPlease,
///     })
/// );
///
/// // "xHugsPlease" will be tokenized as one identifier, so <Symbol<'x'>>::ast fill fail
/// assert!(parse_ast::<_, HugsAmount>("2xHugsPlease", tokenizer).parsed.is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbol<const CHAR: char>;
impl<'source, T, const CHAR: char> Ast<'source, T> for Symbol<CHAR>
where
    T: Tokenize,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        match parser.current_slice().strip_prefix(CHAR) {
            Some("") => {
                parser.step();

                Some(Self)
            }
            _ => E::none(|| parser.error(ErrorKind::ExpectedToken(Self.to_string()))),
        }
    }
}

impl<const CHAR: char> Display for Symbol<CHAR> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{CHAR}'")
    }
}

/// [`Ast`] node expecting the parser to be fully finished after parsing `T`
///
/// This usually won't actually be part of your Ast, rather it is returned by [`parse_ast`].
///
/// After calling [`NoTrailingInput::ast`], the `parser` will always be exhausted.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, NoTrailingInput, parse_ast, ParseIntToken, Symbol};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"\d+")]
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
/// let result = parse_ast::<_, u8>("5$", tokenizer);
/// assert_eq!(result.parsed, Some(5));
/// assert!(!result.errors.is_empty());
/// ```
///
/// [`parse_ast`]: crate::parse_ast
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoTrailingInput<V>(pub V);

impl<'source, T, V> Ast<'source, T> for NoTrailingInput<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        V::ast_impl::<E>(parser).map(|value| {
            if !parser.is_finished() {
                E::err(|| parser.error(ErrorKind::ExpectedToken("end of input".to_string())));

                parser.jump(parser.end());
            }

            Self(value)
        })
    }
}
