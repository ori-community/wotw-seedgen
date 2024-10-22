use std::fs::File;

use log::LevelFilter;
use serde::de::DeserializeOwned;
use wotw_seedgen::seed::{assembly::Assembly, SeedgenInfo};
use wotw_seedgen_assets::file_err;
use zip::{read::ZipFile, ZipArchive};

use crate::{cli::RegenerateArgs, log_init::initialize_log, seed::generate, Error};

pub fn regenerate(args: RegenerateArgs) -> Result<(), Error> {
    let RegenerateArgs { path, verbose } = args;

    initialize_log(
        verbose.then_some("seedgen_log.txt"),
        LevelFilter::Info,
        false,
    )?;

    let file = File::open(&path).map_err(|err| file_err("open", &path, err))?;
    let mut archive = ZipArchive::new(file).map_err(|err| file_err("read", &path, err))?;
    let seedgen_info = json_by_name::<SeedgenInfo>(&mut archive, "seedgen_info.json")?;
    let assembly = json_by_name::<Assembly>(&mut archive, "assembly.json")?;

    // TODO compare seedgen commit hash

    let seed_universe = generate(&seedgen_info.universe_settings, true)?;
    if assembly != seed_universe.worlds[seedgen_info.world_index].assembly {
        return Err(Error("Regenerated seed did not match".to_string()));
    }

    Ok(())
}

fn json_by_name<T: DeserializeOwned>(
    archive: &mut ZipArchive<File>,
    name: &str,
) -> Result<T, Error> {
    Ok(serde_json::from_reader(by_name(archive, name)?)?)
}

fn by_name<'a>(archive: &'a mut ZipArchive<File>, name: &str) -> Result<ZipFile<'a>, Error> {
    Ok(archive
        .by_name(name)
        .map_err(|err| format!("failed to read \"{name}\" from seed: {err}"))?)
}
