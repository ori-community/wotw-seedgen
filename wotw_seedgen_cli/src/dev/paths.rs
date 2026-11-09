use wotw_seedgen::data::assets::{EXECUTABLE_DIR, LOG_DATA_DIR, SEEDGEN_USER_DATA_DIR};

use crate::{
    cli::dev::{PathsArgs, SeedgenPath},
    Error,
};

pub fn paths(args: PathsArgs) -> Result<(), Error> {
    match args.path {
        Some(SeedgenPath::Install) => println!("{}", EXECUTABLE_DIR.display()),
        Some(SeedgenPath::User) => println!("{}", SEEDGEN_USER_DATA_DIR.display()),
        Some(SeedgenPath::Logs) => println!("{}", LOG_DATA_DIR.display()),
        None => {
            println!("Install data: {}", EXECUTABLE_DIR.display());
            println!("User data: {}", SEEDGEN_USER_DATA_DIR.display());
            println!("Logs: {}", LOG_DATA_DIR.display());
        }
    }

    Ok(())
}
