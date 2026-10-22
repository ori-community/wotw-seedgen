use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::{
    data::{
        seed_language::output::{CommonItem, ContainedWrites, ContainedWritesExt},
        Area,
    },
    spoiler::SeedSpoiler,
};

/// Analyzes how much Spirit Light is in an area
pub struct AreaSpiritLightStats {
    pub area: Area,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for AreaSpiritLightStats {
    fn title(&self) -> String {
        format!("Spirit Light in {}", self.area)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let spirit_light = seed
            .groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.area == self.area)
            .flat_map(|placement| {
                placement
                    .item
                    .command
                    .direct_contained_uber_state_writes(&[])
                    .common_items()
            })
            .filter_map(|item| match item {
                CommonItem::SpiritLight(amount) => Some(amount),
                _ => None,
            })
            .sum::<i32>();

        vec![super::group_result(
            spirit_light as usize,
            self.result_bucket_size,
        )]
    }
}
