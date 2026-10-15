use std::{
    borrow::Cow,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use wotw_seedgen::data::assets::{self, file_err, CLIENT_USER_DATA_DIR};

use crate::Error;

pub fn read_ngss() -> Result<PathBuf, Error> {
    let mut line = String::new();

    let ngss_path = CLIENT_USER_DATA_DIR.join(".newgameseedsource");
    BufReader::new(assets::file_open(&ngss_path)?)
        .read_line(&mut line)
        .map_err(|err| file_err("read", &ngss_path, err))?;

    line.trim()
        .strip_prefix("file:")
        .map(PathBuf::from)
        .ok_or_else(|| Error(format!("cannot access seed source \"{line}\"")))
}

pub fn write_ngss(path: &Path) -> Result<(), Error> {
    let path = path.canonicalize().map_or(Cow::Borrowed(path), Cow::Owned);

    let path = path.to_str().ok_or_else(|| {
        format!(
            "failed to write newgameseedsource for \"{}\": invalid unicode",
            path.display()
        )
    })?;

    let client_user_data_dir = &*CLIENT_USER_DATA_DIR;
    assets::create_dir_all(client_user_data_dir)?;

    let ngss_path = client_user_data_dir.join(".newgameseedsource");
    assets::write(ngss_path, format!("file:{path}")).map_err(Error)
}
