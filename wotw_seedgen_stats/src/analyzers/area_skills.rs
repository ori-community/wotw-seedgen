use super::Analyzer;
use wotw_seedgen::{
    data::{
        seed_language::output::{CommonItem, ContainedWrites, ContainedWritesExt},
        Area,
    },
    spoiler::SeedSpoiler,
};

/// Analyzes how many skills are in an area
pub struct AreaSkillsStats {
    pub area: Area,
}

impl Analyzer for AreaSkillsStats {
    fn title(&self) -> String {
        format!("Skills in {}", self.area)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let skills = seed
            .groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.location.area == self.area)
            .flat_map(|placement| {
                placement
                    .item
                    .command
                    .direct_contained_uber_state_writes(&[])
                    .common_items()
            })
            .filter(|item| matches!(item, CommonItem::Skill(_)))
            .count();

        vec![skills.to_string()]
    }
}
