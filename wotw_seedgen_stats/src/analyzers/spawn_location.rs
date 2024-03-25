use super::Analyzer;
use std::cmp::Ordering;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes the spawn location
pub struct SpawnLocationStats;
impl Analyzer for SpawnLocationStats {
    fn title(&self) -> String {
        "Spawn Location".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.spawns.clone()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_location
    }
}
