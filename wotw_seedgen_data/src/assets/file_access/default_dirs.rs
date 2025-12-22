use std::{array, env, path::PathBuf, sync::LazyLock};

use crate::assets::{AssetFileAccess, PresetFileAccess, SnippetFileAccess};

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| match env::var_os("DATA_DIR") {
    None => {
        let mut data_dir = dirs::data_dir().expect("cannot determine data directory");
        data_dir.push("Ori and the Will of the Wisps Randomizer/seedgen");
        data_dir
    }
    Some(data_dir) => PathBuf::from(data_dir),
});

pub static EXECUTABLE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut executable_dir =
        env::current_exe().unwrap_or_else(|err| panic!("failed to read executable path: {err}"));
    executable_dir.pop();
    executable_dir
});

pub struct DefaultFileAccess;

impl AssetFileAccess for DefaultFileAccess {
    type Folders = array::IntoIter<Self::Path, 2>;
    type Path = &'static PathBuf;

    fn asset_folders(&self) -> Self::Folders {
        [&*DATA_DIR, &*EXECUTABLE_DIR].into_iter()
    }
}

impl SnippetFileAccess for DefaultFileAccess {
    type Folders = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn snippet_folders(&self) -> Self::Folders {
        subfolders("snippets")
    }
}

impl PresetFileAccess for DefaultFileAccess {
    type Folders = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn universe_folders(&self) -> Self::Folders {
        subfolders("universe_presets")
    }

    fn world_folders(&self) -> Self::Folders {
        subfolders("world_presets")
    }
}

fn subfolders(prefix: &str) -> array::IntoIter<PathBuf, 2> {
    [DATA_DIR.join(prefix), EXECUTABLE_DIR.join(prefix)].into_iter()
}
