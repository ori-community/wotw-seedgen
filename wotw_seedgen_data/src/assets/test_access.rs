use std::{
    iter::{self, Once},
    path::Path,
    sync::LazyLock,
};

use constcat::concat;

use crate::assets::{
    AssetCache, AssetFileAccess, DefaultAssetCacheValues, PresetFileAccess, SnippetFileAccess,
};

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets");

pub struct TestAccess;

pub static TEST_ASSETS: LazyLock<AssetCache<TestAccess, DefaultAssetCacheValues>> =
    LazyLock::new(|| AssetCache::new(TestAccess).unwrap());

impl AssetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn asset_folders(&self) -> Self::Folders {
        iter::once(&Path::new(ASSETS))
    }
}

impl SnippetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn snippet_folders(&self) -> Self::Folders {
        iter::once(&Path::new(concat!(ASSETS, "/snippets")))
    }
}

impl PresetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn universe_folders(&self) -> Self::Folders {
        iter::once(&Path::new(concat!(ASSETS, "/universe_presets")))
    }

    fn world_folders(&self) -> Self::Folders {
        iter::once(&Path::new(concat!(ASSETS, "/world_presets")))
    }
}
