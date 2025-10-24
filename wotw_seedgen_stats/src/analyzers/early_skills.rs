use super::Analyzer;
use rustc_hash::FxHashSet;
use wotw_seedgen::{data::Skill, spoiler::SeedSpoiler, CommonItem, ContainedWrites};

/// Analyzes how many skills were placed early on
pub struct EarlySkillsStats {
    pub reachable_limit: usize,
}

impl Analyzer for EarlySkillsStats {
    fn title(&self) -> String {
        format!("Skills within {} reachables", self.reachable_limit)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let first_reachables = seed
            .groups
            .iter()
            .flat_map(|group| group.reachable.iter().flatten())
            .take(self.reachable_limit)
            .map(|node| &node.identifier)
            .collect::<FxHashSet<_>>();

        let early_skills = seed
            .groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| first_reachables.contains(&&placement.location.identifier))
            .flat_map(|placement| placement.item.command.contained_common_items())
            .filter(|item| match item {
                CommonItem::Skill(Skill::GladesAncestralLight | Skill::MarshAncestralLight) => {
                    false
                }
                CommonItem::Skill(_) => true,
                CommonItem::CleanWater => true,
                _ => false,
            })
            .count();

        vec![early_skills.to_string()]
    }
}
