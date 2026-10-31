mod area_skills;
mod area_spirit_light;
mod area_unlock;
mod early_skills;
mod first_weapon;
mod item_area;
mod item_location;
mod item_unlock;
mod location_item;
mod location_unlock;
// TODO progression_item?
mod progression;
mod progression_locations;
mod spawn_area;
mod spawn_item_count;
mod spawn_items;
mod spawn_location;
mod step_size;
mod total_spirit_light;

// TODO use contained_writes for analysis instead of common_items?

pub use area_skills::AreaSkillsStats;
pub use area_spirit_light::AreaSpiritLightStats;
pub use area_unlock::AreaUnlockStats;
pub use early_skills::EarlySkillsStats;
pub use first_weapon::FirstWeaponStats;
pub use item_area::ItemAreaStats;
pub use item_location::ItemLocationStats;
pub use item_unlock::ItemUnlockStats;
pub use location_item::LocationItemStats;
pub use location_unlock::LocationUnlockStats;
pub use progression::ProgressionStats;
pub use progression_locations::ProgressionLocationsStats;
pub use spawn_area::SpawnAreaStats;
pub use spawn_item_count::SpawnItemCountStats;
pub use spawn_items::SpawnItemStats;
pub use spawn_location::SpawnLocationStats;
pub use step_size::StepSizeStats;
pub use total_spirit_light::TotalSpiritLightStats;

use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, ControlFlow, Rem, Sub},
    str::FromStr,
    vec,
};

use wotw_seedgen::{
    data::{
        seed_language::output::{CommonItem, ContainedWrites, ContainedWritesExt},
        Area, VariantArray,
    },
    spoiler::{NodeSummary, SeedSpoiler, SpoilerPlacement},
};

/// Trait for types that may analyze seeds and generate statistics
///
/// Check the types in this module for some provided implementations
pub trait Analyzer: Sync {
    /// A brief title describing what kind of statistic is analyzed
    fn title(&self) -> String;

    /// Analyze a given seed and return one or more keys that this seed should be categorized into
    ///
    /// For instance, [`SpawnLocationStats`] will return the name of the spawn locations here
    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String>;

    /// Compare two keys created by this analyzer
    ///
    /// You can manually implement this to customize how your keys will be ordered in the resulting csv
    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        // We need the indirection of returning a function pointer so the trait can still be made into a trait object
        fn compare(a: &str, b: &str) -> Ordering {
            match a
                .split('-')
                .next()
                .unwrap()
                .parse::<u32>()
                .into_iter()
                .zip(b.split('-').next().unwrap().parse::<u32>())
                .next()
            {
                None => a.cmp(b),
                Some((a, b)) => a.cmp(&b),
            }
        }
        compare
    }
}

pub struct SeedAnalyzer<'s> {
    pub spoiler: &'s SeedSpoiler,
    placements: Vec<&'s SpoilerPlacement>,
}

impl<'s> SeedAnalyzer<'s> {
    pub fn new(spoiler: &'s SeedSpoiler) -> Self {
        Self {
            spoiler,
            placements: spoiler
                .preplacements
                .iter()
                .chain(spoiler.groups.iter().flat_map(|group| &group.placements))
                .collect(),
        }
    }

