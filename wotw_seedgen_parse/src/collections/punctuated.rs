use derivative::Derivative;

use super::{AstCollection, Collection};
use crate::{Ast, AstCollectionInit, ErrorMode, Parser, Tokenize};
use std::{
    cmp::Ordering,
    iter,
    ops::{ControlFlow, Index, IndexMut},
    option, slice, vec,
};

#[derive(Debug, Clone, PartialEq, Eq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct Punctuated<Item, Punctuation> {
    pub items: Vec<(Item, Punctuation)>,
    pub last: Option<Item>,
}

impl<'source, T, Item, Punctuation> AstCollectionInit<'source, T> for Punctuated<Item, Punctuation>
where
    T: Tokenize,
{
    fn ast_first_impl<E: ErrorMode>(_parser: &mut Parser<'source, T>) -> Option<Self> {
        Some(Self::default())
    }
}

impl<'source, T, Item, Punctuation> AstCollection<'source, T> for Punctuated<Item, Punctuation>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Punctuation: Ast<'source, T>,
{
    fn ast_item_impl<E: ErrorMode>(
        &mut self,
        parser: &mut Parser<'source, T>,
    ) -> ControlFlow<Result<(), ()>> {
        match Item::ast_impl::<E>(parser) {
            Some(item) => match Punctuation::ast_no_errors(parser) {
                Some(punctuation) => {
                    self.items.push((item, punctuation));

                    ControlFlow::Continue(())
                }
                None => {
                    self.last = Some(item);

                    ControlFlow::Break(Ok(()))
                }
            },
            None => ControlFlow::Break(Err(())),
        }
    }
}

impl<'source, T, Item, Punctuation> Ast<'source, T> for Punctuated<Item, Punctuation>
where
    T: Tokenize,
    Item: Ast<'source, T>,
    Punctuation: Ast<'source, T>,
{
    #[inline]
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        <Collection<Self>>::ast_impl::<E>(parser).map(|c| c.0)
    }
}

impl<Item, Punctuation> Punctuated<Item, Punctuation> {
    pub fn get(&self, index: usize) -> Option<&Item> {
        match index.cmp(&self.items.len()) {
            Ordering::Less => Some(&self.items[index].0),
            Ordering::Equal => self.last.as_ref(),
            Ordering::Greater => None,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Item> {
        match index.cmp(&self.items.len()) {
            Ordering::Less => Some(&mut self.items[index].0),
            Ordering::Equal => self.last.as_mut(),
            Ordering::Greater => None,
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
        self.items.len() + self.last.is_some() as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.last.is_none()
    }
}

impl<Item, Punctuation> IntoIterator for Punctuated<Item, Punctuation> {
    type Item = Item;
    type IntoIter = iter::Chain<
        iter::Map<vec::IntoIter<(Item, Punctuation)>, fn((Item, Punctuation)) -> Item>,
        option::IntoIter<Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.items
            .into_iter()
            .map((|(item, _)| item) as fn((Item, Punctuation)) -> Item)
            .chain(self.last)
    }
}

impl<'a, Item, Punctuation> IntoIterator for &'a Punctuated<Item, Punctuation> {
    type Item = &'a Item;
    type IntoIter = iter::Chain<
        iter::Map<slice::Iter<'a, (Item, Punctuation)>, fn(&(Item, Punctuation)) -> &Item>,
        option::Iter<'a, Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.items
            .iter()
            .map((|(item, _)| item) as fn(&(Item, Punctuation)) -> &Item)
            .chain(&self.last)
    }
}

impl<'a, Item, Punctuation> IntoIterator for &'a mut Punctuated<Item, Punctuation> {
    type Item = &'a mut Item;
    type IntoIter = iter::Chain<
        iter::Map<
            slice::IterMut<'a, (Item, Punctuation)>,
            fn(&mut (Item, Punctuation)) -> &mut Item,
        >,
        option::IterMut<'a, Item>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.items
            .iter_mut()
            .map((|(item, _)| item) as fn(&mut (Item, Punctuation)) -> &mut Item)
            .chain(&mut self.last)
    }
}

impl<Item, Punctuation> Index<usize> for Punctuated<Item, Punctuation> {
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

impl<Item, Punctuation> IndexMut<usize> for Punctuated<Item, Punctuation> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // TODO feels like this shouldn't be necessary on the successful branch (same for other collections)
        let len = self.len();

        match self.get_mut(index) {
            None => panic!("index out of bounds: the len is {len} but the index is {index}",),
            Some(item) => item,
        }
    }
}
