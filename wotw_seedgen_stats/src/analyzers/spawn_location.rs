use std::cmp::Ordering;

use crate::analyzers::{Analyzer, SeedAnalyzer};

/// Analyzes the spawn location
pub struct SpawnLocationStats;
impl Analyzer for SpawnLocationStats {
    fn title(&self) -> String {
        "Spawn Location".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.spoiler.spawns.iter().map(String::clone).collect()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_location
    }
}
