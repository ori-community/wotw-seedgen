use std::path::Path;
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
