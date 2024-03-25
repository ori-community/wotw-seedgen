use super::Analyzer;
use std::cmp::Ordering;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes the spawn region
pub struct SpawnRegionStats;
impl Analyzer for SpawnRegionStats {
    fn title(&self) -> String {
        "Spawn Region".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.spawns
            .iter()
            .map(|spawn| spawn.split('.').next().unwrap().to_string())
            .collect()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_fixed_order::<super::RegionFixedOrder>
    }
}
