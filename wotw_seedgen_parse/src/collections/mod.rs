mod delimited;
mod punctuated;
mod separated;

pub use delimited::Delimited;
pub use punctuated::Punctuated;
pub use separated::{Separated, SeparatedNonEmpty};

use crate::{
    Ast, ErrorMode, Errors, NoErrors, Parser, Span, SpanEnd, SpanStart, SpannedOption, Tokenize,
};
use std::ops::{ControlFlow, Range};

pub trait AstCollection<'source, T: Tokenize>: AstCollectionInit<'source, T> {
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>>;

    fn ast_item(&mut self, parser: &mut Parser<'source, T>) -> ControlFlow<Result<(), ()>> {
        self.ast_item_impl::<Errors>(parser)
    }

    fn ast_item_no_errors(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        self.ast_item_impl::<NoErrors>(parser)
    }
}

pub trait AstCollectionInit<'source, T: Tokenize>: Sized {
    fn ast_first_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self>;

    fn ast_first_impl_spanned<E: ErrorMode>(
        parser: &mut Parser<'source, T>,
    ) -> SpannedOption<Self> {
        let option = Self::ast_first_impl::<E>(parser);

        SpannedOption::from_option(option, || parser.last_error_span())
    }

    fn ast_first(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_first_impl::<Errors>(parser)
    }

    fn ast_first_spanned(parser: &mut Parser<'source, T>) -> SpannedOption<Self> {
        Self::ast_first_impl_spanned::<Errors>(parser)
    }

    fn ast_first_no_errors(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_first_impl::<NoErrors>(parser)
    }

    fn ast_first_no_errors_spanned(parser: &mut Parser<'source, T>) -> SpannedOption<Self> {
        Self::ast_first_impl_spanned::<NoErrors>(parser)
    }
}

#[derive(Default)]
#[repr(transparent)]
struct Collection<V>(pub V);

impl<'source, T, V> Ast<'source, T> for Collection<V>
where
    T: Tokenize,
    V: AstCollection<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let before = parser.position();

        let mut v = V::ast_first_impl::<E>(parser)?;

        while !parser.is_finished() {
            match V::ast_item_impl::<E>(&mut v, parser) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(Ok(())) => return Some(Self(v)),
                ControlFlow::Break(Err(())) => {
                    parser.jump(before);

                    return None;
                }
            }
        }

        Some(Self(v))
    }
}

impl<'source, T, V> AstCollectionInit<'source, T> for Vec<V>
where
    T: Tokenize,
{
    fn ast_first_impl<E: ErrorMode>(_parser: &mut Parser<'source, T>) -> Option<Self> {
        Some(Vec::new())
    }
}

impl<'source, T, V> AstCollection<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        match V::ast_impl::<E>(parser) {
            Some(value) => {
                self.push(value);

                if parser.is_finished() {
                    ControlFlow::Break(Ok(()))
                } else {
                    ControlFlow::Continue(())
                }
            }
            None => ControlFlow::Break(Err(())),
        }
    }
}

impl<'source, T, V> Ast<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        <Collection<Self>>::ast_impl::<E>(parser).map(|c| c.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Once<V>(pub V);
impl<'source, T, V> AstCollectionInit<'source, T> for Once<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_first_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        V::ast_impl::<E>(parser).map(|v| Self(v))
    }
}

impl<'source, T, V> AstCollection<'source, T> for Once<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        _parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        ControlFlow::Break(Ok(()))
    }
}

impl<V: Span> Span for Once<V> {
    #[inline]
    fn span(&self) -> Range<usize> {
        self.0.span()
    }
}

impl<V: SpanStart> SpanStart for Once<V> {
    #[inline]
    fn span_start(&self) -> usize {
        self.0.span_start()
    }
}

impl<V: SpanEnd> SpanEnd for Once<V> {
    #[inline]
    fn span_end(&self) -> usize {
        self.0.span_end()
    }
}
