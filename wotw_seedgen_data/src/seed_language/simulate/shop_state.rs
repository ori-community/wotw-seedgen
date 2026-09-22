use std::collections::hash_map::Entry;

use rustc_hash::FxHashMap;

use crate::UberIdentifier;

#[derive(Debug, Clone, Default)]
pub struct ShopState {
    inner: FxHashMap<UberIdentifier, ShopItemState>,
}

impl ShopState {
    pub fn visibility(&self, uber_identifier: UberIdentifier) -> ShopItemVisibility {
        match self.inner.get(&uber_identifier) {
            None
            | Some(ShopItemState {
                hidden: false,
                locked: false,
            }) => ShopItemVisibility::Visible,
            Some(ShopItemState {
                hidden: true,
                locked: _,
            }) => ShopItemVisibility::Hidden,
            Some(ShopItemState {
                hidden: false,
                locked: true,
            }) => ShopItemVisibility::Locked,
        }
    }

    pub fn set_hidden(&mut self, uber_identifier: UberIdentifier, hidden: bool) {
        match self.inner.entry(uber_identifier) {
            Entry::Occupied(mut occupied) => occupied.get_mut().hidden = hidden,
            Entry::Vacant(vacant) => {
                if hidden {
                    vacant.insert(ShopItemState {
                        hidden: true,
                        locked: false,
                    });
                }
            }
        }
    }

    pub fn set_locked(&mut self, uber_identifier: UberIdentifier, locked: bool) {
        match self.inner.entry(uber_identifier) {
            Entry::Occupied(mut occupied) => occupied.get_mut().locked = locked,
            Entry::Vacant(vacant) => {
                if locked {
                    vacant.insert(ShopItemState {
                        hidden: false,
                        locked: true,
                    });
                }
            }
        }
    }
}

pub enum ShopItemVisibility {
    Visible,
    Hidden,
    Locked,
}

#[derive(Debug, Clone, Default)]
struct ShopItemState {
    hidden: bool,
    locked: bool,
}
