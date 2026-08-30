use std::sync::LazyLock;

use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use wotw_seedgen_data::{
    env_or,
    seed_language::output::{
        CommonUberStateWrite, CommonWriteCommand, ContainedWritesExt, IntermediateOutput,
        UberStateWrite, UberStateWriteOwned,
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

// TODO decide default using statistic
static HAPPY_SPAWN_SLOTS: LazyLock<usize> = LazyLock::new(|| {
    let happy_spawn_slots = env_or("HAPPY_SPAWN_SLOTS", 3);

    assert!(*SPAWN_SLOTS >= happy_spawn_slots);

    happy_spawn_slots
});

pub fn solution_weights<'graph, 'log>(
    solutions: Vec<Solution<'graph>>,
    item_pool: &ItemPool<'log>,
    output: &IntermediateOutput<'log>,
    slots: usize,
    spawn_slots: usize,
) -> Vec<(Solution<'graph>, f32)> {
    let weight_context = WeightContext::new(item_pool, output, &solutions, slots, spawn_slots);

    solutions
        .into_iter()
        .enumerate()
        .map(|(index, solution)| {
            let weight = weight_context.weight(index, &solution);
            (solution, weight)
        })
        .collect()
}

/// Generator for solution weights, taking into account how frequently the same items
/// appear across solutions to counterweight similar but non-redundant variants
struct WeightContext<'pool, 'output, 'log> {
    item_pool: &'pool ItemPool<'log>,
    output: &'output IntermediateOutput<'log>,
    solution_data: Vec<SolutionData<'pool>>,
    write_counts: FxHashMap<&'pool Vec<UberStateWriteOwned>, f32>,
    slots: usize,
    spawn_slots: usize,
}

struct SolutionData<'pool> {
    item_cost: f32,
    items: FxHashSet<&'pool Vec<UberStateWriteOwned>>,
}

impl<'pool, 'output, 'log> WeightContext<'pool, 'output, 'log> {
    fn new(
        item_pool: &'pool ItemPool<'log>,
        output: &'output IntermediateOutput<'log>,
        solutions: &[Solution],
        slots: usize,
        spawn_slots: usize,
    ) -> Self {
        debug_assert!(slots >= spawn_slots);

        let mut solution_data = Vec::with_capacity(solutions.len());
        let mut write_counts = FxHashMap::<_, f32>::default();

        for solution in solutions {
            let mut item_cost = 0.;

            let items = solution
                .items()
                .iter()
                .map(|item| {
                    let item = &item_pool[*item];

                    item_cost += item.cost();

                    item.writes()
                })
                .collect::<FxHashSet<_>>();

            for writes in &items {
                *write_counts.entry(*writes).or_default() += 1.;
            }

            solution_data.push(SolutionData { item_cost, items });
        }

        Self {
            item_pool,
            output,
            solution_data,
            write_counts,
            slots,
            spawn_slots,
        }
    }

    // TODO separate spirit light slots?
    fn weight(&self, index: usize, solution: &Solution) -> f32 {
        let used_slots = solution.used_slots();
        debug_assert!(self.slots >= used_slots);

        let non_spawn_slots = self.slots - self.spawn_slots;
        let slot_weight = (1 + non_spawn_slots.saturating_sub(used_slots)) as f32;

        // TODO make it less likely to use spawn slots for later progressions?
        let sad_spawn_slots = used_slots.saturating_sub(non_spawn_slots + *HAPPY_SPAWN_SLOTS);

        let data = &self.solution_data[index];
        let cost = solution.spirit_light() as f32 + data.item_cost;
        let similarity = data
            .items
            .iter()
            .map(|writes| self.write_counts[writes])
            .product::<f32>();

        let weight = slot_weight
            * (1 + solution.new_reached) as f32
            * (0.3_f32).powf(sad_spawn_slots as f32)
            / (cost * similarity);

        trace!(
            logger: self.item_pool.log_capture,
            "Weight for {items}: {weight} = (1 + max(non_spawn_slots: {non_spawn_slots} - used_slots: {used_slots}, 0)) * (1 + new_reached: {new_reached}) * (0.3 ^ sad_spawn_slots: {sad_spawn_slots}) / (cost: {cost} * similarity: {similarity})",
            non_spawn_slots = self.slots - self.spawn_slots,
            new_reached = solution.new_reached,
            items = solution.display(self.item_pool, self.output),
        );

        debug_assert!(weight.is_finite());

        weight
    }
}

const DEFAULT_COST: f32 = 200.;

pub(crate) fn cost_from_iter<'a, I: IntoIterator<Item = UberStateWrite<'a>>>(iter: I) -> f32 {
    match iter.common().map(|write| write.cost()).sum() {
        // empty sum is -0.0
        -0.0 => DEFAULT_COST,
        other => other,
    }
}

pub(crate) trait Cost {
    fn cost(&self) -> f32;
}

impl Cost for CommonUberStateWrite {
    fn cost(&self) -> f32 {
        self.uber_identifier.cost() * self.command.cost()
    }
}

impl Cost for CommonUberIdentifier {
    fn cost(&self) -> f32 {
        match self {
            CommonUberIdentifier::SpiritLight => 1.,
            CommonUberIdentifier::BaseMaxHealth => 10.,
            CommonUberIdentifier::GorlekOre => 80.,
            CommonUberIdentifier::BaseMaxEnergy
            | CommonUberIdentifier::Skill(Skill::Regenerate | Skill::WaterBreath) => 100., // Quality-of-Life Skills
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
            CommonUberIdentifier::Skill(
                Skill::GladesAncestralLight | Skill::MarshAncestralLight,
            )
            | CommonUberIdentifier::Shard(_) => 500.,
            CommonUberIdentifier::Skill(Skill::Dash | Skill::Flap) => 600., // Counteracting bias because these unlock rather little
            CommonUberIdentifier::Skill(Skill::Glide | Skill::Grapple) => 700., // Feel-Good Finds
            CommonUberIdentifier::Skill(
                Skill::Sword | Skill::Hammer | Skill::Bow | Skill::Shuriken,
            ) => 800., // Basic Weapons
            CommonUberIdentifier::Skill(
                Skill::Burrow | Skill::WaterDash | Skill::Grenade | Skill::Flash,
            )
            | CommonUberIdentifier::CleanWater => 900., // Key Skills
            CommonUberIdentifier::Skill(Skill::DoubleJump) => 1000., // Good to find, but this is already biased for by being powerful
            CommonUberIdentifier::Skill(Skill::Blaze | Skill::Sentry) => 1400., // Tedious Weapons
            CommonUberIdentifier::Skill(Skill::Bash) => 1500., // Counteracting bias because Bash unlocks a lot
            CommonUberIdentifier::Skill(Skill::Spear) => 2000., // Lowering the frequency of slow Spear starts
            CommonUberIdentifier::Teleporter(
                Teleporter::Den
                | Teleporter::Hollow
                | Teleporter::Glades
                | Teleporter::Wellspring
                | Teleporter::Burrows
                | Teleporter::WoodsEntrance
                | Teleporter::WoodsExit
                | Teleporter::Reach
                | Teleporter::Depths
                | Teleporter::CentralPools
                | Teleporter::PoolsBoss
                | Teleporter::FeedingGrounds
                | Teleporter::CentralWastes
                | Teleporter::OuterRuins
                | Teleporter::InnerRuins
                | Teleporter::Willow
                | Teleporter::Shriek,
            ) => 12000.,
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
