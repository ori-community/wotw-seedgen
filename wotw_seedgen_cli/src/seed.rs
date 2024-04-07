use crate::{
    files::{compile_graph, read_assets, PresetFileAccess, SnippetFileAccess},
    Error,
};
use std::fs::{self, File};
use wotw_seedgen::{
    generate_seed,
    settings::{UniversePreset, UniverseSettings},
};

// TODO enable logging

pub(crate) fn seed(settings: UniversePreset) -> Result<(), Error> {
    let universe_preset = settings;
    let mut settings = UniverseSettings::new("".to_string());
    settings.apply_preset(universe_preset, &PresetFileAccess)?;

    fs::create_dir_all("seeds")?;

    let assets = read_assets()?;
    let graph = compile_graph(assets.loc_data, assets.state_data, &settings.world_settings)?;

    let snippet_access = SnippetFileAccess;
    let mut seed_universe =
        generate_seed(&graph, &assets.uber_state_data, &snippet_access, &settings)?;

    fs::write("seeds/spoiler.txt", seed_universe.spoiler.to_string())?;

    let seed = seed_universe.worlds.pop().unwrap();
    seed.package(&mut File::create("seeds/seed.wotwr")?)?;

    Ok(())
}
