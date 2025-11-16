use crate::{Ast, ErrorMode, Parser, Span, SpanEnd, SpanStart, SpannedOption, Tokenize};
use derivative::Derivative;
use std::{fmt::Debug, marker::PhantomData, ops::Range};

/// Trait responsible for recovering the parser when an [`Ast`] implementation fails
///
/// Use [`Recoverable<T, R>`] (which wraps [`Option<T>`]) in your Ast to recover from failing to parse any Ast node with the specified `Recover` implementor `R`
///
/// There are no implementations provided by this crate, this trait is intended for your own specialized recovery strategies.
/// When `recover` is called, the [`Parser`] will have been backtracked to the position before attempting to parse the [`Ast`] node.
/// (This is because [`Ast::ast`] has returned a [`None`] value, in which case it is responsible to backtrack the parser if necessary.)
///
/// Be careful if your recovery strategy may sometimes not progress the parser at all. This can lead to infinite loops if you are parsing into a collection.
///
/// If you want to design recovery strategies where the parser does not backtrack before recovering, consider a manual [`Ast`] implementation
/// on a custom type containing an [`Option`]. [`Delimited`] is an example of this technique.
///
/// [`Delimited`]: crate::Delimited
pub trait Recover<'source, T: Tokenize> {
    /// Recover the `parser` so it can attempt to continue parsing
    fn recover(parser: &mut Parser<'source, T>);
}

/// [`Ast`] node containing a [`SpannedOption`] and specifiying a recovery strategy if parsing the contained type fails
///
/// `T` should be the wrapped [`Ast`] node, `R` should be a [`Recover`] implementation (usually a zero-sized `struct`).
///
/// [`Recoverable::ast`] will always return [`Some`] and the actual [`SpannedOption`] will be stored within.
///
/// Apart from parsing into partial syntax trees to get more useful errors in one run, this can be used as an optimization.
/// Since [`Recoverable::ast`] always returns [`Some`] you can use it to commit to an `enum` variant after you're certain there
/// is no reason to attempt the other variants anymore, for example after encountering a keyword.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, ParseIntToken, Parser, Recover, Recoverable, SpannedOption, TokenDisplay};
///
/// #[derive(Debug, Clone, Copy, Logos, TokenDisplay)]
/// #[logos(skip r"\s+")]
/// enum Token {
///     #[token("happyness")]
///     Happyness,
///     #[token("sadness")]
///     Sadness,
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
/// enum Statement {
///     Happy(Happyness, Recoverable<u32, RecoverSkipOne>),
///     Sad(Sadness, Recoverable<u32, RecoverSkipOne>),
/// }
///
/// struct RecoverSkipOne;
/// impl<'source> Recover<'source, Tokenizer> for RecoverSkipOne {
///     fn recover(parser: &mut Parser<'source, Tokenizer>) {
///         parser.step();
///     }
/// }
///
/// #[derive(Debug, PartialEq, Ast)]
/// #[ast(token = Token::Happyness)]
/// struct Happyness;
/// #[derive(Debug, PartialEq, Ast)]
/// #[ast(token = Token::Sadness)]
/// struct Sadness;
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("happyness 16", tokenizer).parsed,
///     Some(Statement::Happy(Happyness, Recoverable::some(16)))
/// );
/// assert!(matches!(
///     parse_ast("happyness sadness", tokenizer).parsed,
///     Some(Statement::Happy(Happyness, Recoverable { value: SpannedOption::None(_), .. })) // The Statement::Sad branch was never attempted
/// ));
/// ```
#[derive(Derivative)]
#[derivative(
    Debug(bound = "T: Debug"),
    Clone(bound = "T: Clone"),
    PartialEq(bound = "T: PartialEq"),
    Eq(bound = "T: Eq")
)]
// TODO this T is called V elsewhere
pub struct Recoverable<T, R> {
    pub value: SpannedOption<T>,
    phantom: PhantomData<R>,
}

impl<T, R> Recoverable<T, R> {
    #[inline]
    pub const fn new(value: SpannedOption<T>) -> Self {
        Self {
            value,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn some(value: T) -> Self {
        Self::new(SpannedOption::Some(value))
    }

    #[inline]
    pub const fn none(span: Range<usize>) -> Self {
        Self::new(SpannedOption::None(span))
    }
}

impl<'source, T, V, R> Ast<'source, T> for Recoverable<V, R>
where
    T: Tokenize,
    V: Ast<'source, T>,
    R: Recover<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let v = V::ast_spanned(parser);

        if v.as_option().is_none() {
            R::recover(parser);
        }

        Some(Self::new(v))
    }
}

impl<V, R> Span for Recoverable<V, R>
where
    V: Span,
{
    fn span(&self) -> Range<usize> {
        self.value.span()
    }
}

impl<V, R> SpanStart for Recoverable<V, R>
where
    V: SpanStart,
{
    fn span_start(&self) -> usize {
        self.value.span_start()
    }
}

impl<V, R> SpanEnd for Recoverable<V, R>
where
    V: SpanEnd,
{
    fn span_end(&self) -> usize {
        self.value.span_end()
    }
}
