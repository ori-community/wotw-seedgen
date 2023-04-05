use std::num::NonZeroUsize;

use wotw_seedgen::{generator::SeedSpoiler, util::Zone, Item};

use super::Analyzer;

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
            .filter(|placement| {
                placement
                    .location
                    .zone
                    .map_or(false, |zone| zone == self.zone)
            })
            .filter_map(|placement| match &placement.item {
                Item::SpiritLight(amount) => Some(amount),
                _ => None,
            })
            .sum::<u32>() as usize;

        vec![super::group_result(spirit_light, self.result_bucket_size)]
    }
}
