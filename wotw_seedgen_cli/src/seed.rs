use crate::{
    cli::{GenerationArgs, SeedArgs},
    files::{self, write_seed},
    log_config::LogConfig,
    Error,
};
use rand::{distributions::Uniform, prelude::Distribution};
use wotw_seedgen::{
    generate_seed,
    logic_language::{ast, output::Graph},
    settings::{UniverseSettings, WorldSettings},
    SeedUniverse,
};
use wotw_seedgen_assets::{LocData, Source, StateData, UberStateData};

pub fn seed(args: SeedArgs) -> Result<(), Error> {
    let SeedArgs {
        settings,
        generation_args: GenerationArgs { debug, launch },
        verbose_args,
    } = args;

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
    write_seed(seed_universe, name, debug, launch)
}

pub fn generate(settings: &UniverseSettings, debug: bool) -> Result<SeedUniverse, Error> {
    let (graph, uber_state_data) = logic_assets(&settings.world_settings)?;
    let snippet_access = files::snippet_access("")?;

    let seed_universe = generate_seed(&graph, &uber_state_data, &snippet_access, &settings, debug)?;

    Ok(seed_universe)
}

pub fn logic_assets(settings: &[WorldSettings]) -> Result<(Graph, UberStateData), Error> {
    let LogicFiles {
        loc_data,
        state_data,
        areas_source: source,
        uber_state_data,
    } = LogicFiles::new()?;

    let areas = ast::parse(&source.content).into_result()?;

    let (graph, success) =
        Graph::compile(areas, loc_data, state_data, settings).eprint_errors(&source);
    if !success {
        return Err(Error("failed to compile graph".to_string()));
    }

    Ok((graph, uber_state_data))
}

pub struct LogicFiles {
    pub loc_data: LocData,
    pub state_data: StateData,
    pub areas_source: Source,
    pub uber_state_data: UberStateData,
}

impl LogicFiles {
    pub fn new() -> Result<Self, Error> {
        let logic_access = files::logic_access("")?;

        let loc_data = logic_access.loc_data()?;
        let state_data = logic_access.state_data()?;
        let areas_source = logic_access.areas()?;
        let uber_state_data = logic_access.uber_state_data(&loc_data, &state_data)?;

        Ok(LogicFiles {
            loc_data,
            state_data,
            areas_source,
            uber_state_data,
        })
    }
}
