use std::{num::NonZeroUsize, ops::ControlFlow};

use wotw_seedgen::data::seed_language::output::CommonItem;

use crate::analyzers::{placement_items, Analyzer, GroupKeys, SeedAnalyzer};

/// Analyzes the total spirit light
pub struct TotalSpiritLightStats {
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for TotalSpiritLightStats {
    fn title(&self) -> String {
        "Total Spirit Light".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer()
            .analyze_placements(|spirit_light, placement| {
                *spirit_light += placement_items(placement)
                    .map(|item| match item {
                        CommonItem::SpiritLight(amount) => amount,
                        _ => 0,
                    })
                    .sum::<i32>();

                ControlFlow::Continue(())
            })
            .finish()
            .group_keys(self.result_bucket_size.get() as i32)
    }
}
