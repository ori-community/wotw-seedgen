use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

/// Analyzes the spawn items
pub struct SpawnItemStats;
impl Analyzer for SpawnItemStats {
    fn title(&self) -> String {
        "Spawn Items".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .take_while(|group| group.reachable.is_empty())
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.identifier == "Spawn")
            .map(|placement| placement.item_name.clone())
            .collect()
    }
}
