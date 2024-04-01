// TODO mod seed_storage;

use crate::Error;
use itertools::Itertools;
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};
use wotw_seedgen::{
    assets::{LocData, SnippetAccess, Source, StateData, UberStateData},
    logic_language::output::Graph,
    settings::{PresetAccess, UniversePreset, WorldPreset, WorldSettings},
};

pub struct Assets {
    pub loc_data: LocData,
    pub state_data: StateData,
    pub uber_state_data: UberStateData,
}
pub fn read_assets() -> Result<Assets, Error> {
    let loc_data = LocData::from_reader(
        File::open("loc_data.csv").map_err(|err| format!("{err}: loc_data.csv"))?,
    )?;
    let state_data = StateData::from_reader(
        File::open("state_data.csv").map_err(|err| format!("{err}: state_data.csv"))?,
    )?;
    // TODO rename uber_state_dump -> uber_state_data
    let mut uber_state_data = UberStateData::from_reader(
        File::open("uber_state_dump.json").map_err(|err| format!("{err}: uber_state_dump.json"))?,
    )?;
    uber_state_data.add_loc_data(loc_data.clone());
    uber_state_data.add_state_data(state_data.clone());
    Ok(Assets {
        loc_data,
        state_data,
        uber_state_data,
    })
}

pub fn compile_graph(
    loc_data: LocData,
    state_data: StateData,
    settings: &[WorldSettings],
) -> Result<Graph, Error> {
    use wotw_seedgen::logic_language::ast::parse;

    let source = fs::read_to_string("../wotw_seedgen/areas.wotw") // TODO move areas.wotw
        .map_err(|err| format!("{err}: ../wotw_seedgen/areas.wotw"))?;
    let areas = parse(&source).into_result()?; // TODO report errors
    let graph = Graph::compile(areas, loc_data, state_data, settings).into_result()?;
    Ok(graph)
}

pub struct SnippetFileAccess;
impl SnippetAccess for SnippetFileAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        let mut path = PathBuf::from("snippets");
        path.push(identifier);
        path.set_extension("wotws");
        let id = path.to_string_lossy().to_string();

        let content = fs::read_to_string(&path)
            .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;

        Ok(Source { id, content })
    }
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        let mut full_path = PathBuf::from("snippets");
        full_path.push(path);
        fs::read(&full_path).map_err(|err| format!("Failed to read {}: {err}", full_path.display()))
    }
}
impl SnippetFileAccess {
    pub fn available_snippets() -> Result<Vec<String>, String> {
        files_in_folder("snippets", "wotws")
    }
}

pub struct PresetFileAccess;
impl PresetAccess for PresetFileAccess {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        access_preset("universe_presets", identifier)
    }
    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        access_preset("world_presets", identifier)
    }
}
impl PresetFileAccess {
    pub fn available_universe_presets() -> Result<Vec<String>, String> {
        files_in_folder("universe_presets", "json")
    }
    pub fn available_world_presets() -> Result<Vec<String>, String> {
        files_in_folder("world_presets", "json")
    }
}
fn access_preset<T>(folder: &str, identifier: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut path = PathBuf::from(folder);
    path.push(identifier);
    path.set_extension("json");
    serde_json::from_reader(BufReader::new(
        File::open(&path).map_err(|err| format!("Failed to read {}: {err}", path.display()))?,
    ))
    .map_err(|err| format!("Failed to parse {}: {err}", path.display()))
}
fn files_in_folder(folder: &str, extension: &str) -> Result<Vec<String>, String> {
    fs::read_dir(folder)
        .map(|dir| {
            dir.filter_map_ok(|entry| {
                let name = entry.file_name();
                let path = Path::new(&name);
                if path.extension().map_or(false, |ext| ext == extension) {
                    path.file_stem().map(|s| s.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())
        })
        .or_else(|err| match err.kind() {
            io::ErrorKind::NotFound => Ok(Ok(vec![])),
            err => Err(format!("failed to read \"{folder}\" folder: {err}")),
        })?
}
