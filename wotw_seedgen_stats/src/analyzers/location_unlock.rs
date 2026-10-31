use std::{ num::NonZeroUsize, ops::ControlFlow};

use crate::analyzers::{Analyzer, GroupKeys, SeedAnalyzer};

/// Analyzes how late a location is unlocked
pub struct LocationUnlockStats {
    pub location: String,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for LocationUnlockStats {
    fn title(&self) -> String {
        format!("Reachables on {} unlock", self.location)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer()
            .analyze_reachables(|reachable_count, reachable| {
                if reachable
                    .iter()
                    .any(|node| node.identifier == self.location)
                {
                    ControlFlow::Break(())
                } else {
                    *reachable_count += reachable.len();
                    ControlFlow::Continue(())
                }
            })
            .finish()
            .group_keys(self.result_bucket_size.get())
    }
}
