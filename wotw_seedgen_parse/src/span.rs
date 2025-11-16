use crate::{Ast, ErrorMode, Parser, Tokenize};
use std::{cell::LazyCell, ops::Range};

/// Trait responsible for providing spans on successfully parsed Ast nodes in the form of [`Range<usize>`]
///
/// [`Spanned`] implements both [`Span`] and [`Ast`], it will usually form the foundation of your Ast spans.
///
/// You can use `#[derive(Span)]` to expose spans on your higher-level Ast nodes based on their children or implement it manually.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, ParseIntToken, ParseStringToken, Span, Spanned, TokenDisplay};
///
/// #[derive(Clone, Copy, Logos, TokenDisplay)]
/// #[logos(skip r"\s+")]
/// enum Token {
///     #[token("joke")]
///     Joke,
///     #[regex(r#""[^"]*""#)]
///     String,
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
/// impl ParseStringToken for Token {
///     fn is_string(&self) -> bool {
///         matches!(self, Token::String)
///     }
/// }
///
/// #[derive(Ast, Span)]
/// struct Joke<'source> {
///     keyword: Spanned<JokeKeyword>,
///     content: &'source str, // You can span the content if you want to but it's not necessary for the Span implementation of Joke
///     rating: Spanned<i32>,
/// }
/// #[derive(Ast)]
/// #[ast(token = Token::Joke)]
/// struct JokeKeyword;
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// let source = "joke \"It's been 5 years\" 10";
/// let joke: Joke = parse_ast(source, tokenizer).parsed.unwrap();
/// assert_eq!(
///     joke.keyword.span(),
///     0..4,
/// );
/// assert_eq!(
///     joke.rating.span(),
///     source.len() - 2..source.len(),
/// );
/// assert_eq!(
///     joke.span(),
///     0..source.len(),
/// );
/// ```
pub trait Span {
    fn span(&self) -> Range<usize>;
}

/// Similar to [`Span`], but only determines the start index
pub trait SpanStart {
    fn span_start(&self) -> usize;
}

/// Similar to [`Span`], but only determines the end index
pub trait SpanEnd {
    fn span_end(&self) -> usize;
}

impl Span for Range<usize> {
    #[inline]
    fn span(&self) -> Range<usize> {
        self.clone()
    }
}

impl<T: Span> Span for &T {
    #[inline]
    fn span(&self) -> Range<usize> {
        T::span(self)
    }
}

impl<T: Span> Span for &mut T {
    #[inline]
    fn span(&self) -> Range<usize> {
        T::span(self)
    }
}

impl<T: Span> Span for Box<T> {
    #[inline]
    fn span(&self) -> Range<usize> {
        (**self).span()
    }
}

impl<F: FnOnce() -> Range<usize>> Span for LazyCell<Range<usize>, F> {
    fn span(&self) -> Range<usize> {
        (*self).clone()
    }
}

impl<T: SpanStart> SpanStart for &T {
    #[inline]
    fn span_start(&self) -> usize {
        T::span_start(self)
    }
}

impl<T: SpanStart> SpanStart for &mut T {
    #[inline]
    fn span_start(&self) -> usize {
        T::span_start(self)
    }
}

impl<T: SpanStart> SpanStart for Box<T> {
    #[inline]
    fn span_start(&self) -> usize {
        (**self).span_start()
    }
}

impl<F: FnOnce() -> Range<usize>> SpanStart for LazyCell<Range<usize>, F> {
    fn span_start(&self) -> usize {
        self.start
    }
}

impl<T: SpanEnd> SpanEnd for &T {
    #[inline]
    fn span_end(&self) -> usize {
        T::span_end(self)
    }
}

impl<T: SpanEnd> SpanEnd for &mut T {
    #[inline]
    fn span_end(&self) -> usize {
        T::span_end(self)
    }
}

impl<T: SpanEnd> SpanEnd for Box<T> {
    #[inline]
    fn span_end(&self) -> usize {
        (**self).span_end()
    }
}

impl<F: FnOnce() -> Range<usize>> SpanEnd for LazyCell<Range<usize>, F> {
    fn span_end(&self) -> usize {
        self.end
    }
}

impl<T1: SpanStart, T2: SpanEnd> Span for (T1, T2) {
    #[inline]
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}

impl<T1: SpanStart, T2> SpanStart for (T1, T2) {
    fn span_start(&self) -> usize {
        self.0.span_start()
    }
}

impl<T1, T2: SpanEnd> SpanEnd for (T1, T2) {
    fn span_end(&self) -> usize {
        self.1.span_end()
    }
}

/// [`Ast`] node storing the span of the parsed content alongside the wrapped content itself
///
/// See the [`Span`] trait documentation for more details.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// The parsed content
    pub data: T,
    /// Source span corresponding to the parsed content
    pub span: Range<usize>,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn new(data: T, span: Range<usize>) -> Self {
        Self { data, span }
    }
}

impl<T> Span for Spanned<T> {
    #[inline]
    fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl<T> SpanStart for Spanned<T> {
    #[inline]
    fn span_start(&self) -> usize {
        self.span.start
    }
}

impl<T> SpanEnd for Spanned<T> {
    #[inline]
    fn span_end(&self) -> usize {
        self.span.end
    }
}

impl<'source, T, V> Ast<'source, T> for Spanned<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let start = parser.current().1.start;

        let data = V::ast_impl::<E>(parser)?;

        let end = parser.token_at(parser.position() - 1).1.end;
        let span = start..end;

        Some(Self { data, span })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpannedOption<T> {
    Some(T),
    None(Range<usize>),
}

impl<T> SpannedOption<T> {
    #[inline]
    pub fn from_option<F: FnOnce() -> Range<usize>>(option: Option<T>, span: F) -> Self {
        match option {
            None => Self::None(span()),
            Some(t) => Self::Some(t),
        }
    }

    #[inline]
    pub fn as_option(&self) -> Option<&T> {
        match self {
            Self::Some(t) => Some(t),
            Self::None(_) => None,
        }
    }

    #[inline]
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Some(t) => Some(t),
            Self::None(_) => None,
        }
    }

    #[inline]
    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Some(t) => Some(t),
            Self::None(_) => None,
        }
    }
}

impl<T: Span> Span for SpannedOption<T> {
    #[inline]
    fn span(&self) -> Range<usize> {
        match self {
            Self::Some(t) => t.span(),
            Self::None(span) => span.clone(),
        }
    }
}

impl<T: SpanStart> SpanStart for SpannedOption<T> {
    #[inline]
    fn span_start(&self) -> usize {
        match self {
            Self::Some(t) => t.span_start(),
            Self::None(span) => span.start,
        }
    }
}

impl<T: SpanEnd> SpanEnd for SpannedOption<T> {
    #[inline]
    fn span_end(&self) -> usize {
        match self {
            Self::Some(t) => t.span_end(),
            Self::None(span) => span.end,
        }
    }
}

impl<'source, T, V> Ast<'source, T> for SpannedOption<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        Some(V::ast_impl_spanned::<E>(parser))
    }
}
