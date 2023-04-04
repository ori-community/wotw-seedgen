use std::num::NonZeroUsize;

use wotw_seedgen::{generator::SeedSpoiler, util::Zone};

use super::Analyzer;

/// Analyzes when zones unlock
///
/// A zone is considered unlocked once the first item within that zone is unlocked
pub struct ZoneUnlockStats {
    pub zone: Zone,
    /// How many adjacent result to group together
    pub result_bucket_size: NonZeroUsize,
}
impl Analyzer for ZoneUnlockStats {
    fn title(&self) -> String {
        format!("Reachables on {} unlock", self.zone)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let groups_until_unlocked = seed.groups.iter().take_while(|group| {
            !group
                .reachable
                .iter()
                .flatten()
                .any(|node| node.zone.map_or(false, |zone| zone == self.zone))
        });
        let reachable_count = groups_until_unlocked
            .flat_map(|group| group.reachable.iter().map(|reachable| reachable.len()))
            .sum::<usize>();

        vec![super::group_result(
            reachable_count,
            self.result_bucket_size,
        )]
    }
}
