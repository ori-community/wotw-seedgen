use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::{data::Area, spoiler::SeedSpoiler};

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

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let groups_until_unlocked = seed.groups.iter().take_while(|group| {
            !group
                .reachable
                .iter()
                .flatten()
                .any(|node| node.area == self.area)
        });
        let reachable_count = groups_until_unlocked
            .flat_map(|group| group.reachable.iter().map(Vec::len))
            .sum();

        vec![super::group_result(
            reachable_count,
            self.result_bucket_size,
        )]
    }
}
