use std::ops::ControlFlow;

use wotw_seedgen::data::Area;

use crate::analyzers::{placement_items, Analyzer, Keys, SeedAnalyzer};

/// Analyzes how many skills are in an area
pub struct AreaSkillsStats {
    pub area: Area,
}

impl Analyzer for AreaSkillsStats {
    fn title(&self) -> String {
        format!("Skills in {}", self.area)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer::<usize>()
            .analyze_placements(|skills, placement| {
                if placement.location.area == self.area {
                    *skills += placement_items(placement)
                        .filter(|item| item.is_skill_hinted())
                        .count();
                }

                ControlFlow::Continue(())
            })
            .finish()
            .keys()
    }
}
