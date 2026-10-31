use std::{ cmp::Ordering};

use crate::analyzers::{Analyzer, SeedAnalyzer};

/// Analyzes what location an item get placed on
pub struct ItemLocationStats {
    pub item: String,
}

impl Analyzer for ItemLocationStats {
    fn title(&self) -> String {
        format!("Location of {}", self.item)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.placements()
            .filter(|placement| placement.item.name == self.item)
            .map(|placement| placement.location.identifier.to_string())
            .collect()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_location
    }
}
