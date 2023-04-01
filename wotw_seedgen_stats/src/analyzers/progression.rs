use std::iter;

use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

pub struct ProgressionStats;
/// Analyzes which items get placed as forced progression
impl Analyzer for ProgressionStats {
    fn title(&self) -> String {
        "Progression items".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.forced_items.items.iter())
            .flat_map(|(item, amount)| iter::repeat(item.to_string()).take(*amount as usize))
            .collect()
    }
}
