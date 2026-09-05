use std::sync::LazyLock;

use tokio::sync::RwLockReadGuard;
use wotw_seedgen::data::{
    UniverseSettings,
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
    let mut context = InlineContext::new(cache, universe_settings);
    context.check_snippets();
    context.finish()
}

struct InlineContext<'c, 'a, 's> {
    cache: &'c RwLockReadGuard<'a, Cache>,
    universe_settings: &'s mut UniverseSettings,
    includes: Vec<&'c str>,
    failed: Vec<String>,
}

impl<'c, 'a, 's> InlineContext<'c, 'a, 's> {
    fn new(
        cache: &'c RwLockReadGuard<'a, Cache>,
        universe_settings: &'s mut UniverseSettings,
    ) -> Self {
        Self {
            cache,
            universe_settings,
            includes: Vec::new(),
            failed: Vec::new(),
        }
    }

    fn check_snippets(&mut self) {
        for world_settings in &self.universe_settings.world_settings {
            for identifier in &world_settings.snippets {
                inline_snippet(
                    self.cache,
                    identifier,
                    &mut self.universe_settings.inline_snippets,
                    &mut self.includes,
                    &mut self.failed,
                );
            }

            while let Some(identifier) = self.includes.pop() {
                inline_snippet(
                    self.cache,
                    identifier,
                    &mut self.universe_settings.inline_snippets,
                    &mut self.includes,
                    &mut self.failed,
                );
            }
        }
    }

    fn finish(self) -> Result<()> {
        if self.failed.is_empty() {
            Ok(())
        } else {
            Err(Error::InlineSnippets(self.failed))
        }
    }
}

fn inline_snippet<'c>(
    cache: &'c RwLockReadGuard<Cache>,
    identifier: &str,
    inline_snippets: &mut InlineSnippets,
    includes: &mut Vec<&'c str>,
    failed: &mut Vec<String>,
) {
    if inline_snippets.contains_key(identifier) {
        return;
    };

    let snippet_info = &cache.snippet_info[identifier];

    if !*ALWAYS_INLINE_SNIPPETS && matches!(snippet_info.origin, AssetOrigin::ExecutableDir) {
        return;
    }

    if snippet_info.metadata.requires_local_files {
        failed.push(identifier.to_string());
    } else {
        let mut snippet = cache.read_snippet(identifier).unwrap();
        snippet.id = format!("inlined: {}", snippet.id);
        inline_snippets.insert(identifier.to_string(), snippet);
    }

    includes.extend(snippet_info.metadata.includes.iter().map(String::as_str));
}
