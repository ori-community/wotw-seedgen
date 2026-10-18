use std::{array, env, path::PathBuf, sync::LazyLock};

use crate::assets::{
    FullFileAccess, KnownAssetFileAccess, PlandoFileAccess, PresetFileAccess, SnippetFileAccess,
};

pub static RANDOMIZER_USER_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match env::var_os("RANDOMIZER_USER_DATA_DIR") {
        None => {
            let mut data_dir = dirs::data_dir().expect("cannot determine data directory");
            data_dir.push("ori-wotw-randomizer");
            data_dir
        }
        Some(data_dir) => PathBuf::from(data_dir),
    });

pub static SEEDGEN_USER_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match env::var_os("SEEDGEN_USER_DATA_DIR") {
        None => RANDOMIZER_USER_DATA_DIR.join("seedgen"),
        Some(data_dir) => PathBuf::from(data_dir),
    });

pub static CLIENT_USER_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match env::var_os("CLIENT_USER_DATA_DIR") {
        None => RANDOMIZER_USER_DATA_DIR.join("client"),
        Some(data_dir) => PathBuf::from(data_dir),
    });

pub static LOG_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| match env::var_os("LOG_DATA_DIR") {
    None => RANDOMIZER_USER_DATA_DIR.join("logs"),
    Some(data_dir) => PathBuf::from(data_dir),
});

pub static EXECUTABLE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut executable_dir =
        env::current_exe().unwrap_or_else(|err| panic!("failed to read executable path: {err}"));
    executable_dir.pop();
    executable_dir
});

pub struct DefaultFileAccess;

impl FullFileAccess for DefaultFileAccess {}

impl KnownAssetFileAccess for DefaultFileAccess {
    type Dirs = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn logic_dirs(&self) -> Self::Dirs {
        subdirs("logic")
    }

    fn uber_state_dump_paths(&self) -> Self::Dirs {
        subdirs("uber_state_dump.json")
    }
}

impl SnippetFileAccess for DefaultFileAccess {
    type Dirs = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn snippet_dirs(&self) -> Self::Dirs {
        subdirs("snippets")
    }
}

impl PlandoFileAccess for DefaultFileAccess {
    type Dirs = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn plando_dirs(&self) -> Self::Dirs {
        subdirs("plandos")
    }
}

impl PresetFileAccess for DefaultFileAccess {
    type Dirs = array::IntoIter<Self::Path, 2>;
    type Path = PathBuf;

    fn universe_dirs(&self) -> Self::Dirs {
        subdirs("universe_presets")
    }

    fn world_dirs(&self) -> Self::Dirs {
        subdirs("world_presets")
    }
}

fn subdirs(prefix: &str) -> array::IntoIter<PathBuf, 2> {
    [
        SEEDGEN_USER_DATA_DIR.join(prefix),
        EXECUTABLE_DIR.join(prefix),
    ]
    .into_iter()
}
