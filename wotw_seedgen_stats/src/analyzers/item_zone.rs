use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

/// Analyzes what zone an item gets placed in
pub struct ItemZoneStats {
    pub item: String,
}
impl Analyzer for ItemZoneStats {
    fn title(&self) -> String {
        format!("Zone {} is placed in", self.item)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| &placement.item_name == &self.item)
            .map(|placement| {
                placement
                    .location
                    .zone
                    .map_or("Unknown".to_string(), |zone| zone.to_string())
            })
            .collect()
    }
}
