use crate::analyzers::{Analyzer, SeedAnalyzer};

pub struct ProgressionItemsStats;

/// Analyzes which items get placed as progression
impl Analyzer for ProgressionItemsStats {
    fn title(&self) -> String {
        "Progression items".to_string()
    }

    fn analyze(&self, seed: &SeedAnalyzer) -> Vec<String> {
        seed.spoiler
            .groups
            .iter()
            .flat_map(|group| &group.forced_items)
            .map(|item| item.name.to_string())
            .collect()
    }
}
