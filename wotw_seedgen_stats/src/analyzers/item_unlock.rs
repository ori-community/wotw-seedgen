use super::Analyzer;
use std::num::NonZeroUsize;
use wotw_seedgen::spoiler::SeedSpoiler;

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

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let reachable_until_unlocked = seed
            .groups
            .iter()
            .take_while(|group| {
                !group
                    .placements
                    .iter()
                    .any(|placement| placement.item.name == self.item)
            })
            .flat_map(|group| group.reachable.iter().map(|reachable| reachable.len()))
            .sum::<usize>();

        vec![super::group_result(
            reachable_until_unlocked,
            self.result_bucket_size,
        )]
    }
}
