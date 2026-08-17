use std::{mem, sync::LazyLock};

use tokio::sync::RwLockReadGuard;
use wotw_seedgen::data::{
    UniverseSettings, WorldSettings,
    assets::{InlineSnippets, SnippetAccess},
    env_or,
};

use crate::{
    api::assets::AssetOrigin,
    assets::Cache,
    error::{Error, Result},
};

static ALWAYS_INLINE_SNIPPETS: LazyLock<bool> =
    LazyLock::new(|| env_or("ALWAYS_INLINE_SNIPPETS", false));

pub fn inline_universe_snippets(
    universe_settings: &mut UniverseSettings,
    cache: &RwLockReadGuard<Cache>,
) -> Result<()> {
    for world_settings in &mut universe_settings.world_settings {
        inline_world_snippets(world_settings, cache)?;
    }

    Ok(())
}

pub fn inline_world_snippets(
    world_settings: &mut WorldSettings,
    cache: &RwLockReadGuard<Cache>,
) -> Result<()> {
    let inline_snippets = &mut world_settings.inline_snippets;
    let mut failed = vec![];

    if *ALWAYS_INLINE_SNIPPETS {
        for identifier in world_settings.snippets.drain(..) {
            inline_snippet(inline_snippets, cache, identifier, &mut failed);
        }
    } else {
        world_settings.snippets.retain_mut(|identifier| {
            match cache.snippet_info[identifier].origin {
                AssetOrigin::ExecutableDir => true,
                AssetOrigin::UserDataDir { .. } => {
                    let identifier = mem::take(identifier);
                    inline_snippet(inline_snippets, cache, identifier, &mut failed);
                    false
                }
            }
        });
    }

    if failed.is_empty() {
        Ok(())
    } else {
        Err(Error::InlineSnippets(failed))
    }
}

fn inline_snippet(
    inline_snippets: &mut InlineSnippets,
    cache: &RwLockReadGuard<Cache>,
    identifier: String,
    failed: &mut Vec<String>,
) {
    if cache.snippet_info[&identifier].metadata.can_inline {
        let snippet = cache.read_snippet(&identifier).unwrap();
        inline_snippets.insert(identifier, snippet);
    } else {
        failed.push(identifier);
    }
}
