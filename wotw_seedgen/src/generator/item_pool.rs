use crate::generator::solutions::Cost;

use super::SEED_FAILED_MESSAGE;
use itertools::Itertools;
use log::{trace, warn};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{
    fmt::{self, Display},
    iter, mem,
    ops::Deref,
};
use strum::VariantArray;
use wotw_seedgen_data::{
    seed_language::{
        compile,
        output::{CommandVoid, ContainedWrites, UberStateWrite, UberStateWriteOwned},
    },
    Shard, Skill, WeaponUpgrade,
};

pub struct ItemPoolBuilder {
    item_pool: ItemPool,
}

impl ItemPoolBuilder {
    pub fn new(rng: &mut Pcg64Mcg) -> Self {
        const GORLEK_ORE_AMOUNT: usize = 40;
        const KEYSTONE_AMOUNT: usize = 34;
        const SHARD_SLOT_AMOUNT: usize = 5;
        const HEALTH_FRAGMENT_AMOUNT: usize = 24;
        const ENERGY_FRAGMENT_AMOUNT: usize = 24;
        const SKILLS: [Skill; 22] = [
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
            Skill::MarshAncestralLight,
        ];
        const SHARDS: [Shard; 31] = [
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
            Shard::LightHarvest,
            Shard::Vitality,
            Shard::LifeHarvest,
            Shard::EnergyHarvest,
            Shard::Energy,
            Shard::LifePact,
            Shard::LastStand,
            Shard::Secret,
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
        ];

        const TOTAL_AMOUNT: usize = GORLEK_ORE_AMOUNT
            + KEYSTONE_AMOUNT
            + SHARD_SLOT_AMOUNT
            + HEALTH_FRAGMENT_AMOUNT
            + ENERGY_FRAGMENT_AMOUNT
            + SKILLS.len()
            + 1
            + SHARDS.len()
            + WeaponUpgrade::VARIANTS.len();

        let rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);
        let items = Vec::with_capacity(TOTAL_AMOUNT);
        let item_pool = ItemPool { rng, items };
        let mut builder = Self { item_pool };

        builder.add_amount(compile::gorlek_ore(), GORLEK_ORE_AMOUNT);
        builder.add_amount(compile::keystone(), KEYSTONE_AMOUNT);
        builder.add_amount(compile::shard_slot(), SHARD_SLOT_AMOUNT);
        builder.add_amount(compile::health_fragment(), HEALTH_FRAGMENT_AMOUNT);
        builder.add_amount(compile::energy_fragment(), ENERGY_FRAGMENT_AMOUNT);
        for skill in SKILLS {
            builder.add(compile::skill(skill));
        }
        builder.add(compile::clean_water());
        for shard in SHARDS {
            builder.add(compile::shard(shard));
        }
        for weapon_upgrade in WeaponUpgrade::VARIANTS {
            builder.add(compile::weapon_upgrade(*weapon_upgrade));
        }

        builder
    }

    pub fn add(&mut self, item: CommandVoid) {
        self.item_pool.items.push(Item::new(item));
    }

    pub fn add_amount(&mut self, item: CommandVoid, amount: usize) {
        self.item_pool
            .items
            .extend(iter::repeat_n(Item::new(item), amount));
    }

    pub fn remove(&mut self, item: &CommandVoid) {
        self.item_pool.find_remove(item);
    }

    pub fn remove_amount(&mut self, item: &CommandVoid, amount: usize) {
        self.item_pool.find_remove_amount(item, amount);
    }

    pub fn finish(mut self) -> ItemPool {
        self.item_pool.items.shuffle(&mut self.item_pool.rng);
        self.item_pool
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemPool {
    rng: Pcg64Mcg,
    items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    command: CommandVoid,
    writes: Vec<UberStateWriteOwned>,
    cost: f32,
}

impl Item {
    fn new(command: CommandVoid) -> Self {
        let writes = command.contained_writes_owned().collect::<Vec<_>>();
        let cost = writes.cost();
        Self {
            command,
            writes,
            cost,
        }
    }

    pub fn writes(&self) -> &Vec<UberStateWriteOwned> {
        &self.writes
    }
}

impl Deref for Item {
    type Target = CommandVoid;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl ContainedWrites for Item {
    fn contained_writes(&self) -> impl Iterator<Item = UberStateWrite<'_>> {
        self.writes.contained_writes()
    }
}

impl Cost for Item {
    fn cost(&self) -> f32 {
        self.cost
    }
}

impl ItemPool {
    pub fn find_remove(&mut self, item: &CommandVoid) -> bool {
        match self.items.iter().position(|i| &i.command == item) {
            None => {
                self.log_find_remove_failed(item);
                false
            }
            Some(index) => {
                self.remove(index);
                true
            }
        }
    }

    pub fn find_remove_amount(&mut self, item: &CommandVoid, amount: usize) -> bool {
        let mut last_index = 0;

        for _ in 0..amount {
            let Some(mut index) = self.items[last_index..]
                .iter()
                .position(|i| &i.command == item)
            else {
                self.log_find_remove_failed(item);
                return false;
            };

            index += last_index;
            self.remove(index);
            last_index = index;
        }

        true
    }

    fn log_find_remove_failed(&self, item: &CommandVoid) {
        warn!(
            "Attempted to remove {item} from the item pool, but it didn't exist",
            item = item.log_display()
        );
        trace!("Current item pool: {self}");
    }

    pub fn remove(&mut self, index: usize) -> CommandVoid {
        self.items.swap_remove(index).command
    }

    pub fn choose_random(&mut self) -> Option<CommandVoid> {
        // TODO also precompute reroll chance?
        self.items
            .iter()
            .rposition(|item| {
                let cost = item.writes.cost();

                cost <= 10000. || {
                    let choose = self.rng.gen_bool(10000. / cost as f64);

                    if !choose {
                        trace!("Rerolling random placement {}", item.command.log_display())
                    }

                    choose
                }
            })
            .map(|index| self.remove(index))
    }

    pub fn take(&mut self) -> impl Iterator<Item = CommandVoid> {
        mem::take(&mut self.items)
            .into_iter()
            .map(|item| item.command)
    }
}

impl Deref for ItemPool {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl Display for ItemPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = FxHashMap::default();
        for item in &self.items {
            *items.entry(&item.command).or_insert(0_u32) += 1;
        }

        let items = items
            .into_iter()
            .map(|(item, amount)| (item.to_string(), amount))
            .sorted_unstable_by(|(a, _), (b, _)| a.cmp(b))
            .format_with(", ", |(item, amount), f| {
                f(&format_args!("{amount} {item}"))
            });

        write!(f, "{items}")
    }
}
