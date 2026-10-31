use std::ops::ControlFlow;

use wotw_seedgen::data::{seed_language::output::CommonItem, Skill};

use crate::analyzers::{placement_items, Analyzer, SeedAnalyzer};

/// Analyzes which weapon gets placed first
pub struct FirstWeaponStats;
impl Analyzer for FirstWeaponStats {
    fn title(&self) -> String {
        "First Weapon".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.world_analyzer()
            .analyze_placements(|first_weapon, placement| {
                if placement_items(placement).any(|item| {
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
                                | Skill::Sentry,
                        )
                    )
                }) {
                    *first_weapon = Some(placement.item.name.to_string());
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            })
            .finish()
            .flatten()
            .collect()
    }
}