    pub fn world_analyzer<T: Clone + Default>(&'s self) -> WorldAnalyzer<'s, T> {
        WorldAnalyzer {
            stats: vec![(T::default(), ControlFlow::Continue(())); self.spoiler.spawns.len()],
            inner: self,
        }
    }

    pub fn world_analyzer_with<T: Clone>(&'s self, init: T) -> WorldAnalyzer<'s, T> {
        WorldAnalyzer {
            stats: vec![(init, ControlFlow::Continue(())); self.spoiler.spawns.len()],
            inner: self,
        }
    }

    pub fn placements(&'s self) -> impl Iterator<Item = &'s SpoilerPlacement> {
        self.placements.iter().copied()
    }
}

#[must_use]
pub struct WorldAnalyzer<'s, T> {
    stats: Vec<(T, ControlFlow<()>)>,
    inner: &'s SeedAnalyzer<'s>,
}

impl<'s, T> WorldAnalyzer<'s, T> {
    pub fn analyze_reachables<F>(self, update: F) -> Self
    where
        F: FnMut(&mut T, &'s Vec<NodeSummary>) -> ControlFlow<()>,
    {
        self.analyze_with(
            |seed| {
                seed.spoiler
                    .groups
                    .iter()
                    .flat_map(|group| group.reachable.iter().enumerate())
            },
            update,
        )
    }

    pub fn analyze_placements<F>(self, update: F) -> Self
    where
        F: FnMut(&mut T, &'s SpoilerPlacement) -> ControlFlow<()>,
    {
        self.analyze_with(
            |seed| {
                seed.placements()
                    .map(|placement| (placement.target_world_index, placement))
            },
            update,
        )
    }

    fn analyze_with<FU, FI, I, S>(mut self, inspect: FI, mut update: FU) -> Self
    where
        FU: FnMut(&mut T, S) -> ControlFlow<()>,
        FI: FnOnce(&'s SeedAnalyzer) -> I,
        I: IntoIterator<Item = (usize, S)>,
    {
        self.reset_flow();

        let mut running = self.stats.len();

        for (world_index, s) in inspect(self.inner) {
            let (stat, flow) = &mut self.stats[world_index];

            if flow.is_break() {
                continue;
            }

            if let ControlFlow::Break(()) = update(stat, s) {
                *flow = ControlFlow::Break(());

                running -= 1;
                if running == 0 {
                    break;
                }
            }
        }

        self
    }

    fn reset_flow(&mut self) {
        for (_, flow) in &mut self.stats {
            *flow = ControlFlow::Continue(());
        }
    }

    pub fn finish(self) -> impl Iterator<Item = T> {
        self.stats.into_iter().map(|(stat, _)| stat)
    }
}

pub fn placement_items(placement: &SpoilerPlacement) -> impl Iterator<Item = CommonItem> + use<'_> {
    placement
        .item
        .command
        .direct_contained_uber_state_writes(&[])
        .common_items()
}

pub fn compare_area(a: &str, b: &str) -> Ordering {
    fn index(area: &str) -> u8 {
        Area::from_str(area).map_or(const { Area::VARIANTS.len() as u8 }, |area| area as u8)
    }

    Ord::cmp(&index(a), &index(b))
}

pub fn compare_location(a: &str, b: &str) -> Ordering {
    match (a.split_once('.'), b.split_once('.')) {
        (None, None) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some((a_area, a_remaining)), Some((b_area, b_remaining))) => {
            compare_area(&a_area, &b_area).then_with(|| a_remaining.cmp(b_remaining))
        }
    }
}

pub trait Keys<T> {
    fn keys(self) -> Vec<String>;
}

impl<I, T> Keys<T> for I
where
    I: IntoIterator<Item = T>,
    T: Display,
{
    fn keys(self) -> Vec<String> {
        self.into_iter().map(|t| t.to_string()).collect()
    }
}

pub trait GroupKeys<T> {
    fn group_keys(self, bucket_size: T) -> Vec<String>;
}

impl<I, T> GroupKeys<T> for I
where
    I: IntoIterator<Item = T>,
    T: Default
        + From<u8>
        + Clone
        + Copy
        + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Rem<T, Output = T>
        + Display,
{
    fn group_keys(self, bucket_size: T) -> Vec<String> {
        self.into_iter()
            .map(move |result| bucket_key(result, bucket_size))
            .collect()
    }
}

fn bucket_key<T>(result: T, bucket_size: T) -> String
where
    T: Default
        + From<u8>
        + Clone
        + Copy
        + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Rem<T, Output = T>
        + Display,
{
    if bucket_size == T::from(1) {
        return result.to_string();
    }

    let floor = result - result % bucket_size;
    let ceiling = floor + bucket_size - T::from(1);
    format!("{floor}-{ceiling}")
}
