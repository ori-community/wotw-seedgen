use std::fmt;

use rustc_hash::FxHashMap;

use crate::item::Item;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Inventory {
    pub items: FxHashMap<Item, u16>,
}
impl Inventory {
    pub fn grant(&mut self, mut item: Item, mut amount: u16) {
        let single_instance = item.is_single_instance();
        if single_instance && amount > 1 {
            log::warn!("Granted {} more than once, but that item can only be aquired once...", item);
        }
        if let Item::SpiritLight(stacked_amount) = item {
            amount *= stacked_amount;
            item = Item::SpiritLight(1);
        }
        let prior = self.items.entry(item).or_insert(0);
        if single_instance {
            // TODO I feel like in an efficient implementation this shouldn't exist...
            *prior = amount;
        } else {
            *prior += amount;
        }
    }
    pub fn remove(&mut self, item: &Item, amount: u16) -> u16 {
        match self.items.get_mut(item) {
            Some(prior) => {
                if amount >= *prior {
                    let negative = amount - *prior;
                    self.items.remove(item);
                    negative
                } else {
                    *prior -= amount;
                    0
                }
            },
            None => amount,
        }
    }

    pub fn has(&self, item: &Item, amount: u16) -> bool {
        if let Some(owned) = self.items.get(item) {
            return *owned >= amount;
        }
        false
    }
    pub fn get(&self, item: &Item) -> u16 {
        *self.items.get(item).unwrap_or(&0)
    }

    pub fn item_count(&self) -> usize {
        let mut count = 0;
        for (item, amount) in &self.items {
            if let Item::SpiritLight(stacked_amount) = item {
                count += (amount * stacked_amount + 39) / 40;  // this will usually demand more than necessary, but with the placeholder system that shouldn't be a problem (and underestimating the needed slots can force a retry)
            } else {
                count += amount;
            }
        }

        count.into()
    }
    pub fn world_item_count(&self) -> usize {
        let mut count = 0;
        for (item, amount) in self.items.iter().filter(|&(item, _)| !item.is_multiworld_spread()) {
            if let Item::SpiritLight(stacked_amount) = item {
                count += (amount * stacked_amount + 39) / 40;  // this will usually demand more than necessary, but with the placeholder system that shouldn't be a problem (and underestimating the needed slots can force a retry)
            } else {
                count += amount;
            }
        }

        count.into()
    }

    pub fn cost(&self) -> f32 {
        let mut cost = 0;
        for item in self.items.keys() {
            cost += item.cost() * self.items[item];
        }

        cost.into()
    }

    pub fn contains(&self, other: &Inventory) -> bool {
        for (item, amount) in &other.items {
            if !self.has(item, *amount) {
                return false;
            }
        }
        true
    }

    pub fn merge(&self, other: &Inventory) -> Inventory {
        let mut merged = self.clone();
        for (item, amount) in other.items.clone() {
            merged.grant(item, amount);
        }
        merged
    }
}

impl fmt::Display for Inventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = self.items.iter().map(|(item, amount)| {
            if amount == &1 {
                format!("{}", item)
            } else {
                format!("{} {}", amount, item)
            }
        }).collect::<Vec<_>>();
        write!(f, "{}", display.join(", "))
    }
}

impl From<Item> for Inventory {
    fn from(item: Item) -> Inventory {
        let mut inventory = Inventory::default();
        inventory.grant(item, 1);
        inventory
    }
}
impl From<(Item, u16)> for Inventory {
    fn from(item_amount: (Item, u16)) -> Inventory {
        let mut inventory = Inventory::default();
        let (item, amount) = item_amount;
        inventory.grant(item, amount);
        inventory
    }
}
impl From<Vec<Item>> for Inventory {
    fn from(items: Vec<Item>) -> Inventory {
        let mut inventory = Inventory::default();
        for item in items {
            inventory.grant(item, 1);
        }
        inventory
    }
}
impl From<Vec<(Item, u16)>> for Inventory {
    fn from(items: Vec<(Item, u16)>) -> Inventory {
        let mut inventory = Inventory::default();
        for (item, amount) in items {
            inventory.grant(item, amount);
        }
        inventory
    }
}
