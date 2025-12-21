use tokio::sync::RwLockReadGuard;
use wotw_seedgen::data::{UniverseSettings, WorldSettings, assets::SnippetAccess};

use crate::assets::Cache;

pub fn inline_universe_snippets(
    universe_settings: &mut UniverseSettings,
    cache: &RwLockReadGuard<Cache>,
) {
    for world_settings in &mut universe_settings.world_settings {
        inline_world_snippets(world_settings, cache);
    }
}

pub fn inline_world_snippets(world_settings: &mut WorldSettings, cache: &RwLockReadGuard<Cache>) {
    world_settings.snippets.retain(|identifier| {
        let is_in_data_dir = cache.data_dir_snippets.contains(identifier);

        if is_in_data_dir {
            world_settings
                .inline_snippets
                .insert(identifier.clone(), cache.read_snippet(identifier).unwrap());
        }

        !is_in_data_dir
    });
}
