use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::spoiler::SeedSpoiler;

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

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let reachable_until_unlocked = seed
            .groups
            .iter()
            .take_while(|group| {
                !group
                    .placements
                    .iter()
                    .any(|placement| placement.location.identifier == self.location)
            })
            .flat_map(|group| group.reachable.iter().map(Vec::len))
            .sum::<usize>();

        vec![super::group_result(
            reachable_until_unlocked,
            self.result_bucket_size,
        )]
    }
}
