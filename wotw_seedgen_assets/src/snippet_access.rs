use crate::Source;
use std::path::Path;

/// Resolves include commands in the seed language
pub trait SnippetAccess {
    /// Resolve `!include(<identifier>)`
    fn read_snippet(&self, identifier: &str) -> Result<Source, String>;
    /// Resolve binary includes such as `!bundle_icon(<path>)`
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String>;
    /// Return a `Vec` of identifiers which may be passed to [`SnippetAccess::read_snippet`]
    fn available_snippets(&self) -> Vec<String>;
}

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
