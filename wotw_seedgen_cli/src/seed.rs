use crate::{files, Error};
use std::fs::{self, File};
use wotw_seedgen::{
    assets::UniversePreset, generate_seed, logic_language::ast, logic_language::output::Graph,
    settings::UniverseSettings,
};

// TODO enable logging

pub(crate) fn seed(settings: UniversePreset) -> Result<(), Error> {
    let universe_preset = settings;
    let mut settings = UniverseSettings::new("".to_string());
    universe_preset.apply(&mut settings, &files::preset_access("")?)?;

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
    let mut seed_universe = generate_seed(&graph, &uber_state_data, &snippet_access, &settings)?;

    fs::create_dir_all("seeds")?;
    fs::write("seeds/spoiler.txt", seed_universe.spoiler.to_string())?;

    let seed = seed_universe.worlds.pop().unwrap();
    seed.package(&mut File::create("seeds/seed.wotwr")?, true)?;

    eprintln!("Generated seed to \"seeds/seed.wotwr\"");

    Ok(())
}
