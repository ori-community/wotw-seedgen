use std::io::{self, Write};

use wotw_seedgen_assets::Source;

use crate::{
    Error, ErrorKind, ErrorMode, Errors, NoErrors, NoTrailingInput, ParseBoolToken,
    ParseFloatToken, ParseIntToken, ParseStringToken, Parser, SpannedOption, Tokenize,
};

/// Trait responsible for parsing Ast nodes
///
/// The `'source` lifetime is used to support zero-copy parsing of [`&'source str`](str) into your Ast.
///
/// `T` is a tokenizer you provide. It will usually be the same type among all `Ast` implementations of your Ast nodes,
/// meaning your manual `Ast` implementations don't have to be generic over it, they can use your tokenizer explicitely.
/// [`Tokenize::Token`] also provides your `Token` type, which will be referred to throughout this documentation.
/// Check the [`Tokenize`] documentation for more details.
///
/// # Provided Implementations
///
/// `Ast` is implemented for many common Rust types, often relying on additional trait implementations for your `Token`.
///
/// - [`bool`] implements `Ast` if `Token` implements [`ParseBoolToken`]
/// - Integer types implement `Ast` if `Token` implements [`ParseIntToken`]
/// - [`f32`] and [`f64`] implement `Ast` if `Token` implements [`ParseFloatToken`]
/// - With the `ordered_float` feature, [`OrderedFloat<f32>`] and [`OrderedFloat<f64>`] implement `Ast` if `Token` implements [`ParseFloatToken`]
/// - [`&str`](str) and [`String`] implement `Ast` if `Token` implements [`ParseStringToken`]
/// - [`Box<T>`] implements `Ast` if `T` does
/// - [`Option<T>`] implements `Ast` if `T` does. [`Option::ast`] returns `Some(Some(T))` if `T` succeeds and `Some(None)` if `T` fails to parse
/// - `(T1, T2)` implements `Ast` if `T1` and `T2` do.
/// - [`Vec<T>`] implements `Ast` if `T` does. [`Vec::ast`] will attempt to keep parsing `T` until the entire source is exhausted.
///   This can be useful as a top-level Ast node or as [`Delimited<Open, Vec<T>, Close>`][`Delimited`], which will attempt to parse `T` until the delimited content is exhausted.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseFloatToken, Symbol};
///
/// #[derive(Debug, Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"\d+\.?\d*")]
///     Number,
///     #[regex(r"[A-Za-z_]\w*")]
///     Identifier,
///     Error,
///     Eof,
/// }
///
/// impl ParseFloatToken for Token {
///     fn is_float(&self) -> bool {
///         matches!(self, Token::Number)
///     }
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast::<_, (f32, Option<Symbol<'f'>>)>("4.2f", tokenizer).parsed,
///     Some((4.2, Some(Symbol)))
/// );
/// assert_eq!(
///     parse_ast::<_, (f32, Option<Symbol<'f'>>)>("4.2", tokenizer).parsed,
///     Some((4.2, None))
/// );
/// ```
///
/// This crate provides some additional types implementing `Ast` which are commonly useful.
///
/// - [`Identifier`] uses [`ParseIdentToken::is_ident`] and parses a zero-copy [`&'source str`](str)
/// - [`Symbol`] is generic over any single character and parses that character
/// - [`Separated`], [`SeparatedNonEmpty`] and [`Punctuated`] parse collections of Ast nodes with different semantics
/// - [`Delimited`] parses delimited Ast nodes with a built-in recovery mechanism
///
/// # Deriving
///
/// You can derive this trait for `struct`s and `enum`s of any shape.
///
/// Note that derived `Ast` implementations will use your `Token` type from the scope.
///
/// # Deriving on `struct` with fields
///
/// Deriving on a `struct` with fields will parse all fields in order and fail if any field fails.
/// Unnamed and named fields behave the same.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, ParseIntToken, Symbol};
///
/// #[derive(Debug, Clone, Copy, Logos)]
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
/// #[derive(Debug, PartialEq, Ast)]
/// struct Percentage {
///     value: u8,
///     percent: Symbol<'%'>,
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("80%", tokenizer).parsed,
///     Some(Percentage {
///         value: 80,
///         percent: Symbol,
///     })
/// );
/// ```
///
/// # Deriving on unit `struct`
///
/// Deriving on a unit `struct` will parse it expecting a `Token` containing the `struct`'s name.
/// You can change the casing using the `#[ast(case = ...)]` attribute.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast};
///
/// #[derive(Debug, Clone, Copy, Logos)]
/// enum Token {
///     #[regex(r"\w+")]
///     Identifier,
///     Error,
///     Eof,
/// }
///
/// #[derive(Debug, PartialEq, Ast)]
/// #[ast(case = "snake_case")]
/// struct HappyNoises;
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("happy_noises", tokenizer).parsed,
///     Some(HappyNoises)
/// )
/// ```
///
/// Alternatively, you can add keywords to your `Token` enum and reference them using the `#[ast(token = ...)]` attribute.
/// The difference is that if you add keywords to your `Token` enum, they cannot be used as [`Identifier`] elsewhere anymore.
/// Note that using the `#[ast(token = ...)]` attribute requires your `Token` to implement [`Display`] so error messages can be constructed.
/// You can use `#[derive(TokenDisplay)]` on your `Token` for a specialized [`Display`] implementation.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, TokenDisplay};
///
/// #[derive(Debug, Clone, Copy, Logos, TokenDisplay)]
/// enum Token {
///     #[token("fun")]
///     Fun,
///     Error,
///     Eof,
/// }
///
/// #[derive(Debug, PartialEq, Ast)]
/// #[ast(token = Token::Fun)]
/// struct Fun;
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("fun", tokenizer).parsed,
///     Some(Fun)
/// )
/// ```
///
/// # Deriving on `enum`
///
/// Deriving on an `enum` will attempt to parse all variants of the enum in order until one succeeds or all fail.
/// Variants with fields behave like `struct`s with fields, unit variants behave like unit `struct`s.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, parse_ast, ParseStringToken, TokenDisplay};
///
/// #[derive(Debug, Clone, Copy, Logos, TokenDisplay)]
/// enum Token {
///     #[token("foo")]
///     Foo,
///     #[regex(r#""[^"]*""#)]
///     String,
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
/// #[derive(Debug, PartialEq, Ast)]
/// enum Content<'source> {
///     #[ast(token = Token::Foo)]
///     Foo,
///     String(&'source str),
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
/// let tokenizer = Tokenizer::new(Token::Error, Token::Eof);
///
/// assert_eq!(
///     parse_ast("foo", tokenizer).parsed,
///     Some(Content::Foo)
/// );
///
/// assert_eq!(
///     parse_ast("\"bar\"", tokenizer).parsed,
///     Some(Content::String("bar"))
/// );
/// ```
///
/// You can avoid excessive backtracking by using [`Recoverable`] as soon as you want to commit to a branch.
///
/// # Implementing manually
///
/// Usually your Ast will be parsed using one specific `Token` type, so your `Ast` implementation won't be generic over `Token`.
/// This is what the derive does as well. It will look like:
///
/// ```
/// # extern crate logos;
/// use std::ops::ControlFlow;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, ErrorMode, LogosTokenizer, Parser};
///
/// #[derive(Debug, Clone, Copy, Logos)]
/// enum Token {
///     // Token definitions
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
///
/// pub struct CustomAst;
/// impl<'source> Ast<'source, Tokenizer> for CustomAst {
///     fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, Tokenizer>) -> Option<Self> {
///         todo!()
///     }
/// }
/// ```
///
/// Implementations of [`Ast::ast_impl`] should adhere to the following rules:
///
/// If you return `Ok`, progress `parser` using [`Parser::step`] or nested [`Ast::ast_impl`] calls
/// until the `parser` has progressed past the tokens representing this syntax tree node.
///
/// If you return `Err`, `parser` should be in the same position it was in originally.
/// If you already progressed using [`Parser::step`] or nested [`Ast::ast_impl`] calls and
/// only later determine parsing should fail, consider using a pattern like:
///
/// ```
/// # extern crate logos;
/// use std::ops::ControlFlow;
/// # use logos::Logos;
/// # use wotw_seedgen_parse::LogosTokenizer;
/// use wotw_seedgen_parse::{Ast, ErrorMode, Parser};
///
/// # #[derive(Debug, Clone, Copy, Logos)]
/// # enum Token {}
/// # type Tokenizer = LogosTokenizer<Token>;
///
/// pub struct Example;
///
/// impl<'source> Ast<'source, Tokenizer> for Example {
///     fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, Tokenizer>) -> Option<Self> {
///         let before = parser.position();
///
///         let option = {
///             // attempt at parsing which may consume some tokens before it is certain to be successful
///     #       Some(Self)
///         };
///
///         if option.is_none() {
///             parser.jump(before);
///         }
///
///         option
///     }
/// }
/// ```
///
/// The provided and derived implementations rely on these rules, so if your implementation does not adhere to them, parsing may return useless results.
///
/// [`Display`]: std::fmt::Display
/// [`Identifier`]: crate::Identifier
/// [`Symbol`]: crate::Symbol
/// [`Separated`]: crate::Separated
/// [`SeparatedNonEmpty`]: crate::SeparatedNonEmpty
/// [`Punctuated`]: crate::Punctuated
/// [`Delimited`]: crate::Delimited
/// [`ParseIdentToken::is_ident`]: crate::ParseIdentToken::is_ident
/// [`Recoverable`]: crate::Recoverable
/// [`OrderedFloat<f32>`]: ordered_float::OrderedFloat
/// [`OrderedFloat<f64>`]: ordered_float::OrderedFloat
pub trait Ast<'source, T: Tokenize>: Sized {
    /// Composable parsing implementation
    ///
    /// This is the core function when manually implementing `Ast`
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self>;

    fn ast_impl_spanned<E: ErrorMode>(parser: &mut Parser<'source, T>) -> SpannedOption<Self> {
        let option = Self::ast_impl::<E>(parser);

        SpannedOption::from_option(option, || parser.last_error_span())
    }

    fn ast(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_impl::<Errors>(parser)
    }

    fn ast_spanned(parser: &mut Parser<'source, T>) -> SpannedOption<Self> {
        Self::ast_impl_spanned::<Errors>(parser)
    }

    fn ast_no_errors(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_impl::<NoErrors>(parser)
    }

    fn ast_no_errors_spanned(parser: &mut Parser<'source, T>) -> SpannedOption<Self> {
        Self::ast_impl_spanned::<NoErrors>(parser)
    }
}

