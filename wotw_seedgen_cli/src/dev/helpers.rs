use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

use wotw_seedgen::data::assets::{self, file_err, RANDOMIZER_USER_DATA_DIR};

use crate::Error;

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
