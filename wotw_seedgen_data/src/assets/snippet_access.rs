use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};
use utoipa::ToSchema;
use wotw_seedgen_parse::Source;

use crate::seed_language::metadata::Metadata;

/// Resolves include commands in the seed language
pub trait SnippetAccess {
    /// Resolve `!include(<identifier>)`
    fn read_snippet(&self, identifier: &str) -> Result<Source, String>;

    /// Resolve binary includes such as `!include_icon(<path>)`
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String>;

    /// Return a `Vec` of identifiers which may be passed to [`SnippetAccess::read_snippet`]
    fn available_snippets(&self) -> Vec<String>;

    fn snippet_metadata(&self, identifier: &str) -> Result<Metadata, String> {
        self.read_snippet(identifier)
            .map(|source| Metadata::from_source(&source.content))
    }

    fn available_snippets_metadata(&self) -> Vec<(String, Metadata)> {
        self.available_snippets()
            .into_iter()
            .filter_map(|identifier| {
                self.snippet_metadata(&identifier)
                    .ok()
                    .map(|metadata| (identifier, metadata))
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, Default)]
#[serde(transparent)]
pub struct InlineSnippets {
    pub snippets: FxHashMap<String, Source>,
}

impl InlineSnippets {
    pub fn new(snippets: FxHashMap<String, Source>) -> Self {
        Self { snippets }
    }
}

impl Deref for InlineSnippets {
    type Target = FxHashMap<String, Source>;

    fn deref(&self) -> &Self::Target {
        &self.snippets
    }
}

impl DerefMut for InlineSnippets {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.snippets
    }
}

impl SnippetAccess for InlineSnippets {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        match self.snippets.get(identifier) {
            None => Err(String::new()),
            Some(source) => Ok(source.clone()),
        }
    }

    fn read_file(&self, _path: &Path) -> Result<Vec<u8>, String> {
        Err(String::new())
    }

    fn available_snippets(&self) -> Vec<String> {
        self.snippets.keys().cloned().collect()
    }
}

pub struct ChainedSnippetAccess<'a, 'b, A, B> {
    a: &'a A,
    b: &'b B,
}

impl<'a, 'b, A, B> ChainedSnippetAccess<'a, 'b, A, B> {
    pub fn new(a: &'a A, b: &'b B) -> Self {
        Self { a, b }
    }
}

impl<A, B> SnippetAccess for ChainedSnippetAccess<'_, '_, A, B>
where
    A: SnippetAccess,
    B: SnippetAccess,
{
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.a.read_snippet(identifier).or_else(|a_err| {
            self.b
                .read_snippet(identifier)
                .map_err(|b_err| chained_err(a_err, b_err))
        })
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.a.read_file(path).or_else(|a_err| {
            self.b
                .read_file(path)
                .map_err(|b_err| chained_err(a_err, b_err))
        })
    }

    fn available_snippets(&self) -> Vec<String> {
        let mut available = self.a.available_snippets();
        available.append(&mut self.b.available_snippets());

        available.sort_unstable();
        available.dedup();

        available
    }
}

fn chained_err(a_err: String, b_err: String) -> String {
    match (a_err.as_str(), b_err.as_str()) {
        ("", _) => b_err,
        (_, "") => a_err,
        _ => format!("{a_err} and {b_err}"),
    }
}

// TODO are NoAccess impls used anymore?
/// [`SnippetAccess`] implementation that forbids accessing any snippets
pub struct NoSnippetAccess;
impl SnippetAccess for NoSnippetAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        panic!(
            "Attempted to read snippet \"{identifier}\" while explicitely using NoSnippetAccess"
        );
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        panic!(
            "Attempted to read \"{}\" while explicitely using NoSnippetAccess",
            path.display()
        );
    }

    fn available_snippets(&self) -> Vec<String> {
        vec![]
    }
}
