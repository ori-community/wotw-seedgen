use super::Analyzer;
use std::cmp::Ordering;
use wotw_seedgen::spoiler::SeedSpoiler;

/// Analyzes what area an item gets placed in
pub struct ItemAreaStats {
    pub item: String,
}

impl Analyzer for ItemAreaStats {
    fn title(&self) -> String {
        format!("Area {} is placed in", self.item)
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| group.placements.iter())
            .filter(|placement| placement.item.name == self.item)
            .map(|placement| placement.location.area.to_string())
            .collect()
    }

    fn compare_keys(&self) -> fn(&str, &str) -> Ordering {
        super::compare_fixed_order::<super::AreaFixedOrder>
    }
}
