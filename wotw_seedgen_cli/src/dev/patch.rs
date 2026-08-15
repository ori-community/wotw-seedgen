use std::time::Instant;

use wotw_seedgen::{
    data::assets::{self, file_err},
    seed::SeedgenInfo,
};
use zip::ZipArchive;

use crate::{
    cli::{dev::PatchArgs, CompileArgs},
    dev::helpers::{json_by_name, read_ngss},
    log_config::LogConfig,
    seed::{generate, launch_seed},
    Error,
};

pub fn patch(args: PatchArgs) -> Result<(), Error> {
    let PatchArgs {
        path,
        generation_args,
        verbose_args,
    } = args;
    let CompileArgs { debug, launch_args } = generation_args.compile_args;

    let start = Instant::now();

    LogConfig::from_args(verbose_args).apply()?;

    let path = match path {
        None => read_ngss()?,
        Some(path) => path,
    };

    let file = assets::file_open(&path)?;
    let mut archive = ZipArchive::new(file).map_err(|err| file_err("read", &path, err))?;
    let seedgen_info = json_by_name::<SeedgenInfo>(&mut archive, "seedgen_info.json")?;

    let seed_universe = generate(&seedgen_info.universe_settings, debug)?;

    if seed_universe.worlds.len() == 1 {
        let seed = seed_universe.worlds.into_iter().next().unwrap();
        // TODO BufWriter needed on packages to file?
        let mut file = assets::file_create(&path)?;
        seed.package(&mut file, !debug)?;

        launch_seed(&path, launch_args)?;
    } else {
        let parent = path.parent().unwrap();

        for (index, seed) in seed_universe.worlds.into_iter().enumerate() {
            let path = parent.join(format!("world_{index}.wotwr"));
            let mut file = assets::file_create(&path)?;
            seed.package(&mut file, !debug)?;
        }
    }

    eprintln!(
        "Patched seed \"{path}\" in {elapsed:.1}s",
        path = path.display(),
        elapsed = start.elapsed().as_secs_f32(),
    );

    Ok(())
}
