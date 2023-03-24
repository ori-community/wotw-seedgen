use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

/// Analyzes how late an item is placed
pub struct ItemUnlockStats {
    pub item: String,
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
                    .any(|placement| placement.item_name == self.item)
            })
            .flat_map(|group| group.reachable.iter().map(|reachable| reachable.len()))
            .sum::<usize>();

        vec![reachable_until_unlocked.to_string()]
    }
}
