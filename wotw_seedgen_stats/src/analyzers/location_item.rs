use crate::analyzers::{Analyzer, SeedAnalyzer};

/// Analyzes what item gets placed on a location
pub struct LocationItemStats {
    pub location: String,
}

impl Analyzer for LocationItemStats {
    fn title(&self) -> String {
        format!("Item placed at {}", self.location)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.placements()
            .filter(|placement| placement.location.identifier == self.location)
            .map(|placement| placement.item.name.to_string())
            .collect()
    }
}
