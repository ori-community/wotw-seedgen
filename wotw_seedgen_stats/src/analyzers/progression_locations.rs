use super::Analyzer;
use wotw_seedgen::spoiler::SeedSpoiler;

pub struct ProgressionLocationsStats;
/// Analyzes what locations progression gets placed on
impl Analyzer for ProgressionLocationsStats {
    fn title(&self) -> String {
        "Progression locations".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.groups
            .iter()
            .flat_map(|group| {
                let mut forced_items = group
                    .forced_items
                    .iter()
                    .map(|item| &item.name)
                    .collect::<Vec<_>>();

                group.placements.iter().filter_map(move |placement| {
                    let index = forced_items
                        .iter()
                        .position(|item| *item == &placement.item.name)?;
                    forced_items.swap_remove(index);

                    Some(placement.location.identifier.clone())
                })
            })
            .collect()
    }
}
