mod item_unlock;
mod spawn_items;
mod spawn_location;
mod spawn_region;
mod zone_unlock;

pub use item_unlock::ItemUnlockStats;
pub use spawn_items::SpawnItemStats;
pub use spawn_location::SpawnLocationStats;
pub use spawn_region::SpawnRegionStats;
pub use zone_unlock::ZoneUnlockStats;

use wotw_seedgen::generator::SeedSpoiler;

/// Trait for types that may analyze seeds and generate statistics
///
/// Check the types in this module for some provided implementations
pub trait Analyzer: Sync {
    /// A brief title describing what kind of statistic is analyzed
    fn title(&self) -> String;

    /// Analyze a given seed and return one or more keys that this seed should be categorized into
    ///
    /// For instance, [`SpawnLocationStats`] will return the name of the spawn locations here
    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String>;
}
