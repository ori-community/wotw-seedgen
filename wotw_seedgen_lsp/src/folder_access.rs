use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::Url;
use wotw_seedgen_seed_language::assets::{FileAccess, SnippetAccess, Source};
use wotw_seedgen_static_assets::SNIPPET_ACCESS;

pub struct FolderAccess {
    file_access: FileAccess,
}

impl FolderAccess {
    pub fn new<P: AsRef<Path>>(source: P) -> Self {
        let folder = source.as_ref().parent().unwrap_or(Path::new(""));
        let file_access = FileAccess::new([folder]);
        Self { file_access }
    }
}

impl SnippetAccess for FolderAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.file_access
            .read_snippet(identifier)
            .or_else(|err| SNIPPET_ACCESS.read_snippet(identifier).map_err(|_| err))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.file_access.read_file(path)
    }
}

pub fn url_to_path(url: &Url) -> Result<PathBuf, String> {
    if url.scheme() != "file" {
        return Err(format!("invalid url \"{url}\": not a file scheme"));
    }
    url.to_file_path()
        .map_err(|()| format!("invalid url \"{url}\""))
}
