#[cfg(feature = "loc_data")]
use crate::LocData;
use crate::Source;
#[cfg(feature = "state_data")]
use crate::StateData;
#[cfg(feature = "uber_state_data")]
use crate::UberStateData;
use itertools::Itertools;
#[cfg(feature = "snippet_access")]
use std::io::Read;
use std::{
    ffi::OsStr,
    fmt::Display,
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub fn file_err<E: Display, P: AsRef<Path>>(action: &str, path: P, err: E) -> String {
    format!("failed to {action} \"{}\": {err}", path.as_ref().display())
}

pub struct FileAccess {
    folders: Vec<PathBuf>,
}
impl FileAccess {
    pub fn new<P: AsRef<Path>, I: IntoIterator<Item = P>>(folders: I) -> Self {
        let folders = folders
            .into_iter()
            .map(|folder| folder.as_ref().to_path_buf())
            .collect::<Vec<_>>();
        assert!(!folders.is_empty());
        Self { folders }
    }

    #[cfg(feature = "loc_data")]
    pub fn loc_data(&self) -> Result<LocData, String> {
        let (path, file) = self.open(Path::new("loc_data.csv"))?;
        LocData::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }
    #[cfg(feature = "state_data")]
    pub fn state_data(&self) -> Result<StateData, String> {
        let (path, file) = self.open(Path::new("state_data.csv"))?;
        StateData::from_reader(file).map_err(|err| file_err("parse", &path, err))
    }
    #[cfg(feature = "uber_state_data")]
    pub fn uber_state_data(
        &self,
        loc_data: &LocData,
        state_data: &StateData,
    ) -> Result<UberStateData, String> {
        // TODO rename uber_state_dump -> uber_state_data
        let (path, file) = self.open(Path::new("uber_state_dump.json"))?;
        UberStateData::from_reader(file, loc_data, state_data)
            .map_err(|err| file_err("parse", &path, err))
    }
    pub fn areas(&self) -> Result<Source, String> {
        let (path, content) = self.read_to_string(Path::new("areas.wotw"))?;
        let id = path.to_string_lossy().to_string();
        Ok(Source::new(id, content))
    }

    #[cfg(feature = "snippet_access")]
    pub fn available_snippets(&self) -> Vec<String> {
        self.files_in_folder("wotws")
    }
    #[cfg(feature = "presets")]
    pub fn available_universe_presets(&self) -> Vec<String> {
        self.available_presets("universe_presets")
    }
    #[cfg(feature = "presets")]
    pub fn available_world_presets(&self) -> Vec<String> {
        self.available_presets("world_presets")
    }
    #[cfg(feature = "presets")]
    fn available_presets(&self, folder: &str) -> Vec<String> {
        FileAccess::new(self.folders.iter().map(|path| path.join(folder))).files_in_folder("json")
    }

    fn open(&self, path: &Path) -> Result<(PathBuf, File), String> {
        let mut attempts = vec![];
        for folder in &self.folders {
            let full_path = folder.join(path);
            match File::open(&full_path) {
                Ok(file) => return Ok((full_path, file)),
                Err(err) if err.kind() == ErrorKind::NotFound => attempts.push(full_path),
                Err(err) => return Err(file_err("read", &full_path, err)),
            }
        }
        return Err(format!(
            "\"{}\" not found at \"{}\"",
            path.display(),
            attempts
                .into_iter()
                .format_with("\" or \"", |path, f| f(&path.display()))
        ));
    }
    #[cfg(feature = "snippet_access")]
    fn read(&self, path: &Path) -> Result<(PathBuf, Vec<u8>), String> {
        let (path, mut file) = self.open(path)?;
        let mut buf = vec![];
        file.read_to_end(&mut buf)
            .map_err(|err| file_err("read", &path, err))?;
        Ok((path, buf))
    }
    fn read_to_string(&self, path: &Path) -> Result<(PathBuf, String), String> {
        let (path, mut file) = self.open(path)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|err| file_err("read", &path, err))?;
        Ok((path, buf))
    }

    fn files_in_folder(&self, extension: &str) -> Vec<String> {
        let extension = OsStr::new(extension);
        let mut files = vec![];

        for folder in &self.folders {
            files.extend(
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
                    }),
            );
            if !files.is_empty() {
                break;
            }
        }

        files.sort_unstable();

        files
    }
}

#[cfg(feature = "snippet_access")]
mod snippet_access {
    use super::*;
    use crate::{SnippetAccess, Source};

    impl SnippetAccess for FileAccess {
        fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
            let mut path = PathBuf::from(identifier);
            path.set_extension("wotws");
            let (path, content) = self.read_to_string(&path)?;
            let id = path.to_string_lossy().to_string();
            Ok(Source::new(id, content))
        }
        fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
            self.read(path).map(|(_, content)| content)
        }
    }
}

#[cfg(feature = "presets")]
mod presets {
    use super::*;
    use crate::{PresetAccess, UniversePreset, WorldPreset};
    use serde::de::DeserializeOwned;
    use std::io::BufReader;

    impl PresetAccess for FileAccess {
        fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
            self.preset(identifier, "universe_presets".into())
        }
        fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
            self.preset(identifier, "world_presets".into())
        }
    }

    impl FileAccess {
        fn preset<P: DeserializeOwned>(
            &self,
            identifier: &str,
            mut path: PathBuf,
        ) -> Result<P, String> {
            path.push(identifier);
            path.set_extension("json");
            let (path, file) = self.open(&path)?;
            serde_json::from_reader(BufReader::new(file))
                .map_err(|err| file_err("parse", &path, err))
        }
    }
}
