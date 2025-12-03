use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use rustc_hash::FxHashMap;
use serde::Serialize;
use wotw_seedgen_assets::{LocData, StateData, UberStateData, WorldPreset};

fn main() {
    println!("cargo::rerun-if-changed=../assets");

    let loc_data = loc_data();
    let state_data = state_data();

    uber_state_data(&loc_data, &state_data);

    snippets();

    presets();
}

fn loc_data() -> LocData {
    let loc_data =
        LocData::from_reader(include_bytes!("../assets/loc_data.csv").as_slice()).unwrap();

    write("loc_data", &loc_data);

    loc_data
}

fn state_data() -> StateData {
    let state_data =
        StateData::from_reader(include_bytes!("../assets/state_data.csv").as_slice()).unwrap();

    write("state_data", &state_data);

    state_data
}

fn uber_state_data(loc_data: &LocData, state_data: &StateData) {
    let dump = serde_json::from_slice(include_bytes!("../assets/uber_state_dump.json").as_slice())
        .unwrap();

    let uber_state_data = UberStateData::from_parts(dump, &loc_data, &state_data);

    write("uber_state_data", &uber_state_data);
}

fn snippets() {
    let snippets = fs::read_dir("../assets/snippets")
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("wotws")))
        .map(|path| {
            let identifier = path.file_stem().unwrap().to_str().unwrap().to_string();
            let content = fs::read_to_string(&path).unwrap();

            (identifier, (path, content))
        })
        .collect::<FxHashMap<_, _>>();

    write("snippets", &snippets);
}

fn presets() {
    let presets = fs::read_dir(format!("../assets/world_presets"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("json")))
        .map(|path| {
            let identifier = path.file_stem().unwrap().to_str().unwrap().to_string();
            let preset: WorldPreset =
                serde_json::from_reader(BufReader::new(File::open(&path).unwrap())).unwrap();

            (identifier, preset)
        })
        .collect::<FxHashMap<_, _>>();

    write("world_presets", &presets);
}

fn write<T: Serialize>(path: &str, contents: &T) {
    let file = File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join(path)).unwrap();

    ciborium::into_writer(contents, file).unwrap();
}
