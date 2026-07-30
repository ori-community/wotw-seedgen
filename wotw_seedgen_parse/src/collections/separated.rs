use derivative::Derivative;

use super::{AstCollection, AstCollectionInit, Collection};
use crate::{Ast, ErrorMode, Parser, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    iter,
    ops::{ControlFlow, Index, IndexMut, Range},
    option, slice, vec,
};

pub type Separated<Item, Separator> = SeparatedGeneric<Item, Item, Separator>;
pub type SeparatedSmall<Item, Separator> = SeparatedGeneric<Box<Item>, Item, Separator>;

#[derive(Debug, Clone, PartialEq, Eq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct SeparatedGeneric<First, Item, Separator> {
    pub first: Option<First>,
    pub more: Vec<(Separator, Item)>,
}

// TODO PunctuatedNonEmpty? MIN_VALUES const generic?
pub type SeparatedNonEmpty<Item, Separator> = SeparatedNonEmptyGeneric<Item, Item, Separator>;
pub type SeparatedNonEmptySmall<Item, Separator> =
    SeparatedNonEmptyGeneric<Box<Item>, Item, Separator>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatedNonEmptyGeneric<First, Item, Separator> {
    pub first: First,
    pub more: Vec<(Separator, Item)>,
}

pub trait AsItem<Item> {
    fn new(item: Item) -> Self;

    fn as_item(&self) -> &Item;

    fn as_item_mut(&mut self) -> &mut Item;

    fn into_item(self) -> Item;
}

impl<Item> AsItem<Item> for Item {
    fn new(item: Item) -> Self {
        item
    }

    fn as_item(&self) -> &Item {
        self
    }

    fn as_item_mut(&mut self) -> &mut Item {
        self
    }

    fn into_item(self) -> Item {
        self
    }
}

impl<Item> AsItem<Item> for Box<Item> {
    fn new(item: Item) -> Self {
        Box::new(item)
    }

    fn as_item(&self) -> &Item {
        self
    }

    fn as_item_mut(&mut self) -> &mut Item {
        self
    }

    fn into_item(self) -> Item {
        *self
    }
}

impl<'source, T, First, Item, Separator> AstCollectionInit<'source, T>
    for SeparatedGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
{
    fn ast_first_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let first = First::ast_impl::<E>(parser);

        Some(Self {
            first,
            more: Vec::new(),
        })
    }
}

impl<'source, T, First, Item, Separator> AstCollection<'source, T>
    for SeparatedGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        shared_ast_item_impl::<_, _, _, E>(&mut self.more, parser)
    }
}

impl<'source, T, First, Item, Separator> Ast<'source, T>
    for SeparatedGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        <Collection<Self>>::ast_impl::<E>(parser).map(|c| c.0)
    }
}

impl<First, Item, Separator> SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            self.first()
        } else {
            self.more.get(index - 1).map(|(_, item)| item)
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        if index == 0 {
            self.first_mut()
        } else {
            self.more.get_mut(index - 1).map(|(_, item)| item)
        }
    }

    #[inline]
    pub fn first(&self) -> Option<&Item> {
        self.first.as_ref().map(AsItem::as_item)
    }

    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut Item> {
        self.first.as_mut().map(AsItem::as_item_mut)
    }

    #[inline]
    pub fn last(&self) -> Option<&Item> {
        self.more
            .last()
            .map(|(_, item)| item)
            .or_else(|| self.first())
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut Item> {
        self.more
            .last_mut()
            .map(|(_, item)| item)
            .or_else(|| self.first.as_mut().map(AsItem::as_item_mut))
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
        usize::from(self.first.is_some()) + self.more.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.first.is_none()
    }
}

impl<First, Item, Separator> IntoIterator for SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = Item;
    type IntoIter = iter::Chain<
        option::IntoIter<Item>,
        iter::Map<vec::IntoIter<(Separator, Item)>, fn((Separator, Item)) -> Item>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.first.map(AsItem::into_item).into_iter().chain(
            self.more
                .into_iter()
                .map((|(_, item)| item) as fn((Separator, Item)) -> Item),
        )
    }
}

impl<'a, First, Item, Separator> IntoIterator for &'a SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = &'a Item;
    type IntoIter = iter::Chain<
        option::IntoIter<&'a Item>,
        iter::Map<slice::Iter<'a, (Separator, Item)>, fn(&'a (Separator, Item)) -> &'a Item>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.first().into_iter().chain(
            self.more
                .iter()
                .map((|(_, item)| item) as fn(&'a (Separator, Item)) -> &'a Item),
        )
    }
}

impl<'a, First, Item, Separator> IntoIterator for &'a mut SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = &'a mut Item;
    type IntoIter = iter::Chain<
        option::IntoIter<&'a mut Item>,
        iter::Map<
            slice::IterMut<'a, (Separator, Item)>,
            fn(&'a mut (Separator, Item)) -> &'a mut Item,
        >,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.first
            .as_mut()
            .map(AsItem::as_item_mut)
            .into_iter()
            .chain(
                self.more
                    .iter_mut()
                    .map((|(_, item)| item) as fn(&'a mut (Separator, Item)) -> &'a mut Item),
            )
    }
}

