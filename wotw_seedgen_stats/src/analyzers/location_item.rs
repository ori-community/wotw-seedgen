use super::Analyzer;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes what item gets placed on a location
pub struct LocationItemStats {
    pub location: String,
}
impl Analyzer for LocationItemStats {
    fn title(&self) -> String {
        format!("Item placed at {}", self.location)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .find(|placement| placement.location.identifier == self.location)
            .map(|placement| placement.item.name.clone())
            .into_iter()
            .collect()
    }
}
