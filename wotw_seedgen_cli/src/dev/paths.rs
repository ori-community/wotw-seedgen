use wotw_seedgen::data::assets::{EXECUTABLE_DIR, LOG_DATA_DIR, SEEDGEN_USER_DATA_DIR};

use crate::Error;

pub fn paths() -> Result<(), Error> {
    println!("Install data: {}", EXECUTABLE_DIR.display());
    println!("User data: {}", SEEDGEN_USER_DATA_DIR.display());
    println!("Logs: {}", LOG_DATA_DIR.display());

    Ok(())
}
