use super::{AstCollection, AstCollectionInit, Collection};
use crate::{Ast, Error, Parser, Result, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    iter,
    ops::{ControlFlow, Range},
    option, slice, vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Separated<Item, Separator> {
    pub first: Option<Item>,
    pub more: Vec<(Separator, Item)>,
}
impl<Item, Separator> Default for Separated<Item, Separator> {
    #[inline]
    fn default() -> Self {
        Self {
            first: Default::default(),
            more: Default::default(),
        }
    }
}
impl<'source, T, Item, Separator> AstCollection<'source, T> for Separated<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    fn ast_item(&mut self, parser: &mut Parser<'source, T>) -> ControlFlow<Option<Error>> {
        match self.first {
            None => match Item::ast(parser) {
                Ok(item) => {
                    self.first = Some(item);
                    ControlFlow::Continue(())
                }
                Err(_) => ControlFlow::Break(None),
            },
            Some(_) => separated_ast_item(&mut self.more, parser),
        }
    }
}
fn separated_ast_item<'source, T, Item, Separator>(
    items: &mut Vec<(Separator, Item)>,
    parser: &mut Parser<'source, T>,
) -> ControlFlow<Option<Error>>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    match Separator::ast(parser) {
        Ok(separator) => match Item::ast(parser) {
            Ok(item) => {
                items.push((separator, item));
                ControlFlow::Continue(())
            }
            Err(err) => ControlFlow::Break(Some(err)),
        },
        Err(_) => ControlFlow::Break(None),
    }
}
impl<'source, T, Item, Separator> Ast<'source, T> for Separated<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast(parser: &mut Parser<'source, T>) -> Result<Self> {
        <Collection<Self>>::ast(parser).map(|c| c.0)
    }
}
impl<Item, Separator> Separated<Item, Separator> {
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    #[inline]
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    #[inline]
    pub fn len(&self) -> usize {
        match self.first {
            None => 0,
            Some(_) => 1 + self.more.len(),
        }
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }
}
impl<Item, Separator> IntoIterator for Separated<Item, Separator> {
    type Item = Item;
    type IntoIter = iter::Chain<
        option::IntoIter<Item>,
        iter::Map<vec::IntoIter<(Separator, Item)>, fn((Separator, Item)) -> Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.first.into_iter().chain(
            self.more
                .into_iter()
                .map((|(_, item)| item) as fn((Separator, Item)) -> Item),
        )
    }
}
impl<'a, Item, Separator> IntoIterator for &'a Separated<Item, Separator> {
    type Item = &'a Item;
    type IntoIter = iter::Chain<
        option::Iter<'a, Item>,
        iter::Map<slice::Iter<'a, (Separator, Item)>, fn(&(Separator, Item)) -> &Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.first.iter().chain(
            self.more
                .iter()
                .map((|(_, item)| item) as fn(&(Separator, Item)) -> &Item),
        )
    }
}
impl<'a, Item, Separator> IntoIterator for &'a mut Separated<Item, Separator> {
    type Item = &'a mut Item;
    type IntoIter = iter::Chain<
        option::IterMut<'a, Item>,
        iter::Map<slice::IterMut<'a, (Separator, Item)>, fn(&mut (Separator, Item)) -> &mut Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.first.iter_mut().chain(
            self.more
                .iter_mut()
                .map((|(_, item)| item) as fn(&mut (Separator, Item)) -> &mut Item),
        )
    }
}

// TODO PunctuatedNonEmpty? MIN_VALUES const generic?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatedNonEmpty<Item, Separator> {
    pub first: Item,
    pub more: Vec<(Separator, Item)>,
}
impl<'source, T, Item, Separator> AstCollectionInit<'source, T>
    for SeparatedNonEmpty<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
{
    fn ast_first(parser: &mut Parser<'source, T>) -> Result<Self> {
        Ok(Self {
            first: Item::ast(parser)?,
            more: Default::default(),
        })
    }
}
impl<'source, T, Item, Separator> AstCollection<'source, T> for SeparatedNonEmpty<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast_item(&mut self, parser: &mut Parser<'source, T>) -> ControlFlow<Option<Error>> {
        separated_ast_item(&mut self.more, parser)
    }
}
impl<'source, T, Item, Separator> Ast<'source, T> for SeparatedNonEmpty<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast(parser: &mut Parser<'source, T>) -> Result<Self> {
        <Collection<Self>>::ast(parser).map(|c| c.0)
    }
}
impl<Item: Span, Separator> Span for SeparatedNonEmpty<Item, Separator> {
    fn span(&self) -> Range<usize> {
        let first_span = self.first.span();
        match self.more.last() {
            None => first_span,
            Some((_, last)) => first_span.start..last.span().end,
        }
    }
}
impl<Item: SpanStart, Separator> SpanStart for SeparatedNonEmpty<Item, Separator> {
    fn span_start(&self) -> usize {
        self.first.span_start()
    }
}
impl<Item: SpanEnd, Separator> SpanEnd for SeparatedNonEmpty<Item, Separator> {
    fn span_end(&self) -> usize {
        self.last().span_end()
    }
}
impl<Item, Separator> SeparatedNonEmpty<Item, Separator> {
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    #[inline]
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        1 + self.more.len()
    }
    pub fn last(&self) -> &Item {
        self.more.last().map_or(&self.first, |(_, item)| item)
    }
}
impl<Item, Separator> IntoIterator for SeparatedNonEmpty<Item, Separator> {
    type Item = Item;
    type IntoIter = iter::Chain<
        iter::Once<Item>,
        iter::Map<vec::IntoIter<(Separator, Item)>, fn((Separator, Item)) -> Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(self.first).chain(
            self.more
                .into_iter()
                .map((|(_, item)| item) as fn((Separator, Item)) -> Item),
        )
    }
}
impl<'a, Item, Separator> IntoIterator for &'a SeparatedNonEmpty<Item, Separator> {
    type Item = &'a Item;
    type IntoIter = iter::Chain<
        iter::Once<&'a Item>,
        iter::Map<slice::Iter<'a, (Separator, Item)>, fn(&(Separator, Item)) -> &Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(&self.first).chain(
            self.more
                .iter()
                .map((|(_, item)| item) as fn(&(Separator, Item)) -> &Item),
        )
    }
}
impl<'a, Item, Separator> IntoIterator for &'a mut SeparatedNonEmpty<Item, Separator> {
    type Item = &'a mut Item;
    type IntoIter = iter::Chain<
        iter::Once<&'a mut Item>,
        iter::Map<slice::IterMut<'a, (Separator, Item)>, fn(&mut (Separator, Item)) -> &mut Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(&mut self.first).chain(
            self.more
                .iter_mut()
                .map((|(_, item)| item) as fn(&mut (Separator, Item)) -> &mut Item),
        )
    }
}
