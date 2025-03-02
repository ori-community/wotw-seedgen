mod delimited;
mod punctuated;
mod separated;

pub use delimited::Delimited;
pub use punctuated::Punctuated;
pub use separated::{Separated, SeparatedNonEmpty};

use crate::{Ast, Error, Parser, Result, Span, SpanEnd, SpanStart, Tokenize};
use std::ops::{ControlFlow, Range};

pub trait AstCollection<'source, T: Tokenize>: AstCollectionInit<'source, T> {
    fn ast_item(&mut self, parser: &mut Parser<'source, T>) -> ControlFlow<Option<Error>>;
}
pub trait AstCollectionInit<'source, T: Tokenize>: Sized {
    fn ast_first(parser: &mut Parser<'source, T>) -> Result<Self>;
}
impl<'source, T, V> AstCollectionInit<'source, T> for V
where
    T: Tokenize,
    V: Default,
{
    #[inline]
    fn ast_first(_parser: &mut Parser<'source, T>) -> Result<Self> {
        Ok(Self::default())
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
    fn ast(parser: &mut Parser<'source, T>) -> Result<Self> {
        let before = parser.position();
        let mut v = V::ast_first(parser)?;
        while !parser.is_finished() {
            match v.ast_item(parser) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(None) => return Ok(Self(v)),
                ControlFlow::Break(Some(err)) => {
                    parser.jump(before);
                    return Err(err);
                }
            }
        }
        Ok(Self(v))
    }
}

impl<'source, T, V> AstCollection<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_item(&mut self, parser: &mut Parser<'source, T>) -> ControlFlow<Option<Error>> {
        match V::ast(parser) {
            Ok(value) => {
                self.push(value);

                if parser.is_finished() {
                    ControlFlow::Break(None)
                } else {
                    ControlFlow::Continue(())
                }
            }
            Err(err) => ControlFlow::Break(Some(err)),
        }
    }
}
impl<'source, T, V> Ast<'source, T> for Vec<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast(parser: &mut Parser<'source, T>) -> Result<Self> {
        <Collection<Self>>::ast(parser).map(|c| c.0)
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
    fn ast_first(parser: &mut Parser<'source, T>) -> Result<Self> {
        V::ast(parser).map(Self)
    }
}
impl<'source, T, V> AstCollection<'source, T> for Once<V>
where
    T: Tokenize,
    V: Ast<'source, T>,
{
    fn ast_item(&mut self, _parser: &mut Parser<'source, T>) -> ControlFlow<Option<Error>> {
        ControlFlow::Break(None)
    }
}
impl<V: Span> Span for Once<V> {
    fn span(&self) -> Range<usize> {
        self.0.span()
    }
}
impl<V: SpanStart> SpanStart for Once<V> {
    fn span_start(&self) -> usize {
        self.0.span_start()
    }
}
impl<V: SpanEnd> SpanEnd for Once<V> {
    fn span_end(&self) -> usize {
        self.0.span_end()
    }
}
