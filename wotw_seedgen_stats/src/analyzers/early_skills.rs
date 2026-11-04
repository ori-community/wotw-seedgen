use std::ops::ControlFlow;

use rustc_hash::{FxBuildHasher, FxHashSet};
use wotw_seedgen::data::Area;

use crate::analyzers::{placement_items, Analyzer, Keys, SeedAnalyzer};

/// Analyzes how many skills were placed early on
pub struct EarlySkillsStats {
    pub reachable_limit: usize,
}

impl Analyzer for EarlySkillsStats {
    fn title(&self) -> String {
        format!("Skills within {} reachables", self.reachable_limit)
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer_with((
            FxHashSet::with_capacity_and_hasher(self.reachable_limit, FxBuildHasher),
            0,
        ))
        .analyze_reachables(|(early_locations, _), reachable| {
            for node in reachable {
                early_locations.insert(&node.identifier);

                if early_locations.len() == self.reachable_limit {
                    return ControlFlow::Break(());
                }
            }

            ControlFlow::Continue(())
        })
        .reset_flow()
        .analyze_placements(|(early_locations, early_skills), placement| {
            if matches!(placement.location.area, Area::Spawn)
                || early_locations.contains(&placement.location.identifier)
            {
                *early_skills += placement_items(placement)
                    .filter(|item| item.is_skill_hinted())
                    .count()
            }

            ControlFlow::Continue(())
        })
        .finish()
        .map(|(_, early_skills)| early_skills)
        .keys()
    }
}