/// Convenience function to parse any type implementing [`Ast`] from a [`&str`](str).
pub fn parse_ast<'source, T, V>(source: &'source str, tokenizer: T) -> ParseResult<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    let mut parser = Parser::new(source, tokenizer);

    let parsed = NoTrailingInput::<V>::ast(&mut parser).map(|v| v.0);

    ParseResult {
        parsed,
        errors: parser.errors,
    }
}

#[derive(Debug, Clone)]
pub struct ParseResult<V> {
    pub parsed: Option<V>,
    pub errors: Vec<Error>,
}

impl<V> ParseResult<V> {
    pub fn eprint_errors(self, source: &Source) -> Option<V> {
        let mut stderr = io::stderr().lock();

        let error_count = self.errors.len();

        for error in self.errors {
            error.write_pretty(source, &mut stderr).unwrap();
        }

        let success = error_count == 0;
        if !success {
            writeln!(
                &mut stderr,
                "Failed to compile Logic with {error_count} errors."
            )
            .unwrap();
        }

        if success {
            self.parsed
        } else {
            None
        }
    }
}

impl<'source, T> Ast<'source, T> for bool
where
    T: Tokenize,
    T::Token: ParseBoolToken,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let value = parser.current().0.bool();

        match value {
            None => E::err(|| parser.error(ErrorKind::ExpectedToken("boolean".to_string()))),
            Some(_) => parser.step(),
        }

        value
    }
}

