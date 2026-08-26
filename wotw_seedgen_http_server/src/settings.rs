use std::{mem, sync::LazyLock};

use tokio::sync::RwLockReadGuard;
use wotw_seedgen::data::{UniverseSettings, WorldSettings, assets::SnippetAccess, env_or};

use crate::{
    api::{assets::AssetOrigin, snippets::SnippetInfo},
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
    let mut context = InlineContext::new(cache, world_settings);

    if *ALWAYS_INLINE_SNIPPETS {
        context.inline_all_world_snippets();
    } else {
        context.check_world_snippets();
    }

    context.finish()
}

struct InlineContext<'c, 'a, 's> {
    cache: &'c RwLockReadGuard<'a, Cache>,
    world_settings: &'s mut WorldSettings,
    includes: Vec<String>,
    failed: Vec<String>,
}

impl<'c, 'a, 's> InlineContext<'c, 'a, 's> {
    fn new(cache: &'c RwLockReadGuard<'a, Cache>, world_settings: &'s mut WorldSettings) -> Self {
        Self {
            cache,
            world_settings,
            includes: Vec::new(),
            failed: Vec::new(),
        }
    }

    fn inline_all_world_snippets(&mut self) {
        self.includes = mem::take(&mut self.world_settings.snippets);

        while let Some(identifier) = self.includes.pop() {
            let snippet_info = &self.cache.snippet_info[&identifier];

            self.inline_snippet(snippet_info, identifier);
        }
    }

    fn check_world_snippets(&mut self) {
        let mut snippets = mem::take(&mut self.world_settings.snippets);
        snippets.retain_mut(|identifier| self.check_snippet(identifier));
        self.world_settings.snippets = snippets;

        while let Some(mut identifier) = self.includes.pop() {
            self.check_snippet(&mut identifier);
        }
    }

    fn check_snippet(&mut self, identifier: &mut String) -> bool {
        let snippet_info = &self.cache.snippet_info[identifier];

        match snippet_info.origin {
            AssetOrigin::ExecutableDir => true,
            AssetOrigin::UserDataDir { .. } => {
                self.inline_snippet(snippet_info, mem::take(identifier));
                false
            }
        }
    }

    fn inline_snippet(&mut self, snippet_info: &SnippetInfo, identifier: String) {
        if snippet_info.metadata.requires_local_files {
            self.failed.push(identifier);
        } else {
            let snippet = self.cache.read_snippet(&identifier).unwrap();
            self.world_settings
                .inline_snippets
                .insert(identifier, snippet);
        }

        self.includes
            .extend(snippet_info.metadata.includes.iter().cloned());
    }

    fn finish(self) -> Result<()> {
        if self.failed.is_empty() {
            Ok(())
        } else {
            Err(Error::InlineSnippets(self.failed))
        }
    }
}
