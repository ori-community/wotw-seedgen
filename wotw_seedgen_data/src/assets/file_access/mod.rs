mod cache;
mod default_dirs;
mod dirs;
mod plando;
mod watch;

pub use cache::{AssetCache, AssetCacheValues, ChangedAssets, DefaultAssetCacheValues};
pub use default_dirs::{
    DefaultFileAccess, CLIENT_USER_DATA_DIR, EXECUTABLE_DIR, LOG_DATA_DIR,
    RANDOMIZER_USER_DATA_DIR, SEEDGEN_USER_DATA_DIR,
};
pub use plando::PlandoDirAccess;
pub use watch::{Watcher, WatcherError, WatcherMessage, WatcherResult};

use crate::assets::{
    known_asset_access::KnownAssetAccess, Pickups, PlandoAccess, PresetAccess, SnippetAccess,
    States, UberStateDump, UniversePreset, WorldPreset,
};
use std::{
    borrow::Cow,
    fmt::Display,
    fs::{self, DirEntry, File, Metadata},
    io::{self, BufReader},
    path::{Path, PathBuf},
};
use wotw_seedgen_parse::Source;

pub fn file_err<E: Display, P: AsRef<Path>>(action: &str, path: P, err: E) -> String {
    format!("failed to {action} \"{}\": {err}", path.as_ref().display())
}

pub fn file_create<P: AsRef<Path>>(path: P) -> Result<File, String> {
    create_with(path.as_ref(), File::create)
}

pub fn file_open<P: AsRef<Path>>(path: P) -> Result<File, String> {
    action_with("open", path.as_ref(), File::open)
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    read_with(path.as_ref(), fs::read_to_string)
}

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    create_with(path.as_ref(), fs::create_dir_all)
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> Result<ReadDir<P>, String> {
    let inner = read_with(path.as_ref(), fs::read_dir)?;
    Ok(ReadDir { path, inner })
}

pub struct ReadDir<P> {
    path: P,
    inner: fs::ReadDir,
}

impl<P: AsRef<Path>> Iterator for ReadDir<P> {
    type Item = Result<DirEntry, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.inner.next()?;
        Some(entry.map_err(|err| file_err("read", &self.path, err)))
    }
}

pub fn canonicalize<P: AsRef<Path>>(path: P) -> Result<PathBuf, String> {
    action_with("canonicalize", path.as_ref(), fs::canonicalize)
}

pub fn metadata<P: AsRef<Path>>(path: P) -> Result<Metadata, String> {
    read_with(path.as_ref(), fs::metadata)
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), String> {
    action_with("write", path.as_ref(), |path| fs::write(path, contents))
}

fn create_with<'p, O, F: FnOnce(&'p Path) -> io::Result<O>>(
    path: &'p Path,
    f: F,
) -> Result<O, String> {
    action_with("create", path, f)
}

fn read_with<'p, O, F: FnOnce(&'p Path) -> io::Result<O>>(
    path: &'p Path,
    f: F,
) -> Result<O, String> {
    action_with("read", path, f)
}

fn action_with<'p, O, F: FnOnce(&'p Path) -> io::Result<O>>(
    action: &str,
    path: &'p Path,
    f: F,
) -> Result<O, String> {
    f(path).map_err(|err| file_err(action, path, err))
}

pub trait FullFileAccess:
    KnownAssetFileAccess + SnippetFileAccess + PlandoFileAccess + PresetFileAccess
{
}

pub trait KnownAssetFileAccess {
    type Dirs: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn logic_dirs(&self) -> Self::Dirs;

    fn uber_state_dump_paths(&self) -> Self::Dirs;
}

impl<T: KnownAssetFileAccess> KnownAssetAccess for T {
    fn pickups(&self) -> Result<Pickups, String> {
        let (path, file) = dirs::join(self.logic_dirs(), Path::new("pickups.csv")).open()?;

        Pickups::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }

    fn states(&self) -> Result<States, String> {
        let (path, file) = dirs::join(self.logic_dirs(), Path::new("states.csv")).open()?;

        States::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }

    fn paths(&self) -> Result<Source, String> {
        let (path, content) =
            dirs::join(self.logic_dirs(), Path::new("paths.wotwl")).read_to_string()?;

        let id = path.to_string_lossy().to_string();
        Ok(Source::new(id, content))
    }

    fn uber_state_dump(&self) -> Result<UberStateDump, String> {
        let (path, file) = dirs::open(
            self.uber_state_dump_paths()
                .map(|p| p.as_ref().to_path_buf()),
            "uber_state_dump.json",
        )?;

        serde_json::from_reader(BufReader::new(file)).map_err(|err| file_err("parse", &path, err))
    }
}

pub trait SnippetFileAccess {
    type Dirs: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn snippet_dirs(&self) -> Self::Dirs;
}

impl<T: SnippetFileAccess> SnippetAccess for T {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        let mut path = Cow::Borrowed(Path::new(identifier));

        if path.extension().is_none() {
            path.to_mut().set_extension("wotws");
        }

        let (path, content) = dirs::join(self.snippet_dirs(), &path).read_to_string()?;

        let id = path.to_string_lossy().to_string();
        Ok(Source::new(id, content))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        let (_, content) = dirs::join(self.snippet_dirs(), path).read()?;

        Ok(content)
    }

    fn available_snippets(&self) -> Vec<String> {
        dirs::available_files(self.snippet_dirs(), "wotws")
    }
}

pub trait PlandoFileAccess {
    type Dirs: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn plando_dirs(&self) -> Self::Dirs;
}

impl<T: PlandoFileAccess> PlandoAccess for T {
    fn read_plando(&self, identifier: &str) -> Result<Vec<u8>, String> {
        let mut path = Cow::Borrowed(Path::new(identifier));

        if path.extension().is_none() {
            path.to_mut().set_extension("wotwr");
        }

        let (_, content) = dirs::join(self.plando_dirs(), &path).read()?;

        Ok(content)
    }

    fn available_plandos(&self) -> Vec<String> {
        dirs::available_files(self.plando_dirs(), "wotwr")
    }
}

pub trait PresetFileAccess {
    type Dirs: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn universe_dirs(&self) -> Self::Dirs;

    fn world_dirs(&self) -> Self::Dirs;
}

impl<T: PresetFileAccess> PresetAccess for T {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        let mut path = Cow::Borrowed(Path::new(identifier));

        if path.extension().is_none() {
            path.to_mut().set_extension("json");
        }

        dirs::join(self.universe_dirs(), &path).read_json()
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        let mut path = Cow::Borrowed(Path::new(identifier));

        if path.extension().is_none() {
            path.to_mut().set_extension("json");
        }

        dirs::join(self.world_dirs(), &path).read_json()
    }

    fn available_universe_presets(&self) -> Vec<String> {
        dirs::available_files(self.universe_dirs(), "json")
    }

    fn available_world_presets(&self) -> Vec<String> {
        dirs::available_files(self.world_dirs(), "json")
    }
}
