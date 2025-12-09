use std::{
    ffi::OsStr,
    fs::{self, File},
    io,
    path::Path,
    time::Instant,
};

use crate::{
    cli::{GenerationArgs, SeedArgs},
    log_config::LogConfig,
    Error,
};
use rand::{distributions::Uniform, prelude::Distribution};
use wotw_seedgen::{
    generate_seed,
    logic_language::{ast::Areas, output::Graph},
    settings::{UniverseSettings, WorldSettings},
    SeedUniverse,
};
use wotw_seedgen_assets::{
    file_err, AssetFileAccess, DefaultFileAccess, LocData, Source, StateData, UberStateData,
};

pub fn seed(args: SeedArgs) -> Result<(), Error> {
    let SeedArgs {
        settings,
        generation_args: GenerationArgs { debug, launch },
        verbose_args,
    } = args;

    let start = Instant::now();

    LogConfig::from_args(verbose_args, "seedgen_log.txt").apply()?;

    let mut settings = settings.into_universe_settings()?;
    let name = if settings.seed.is_empty() {
        let distribution = Uniform::from('0'..='9');
        settings.seed = distribution
            .sample_iter(rand::thread_rng())
            .take(12)
            .collect();

        "seed"
    } else {
        &settings.seed
    };

    let seed_universe = generate(&settings, debug)?;
    write_seed(seed_universe, name, debug, launch, start)
}

fn write_seed(
    mut seed_universe: SeedUniverse,
    name: &str,
    debug: bool,
    launch: bool,
    start: Instant,
) -> Result<(), Error> {
    fs::create_dir_all("seeds")?;

    let path = if seed_universe.worlds.len() == 1 {
        let (mut file, path) = create_unique_file(&format!("seeds/{name}"))?;
        let seed = seed_universe.worlds.pop().unwrap();
        seed.package(&mut file, !debug)?;

        if launch {
            launch_seed(&path)?;
        }

        let spoiler_path = format!("{}_spoiler.txt", &path[..path.len() - ".wotwr".len()]);
        fs::write(&spoiler_path, seed_universe.spoiler.to_string())
            .map_err(|err| file_err("write", &spoiler_path, err))?;

        path
    } else {
        let path = create_unique_dir(&format!("seeds/{name}"))?;

        for (index, seed) in seed_universe.worlds.into_iter().enumerate() {
            let path = format!("{path}/world_{index}.wotwr");
            let mut file = File::create(&path).map_err(|err| file_err("create", path, err))?;
            seed.package(&mut file, !debug)?;
        }

        let spoiler_path = format!("{path}/spoiler.txt");
        fs::write(&spoiler_path, seed_universe.spoiler.to_string())
            .map_err(|err| file_err("write", &spoiler_path, err))?;

        path
    };

    eprintln!(
        "Generated seed in {:.1}s to \"{path}\"",
        start.elapsed().as_secs_f32()
    );

    Ok(())
}

fn create_unique_file(path: &str) -> Result<(File, String), Error> {
    create_unique::<_, File>(path, ".wotwr", |path| File::create_new(path))
}

fn create_unique_dir(path: &str) -> Result<String, Error> {
    create_unique::<_, ()>(path, "", |path| fs::create_dir(path)).map(|(_, path)| path)
}

fn create_unique<F, T>(path: &str, extension: &str, mut f: F) -> Result<(T, String), Error>
where
    F: FnMut(&str) -> io::Result<T>,
{
    for attempt in 0_u32.. {
        let path = if attempt == 0 {
            format!("{path}{extension}")
        } else {
            format!("{path}_{attempt}{extension}")
        };

        match f(&path) {
            Ok(t) => return Ok((t, path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(Error(file_err("create", path, err))),
        }
    }

    unreachable!()
}

pub fn launch_seed<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Result<(), Error> {
    Ok(open::that_detached(&path).map_err(|err| file_err("launch", path, err))?)
}

pub fn generate(settings: &UniverseSettings, debug: bool) -> Result<SeedUniverse, Error> {
    let (graph, loc_data, uber_state_data) = logic_assets(&settings.world_settings)?;

    let seed_universe = generate_seed(
        &graph,
        &loc_data,
        &uber_state_data,
        &DefaultFileAccess,
        settings,
        debug,
    )?;

    Ok(seed_universe)
}

pub fn areas(source: &Source) -> Result<Areas<'_>, Error> {
    Areas::parse(&source.content)
        .eprint_errors(&source)
        .ok_or_else(|| Error("failed to parse areas".to_string()))
}

pub fn graph(
    source: &Source,
    areas: Areas,
    loc_data: LocData,
    state_data: StateData,
    settings: &[WorldSettings],
) -> Result<Graph, Error> {
    Graph::compile(areas, loc_data, state_data, settings)
        .eprint_errors(source)
        .ok_or_else(|| Error("failed to compile graph".to_string()))
}

pub fn logic_assets(settings: &[WorldSettings]) -> Result<(Graph, LocData, UberStateData), Error> {
    let LogicFiles {
        loc_data,
        state_data,
        areas_source: source,
        uber_state_data,
    } = LogicFiles::new()?;

    let areas = areas(&source)?;
    let graph = graph(&source, areas, loc_data.clone(), state_data, settings)?;

    Ok((graph, loc_data, uber_state_data))
}

pub struct LogicFiles {
    pub loc_data: LocData,
    pub state_data: StateData,
    pub areas_source: Source,
    pub uber_state_data: UberStateData,
}

impl LogicFiles {
    pub fn new() -> Result<Self, Error> {
        let loc_data = DefaultFileAccess.loc_data()?;
        let state_data = DefaultFileAccess.state_data()?;
        let areas_source = DefaultFileAccess.areas()?;
        let uber_state_data = DefaultFileAccess.uber_state_data(&loc_data, &state_data)?;

        Ok(LogicFiles {
            loc_data,
            state_data,
            areas_source,
            uber_state_data,
        })
    }
}
