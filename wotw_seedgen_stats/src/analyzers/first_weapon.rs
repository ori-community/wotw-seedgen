use super::Analyzer;
use wotw_seedgen::{
    data::Skill,
    seed_language::output::{CommonItem, ContainedWrites},
    spoiler::SeedSpoiler,
};

/// Analyzes which weapon gets placed first
pub struct FirstWeaponStats;
impl Analyzer for FirstWeaponStats {
    fn title(&self) -> String {
        "First Weapon".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| &group.placements)
            .find(|placement| {
                placement.item.command.contained_common_items().any(|item| {
                    matches!(
                        item,
                        CommonItem::Skill(
                            Skill::Grenade
                                | Skill::Spear
                                | Skill::Bow
                                | Skill::Hammer
                                | Skill::Sword
                                | Skill::Shuriken
                                | Skill::Blaze
                                | Skill::Sentry
                        )
                    )
                })
            })
            .map(|placement| placement.item.name.clone())
            .into_iter()
            .collect()
    }
}