macro_rules! impl_ast_number {
    ($ty:ty, $parse:ident, $trait:ident, $fn:ident, $expected:literal) => {
        impl<'source, T> Ast<'source, T> for $ty
        where
            T: Tokenize,
            T::Token: $trait,
        {
            fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
                let (token, span) = parser.current();

                if token.$fn() {
                    let result = T::Token::$parse(parser.slice(span.clone()));
                    let value = E::consume_result(result, |err| {
                        parser.error(ErrorKind::InvalidNumber(err))
                    })?;

                    let result = Self::try_from(value);
                    let value = E::consume_result(result, |err| {
                        parser.error(ErrorKind::InvalidNumber(err.to_string()))
                    })?;

                    parser.step();

                    Some(value)
                } else {
                    E::none(|| parser.error(ErrorKind::ExpectedToken($expected.to_string())))
                }
            }
        }
    };
}

macro_rules! impl_ast_integers {
    () => {};
    ($ty:ident $parse:ident $($more:tt)*) => {
        impl_ast_number!($ty, $parse, ParseIntToken, is_int, "integer");
        impl_ast_integers!($($more)*);
    };
}

macro_rules! impl_ast_floats {
    () => {};
    ($ty:ident $parse:ident $($more:tt)*) => {
        impl_ast_number!($ty, $parse, ParseFloatToken, is_float, "float");
        impl_ast_floats!($($more)*);
    };
}

