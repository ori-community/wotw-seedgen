use super::Analyzer;
use wotw_seedgen::{data::Skill, spoiler::SeedSpoiler, CommonItem};

/// Analyzes which weapon gets placed first
pub struct FirstWeaponStats;
impl Analyzer for FirstWeaponStats {
    fn title(&self) -> String {
        "First Weapon".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .find_map(|group| {
                group
                    .placements
                    .iter()
                    .find(|placement| {
                        CommonItem::from_command(&placement.command)
                            .into_iter()
                            .any(|item| {
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
                    .map(|placement| placement.item_name.clone())
            })
            .into_iter()
            .collect()
    }
}
