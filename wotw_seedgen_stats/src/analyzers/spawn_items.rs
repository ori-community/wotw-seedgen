use super::Analyzer;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes the spawn items
pub struct SpawnItemStats;
impl Analyzer for SpawnItemStats {
    fn title(&self) -> String {
        "Spawn Items".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.identifier == "Spawn")
            .map(|placement| placement.item.name.clone())
            .collect()
    }
}
