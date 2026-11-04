use std::{num::NonZeroUsize, ops::ControlFlow};

use crate::analyzers::{Analyzer, GroupKeys, SeedAnalyzer};

/// Analyzes how late an item is placed
pub struct ItemUnlockStats {
    pub item: String,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for ItemUnlockStats {
    fn title(&self) -> String {
        format!("Reachables on {} unlock", self.item)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer_with((Vec::with_capacity(1), 0))
            .analyze_placements(|(locations, _), placement| {
                if placement.item.name == self.item {
                    locations.push(&placement.location.identifier);
                }

                ControlFlow::Continue(())
            })
            .analyze_reachables(|(location, reachable_count), reachable| {
                if location
                    .iter()
                    .any(|&location| reachable.iter().any(|node| &node.identifier == location))
                {
                    ControlFlow::Break(())
                } else {
                    *reachable_count += reachable.len();
                    ControlFlow::Continue(())
                }
            })
            .finish_filter_break()
            .map(|(_, reachable_count)| reachable_count)
            .group_keys(self.result_bucket_size.get())
    }
}
