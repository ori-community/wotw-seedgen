use crate::{
    cli::{GenerationArgs, SeedArgs, SeedSettings},
    files,
    log_init::initialize_log,
    Error,
};
use log::LevelFilter;
use rand::{distributions::Uniform, prelude::Distribution};
use std::fs::{self, File};
use wotw_seedgen::{
    generate_seed,
    logic_language::{ast, output::Graph},
    settings::UniverseSettings,
    SeedUniverse,
};
use wotw_seedgen_assets::file_err;

pub fn seed(args: SeedArgs) -> Result<(), Error> {
    let SeedArgs {
        settings: SeedSettings(universe_preset_settings),
        generation_args: GenerationArgs { debug, launch },
        verbose,
    } = args;

    initialize_log(
        verbose.then_some("seedgen_log.txt"),
        LevelFilter::Info,
        false,
    )?;

    let mut settings = UniverseSettings::new("".to_string());
    universe_preset_settings.apply(&mut settings, &files::preset_access("")?)?;
    if settings.seed.is_empty() {
        let distribution = Uniform::from('0'..='9');
        settings.seed = distribution
            .sample_iter(rand::thread_rng())
            .take(12)
            .collect();
    }

    let seed_universe = generate(&settings, debug)?;
    write(seed_universe, debug, launch)
}

pub fn generate(settings: &UniverseSettings, debug: bool) -> Result<SeedUniverse, Error> {
    let logic_access = files::logic_access("")?;
    let loc_data = logic_access.loc_data()?;
    let state_data = logic_access.state_data()?;
    let source = logic_access.areas()?;
    let areas = ast::parse(&source.content).into_result()?;
    let (graph, success) = Graph::compile(
        areas,
        loc_data.clone(),
        state_data.clone(),
        &settings.world_settings,
    )
    .eprint_errors(&source);
    if !success {
        return Err(Error("failed to compile graph".to_string()));
    }

    let uber_state_data = logic_access.uber_state_data(loc_data, state_data)?;
    let snippet_access = files::snippet_access("")?;
    let seed_universe = generate_seed(&graph, &uber_state_data, &snippet_access, &settings, debug)?;

    Ok(seed_universe)
}

pub fn write(mut seed_universe: SeedUniverse, debug: bool, launch: bool) -> Result<(), Error> {
    fs::create_dir_all("seeds")?;
    fs::write("seeds/spoiler.txt", seed_universe.spoiler.to_string())?;

    let path = "seeds/seed.wotwr";
    let seed = seed_universe.worlds.pop().unwrap(); // TODO multiworld output
    let mut file = File::create(path).map_err(|err| file_err("create", path, err))?;
    seed.package(&mut file, !debug)?;

    eprintln!("Generated seed to \"{path}\"");

    if launch {
        open::that_detached(path).map_err(|err| format!("failed to open \"{path}\": {err}"))?;
    }

    Ok(())
}
