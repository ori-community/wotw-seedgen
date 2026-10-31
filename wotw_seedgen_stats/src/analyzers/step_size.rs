use std::num::NonZeroUsize;

use crate::analyzers::{Analyzer, Keys, SeedAnalyzer};

/// Analyzes how big the steps of progression are
pub struct StepSizeStats {
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for StepSizeStats {
    fn title(&self) -> String {
        "Size of progression steps".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.spoiler
            .groups
            .iter()
            .flat_map(|group| group.reachable.iter().map(Vec::len))
            .filter(|step_size| *step_size > 0)
            .keys()
    }
}
