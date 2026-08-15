use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use serde::de::DeserializeOwned;
use wotw_seedgen::data::assets::{self, file_err, RANDOMIZER_USER_DATA_DIR};
use zip::{read::ZipFile, ZipArchive};

use crate::Error;

pub fn json_by_name<T: DeserializeOwned>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> Result<T, Error> {
    Ok(serde_json::from_reader(by_name(archive, name)?)?)
}

fn by_name<'a>(archive: &'a mut ZipArchive<File>, name: &str) -> Result<ZipFile<'a, File>, Error> {
    Ok(archive
        .by_name(name)
        .map_err(|err| format!("failed to read \"{name}\" from seed: {err}"))?)
}

pub fn read_ngss() -> Result<PathBuf, Error> {
    let mut line = String::new();

    let ngss_path = RANDOMIZER_USER_DATA_DIR.join("randomizer/.newgameseedsource");
    BufReader::new(assets::file_open(&ngss_path)?)
        .read_line(&mut line)
        .map_err(|err| file_err("read", &ngss_path, err))?;

    line.trim()
        .strip_prefix("file:")
        .map(PathBuf::from)
        .ok_or_else(|| Error(format!("cannot access seed source \"{line}\"")))
}