impl<First, Item, Separator> Index<usize> for SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
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

impl<First, Item, Separator> IndexMut<usize> for SeparatedGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();

        match self.get_mut(index) {
            None => panic!("index out of bounds: the len is {len} but the index is {index}"),
            Some(item) => item,
        }
    }
}

impl<'source, T, First, Item, Separator> AstCollectionInit<'source, T>
    for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
{
    fn ast_first_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let first = First::ast_impl::<E>(parser);

        first.map(|first| Self {
            first,
            more: Vec::new(),
        })
    }
}

impl<'source, T, First, Item, Separator> AstCollection<'source, T>
    for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        shared_ast_item_impl::<_, _, _, E>(&mut self.more, parser)
    }
}

impl<'source, T, First, Item, Separator> Ast<'source, T>
    for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    T: Tokenize,
    First: Ast<'source, T>,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        <Collection<Self>>::ast_impl::<E>(parser).map(|c| c.0)
    }
}

impl<First, Item, Separator> SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    #[inline]
    pub fn new(first: Item) -> Self {
        Self {
            first: First::new(first),
            more: Vec::new(),
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Item> {
        if index == 0 {
            Some(self.first.as_item())
        } else {
            self.more.get(index - 1).map(|(_, item)| item)
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        if index == 0 {
            Some(self.first.as_item_mut())
        } else {
            self.more.get_mut(index - 1).map(|(_, item)| item)
        }
    }

    #[inline]
    pub fn last(&self) -> &Item {
        self.more
            .last()
            .map_or_else(|| self.first.as_item(), |(_, item)| item)
    }

    #[inline]
    pub fn last_mut(&mut self) -> &mut Item {
        self.more
            .last_mut()
            .map_or_else(|| self.first.as_item_mut(), |(_, item)| item)
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
}

impl<First, Item, Separator> IntoIterator for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = Item;
    type IntoIter = iter::Chain<
        iter::Once<Item>,
        iter::Map<vec::IntoIter<(Separator, Item)>, fn((Separator, Item)) -> Item>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self.first.into_item()).chain(
            self.more
                .into_iter()
                .map((|(_, item)| item) as fn((Separator, Item)) -> Item),
        )
    }
}

impl<'a, First, Item, Separator> IntoIterator
    for &'a SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = &'a Item;
    type IntoIter = iter::Chain<
        iter::Once<&'a Item>,
        iter::Map<slice::Iter<'a, (Separator, Item)>, fn(&'a (Separator, Item)) -> &'a Item>,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self.first.as_item()).chain(
            self.more
                .iter()
                .map((|(_, item)| item) as fn(&'a (Separator, Item)) -> &'a Item),
        )
    }
}

impl<'a, First, Item, Separator> IntoIterator
    for &'a mut SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    type Item = &'a mut Item;
    type IntoIter = iter::Chain<
        iter::Once<&'a mut Item>,
        iter::Map<
            slice::IterMut<'a, (Separator, Item)>,
            fn(&'a mut (Separator, Item)) -> &'a mut Item,
        >,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        iter::once(self.first.as_item_mut()).chain(
            self.more
                .iter_mut()
                .map((|(_, item)| item) as fn(&'a mut (Separator, Item)) -> &'a mut Item),
        )
    }
}

impl<First, Item, Separator> Index<usize> for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
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

impl<First, Item, Separator> IndexMut<usize> for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();

        match self.get_mut(index) {
            None => panic!("index out of bounds: the len is {len} but the index is {index}"),
            Some(item) => item,
        }
    }
}

impl<First, Item, Separator> Span for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
    Item: SpanStart + SpanEnd,
{
    #[inline]
    fn span(&self) -> Range<usize> {
        self.first.as_item().span_start()..self.last().span_end()
    }
}

impl<First, Item, Separator> SpanStart for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
    Item: SpanStart,
{
    #[inline]
    fn span_start(&self) -> usize {
        self.first.as_item().span_start()
    }
}

impl<First, Item, Separator> SpanEnd for SeparatedNonEmptyGeneric<First, Item, Separator>
where
    First: AsItem<Item>,
    Item: SpanEnd,
{
    #[inline]
    fn span_end(&self) -> usize {
        self.last().span_end()
    }
}

fn shared_ast_item_impl<'source, Item, Separator, T, E: ErrorMode>(
    more: &mut Vec<(Separator, Item)>,
    parser: &mut Parser<'source, T>,
) -> ControlFlow<Result<(), ()>>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Separator: Ast<'source, T>,
{
    match Separator::ast_no_errors(parser) {
        Some(separator) => match Item::ast_impl::<E>(parser) {
            Some(item) => {
                more.push((separator, item));

                ControlFlow::Continue(())
            }
            None => ControlFlow::Break(Err(())),
        },
        None => ControlFlow::Break(Ok(())),
    }
}
