use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes how big the steps of progression are
pub struct StepSizeStats {
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}

impl Analyzer for StepSizeStats {
    fn title(&self) -> String {
        "Size of progression steps".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .map(|group| {
                super::group_result(
                    group
                        .reachable
                        .iter()
                        .map(|reachable| reachable.len())
                        .sum(),
                    self.result_bucket_size,
                )
            })
            .collect()
    }
}
