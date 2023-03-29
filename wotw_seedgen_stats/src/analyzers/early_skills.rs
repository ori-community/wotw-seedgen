use rustc_hash::FxHashSet;
use wotw_seedgen::{generator::SeedSpoiler, item::Skill, Item};

use super::Analyzer;

/// Analyzes how many skills were placed early on
pub struct EarlySkillsStats {
    pub reachable_limit: usize,
}
impl Analyzer for EarlySkillsStats {
    fn title(&self) -> String {
        format!("Skills within {} reachables", self.reachable_limit)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        #[inline]
        fn is_skill(item: &Item) -> bool {
            match item {
                Item::Skill(skill) => !matches!(
                    skill,
                    Skill::InkwaterAncestralLight | Skill::GladesAncestralLight
                ),
                Item::Water => true,
                _ => false,
            }
        }

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
            .filter(|placement| {
                is_skill(&placement.item)
                    && first_reachables.contains(&&placement.location.identifier)
            })
            .count();

        vec![early_skills.to_string()]
    }
}
