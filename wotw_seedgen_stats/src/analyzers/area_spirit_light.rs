use std::{num::NonZeroUsize, ops::ControlFlow};

use wotw_seedgen::data::{seed_language::output::CommonItem, Area};

use crate::analyzers::{placement_items, Analyzer, GroupKeys, SeedAnalyzer};

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

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer()
            .analyze_placements(|spirit_light, placement| {
                if placement.location.area == self.area {
                    *spirit_light += placement_items(placement)
                        .map(|item| match item {
                            CommonItem::SpiritLight(amount) => amount,
                            _ => 0,
                        })
                        .sum::<i32>();
                }

                ControlFlow::Continue(())
            })
            .finish()
            .group_keys(self.result_bucket_size.get() as i32)
    }
}
