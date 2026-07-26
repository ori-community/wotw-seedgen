use std::{fs::File, time::Instant};

use serde::de::DeserializeOwned;
use wotw_seedgen::{
    data::assets::{self, file_err},
    seed::{assembly::Assembly, SeedgenInfo},
};
use zip::{read::ZipFile, ZipArchive};

use crate::{
    cli::{dev::RegenerateArgs, GenerationArgs},
    log_config::LogConfig,
    seed::{generate, write_new_game_seed_source, write_seed},
    Error,
};

pub fn regenerate(args: RegenerateArgs) -> Result<(), Error> {
    let RegenerateArgs {
        path,
        generation_args: GenerationArgs { debug, launch },
        verbose_args,
    } = args;

    let start = Instant::now();

    LogConfig::from_args(verbose_args).apply()?;

    let file = assets::file_open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| file_err("read", &path, err))?;
    let seedgen_info = json_by_name::<SeedgenInfo>(&mut archive, "seedgen_info.json")?;
    let assembly = json_by_name::<Assembly>(&mut archive, "assembly.json")?;

    // TODO compare seedgen commit hash

    let seed_universe = generate(&seedgen_info.universe_settings, debug)?;
    if assembly != seed_universe.worlds[seedgen_info.world_index].assembly {
        return Err(Error("Regenerated seed did not match".to_string()));
    }

    let path = if debug || launch.launch {
        let name = format!("{}_regenerate", path.file_stem().unwrap().display());

        Some(write_seed(seed_universe, &name, debug, launch)?)
    } else {
        if launch.new_game_seed_source {
            write_new_game_seed_source(&path)?;
        }

        None
    };

    eprint!("Regenerated seed in {:.1}s", start.elapsed().as_secs_f32());

    if let Some(path) = path {
        eprint!(" to \"{}\"", path.display());
    }

    eprintln!();

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
