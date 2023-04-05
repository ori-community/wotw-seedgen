mod early_skills;
mod first_weapon;
mod item_location;
mod item_unlock;
mod item_zone;
mod location_item;
mod progression;
mod spawn_item_count;
mod spawn_items;
mod spawn_location;
mod spawn_region;
mod step_size;
mod zone_spirit_light;
mod zone_unlock;

pub use early_skills::EarlySkillsStats;
pub use first_weapon::FirstWeaponStats;
pub use item_location::ItemLocationStats;
pub use item_unlock::ItemUnlockStats;
pub use item_zone::ItemZoneStats;
pub use location_item::LocationItemStats;
pub use progression::ProgressionStats;
pub use spawn_item_count::SpawnItemCountStats;
pub use spawn_items::SpawnItemStats;
pub use spawn_location::SpawnLocationStats;
pub use spawn_region::SpawnRegionStats;
pub use step_size::StepSizeStats;
pub use zone_spirit_light::ZoneSpiritLightStats;
pub use zone_unlock::ZoneUnlockStats;

use std::num::NonZeroUsize;

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

fn group_result(result: usize, bucket_size: NonZeroUsize) -> String {
    let bucket_size = bucket_size.get();
    if bucket_size == 1 {
        return result.to_string();
    }

    let floor = result - result % bucket_size;
    let ceiling = floor + bucket_size - 1;
    format!("{floor}-{ceiling}")
}
