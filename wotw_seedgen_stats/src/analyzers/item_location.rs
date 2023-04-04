use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

/// Analyzes what location an item get placed on
pub struct ItemLocationStats {
    pub item: String,
}
impl Analyzer for ItemLocationStats {
    fn title(&self) -> String {
        format!("Location of {}", self.item)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| &placement.item_name == &self.item)
            .map(|placement| placement.location.identifier.clone())
            .collect()
    }
}
