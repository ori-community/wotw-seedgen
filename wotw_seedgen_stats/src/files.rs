use crate::Result;
use wotw_seedgen::{settings::UniverseSettings, spoiler::SeedSpoiler};

// TODO other access traits took a &self reference everywhere, makes it more flexible
/// Access seed files across stats runs
///
/// When generating stats multiple times with the same settings, seeds generated for previous runs can be reused  
/// These trait methods will be used to store and reuse seeds across stats runs
pub trait SeedStorageAccess {
    type Iter: Iterator<Item = Result<SeedSpoiler>>;

    /// fetch seeds that have been previously generated with these settings
    fn read_seeds(settings: &UniverseSettings, limit: usize) -> Result<Self::Iter>;
    /// write a seed generated from these settings for later use
    ///
    /// `key` should be unique, although it is recommended you don't rely on this being true and take it as a hint for what key you could use
    fn write_seed(seed: &SeedSpoiler, settings: &UniverseSettings, key: usize) -> Result<()>;
    /// clean all seeds that have previously been generated with these settings
    fn clean_seeds(settings: &UniverseSettings) -> Result<()>;
    /// clean all seeds that have previously been generated
    fn clean_all_seeds() -> Result<()>;
}
