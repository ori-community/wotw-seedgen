use log::trace;
use wotw_seedgen_data::{
    seed_language::output::{
        CommonUberStateWrite, CommonWriteCommand, ContainedWrites, UberStateWriteOwned,
    },
    CommonUberIdentifier, Skill, Teleporter,
};

use crate::{
    generator::{
        placement::SPAWN_SLOTS,
        solutions::{Solution, SolutionLike},
    },
    item_pool::ItemPool,
};

const HAPPY_SPAWN_SLOTS: usize = 3;
const _: usize = SPAWN_SLOTS - HAPPY_SPAWN_SLOTS; // check that SPAWN_SLOTS >= PREFERRED_SPAWN_SLOTS

impl Solution {
    // TODO separate spirit light slots?
    pub(crate) fn weight(&self, item_pool: &ItemPool, slots: usize, spawn_slots: usize) -> f32 {
        let used_slots = self.used_slots();
        debug_assert!(slots >= used_slots && slots >= spawn_slots);

        let non_spawn_slots = slots - spawn_slots;
        let slot_weight = (1 + non_spawn_slots.saturating_sub(used_slots)) as f32;

        // TODO make it less likely to use spawn slots for later progressions?
        let sad_spawn_slots = used_slots.saturating_sub(non_spawn_slots + HAPPY_SPAWN_SLOTS);

        let cost = self.spirit_light as f32
            + self
                .items
                .iter()
                .map(|item| item_pool[*item].cost())
                .sum::<f32>();

        let weight =
            slot_weight * (1 + self.new_reached) as f32 * (0.3_f32).powf(sad_spawn_slots as f32)
                / cost;

        trace!(
            "Weight for {items}: {weight} = (1 + max(non_spawn_slots: {non_spawn_slots} - used_slots: {used_slots}, 0)) * (1 + new_reached: {new_reached}) * (0.3 ^ sad_spawn_slots: {sad_spawn_slots}) / (cost: {cost})",
            non_spawn_slots = slots - spawn_slots,
            new_reached = self.new_reached,
            items = self.display(item_pool, None),
        );

        debug_assert!(weight.is_finite());

        weight
    }
}

const DEFAULT_COST: f32 = 200.;

pub(crate) trait Cost {
    fn cost(&self) -> f32;
}

impl Cost for Vec<UberStateWriteOwned> {
    fn cost(&self) -> f32 {
        match self
            .contained_common_writes()
            .map(|write| write.cost())
            .sum()
        {
            // empty sum is -0.0
            -0.0 => DEFAULT_COST,
            other => other,
        }
    }
}

impl Cost for CommonUberStateWrite {
    fn cost(&self) -> f32 {
        self.uber_identifier.cost() * self.command.cost()
    }
}

impl Cost for CommonUberIdentifier {
    fn cost(&self) -> f32 {
        match self {
            CommonUberIdentifier::Health | CommonUberIdentifier::Energy => 0.,
            CommonUberIdentifier::SpiritLight => 1.,
            CommonUberIdentifier::MaxHealth => 12.,
            CommonUberIdentifier::GorlekOre => 80.,
            CommonUberIdentifier::Skill(Skill::Regenerate)
            | CommonUberIdentifier::Skill(Skill::WaterBreath) => 100., // Quality-of-Life Skills
            CommonUberIdentifier::MaxEnergy => 120.,
            CommonUberIdentifier::Keystones => 160.,
            CommonUberIdentifier::WeaponUpgrade(_) => 200.,
            CommonUberIdentifier::Skill(
                Skill::WallJump
                | Skill::SpiritFlame
                | Skill::Seir
                | Skill::BowCharge
                | Skill::WeaponCharge,
            ) => DEFAULT_COST,
            CommonUberIdentifier::ShardSlots => 240.,
            CommonUberIdentifier::Skill(Skill::GladesAncestralLight)
            | CommonUberIdentifier::Skill(Skill::MarshAncestralLight)
            | CommonUberIdentifier::Shard(_) => 500.,
            CommonUberIdentifier::Skill(Skill::Dash) | CommonUberIdentifier::Skill(Skill::Flap) => {
                600.
            } // Counteracting bias because these unlock rather little
            CommonUberIdentifier::Skill(Skill::Glide)
            | CommonUberIdentifier::Skill(Skill::Grapple) => 700., // Feel-Good Finds
            CommonUberIdentifier::Skill(Skill::Sword)
            | CommonUberIdentifier::Skill(Skill::Hammer)
            | CommonUberIdentifier::Skill(Skill::Bow)
            | CommonUberIdentifier::Skill(Skill::Shuriken) => 800., // Basic Weapons
            CommonUberIdentifier::Skill(Skill::Burrow)
            | CommonUberIdentifier::Skill(Skill::WaterDash)
            | CommonUberIdentifier::Skill(Skill::Grenade)
            | CommonUberIdentifier::Skill(Skill::Flash)
            | CommonUberIdentifier::CleanWater => 900., // Key Skills
            CommonUberIdentifier::Skill(Skill::DoubleJump) => 1000., // Good to find, but this is already biased for by being powerful
            CommonUberIdentifier::Skill(Skill::Blaze)
            | CommonUberIdentifier::Skill(Skill::Sentry) => 1400., // Tedious Weapons
            CommonUberIdentifier::Skill(Skill::Bash) => 1500., // Counteracting bias because Bash unlocks a lot
            CommonUberIdentifier::Skill(Skill::Spear) => 2000., // Lowering the frequency of slow Spear starts
            CommonUberIdentifier::Teleporter(Teleporter::Den)
            | CommonUberIdentifier::Teleporter(Teleporter::Hollow)
            | CommonUberIdentifier::Teleporter(Teleporter::Glades)
            | CommonUberIdentifier::Teleporter(Teleporter::Wellspring)
            | CommonUberIdentifier::Teleporter(Teleporter::Burrows)
            | CommonUberIdentifier::Teleporter(Teleporter::WoodsEntrance)
            | CommonUberIdentifier::Teleporter(Teleporter::WoodsExit)
            | CommonUberIdentifier::Teleporter(Teleporter::Reach)
            | CommonUberIdentifier::Teleporter(Teleporter::Depths)
            | CommonUberIdentifier::Teleporter(Teleporter::CentralPools)
            | CommonUberIdentifier::Teleporter(Teleporter::PoolsBoss)
            | CommonUberIdentifier::Teleporter(Teleporter::FeedingGrounds)
            | CommonUberIdentifier::Teleporter(Teleporter::CentralWastes)
            | CommonUberIdentifier::Teleporter(Teleporter::OuterRuins)
            | CommonUberIdentifier::Teleporter(Teleporter::InnerRuins)
            | CommonUberIdentifier::Teleporter(Teleporter::Willow)
            | CommonUberIdentifier::Teleporter(Teleporter::Shriek) => 12000.,
            CommonUberIdentifier::Teleporter(Teleporter::Marsh) => 15000.,
            CommonUberIdentifier::Skill(Skill::Launch) => 20000., // Absolutely Broken
        }
    }
}

impl Cost for CommonWriteCommand {
    fn cost(&self) -> f32 {
        match self {
            CommonWriteCommand::SetBooleanTrue => 1.,
            CommonWriteCommand::AddInteger(amount) => *amount as f32,
            CommonWriteCommand::AddFloat(amount) => **amount,
        }
    }
}
