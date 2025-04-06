use crate::{contained_uber_identifiers::ContainedWrites, generator::weight::cost};

use super::SEED_FAILED_MESSAGE;
use itertools::Itertools;
use log::{trace, warn};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{
    fmt::{self, Display},
    iter,
    ops::{Deref, DerefMut},
};
use wotw_seedgen_data::{Shard, Skill, UberIdentifier, WeaponUpgrade};
use wotw_seedgen_seed_language::{compile, output::CommandVoid};

// TODO don't really think this should be public
#[derive(Debug, Clone, PartialEq)]
pub struct ItemPool {
    rng: Pcg64Mcg,
    items: Vec<CommandVoid>,
}

impl ItemPool {
    // TODO check where it should be shuffled after creation

    pub fn new(rng: &mut Pcg64Mcg) -> Self {
        let rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);

        let items = iter::repeat(compile::gorlek_ore())
            .take(40)
            .chain(iter::repeat(compile::keystone()).take(34))
            .chain(iter::repeat(compile::shard_slot()).take(5))
            .chain(iter::repeat(compile::health_fragment()).take(24))
            .chain(iter::repeat(compile::energy_fragment()).take(24))
            .chain([
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
                compile::skill(Skill::MarshAncestralLight),
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
            ])
            .collect();

        Self { rng, items }
    }
}

impl ItemPool {
    pub fn remove_command(&mut self, command: &CommandVoid) -> Option<CommandVoid> {
        match self.items.iter().position(|item| item == command) {
            None => {
                warn!("Attempted to remove {command} from the item pool, but it didn't exist");
                trace!("Current item pool: {self}");
                None
            }
            Some(index) => Some(self.items.swap_remove(index)),
        }
    }

    pub fn choose_random(&mut self) -> Option<CommandVoid> {
        loop {
            let command = self.items.pop()?;

            let cost = command
                .contained_common_write_identifiers()
                .map(cost)
                .sum::<usize>();
            if cost > 10000 {
                let reroll_chance = -10000.0 / cost as f64 + 1.0;

                if self.rng.gen_bool(reroll_chance) {
                    self.items.push(command);
                    continue;
                }
            }

            return Some(command);
        }
    }

    // TODO only consider positive writes, this would break for instance when shuffling remove skills into the item pool
    pub fn progression_for(
        &self,
        uber_identifier: UberIdentifier,
    ) -> impl Iterator<Item = (usize, &CommandVoid)> {
        self.iter().enumerate().filter(move |(_, item)| {
            item.contained_write_identifiers()
                .contains(&uber_identifier)
        })
    }
}

impl Deref for ItemPool {
    type Target = Vec<CommandVoid>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for ItemPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl Display for ItemPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut items = FxHashMap::default();
        for item in &**self {
            *items.entry(item).or_insert(0_u32) += 1;
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
