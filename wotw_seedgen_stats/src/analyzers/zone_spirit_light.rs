use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::{
    data::{
        seed_language::output::{CommonItem, ContainedWrites},
        Zone,
    },
    spoiler::SeedSpoiler,
};

/// Analyzes how much Spirit Light is in a zone
pub struct ZoneSpiritLightStats {
    pub zone: Zone,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for ZoneSpiritLightStats {
    fn title(&self) -> String {
        format!("Spirit Light in {}", self.zone)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let spirit_light = seed
            .groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.zone == Some(self.zone))
            .flat_map(|placement| placement.item.command.contained_common_items())
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
