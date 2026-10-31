use std::cmp::Ordering;

use crate::analyzers::{Analyzer, Keys, SeedAnalyzer};

/// Analyzes what area an item gets placed in
pub struct ItemAreaStats {
    pub item: String,
}

impl Analyzer for ItemAreaStats {
    fn title(&self) -> String {
        format!("Area {} is placed in", self.item)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.placements()
            .filter(|placement| placement.item.name == self.item)
            .map(|placement| placement.location.area)
            .keys()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_area
    }
}
