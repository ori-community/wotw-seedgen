use std::{mem, sync::LazyLock};

use tokio::sync::RwLockReadGuard;
use wotw_seedgen::data::{
    UniverseSettings, WorldSettings,
    assets::{InlineSnippets, SnippetAccess},
    env_or,
};

use crate::{api::assets::AssetOrigin, assets::Cache};

static ALWAYS_INLINE_SNIPPETS: LazyLock<bool> =
    LazyLock::new(|| env_or("ALWAYS_INLINE_SNIPPETS", false));

pub fn inline_universe_snippets(
    universe_settings: &mut UniverseSettings,
    cache: &RwLockReadGuard<Cache>,
) {
    for world_settings in &mut universe_settings.world_settings {
        inline_world_snippets(world_settings, cache);
    }
}

pub fn inline_world_snippets(world_settings: &mut WorldSettings, cache: &RwLockReadGuard<Cache>) {
    if *ALWAYS_INLINE_SNIPPETS {
        for identifier in world_settings.snippets.drain(..) {
            inline_snippet(&mut world_settings.inline_snippets, cache, identifier);
        }
    } else {
        world_settings.snippets.retain_mut(|identifier| {
            match cache.snippet_info[identifier].origin {
                AssetOrigin::ExecutableDir => true,
                AssetOrigin::UserDataDir { .. } => {
                    let identifier = mem::take(identifier);
                    inline_snippet(&mut world_settings.inline_snippets, cache, identifier);
                    false
                }
            }
        });
    }
}

fn inline_snippet(
    inline_snippets: &mut InlineSnippets,
    cache: &RwLockReadGuard<Cache>,
    identifier: String,
) {
    let snippet = cache.read_snippet(&identifier).unwrap();
    inline_snippets.insert(identifier, snippet);
}
