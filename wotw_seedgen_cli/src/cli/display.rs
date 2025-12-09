use std::{
    fmt::{self, Display},
    sync::LazyLock,
};

use clap::builder::styling::Reset;
use wotw_seedgen_assets::{DefaultFileAccess, PresetAccess, PresetInfo, SnippetAccess};
use wotw_seedgen_seed_language::metadata::Metadata;

use crate::cli::LITERAL;

pub static AVAILABLE_UNIVERSE_PRESETS: LazyLock<Vec<AvailablePreset>> =
    LazyLock::new(|| AvailablePreset::all_universe());

pub static AVAILABLE_WORLD_PRESETS: LazyLock<Vec<AvailablePreset>> =
    LazyLock::new(|| AvailablePreset::all_world());

pub static AVAILABLE_SNIPPETS: LazyLock<Vec<AvailableSnippet>> =
    LazyLock::new(|| AvailableSnippet::all());

#[derive(Debug, Clone)]
pub struct AvailablePreset {
    pub identifier: String,
    pub info: Result<Option<PresetInfo>, String>,
}

impl AvailablePreset {
    pub fn all_universe() -> Vec<Self> {
        DefaultFileAccess
            .available_universe_presets()
            .into_iter()
            .map(Self::new_universe)
            .collect()
    }

    pub fn all_world() -> Vec<Self> {
        DefaultFileAccess
            .available_world_presets()
            .into_iter()
            .map(Self::new_world)
            .collect()
    }

    pub fn new_universe(identifier: String) -> Self {
        let info = DefaultFileAccess
            .universe_preset(&identifier)
            .map(|preset| preset.info);

        Self { identifier, info }
    }

    pub fn new_world(identifier: String) -> Self {
        let info = DefaultFileAccess
            .world_preset(&identifier)
            .map(|preset| preset.info);

        Self { identifier, info }
    }
}

impl Display for AvailablePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{LITERAL}{identifier}{Reset}: ",
            identifier = self.identifier,
        )?;

        match &self.info {
            Ok(info) => match info {
                None => write!(f, "(no details provided by preset)"),
                Some(info) => match &info.description {
                    None => write!(f, "(no description provided by preset)"),
                    Some(description) => description.fmt(f),
                },
            },
            Err(err) => write!(f, "failed to read details: {err}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvailableSnippet {
    pub identifier: String,
    pub metadata: Metadata,
}

impl Display for AvailableSnippet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{LITERAL}{identifier}{Reset}: {description}",
            identifier = self.identifier,
            description = self
                .metadata
                .description
                .as_deref()
                .unwrap_or("(no description provided by snippet)")
        )
    }
}

impl AvailableSnippet {
    pub fn all() -> Vec<Self> {
        let mut available_snippets = DefaultFileAccess
            .available_snippets()
            .into_iter()
            .map(Self::new)
            .filter(|available_snippet| !available_snippet.metadata.hidden)
            .collect::<Vec<_>>();

        available_snippets.sort_unstable_by(|a, b| {
            a.metadata
                .category
                .cmp(&b.metadata.category)
                .then_with(|| a.identifier.cmp(&b.identifier))
        });

        available_snippets
    }

    pub fn new(identifier: String) -> Self {
        let metadata = DefaultFileAccess
            .read_snippet(&identifier)
            .map(|source| Metadata::from_source(&source.content))
            .unwrap_or_default();

        Self {
            identifier,
            metadata,
        }
    }
}
