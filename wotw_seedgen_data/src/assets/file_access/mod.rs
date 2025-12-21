mod cache;
mod default_dirs;
mod plando;
mod watch;

pub use cache::{AssetCache, AssetCacheValues, ChangedAssets, DefaultAssetCacheValues};
pub use default_dirs::{DefaultFileAccess, DATA_DIR, EXECUTABLE_DIR};
pub use plando::PlandoFileAccess;
pub use watch::{Watcher, WatcherError};

use crate::assets::{
    LocData, PresetAccess, SnippetAccess, StateData, UberStateData, UberStateDump, UniversePreset,
    WorldPreset,
};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::{BufReader, ErrorKind, Read},
    path::{Path, PathBuf},
    vec,
};
use wotw_seedgen_parse::Source;

pub fn file_err<E: Display, P: AsRef<Path>>(action: &str, path: P, err: E) -> String {
    format!("failed to {action} \"{}\": {err}", path.as_ref().display())
}

// TODO this trait for non-file access?
pub trait AssetFileAccess {
    type Folders: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn folders(&self) -> Self::Folders;

    const LOC_DATA_PATH: &str = "logic/loc_data.csv";

    fn loc_data(&self) -> Result<LocData, String> {
        let (path, file) = open(self.folders(), Path::new(Self::LOC_DATA_PATH))?;

        LocData::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }

    const STATE_DATA_PATH: &str = "logic/state_data.csv";

    fn state_data(&self) -> Result<StateData, String> {
        let (path, file) = open(self.folders(), Path::new(Self::STATE_DATA_PATH))?;

        StateData::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }

    const UBER_STATE_DUMP_PATH: &str = "uber_state_dump.json";

    fn uber_state_dump(&self) -> Result<UberStateDump, String> {
        let (path, file) = open(self.folders(), Path::new(Self::UBER_STATE_DUMP_PATH))?;

        serde_json::from_reader(BufReader::new(file)).map_err(|err| file_err("parse", &path, err))
    }

    fn uber_state_data(
        &self,
        loc_data: &LocData,
        state_data: &StateData,
    ) -> Result<UberStateData, String> {
        let dump = self.uber_state_dump()?;

        Ok(UberStateData::from_parts(dump, loc_data, state_data))
    }

    const AREAS_PATH: &str = "logic/areas.wotw";

    fn areas(&self) -> Result<Source, String> {
        let (path, content) = read_to_string(self.folders(), Path::new(Self::AREAS_PATH))?;

        let id = path.to_string_lossy().to_string();
        Ok(Source::new(id, content))
    }
}

pub trait SnippetFileAccess {
    type Folders: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn folders(&self) -> Self::Folders;
}

impl<T: SnippetFileAccess> SnippetAccess for T {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        let mut path = Cow::Borrowed(Path::new(identifier));

        if path.extension().is_none() {
            path.to_mut().set_extension("wotws");
        }

        let (path, content) = read_to_string(self.folders(), &path)?;

        let id = path.to_string_lossy().to_string();
        Ok(Source::new(id, content))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        read(self.folders(), path).map(|(_, content)| content)
    }

    fn available_snippets(&self) -> Vec<String> {
        available_files(self.folders(), "wotws")
    }
}

pub trait PresetFileAccess {
    type Folders: Iterator<Item = Self::Path>;
    type Path: AsRef<Path>;

    fn universe_folders(&self) -> Self::Folders;

    fn world_folders(&self) -> Self::Folders;
}

impl<T: PresetFileAccess> PresetAccess for T {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        read_json(self.universe_folders(), identifier)
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        read_json(self.world_folders(), identifier)
    }

    fn available_universe_presets(&self) -> Vec<String> {
        available_files(self.universe_folders(), "json")
    }

    fn available_world_presets(&self) -> Vec<String> {
        available_files(self.world_folders(), "json")
    }
}

fn open(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, File), String> {
    let mut attempts = vec![];

    for folder in folders {
        let full_path = folder.as_ref().join(path);

        match File::open(&full_path) {
            Ok(file) => return Ok((full_path, file)),
            Err(err) if err.kind() == ErrorKind::NotFound => attempts.push(full_path),
            Err(err) => return Err(file_err("read", &full_path, err)),
        }
    }

    Err(format!(
        "\"{}\" not found at \"{}\"",
        path.display(),
        attempts
            .into_iter()
            .format_with("\" or \"", |path, f| f(&path.display()))
    ))
}

fn read(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, Vec<u8>), String> {
    let (path, mut file) = open(folders, path)?;

    let mut buf = vec![];
    file.read_to_end(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

fn read_to_string(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, String), String> {
    let (path, mut file) = open(folders, path)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

fn read_json<P: DeserializeOwned>(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    identifier: &str,
) -> Result<P, String> {
    let mut path = Cow::Borrowed(Path::new(identifier));

    if path.extension().is_none() {
        path.to_mut().set_extension("json");
    }

    let (path, file) = open(folders, &path)?;

    serde_json::from_reader(BufReader::new(file)).map_err(|err| file_err("parse", &path, err))
}

fn available_files(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    extension: &str,
) -> Vec<String> {
    let extension = OsStr::new(extension);

    let mut files = folders
        .into_iter()
        .flat_map(|folder| {
            fs::read_dir(folder)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.file_name())
                .filter(|file_name| Path::new(file_name).extension() == Some(extension))
                .map(|file_name| {
                    Path::new(&file_name)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
        })
        .collect::<Vec<_>>();

    files.sort_unstable();

    files
}
