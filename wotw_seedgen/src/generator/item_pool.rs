use super::cost::Cost;
use crate::{common_item::CommonItem, inventory::Inventory};
use rand::{seq::SliceRandom, Rng};
use rand_pcg::Pcg64Mcg;
use std::iter;
use wotw_seedgen_data::{Shard, Skill, WeaponUpgrade};
use wotw_seedgen_seed_language::{compile, output::CommandVoid};

// TODO not so sure this is an efficient item pool, maybe try something else once it's possible to benchmark seedgen again
// in particular, I think maybe simply cloning the items would be more efficient than the item_lookup
// TODO don't really think this should be public
#[derive(Debug, Clone, PartialEq)]
pub struct ItemPool {
    items: Vec<usize>,
    item_lookup: Vec<CommandVoid>,
    inventory: Inventory,
}
impl Default for ItemPool {
    fn default() -> Self {
        Self {
            items: [
                iter::repeat(0).take(40),
                iter::repeat(1).take(34),
                iter::repeat(2).take(5),
                iter::repeat(3).take(24),
                iter::repeat(4).take(24),
            ]
            .into_iter()
            .flatten()
            .chain(5..=63)
            .collect(),
            item_lookup: [
                compile::gorlek_ore(),
                compile::keystone(),
                compile::shard_slot(),
                compile::health_fragment(),
                compile::energy_fragment(),
                compile::skill(Skill::Bash),
                compile::skill(Skill::DoubleJump),
                compile::skill(Skill::Launch),
                compile::skill(Skill::Glide),
                compile::skill(Skill::WaterBreath),
                compile::skill(Skill::Grenade),
                compile::skill(Skill::Grapple),
                compile::skill(Skill::Flash),
                compile::skill(Skill::Spear),
                compile::skill(Skill::Regenerate),
                compile::skill(Skill::Bow),
                compile::skill(Skill::Hammer),
                compile::skill(Skill::Sword),
                compile::skill(Skill::Burrow),
                compile::skill(Skill::Dash),
                compile::skill(Skill::WaterDash),
                compile::skill(Skill::Shuriken),
                compile::skill(Skill::Blaze),
                compile::skill(Skill::Sentry),
                compile::skill(Skill::Flap),
                compile::skill(Skill::GladesAncestralLight),
                compile::skill(Skill::InkwaterAncestralLight),
                compile::clean_water(),
                compile::shard(Shard::Overcharge),
                compile::shard(Shard::TripleJump),
                compile::shard(Shard::Wingclip),
                compile::shard(Shard::Bounty),
                compile::shard(Shard::Swap),
                compile::shard(Shard::Magnet),
                compile::shard(Shard::Splinter),
                compile::shard(Shard::Reckless),
                compile::shard(Shard::Quickshot),
                compile::shard(Shard::Resilience),
                compile::shard(Shard::SpiritLightHarvest),
                compile::shard(Shard::Vitality),
                compile::shard(Shard::LifeHarvest),
                compile::shard(Shard::EnergyHarvest),
                compile::shard(Shard::Energy),
                compile::shard(Shard::LifePact),
                compile::shard(Shard::LastStand),
                compile::shard(Shard::Sense),
                compile::shard(Shard::UltraBash),
                compile::shard(Shard::UltraGrapple),
                compile::shard(Shard::Overflow),
                compile::shard(Shard::Thorn),
                compile::shard(Shard::Catalyst),
                compile::shard(Shard::Turmoil),
                compile::shard(Shard::Sticky),
                compile::shard(Shard::Finesse),
                compile::shard(Shard::SpiritSurge),
                compile::shard(Shard::Lifeforce),
                compile::shard(Shard::Deflector),
                compile::shard(Shard::Fracture),
                compile::shard(Shard::Arcing),
                compile::weapon_upgrade(WeaponUpgrade::ExplodingSpear),
                compile::weapon_upgrade(WeaponUpgrade::HammerShockwave),
                compile::weapon_upgrade(WeaponUpgrade::StaticShuriken),
                compile::weapon_upgrade(WeaponUpgrade::ChargeBlaze),
                compile::weapon_upgrade(WeaponUpgrade::RapidSentry),
            ]
            .into_iter()
            .collect(),
            inventory: Inventory {
                spirit_light: 0,
                gorlek_ore: 40,
                keystones: 34,
                shard_slots: 5,
                health: 24 * 5,
                energy: 24. * 0.5,
                skills: [
                    Skill::Bash,
                    Skill::DoubleJump,
                    Skill::Launch,
                    Skill::Glide,
                    Skill::WaterBreath,
                    Skill::Grenade,
                    Skill::Grapple,
                    Skill::Flash,
                    Skill::Spear,
                    Skill::Regenerate,
                    Skill::Bow,
                    Skill::Hammer,
                    Skill::Sword,
                    Skill::Burrow,
                    Skill::Dash,
                    Skill::WaterDash,
                    Skill::Shuriken,
                    Skill::Blaze,
                    Skill::Sentry,
                    Skill::Flap,
                    Skill::GladesAncestralLight,
                    Skill::InkwaterAncestralLight,
                ]
                .into_iter()
                .collect(),
                shards: [
                    Shard::Overcharge,
                    Shard::TripleJump,
                    Shard::Wingclip,
                    Shard::Bounty,
                    Shard::Swap,
                    Shard::Magnet,
                    Shard::Splinter,
                    Shard::Reckless,
                    Shard::Quickshot,
                    Shard::Resilience,
                    Shard::SpiritLightHarvest,
                    Shard::Vitality,
                    Shard::LifeHarvest,
                    Shard::EnergyHarvest,
                    Shard::Energy,
                    Shard::LifePact,
                    Shard::LastStand,
                    Shard::Sense,
                    Shard::UltraBash,
                    Shard::UltraGrapple,
                    Shard::Overflow,
                    Shard::Thorn,
                    Shard::Catalyst,
                    Shard::Turmoil,
                    Shard::Sticky,
                    Shard::Finesse,
                    Shard::SpiritSurge,
                    Shard::Lifeforce,
                    Shard::Deflector,
                    Shard::Fracture,
                    Shard::Arcing,
                ]
                .into_iter()
                .collect(),
                teleporters: Default::default(),
                clean_water: true,
                weapon_upgrades: [
                    WeaponUpgrade::ExplodingSpear,
                    WeaponUpgrade::HammerShockwave,
                    WeaponUpgrade::StaticShuriken,
                    WeaponUpgrade::ChargeBlaze,
                    WeaponUpgrade::RapidSentry,
                ]
                .into_iter()
                .collect(),
            },
        }
    }
}
impl ItemPool {
    // TODO when doing forced keystone placements this should only need a reference to remove
    pub fn change(&mut self, command: CommandVoid, mut amount: i32) {
        let common_items = CommonItem::from_command(&command);

        let index = self
            .item_lookup
            .iter()
            .enumerate()
            .find(|(_, a)| *a == &command)
            .map(|(index, _)| index);

        if amount > 0 {
            for common_item in iter::repeat(common_items).take(amount as usize).flatten() {
                common_item.grant(&mut self.inventory);
            }

            let index = match index {
                None => {
                    let index = self.item_lookup.len();
                    self.item_lookup.push(command);
                    index
                }
                Some(index) => index,
            };
            self.items.extend(iter::repeat(index).take(amount as usize));
        } else if let Some(index) = index {
            for common_item in iter::repeat(common_items).take(-amount as usize).flatten() {
                common_item.remove(&mut self.inventory);
            }

            self.items.retain(|i| {
                amount == 0
                    || if *i == index {
                        amount += 1;
                        false
                    } else {
                        true
                    }
            });
        }
    }

