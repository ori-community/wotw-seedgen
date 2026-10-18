use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, ErrorKind, Read},
    path::{Path, PathBuf},
    vec,
};

use crate::assets::file_err;

pub fn join<'p, P, I>(dirs: I, path: &'p Path) -> JoinedDirs<'p, I>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
{
    JoinedDirs { dirs, path }
}

pub struct JoinedDirs<'p, I> {
    dirs: I,
    path: &'p Path,
}

impl<I, P> Iterator for JoinedDirs<'_, I>
where
    I: Iterator<Item = P>,
    P: AsRef<Path>,
{
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.dirs.next().map(|dir| dir.as_ref().join(self.path))
    }
}

impl<I, P> JoinedDirs<'_, I>
where
    I: Iterator<Item = P>,
    P: AsRef<Path>,
{
    pub fn open(self) -> Result<(PathBuf, File), String> {
        let target = self.path;
        open(self, target)
    }

    pub fn read(self) -> Result<(PathBuf, Vec<u8>), String> {
        let target = self.path;
        read(self, target)
    }

    pub fn read_to_string(self) -> Result<(PathBuf, String), String> {
        let target = self.path;
        read_to_string(self, target)
    }

    pub fn read_json<T: DeserializeOwned>(self) -> Result<T, String> {
        let target = self.path;
        read_json(self, target)
    }
}

pub fn open<I, P>(mut paths: I, target: P) -> Result<(PathBuf, File), String>
where
    I: Iterator<Item = PathBuf>,
    P: AsRef<Path>,
{
    let mut attempts = vec![];

    for path in &mut paths {
        match File::open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(err) if err.kind() == ErrorKind::NotFound => attempts.push(path),
            Err(err) => return Err(file_err("open", path, err)),
        }
    }

    Err(format!(
        "\"{}\" not found at \"{}\"",
        target.as_ref().display(),
        attempts
            .into_iter()
            .format_with("\" or \"", |path, f| f(&path.display()))
    ))
}

pub fn read<I, P>(paths: I, target: P) -> Result<(PathBuf, Vec<u8>), String>
where
    I: Iterator<Item = PathBuf>,
    P: AsRef<Path>,
{
    let (path, mut file) = open(paths, target)?;

    let mut buf = vec![];
    file.read_to_end(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

pub fn read_to_string<I, P>(paths: I, target: P) -> Result<(PathBuf, String), String>
where
    I: Iterator<Item = PathBuf>,
    P: AsRef<Path>,
{
    let (path, mut file) = open(paths, target)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|err| file_err("read", &path, err))?;

    Ok((path, buf))
}

pub fn read_json<I, P, T>(paths: I, target: P) -> Result<T, String>
where
    I: Iterator<Item = PathBuf>,
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let (path, file) = open(paths, target)?;

    serde_json::from_reader(BufReader::new(file)).map_err(|err| file_err("parse", &path, err))
}

pub fn available_files(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
    extension: &str,
) -> Vec<String> {
    let extension = OsStr::new(extension);

    let mut files = dirs
        .into_iter()
        .flat_map(|dir| {
            fs::read_dir(dir)
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
