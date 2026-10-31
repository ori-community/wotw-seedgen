use std::cmp::Ordering;

use crate::analyzers::{Analyzer, SeedAnalyzer};

/// Analyzes the spawn area
pub struct SpawnAreaStats;
impl Analyzer for SpawnAreaStats {
    fn title(&self) -> String {
        "Spawn Area".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.spoiler
            .spawns
            .iter()
            .map(|spawn| spawn.split('.').next().unwrap().to_string())
            .collect()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_area
    }
}
