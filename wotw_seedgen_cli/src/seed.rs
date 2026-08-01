use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{
    cli::{GenerationArgs, LaunchArgs, SeedArgs, SeedSettingsArgs},
    log_config::LogConfig,
    Error,
};
use rand::{distributions::Uniform, prelude::Distribution};
use wotw_seedgen::{
    data::{
        assets::{
            self, file_err, write, AssetFileAccess, DefaultFileAccess, LocData, StateData,
            UberStateData, RANDOMIZER_USER_DATA_DIR, SEEDGEN_USER_DATA_DIR,
        },
        logic_language::{ast::Paths, output::Graph},
        parse::Source,
        UniverseSettings, WorldSettings,
    },
    Generator, SeedUniverse,
};

pub fn seed(args: SeedArgs) -> Result<(), Error> {
    let SeedArgs {
        settings_args,
        generation_args: GenerationArgs { debug, launch },
        verbose_args,
    } = args;

    let start = Instant::now();

    LogConfig::from_args(verbose_args).apply()?;

    let mut settings = settings_args.into_universe_settings()?;

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
    let path = write_seed(seed_universe, name, debug, launch)?;

    eprintln!(
        "Generated seed in {:.1}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        path.display()
    );

    Ok(())
}

impl SeedSettingsArgs {
    pub fn into_universe_settings(self) -> Result<UniverseSettings, Error> {
        let Self {
            stdin_settings,
            settings,
        } = self;

        let mut universe_settings = if stdin_settings {
            serde_json::from_reader(io::stdin().lock())
                .map_err(|err| format!("failed to read settings from stdin: {err}"))?
        } else {
            UniverseSettings::new(String::new())
        };

        settings
            .0
            .apply(&mut universe_settings, &DefaultFileAccess)?;

        Ok(universe_settings)
    }
}

pub fn write_seed(
    seed_universe: SeedUniverse,
    name: &str,
    debug: bool,
    launch: LaunchArgs,
) -> Result<PathBuf, Error> {
    let seeds_dir = SEEDGEN_USER_DATA_DIR.join("seeds");
    assets::create_dir_all(&seeds_dir)?;

    if seed_universe.worlds.len() == 1 {
        let (mut file, path) = create_unique_file(seeds_dir, name, ".wotwr")?;
        let seed = seed_universe.worlds.into_iter().next().unwrap();
        // TODO BufWriter needed on packages to file?
        seed.package(&mut file, !debug)?;

        launch_seed(&path, launch)?;

        let spoiler_path = path.with_extension("spoiler.txt");
        assets::write(&spoiler_path, seed_universe.spoiler.to_string())?;

        Ok(path)
    } else {
        let path = create_unique_dir(seeds_dir, name)?;

        for (index, seed) in seed_universe.worlds.into_iter().enumerate() {
            let path = path.join(format!("world_{index}.wotwr"));
            let mut file = assets::file_create(&path)?;
            seed.package(&mut file, !debug)?;
        }

        let spoiler_path = path.join("spoiler.txt");
        assets::write(&spoiler_path, seed_universe.spoiler.to_string())?;

        Ok(path)
    }
}

fn create_unique_file(dir: PathBuf, name: &str, extension: &str) -> Result<(File, PathBuf), Error> {
    create_unique::<_, File>(dir, name, extension, |path| File::create_new(path))
}

fn create_unique_dir(dir: PathBuf, name: &str) -> Result<PathBuf, Error> {
    create_unique::<_, ()>(dir, name, "", |path| fs::create_dir(path)).map(|((), path)| path)
}

fn create_unique<F, T>(
    mut dir: PathBuf,
    name: &str,
    extension: &str,
    mut f: F,
) -> Result<(T, PathBuf), Error>
where
    F: FnMut(&Path) -> io::Result<T>,
{
    dir.push(format!("{name}{extension}"));

    for attempt in 0_u32.. {
        match f(&dir) {
            Ok(t) => return Ok((t, dir)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(Error(file_err("create", dir, err))),
        }

        dir.set_file_name(format!(
            "{name}_{attempt}{extension}",
            attempt = attempt + 1
        ));
    }

    unreachable!()
}

pub fn launch_seed(path: &Path, args: LaunchArgs) -> Result<(), Error> {
    let LaunchArgs {
        launch,
        new_game_seed_source,
    } = args;

    if new_game_seed_source {
        write_new_game_seed_source(path)?;
    }

    if launch {
        open::that_detached(path).map_err(|err| file_err("launch", path, err))?;
    }

    Ok(())
}

pub fn write_new_game_seed_source(path: &Path) -> Result<(), Error> {
    let path = path.to_str().ok_or_else(|| {
        format!(
            "failed to write newgameseedsource for {}: invalid unicode",
            path.display()
        )
    })?;

    let newgameseedsource = RANDOMIZER_USER_DATA_DIR.join("randomizer/.newgameseedsource");

    write(newgameseedsource, format!("file:{path}"))?;

    Ok(())
}

pub fn generate(settings: &UniverseSettings, debug: bool) -> Result<SeedUniverse, Error> {
    let (graph, loc_data, uber_state_data) = logic_assets(&settings.world_settings)?;

    let seed_universe = Generator::new(
        &graph,
        &loc_data,
        &uber_state_data,
        &DefaultFileAccess,
        settings,
    )
    .with_debug_symbols(debug)
    .generate()?;

    Ok(seed_universe)
}

pub fn paths(source: &Source) -> Result<Paths<'_>, Error> {
    Paths::parse(&source.content)
        .eprint_errors(source)
        .ok_or_else(|| Error("failed to parse paths".to_string()))
}

pub fn graph(
    source: &Source,
    paths: Paths,
    loc_data: LocData,
    state_data: StateData,
    settings: &[WorldSettings],
) -> Result<Graph, Error> {
    Graph::compile(paths, loc_data, state_data, settings)
        .eprint_errors(source)
        .ok_or_else(|| Error("failed to compile graph".to_string()))
}

pub fn logic_assets(settings: &[WorldSettings]) -> Result<(Graph, LocData, UberStateData), Error> {
    let LogicFiles {
        loc_data,
        state_data,
        paths_source: source,
        uber_state_data,
    } = LogicFiles::new()?;

    let paths = paths(&source)?;
    let graph = graph(&source, paths, loc_data.clone(), state_data, settings)?;

    Ok((graph, loc_data, uber_state_data))
}

pub struct LogicFiles {
    pub loc_data: LocData,
    pub state_data: StateData,
    pub paths_source: Source,
    pub uber_state_data: UberStateData,
}

impl LogicFiles {
    pub fn new() -> Result<Self, Error> {
        let loc_data = DefaultFileAccess.loc_data()?;
        let state_data = DefaultFileAccess.state_data()?;
        let paths_source = DefaultFileAccess.paths()?;
        let uber_state_data = DefaultFileAccess.uber_state_data(&loc_data, &state_data)?;

        Ok(LogicFiles {
            loc_data,
            state_data,
            paths_source,
            uber_state_data,
        })
    }
}
