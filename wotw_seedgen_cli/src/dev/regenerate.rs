use std::{fs::File, time::Instant};

use log::{trace, warn};
use serde::de::DeserializeOwned;
use wotw_seedgen::{
    data::assets::{self, file_err},
    seed::{assembly::Assembly, SeedgenInfo},
};
use wotw_seedgen_git_info::{GitInfo, GIT_HEAD, GIT_STATUS};
use zip::{read::ZipFile, ZipArchive};

use crate::{
    cli::{dev::RegenerateArgs, CompileArgs},
    log_config::LogConfig,
    seed::{generate, write_new_game_seed_source, write_seed},
    Error,
};

pub fn regenerate(args: RegenerateArgs) -> Result<(), Error> {
    let RegenerateArgs {
        path,
        generation_args,
        verbose_args,
    } = args;
    let CompileArgs { debug, launch_args } = generation_args.compile_args;

    let start = Instant::now();

    LogConfig::from_args(verbose_args).apply()?;

    let file = assets::file_open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| file_err("read", &path, err))?;
    let seedgen_info = json_by_name::<SeedgenInfo>(&mut archive, "seedgen_info.json")?;

    check_git_info(seedgen_info.git_info);

    let assembly = json_by_name::<Assembly>(&mut archive, "assembly.json")?;

    let seed_universe = generate(&seedgen_info.universe_settings, debug)?;
    if assembly != seed_universe.worlds[seedgen_info.world_index].assembly {
        return Err(Error("Regenerated seed did not match".to_string()));
    }

    let path = if debug || launch_args.launch {
        let name = format!("{}_regenerate", path.file_stem().unwrap().display());

        Some(write_seed(seed_universe, &name, generation_args)?)
    } else {
        if launch_args.new_game_seed_source {
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

fn check_git_info(git_info: Option<GitInfo>) {
    let Some(GitInfo { head, status }) = git_info else {
        warn!("This seed contains no git information, it probably cannot be regenerated!");

        return;
    };

    if !status.is_empty() {
        warn!("The seedgen used to generate this seed did not have a clean git status!");
        trace!("Seed's git status:\n{status}");
    }

    if !GIT_STATUS.is_empty() {
        warn!("The currently running seedgen does not have a clean git status!");
        trace!("Current seedgen's git status:\n{status}");
    }

    if head != GIT_HEAD {
        warn!("The seedgen used to generate this seed was on commit {head}, but the seedgen currently running is on commit {GIT_HEAD}");
    }
}
