use wotw_seedgen::data::assets::{file_err, EXECUTABLE_DIR, LOG_DATA_DIR, SEEDGEN_USER_DATA_DIR};

use crate::{
    cli::{OpenArgs, OpenDirectory},
    Error,
};

pub fn open(args: OpenArgs) -> Result<(), Error> {
    let OpenArgs { directory } = args;

    let path = match directory {
        OpenDirectory::Install => &*EXECUTABLE_DIR,
        OpenDirectory::UserData => &*SEEDGEN_USER_DATA_DIR,
        OpenDirectory::Logs => &*LOG_DATA_DIR,
    };

    open::that_detached(path).map_err(|err| Error(file_err("open", path, err)))
}
