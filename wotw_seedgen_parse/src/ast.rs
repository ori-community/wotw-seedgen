use std::ops::ControlFlow;

use crate::{
    ErrorKind, Mode, NoTrailingInput, OptionMode, ParseBoolToken, ParseFloatToken, ParseIntToken,
    ParseStringToken, Parser, Result, ResultMode, Tokenize,
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
/// - [`Option<T>`] implements `Ast` if `T` does. [`Option::ast`] returns `Ok(Some(T))` if `T` succeeds and `Ok(None)` if `T` fails to parse
/// - `(T1, T2)` implements `Ast` if `T1` and `T2` do.
/// - [`Vec<T>`] implements `Ast` if `T` does. [`Vec::ast`] will attempt to keep parsing `T` until the entire source is exhausted.
///   This can be useful as a top-level Ast node or as [`Delimited<Open, Vec<T>, Close>`][`Delimited`], which will attempt to parse `T` until the delimited content is exhausted.
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{LogosTokenizer, parse_ast, ParseFloatToken, Symbol};
///
/// #[derive(Clone, Copy, Logos)]
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
///     parse_ast::<_, (f32, Option<Symbol<'f'>>)>("4.2f", tokenizer).into_result(),
///     Ok((4.2, Some(Symbol)))
/// );
/// assert_eq!(
///     parse_ast::<_, (f32, Option<Symbol<'f'>>)>("4.2", tokenizer).into_result(),
///     Ok((4.2, None))
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
///     parse_ast("80%", tokenizer).into_result(),
///     Ok(Percentage {
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
/// #[derive(Clone, Copy, Logos)]
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
///     parse_ast("happy_noises", tokenizer).into_result(),
///     Ok(HappyNoises)
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
/// #[derive(Clone, Copy, Logos, TokenDisplay)]
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
///     parse_ast("fun", tokenizer).into_result(),
///     Ok(Fun)
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
/// #[derive(Clone, Copy, Logos, TokenDisplay)]
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
///     parse_ast("foo", tokenizer).into_result(),
///     Ok(Content::Foo)
/// );
///
/// assert_eq!(
///     parse_ast("\"bar\"", tokenizer).into_result(),
///     Ok(Content::String("bar"))
/// );
/// ```
///
/// You can avoid excessive backtracking by using [`Recoverable`] or [`Result`] as soon as you want to commit to a branch.
///
/// # Implementing manually
///
/// Usually your Ast will be parsed using one specific `Token` type, so your `Ast` implementation won't be generic over `Token`.
/// This is what the derive does as well. It will look like:
///
/// ```
/// # extern crate logos;
/// use logos::Logos;
/// use wotw_seedgen_parse::{Ast, LogosTokenizer, Parser, Result};
///
/// #[derive(Clone, Copy, Logos)]
/// enum Token {
///     // Token definitions
/// }
///
/// type Tokenizer = LogosTokenizer<Token>;
///
/// pub struct CustomAst;
/// impl<'source> Ast<'source, Tokenizer> for CustomAst {
///     fn ast(parser: &mut Parser<'source, Tokenizer>) -> Result<Self> {
///         todo!()
///     }
/// }
/// ```
///
/// Implementations of [`Ast::ast`] should adhere to the following rules:
///
/// If you return `Ok`, progress `parser` using [`Parser::step`] or nested [`Ast::ast`] calls
/// until the `parser` has progressed past the tokens representing this syntax tree node.
///
/// If you return `Err`, `parser` should be in the same position it was in originally.
/// If you already progressed using [`Parser::step`] or nested [`Ast::ast`] calls and
/// only later determine parsing should fail, consider using a pattern like:
///
// TODO not sure how to adjust this doctest
/// ```
/// # extern crate logos;
/// # use logos::Logos;
/// # use wotw_seedgen_parse::{Ast, LogosTokenizer, Parser, Result};
/// # #[derive(Clone, Copy, Logos)]
/// # enum Token {}
/// # type Tokenizer = LogosTokenizer<Token>;
/// # pub struct Example;
/// # impl<'source> Ast<'source, Tokenizer> for Example {
/// fn ast(parser: &mut Parser<'source, Tokenizer>) -> Result<Self> {
///     let before = parser.position();
///
///     let result = {
///         // attempt at parsing which may consume some tokens before it is certain to be successful
/// #       Ok(Self)
///     };
///
///     if result.is_err() {
///         parser.jump(before);
///     }
///     result
/// }
/// # }
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
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self>;

    #[inline]
    /// Attempt to parse `Self`, only progressing `parser` if successful.
    ///
    /// Unless when implementing custom parser [`Mode`]s, you probably want to use
    /// [`Ast::ast_result`] or [`Ast::ast_option`] depending on whether you need errors.
    fn ast_output<M: Mode>(parser: &mut Parser<'source, T>) -> M::Output<Self> {
        M::output(Self::ast_impl::<M>(parser))
    }

    #[inline]
    /// Attempt to parse `Self`, only progressing `parser` if successful.
    ///
    /// If parsing fails, an [`Error`] will be returned
    ///
    /// [`Error`]: crate::Error
    fn ast_result(parser: &mut Parser<'source, T>) -> Result<Self> {
        Self::ast_output::<ResultMode>(parser)
    }

    #[inline]
    /// Attempt to parse `Self`, only progressing `parser` if successful.
    ///
    /// If parsing fails, `None` will be returned
    fn ast_option(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_output::<OptionMode>(parser)
    }
}

