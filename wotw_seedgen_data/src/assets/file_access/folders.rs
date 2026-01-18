use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, ErrorKind, Read},
    path::{Path, PathBuf},
    vec,
};

use crate::assets::file_err;

pub fn open(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, File), String> {
    let mut attempts = vec![];

    for folder in folders {
        let full_path = folder.as_ref().join(path);

        match File::open(&full_path) {
            Ok(file) => return Ok((full_path, file)),
            Err(err) if err.kind() == ErrorKind::NotFound => attempts.push(full_path),
            Err(err) => return Err(file_err("open", &full_path, err)),
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

pub fn read(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, Vec<u8>), String> {
    let (path, mut file) = open(folders, path)?;

    let mut buf = vec![];
    file.read_to_end(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

pub fn read_to_string(
    folders: impl IntoIterator<Item = impl AsRef<Path>>,
    path: &Path,
) -> Result<(PathBuf, String), String> {
    let (path, mut file) = open(folders, path)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

pub fn read_json<P: DeserializeOwned>(
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

pub fn available_files(
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
