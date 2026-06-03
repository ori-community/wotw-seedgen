use super::Analyzer;
use std::{cmp::Ordering, num::NonZeroUsize};
use wotw_seedgen::{
    data::seed_language::output::{CommonItem, ContainedWrites},
    spoiler::SeedSpoiler,
};

/// Analyzes the total spirit light
pub struct TotalSpiritLightStats {
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for TotalSpiritLightStats {
    fn title(&self) -> String {
        "Total Spirit Light".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        // TODO preplacements seem to be ignored elsewhere? maybe we want a helper function for contained placements?
        let total_spirit_light = seed
            .preplacements
            .iter()
            .chain(seed.groups.iter().flat_map(|group| group.placements.iter()))
            .flat_map(|placement| placement.item.command.contained_common_items())
            .filter_map(|common_item| match common_item {
                CommonItem::SpiritLight(amount) => Some(amount),
                _ => None,
            })
            .sum::<i32>();

        vec![super::group_result(
            total_spirit_light as usize,
            self.result_bucket_size,
        )]
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_location
    }
}
