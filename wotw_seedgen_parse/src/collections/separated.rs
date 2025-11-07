use super::{AstCollection, AstCollectionInit, Collection};
use crate::{Ast, Mode, Parser, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    iter,
    ops::{ControlFlow, Index, IndexMut, Range},
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
    fn ast_item_impl<M: Mode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Option<M::Error>> {
        match self.first {
            None => match Item::ast_option(parser) {
                Some(item) => {
                    self.first = Some(item);
                    ControlFlow::Continue(())
                }
                None => ControlFlow::Break(None),
            },
            Some(_) => separated_ast_item::<_, M, _, _>(&mut self.more, parser),
        }
    }
}

fn separated_ast_item<'source, T, M, Item, Separator>(
    items: &mut Vec<(Separator, Item)>,
    parser: &mut Parser<'source, T>,
) -> ControlFlow<Option<M::Error>>
where
    T: Tokenize,
    M: Mode,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    match Separator::ast_option(parser) {
        Some(separator) => match Item::ast_impl::<M>(parser) {
            ControlFlow::Continue(item) => {
                items.push((separator, item));
                ControlFlow::Continue(())
            }
            ControlFlow::Break(err) => ControlFlow::Break(Some(err)),
        },
        None => ControlFlow::Break(None),
    }
}

impl<'source, T, Item, Separator> Ast<'source, T> for Separated<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<M: crate::Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        <Collection<Self>>::ast_impl::<M>(parser).map_continue(|c| c.0)
    }
}

impl<Item, Separator> Separated<Item, Separator> {
    pub fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            self.first.as_ref()
        } else {
            self.more.get(index - 1).map(|(_, item)| item)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        if index == 0 {
            self.first.as_mut()
        } else {
            self.more.get_mut(index - 1).map(|(_, item)| item)
        }
    }

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

impl<Item, Separator> Index<usize> for Separated<Item, Separator> {
    type Output = Item;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            None => panic!(
                "index out of bounds: the len is {} but the index is {index}",
                self.len()
            ),
            Some(item) => item,
        }
    }
}

impl<Item, Separator> IndexMut<usize> for Separated<Item, Separator> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();

        match self.get_mut(index) {
            None => panic!("index out of bounds: the len is {len} but the index is {index}",),
            Some(item) => item,
        }
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
    fn ast_first_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let first = Item::ast_impl::<M>(parser)?;

        ControlFlow::Continue(Self {
            first,
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
    fn ast_item_impl<M: Mode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Option<M::Error>> {
        separated_ast_item::<_, M, _, _>(&mut self.more, parser)
    }
}

impl<'source, T, Item, Separator> Ast<'source, T> for SeparatedNonEmpty<Item, Separator>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        <Collection<Self>>::ast_impl::<M>(parser).map_continue(|c| c.0)
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
    #[inline]
    fn span_start(&self) -> usize {
        self.first.span_start()
    }
}

impl<Item: SpanEnd, Separator> SpanEnd for SeparatedNonEmpty<Item, Separator> {
    #[inline]
    fn span_end(&self) -> usize {
        self.last().span_end()
    }
}

impl<Item, Separator> SeparatedNonEmpty<Item, Separator> {
    pub fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            Some(&self.first)
        } else {
            self.more.get(index - 1).map(|(_, item)| item)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        if index == 0 {
            Some(&mut self.first)
        } else {
            self.more.get_mut(index - 1).map(|(_, item)| item)
        }
    }

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

impl<Item, Separator> Index<usize> for SeparatedNonEmpty<Item, Separator> {
    type Output = Item;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            None => panic!(
                "index out of bounds: the len is {} but the index is {index}",
                self.len()
            ),
            Some(item) => item,
        }
    }
}

impl<Item, Separator> IndexMut<usize> for SeparatedNonEmpty<Item, Separator> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();

        match self.get_mut(index) {
            None => panic!("index out of bounds: the len is {len} but the index is {index}",),
            Some(item) => item,
        }
    }
}
