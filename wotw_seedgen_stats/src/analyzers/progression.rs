use super::Analyzer;
use wotw_seedgen::spoiler::SeedSpoiler;

pub struct ProgressionStats;
/// Analyzes which progressions get placed
impl Analyzer for ProgressionStats {
    fn title(&self) -> String {
        "Progression items".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| &group.forced_items)
            .map(|item| item.name.to_string())
            .collect()
    }
}