/// Convenience function to parse any type implementing [`Ast`] from a [`&str`](str).
// TODO we are not assuming anymore that the library user implements ParseToken no their token.
#[inline]
pub fn parse_ast<'source, T, V>(source: &'source str, tokenizer: T) -> NoTrailingInput<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    let mut parser = Parser::new(source, tokenizer);
    NoTrailingInput::ast_option(&mut parser).unwrap() // NoTrailingInput parsing never fails
}

impl<'source, T> Ast<'source, T> for bool
where
    T: Tokenize,
    T::Token: ParseBoolToken,
{
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let value = M::opt(parser.current().0.bool(), || {
            parser.error(ErrorKind::ExpectedToken("boolean".to_string()))
        })?;

        parser.step();

        ControlFlow::Continue(value)
    }
}

macro_rules! impl_ast_number {
    ($ty:ty, $parse:ident, $trait:ident, $fn:ident, $expected:literal) => {
        impl<'source, T> Ast<'source, T> for $ty
        where
            T: Tokenize,
            T::Token: $trait,
        {
            fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
                let (token, span) = parser.current();
                if token.$fn() {
                    let value = M::res(T::Token::$parse(parser.slice(span.clone())), |err| {
                        parser.error(ErrorKind::InvalidNumber(err))
                    })?;
                    let value = M::res(Self::try_from(value), |err| {
                        parser.error(ErrorKind::InvalidNumber(err.to_string()))
                    })?;

                    parser.step();

                    ControlFlow::Continue(value)
                } else {
                    ControlFlow::Break(M::err(|| {
                        parser.error(ErrorKind::ExpectedToken($expected.to_string()))
                    }))
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
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let (token, span) = parser.current();
        if token.is_string() {
            let slice = M::res(T::Token::parse_str(parser.slice(span.clone())), |err| {
                parser.custom_error(err)
            })?;

            parser.step();

            ControlFlow::Continue(slice)
        } else {
            ControlFlow::Break(M::err(|| {
                parser.error(ErrorKind::ExpectedToken("string".to_string()))
            }))
        }
    }
}

impl<'source, T> Ast<'source, T> for String
where
    T: Tokenize,
    T::Token: ParseStringToken,
{
    #[inline]
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        <&str>::ast_impl::<M>(parser).map_continue(str::to_string)
    }
}

impl<'source, T, V> Ast<'source, T> for Box<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        V::ast_impl::<M>(parser).map_continue(Box::new)
    }
}

// TODO error messages aren't great when options are involved because if both the option and the non-option parse fail at the same position, you only get the error for the non-option one
impl<'source, T, V> Ast<'source, T> for Option<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        match V::ast_impl::<OptionMode>(parser) {
            ControlFlow::Continue(v) => ControlFlow::Continue(Some(v)),
            ControlFlow::Break(()) => ControlFlow::Continue(None),
        }
    }
}

impl<'source, T, V1, V2> Ast<'source, T> for (V1, V2)
where
    T: Tokenize,
    V1: Ast<'source, T>,
    V2: Ast<'source, T>,
{
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let before = parser.position();
        let a = V1::ast_impl::<M>(parser)?;
        match V2::ast_impl::<M>(parser) {
            ControlFlow::Continue(b) => ControlFlow::Continue((a, b)),
            ControlFlow::Break(err) => {
                parser.jump(before);
                ControlFlow::Break(err)
            }
        }
    }
}
