use std::cmp::Ordering;

use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

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

    fn compare_keys(&self) -> fn(&String, &String) -> Ordering {
        super::compare_fixed_order::<super::RegionFixedOrder>
    }
}
