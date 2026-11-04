use std::ops::ControlFlow;

use wotw_seedgen::data::Area;

use crate::analyzers::{Analyzer, Keys, SeedAnalyzer};

/// Analyzes the amount of spawn items
pub struct SpawnItemCountStats;
impl Analyzer for SpawnItemCountStats {
    fn title(&self) -> String {
        "Spawn Item Count".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer::<usize>()
            .analyze_placements(|spawn_item_count, placement| {
                if matches!(placement.location.area, Area::Spawn) {
                    *spawn_item_count += 1;
                }

                ControlFlow::Continue(())
            })
            .finish()
            .keys()
    }
}
