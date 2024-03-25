use super::Analyzer;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes the spawn items
pub struct SpawnItemCountStats;
impl Analyzer for SpawnItemCountStats {
    fn title(&self) -> String {
        "Spawn Item Count".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let count = seed
            .groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.identifier == "Spawn")
            .count();

        vec![count.to_string()]
    }
}
