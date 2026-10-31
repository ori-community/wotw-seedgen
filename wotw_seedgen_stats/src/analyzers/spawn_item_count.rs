use std::{ ops::ControlFlow};

use crate::analyzers::{Analyzer, Keys, SeedAnalyzer};

/// Analyzes the spawn items
pub struct SpawnItemCountStats;
impl Analyzer for SpawnItemCountStats {
    fn title(&self) -> String {
        "Spawn Item Count".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer::<usize>()
            .analyze_placements(|spawn_item_count, placement| {
                if placement.location.identifier == "Spawn" {
                    *spawn_item_count += 1;
                }

                ControlFlow::Continue(())
            })
            .finish()
            .keys()
    }
}