impl_ast_integers! {
    u8 parse_u8 u16 parse_u16 u32 parse_u32 u64 parse_u64 u128 parse_u128
    i8 parse_i8 i16 parse_i16 i32 parse_i32 i64 parse_i64 i128 parse_i128
    usize parse_usize isize parse_isize
}

impl_ast_floats! { f32 parse_f32 f64 parse_f64 }
#[cfg(feature = "ordered_float")]
mod impl_ordered_float {
    use super::*;
    use ordered_float::OrderedFloat;
    type OrderedF32 = OrderedFloat<f32>;
    type OrderedF64 = OrderedFloat<f64>;
    impl_ast_floats! { OrderedF32 parse_f32 OrderedF64 parse_f64 }
}

impl<'source, T> Ast<'source, T> for &'source str
where
    T: Tokenize,
    T::Token: ParseStringToken,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let (token, span) = parser.current();

        if token.is_string() {
            let result = T::Token::parse_str(parser.slice(span.clone()));
            let slice = E::consume_result(result, |err| parser.error(ErrorKind::Other(err)))?;

            parser.step();

            Some(slice)
        } else {
            E::none(|| parser.error(ErrorKind::ExpectedToken("string".to_string())))
        }
    }
}

impl<'source, T> Ast<'source, T> for String
where
    T: Tokenize,
    T::Token: ParseStringToken,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        <&str>::ast_impl::<E>(parser).map(str::to_string)
    }
}

impl<'source, T, V> Ast<'source, T> for Box<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        V::ast_impl::<E>(parser).map(Box::new)
    }
}

// TODO error messages aren't great when options are involved because if both the option and the non-option parse fail at the same position, you only get the error for the non-option one
impl<'source, T, V> Ast<'source, T> for Option<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        Some(V::ast_no_errors(parser))
    }
}

impl<'source, T, V1, V2> Ast<'source, T> for (V1, V2)
where
    T: Tokenize,
    V1: Ast<'source, T>,
    V2: Ast<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let before = parser.position();

        let a = V1::ast_impl::<E>(parser)?;

        match V2::ast_impl::<E>(parser) {
            Some(b) => Some((a, b)),
            None => {
                parser.jump(before);

                None
            }
        }
    }
}
