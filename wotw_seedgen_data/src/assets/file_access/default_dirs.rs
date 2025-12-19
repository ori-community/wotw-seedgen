use std::{array, env, path::PathBuf, sync::LazyLock};

use crate::assets::{AssetFileAccess, PresetFileAccess, SnippetFileAccess};

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| match env::var_os("CONFIG_DIR") {
    None => {
        let mut config_dir = dirs::config_dir().expect("cannot determine config directory");
        config_dir.push("Ori and the Will of the Wisps Randomizer/seedgen");
        config_dir
    }
    Some(config_dir) => PathBuf::from(config_dir),
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

    fn folders(&self) -> Self::Folders {
        [&*CONFIG_DIR, &*EXECUTABLE_DIR].into_iter()
    }
}

impl SnippetFileAccess for DefaultFileAccess {
    type Folders = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn folders(&self) -> Self::Folders {
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
    [CONFIG_DIR.join(prefix), EXECUTABLE_DIR.join(prefix)].into_iter()
}
