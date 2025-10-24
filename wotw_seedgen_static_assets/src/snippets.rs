use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::path::Path;
use wotw_seedgen_assets::{SnippetAccess, Source};

pub struct StaticSnippetAccess {
    snippets: FxHashMap<String, (String, String)>, // TODO can we really not have &'static str here :( Maybe with a different library. Many don't work with the preset format, but flexbuffers and bendy could be worth trying
}

lazy_static! {
    pub static ref SNIPPET_ACCESS: StaticSnippetAccess = StaticSnippetAccess {
        snippets: ciborium::from_reader(
            include_bytes!(concat!(env!("OUT_DIR"), "/snippets")).as_slice()
        )
        .unwrap()
    };
}

impl SnippetAccess for StaticSnippetAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.snippets
            .get(identifier)
            .cloned()
            .map(|(id, content)| Source::new(id, content))
            .ok_or_else(|| format!("unknown snippet \"{identifier}\""))
    }

    fn read_file(&self, _path: &Path) -> Result<Vec<u8>, String> {
        Err("cannot read arbitrary files with static file access".to_string())
    }

    fn available_snippets(&self) -> Vec<String> {
        self.snippets.keys().map(String::clone).collect()
    }
}
