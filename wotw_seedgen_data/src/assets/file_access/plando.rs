use std::{
    borrow::Cow,
    iter::{self, Chain, Empty, Map, Once},
    path::{Path, PathBuf},
};

use crate::assets::{
    DefaultFileAccess, FullFileAccess, KnownAssetFileAccess, PlandoFileAccess, PresetFileAccess,
    SnippetFileAccess,
};

pub struct PlandoDirAccess<'a> {
    root: &'a Path,
}

impl<'a> PlandoDirAccess<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self { root }
    }
}

impl FullFileAccess for PlandoDirAccess<'_> {}

impl KnownAssetFileAccess for PlandoDirAccess<'_> {
    type Dirs = <DefaultFileAccess as KnownAssetFileAccess>::Dirs;
    type Path = <DefaultFileAccess as KnownAssetFileAccess>::Path;

    fn logic_dirs(&self) -> Self::Dirs {
        DefaultFileAccess.logic_dirs()
    }

    fn uber_state_dump_paths(&self) -> Self::Dirs {
        DefaultFileAccess.uber_state_dump_paths()
    }
}

impl<'a> SnippetFileAccess for PlandoDirAccess<'a> {
    type Dirs = Chain<
        Once<Cow<'a, Path>>,
        Map<<DefaultFileAccess as SnippetFileAccess>::Dirs, fn(PathBuf) -> Cow<'a, Path>>,
    >;
    type Path = Cow<'a, Path>;

    fn snippet_dirs(&self) -> Self::Dirs {
        iter::once(Cow::Borrowed(self.root)).chain(
            DefaultFileAccess
                .snippet_dirs()
                .map(Cow::Owned as fn(_) -> _),
        )
    }
}

impl PlandoFileAccess for PlandoDirAccess<'_> {
    type Dirs = Empty<PathBuf>;
    type Path = PathBuf;

    fn plando_dirs(&self) -> Self::Dirs {
        iter::empty()
    }
}

impl PresetFileAccess for PlandoDirAccess<'_> {
    type Dirs = Empty<PathBuf>;
    type Path = PathBuf;

    fn universe_dirs(&self) -> Self::Dirs {
        iter::empty()
    }

    fn world_dirs(&self) -> Self::Dirs {
        iter::empty()
    }
}
