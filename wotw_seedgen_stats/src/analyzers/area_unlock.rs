use std::{num::NonZeroUsize, ops::ControlFlow};

use wotw_seedgen::data::Area;

use crate::analyzers::{Analyzer, GroupKeys, SeedAnalyzer};

/// Analyzes when areas unlock
///
/// An area is considered unlocked once the first item within that area is unlocked
pub struct AreaUnlockStats {
    pub area: Area,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for AreaUnlockStats {
    fn title(&self) -> String {
        format!("Reachables on {} unlock", self.area)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer()
            .analyze_reachables(|reachable_count, reachable| {
                if reachable.iter().any(|node| node.area == self.area) {
                    ControlFlow::Break(())
                } else {
                    *reachable_count += reachable.len();
                    ControlFlow::Continue(())
                }
            })
            .finish_filter_break()
            .group_keys(self.result_bucket_size.get())
    }
}
