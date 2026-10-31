use crate::analyzers::{Analyzer, SeedAnalyzer};

/// Analyzes the spawn items
pub struct SpawnItemStats;
impl Analyzer for SpawnItemStats {
    fn title(&self) -> String {
        "Spawn Items".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.placements()
            .filter(|placement| placement.location.identifier == "Spawn")
            .map(|placement| placement.item.name.to_string())
            .collect()
    }
}
