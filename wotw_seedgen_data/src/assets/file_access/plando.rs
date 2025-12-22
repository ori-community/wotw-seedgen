use std::{
    borrow::Cow,
    iter::{self, Chain, Map, Once},
    path::{Path, PathBuf},
};

use crate::assets::{AssetFileAccess, DefaultFileAccess, PresetFileAccess, SnippetFileAccess};

pub struct PlandoFileAccess<'a> {
    root: &'a Path,
}

impl<'a> PlandoFileAccess<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }
}

impl AssetFileAccess for PlandoFileAccess<'_> {
    type Folders = <DefaultFileAccess as AssetFileAccess>::Folders;
    type Path = <DefaultFileAccess as AssetFileAccess>::Path;

    fn asset_folders(&self) -> Self::Folders {
        DefaultFileAccess.asset_folders()
    }
}

impl<'a> SnippetFileAccess for PlandoFileAccess<'a> {
    type Folders = Chain<
        Once<Cow<'a, Path>>,
        Map<<DefaultFileAccess as SnippetFileAccess>::Folders, fn(PathBuf) -> Cow<'a, Path>>,
    >;
    type Path = Cow<'a, Path>;

    fn snippet_folders(&self) -> Self::Folders {
        iter::once(Cow::Borrowed(self.root)).chain(
            DefaultFileAccess
                .snippet_folders()
                .map(Cow::Owned as fn(_) -> _),
        )
    }
}

impl PresetFileAccess for PlandoFileAccess<'_> {
    type Folders = <DefaultFileAccess as PresetFileAccess>::Folders;
    type Path = <DefaultFileAccess as PresetFileAccess>::Path;

    fn universe_folders(&self) -> Self::Folders {
        DefaultFileAccess.universe_folders()
    }

    fn world_folders(&self) -> Self::Folders {
        DefaultFileAccess.world_folders()
    }
}
