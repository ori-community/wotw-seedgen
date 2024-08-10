use super::Analyzer;
use rustc_hash::FxHashSet;
use wotw_seedgen::{data::Skill, spoiler::SeedSpoiler, CommonItem};

/// Analyzes how many skills were placed early on
pub struct EarlySkillsStats {
    pub reachable_limit: usize,
}
impl Analyzer for EarlySkillsStats {
    fn title(&self) -> String {
        format!("Skills within {} reachables", self.reachable_limit)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        let mut relevant_groups = 0;
        let first_reachables = seed
            .groups
            .iter()
            .enumerate()
            .flat_map(|(index, group)| {
                relevant_groups = usize::max(relevant_groups, index);
                group.reachable.iter().flatten()
            })
            .take(self.reachable_limit)
            .map(|node| &node.identifier)
            .collect::<FxHashSet<_>>();

        let mut iter = seed.groups.iter();
        let last = iter
            .next_back()
            .into_iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| first_reachables.contains(&&placement.location.identifier));
        let early_skills = iter
            .take(relevant_groups.saturating_sub(1))
            .flat_map(|group| group.placements.iter())
            .chain(last)
            .flat_map(|placement| CommonItem::from_command(&placement.command))
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
