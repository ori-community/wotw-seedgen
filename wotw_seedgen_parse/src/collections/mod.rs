mod delimited;
mod punctuated;
mod separated;

pub use delimited::Delimited;
pub use punctuated::Punctuated;
pub use separated::{Separated, SeparatedNonEmpty};

use crate::{
    Ast, Mode, OptionMode, Parser, Result, ResultMode, Span, SpanEnd, SpanStart, Tokenize,
};
use std::ops::{ControlFlow, Range};

pub trait AstCollection<'source, T: Tokenize>: AstCollectionInit<'source, T> {
    fn ast_item_impl<M: Mode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Option<M::Error>>;
}

pub trait AstCollectionInit<'source, T: Tokenize>: Sized {
    fn ast_first_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self>;

    #[inline]
    fn ast_first_output<M: Mode>(parser: &mut Parser<'source, T>) -> M::Output<Self> {
        M::output(Self::ast_first_impl::<M>(parser))
    }

    #[inline]
    fn ast_first_result(parser: &mut Parser<'source, T>) -> Result<Self> {
        Self::ast_first_output::<ResultMode>(parser)
    }

    #[inline]
    fn ast_first_option(parser: &mut Parser<'source, T>) -> Option<Self> {
        Self::ast_first_output::<OptionMode>(parser)
    }
}

impl<'source, T, V> AstCollectionInit<'source, T> for V
where
    T: Tokenize,
    V: Default,
{
    #[inline]
    fn ast_first_impl<M: Mode>(_parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        ControlFlow::Continue(Self::default())
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
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let before = parser.position();
        let mut v = V::ast_first_impl::<M>(parser)?;
        while !parser.is_finished() {
            match v.ast_item_impl::<M>(parser) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(None) => return ControlFlow::Continue(Self(v)),
                ControlFlow::Break(Some(err)) => {
                    parser.jump(before);
                    return ControlFlow::Break(err);
                }
            }
        }
        ControlFlow::Continue(Self(v))
    }
}

impl<'source, T, V> AstCollection<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_item_impl<M: Mode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Option<M::Error>> {
        match V::ast_impl::<M>(parser) {
            ControlFlow::Continue(value) => {
                self.push(value);

                if parser.is_finished() {
                    ControlFlow::Break(None)
                } else {
                    ControlFlow::Continue(())
                }
            }
            ControlFlow::Break(err) => ControlFlow::Break(Some(err)),
        }
    }
}

impl<'source, T, V> Ast<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        <Collection<Self>>::ast_impl::<M>(parser).map_continue(|c| c.0)
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
    fn ast_first_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        V::ast_impl::<M>(parser).map_continue(Self)
    }
}

impl<'source, T, V> AstCollection<'source, T> for Once<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    #[inline]
    fn ast_item_impl<M: Mode>(
        &mut self,
        _parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Option<M::Error>> {
        ControlFlow::Break(None)
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