    pub fn choose_random(&mut self, rng: &mut Pcg64Mcg) -> Option<CommandVoid> {
        if self.items.is_empty() {
            return None;
        }
        let index = self.items.swap_remove(rng.gen_range(0..self.items.len()));
        let command = self.item_lookup[index].clone();

        let cost = command.cost();
        if cost > 10000 {
            let reroll_chance = -10000.0 / cost as f64 + 1.0;

            if rng.gen_bool(reroll_chance) {
                self.items.push(index);
                return self.choose_random(rng);
            }
        }

        for common_item in CommonItem::from_command(&command) {
            common_item.remove(&mut self.inventory); // TODO mild inaccuracy: if the pool has multiple, say, skills, placing the first will remove it from the inventory
        }

        Some(command)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline]
    pub fn items(&self) -> impl Iterator<Item = &CommandVoid> {
        self.items.iter().map(|index| &self.item_lookup[*index])
    }
    #[inline]
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    #[inline]
    pub fn drain(&mut self) -> Drain<'_> {
        self.inventory.clear();
        Drain::new(self)
    }
    #[inline]
    pub fn drain_random<'pool>(&'pool mut self, rng: &mut Pcg64Mcg) -> Drain<'pool> {
        self.items.shuffle(rng);
        self.drain()
    }
}

pub struct Drain<'pool> {
    item_pool: &'pool mut ItemPool,
}
impl<'pool> Drain<'pool> {
    fn new(item_pool: &'pool mut ItemPool) -> Self {
        Self { item_pool }
    }
}
impl Iterator for Drain<'_> {
    type Item = CommandVoid;

    fn next(&mut self) -> Option<Self::Item> {
        self.item_pool
            .items
            .pop()
            .map(|index| self.item_pool.item_lookup[index].clone())
    }
}
