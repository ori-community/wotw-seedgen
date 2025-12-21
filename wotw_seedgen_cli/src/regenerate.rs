use std::fs::File;

use serde::de::DeserializeOwned;
use wotw_seedgen::{
    data::assets::{self, file_err},
    seed::{assembly::Assembly, SeedgenInfo},
};
use zip::{read::ZipFile, ZipArchive};

use crate::{cli::RegenerateArgs, log_config::LogConfig, seed::generate, Error};

pub fn regenerate(args: RegenerateArgs) -> Result<(), Error> {
    let RegenerateArgs { path, verbose_args } = args;

    LogConfig::from_args(verbose_args).apply()?;

    let file = assets::file_open(&path)?;
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

fn by_name<'a>(archive: &'a mut ZipArchive<File>, name: &str) -> Result<ZipFile<'a, File>, Error> {
    Ok(archive
        .by_name(name)
        .map_err(|err| format!("failed to read \"{name}\" from seed: {err}"))?)
}
