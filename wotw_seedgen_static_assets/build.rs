fn main() {
    #[allow(unused)]
    let loc_data = {
        println!("cargo::rerun-if-changed=../assets/loc_data.csv");

        let loc_data = wotw_seedgen_assets::LocData::from_reader(
            include_bytes!("../assets/loc_data.csv").as_slice(),
        )
        .unwrap();

        write("loc_data", &loc_data);
        loc_data
    };

    #[allow(unused)]
    let state_data = {
        println!("cargo::rerun-if-changed=../assets/state_data.csv");

        let state_data = wotw_seedgen_assets::StateData::from_reader(
            include_bytes!("../assets/state_data.csv").as_slice(),
        )
        .unwrap();

        write("state_data", &state_data);
        state_data
    };

    {
        println!("cargo::rerun-if-changed=../assets/uber_state_dump.json");

        let dump =
            serde_json::from_slice(include_bytes!("../assets/uber_state_dump.json").as_slice())
                .unwrap();

        let uber_state_data =
            wotw_seedgen_assets::UberStateData::from_parts(dump, &loc_data, &state_data);

        write("uber_state_data", &uber_state_data);
    }

    {
        println!("cargo::rerun-if-changed=../assets/snippets");

        use itertools::Itertools;
        use rustc_hash::FxHashMap;
        use std::path::PathBuf;

        let snippets = read_optional_dir("../assets/snippets")
            .into_iter()
            .flatten()
            .filter_map_ok(|entry| {
                let name = PathBuf::from(entry.file_name());
                if name.extension()? != "wotws" {
                    return None;
                }

                let id = entry.path();
                let content = std::fs::read_to_string(entry.path()).unwrap();
                let identifier = name.file_stem()?.to_string_lossy().to_string();

                Some((identifier, (id.to_string_lossy().to_string(), content)))
            })
            .collect::<Result<FxHashMap<_, _>, _>>()
            .unwrap();

        write("snippets", &snippets);
    }

    {
        // TODO create logic folder
        println!("cargo::rerun-if-changed=../assets/logic");

        use itertools::Itertools;
        use std::path::PathBuf;

        let logic = read_optional_dir("../assets/logic")
            .into_iter()
            .flatten()
            .filter_map_ok(|entry| {
                let name = PathBuf::from(entry.file_name());
                if name.extension()? != "wotws" {
                    return None;
                }
                let id = entry.path();
                let content = std::fs::read_to_string(entry.path()).unwrap();
                Some((id.to_string_lossy().to_string(), content))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        write("logic", &logic);
    }

    {
        // cargo will always rerun this build script if told to scan a directory that doesn't exist, so this needs to be inactive as long as there are no universe presets in the assets
        // println!("cargo::rerun-if-changed=../assets/universe_presets");
        println!("cargo::rerun-if-changed=../assets/world_presets");

        use itertools::Itertools;
        use rustc_hash::FxHashMap;
        use serde::{Deserialize, Serialize};
        use std::{fs::File, io::BufReader, path::PathBuf};
        use wotw_seedgen_assets::{UniversePreset, WorldPreset};

        fn process_presets<T: Serialize + for<'de> Deserialize<'de>>(folder: &str) {
            let presets = read_optional_dir(format!("../assets/{folder}"))
                .into_iter()
                .flatten()
                .filter_map_ok(|entry| {
                    let name = PathBuf::from(entry.file_name());
                    if name.extension()? != "json" {
                        return None;
                    }

                    let preset: T =
                        serde_json::from_reader(BufReader::new(File::open(entry.path()).unwrap()))
                            .unwrap();
                    let identifier = name.file_stem()?.to_string_lossy().to_string();

                    Some((identifier, preset))
                })
                .collect::<Result<FxHashMap<_, _>, _>>()
                .unwrap();

            write(folder, &presets);
        }

        process_presets::<UniversePreset>("universe_presets");
        process_presets::<WorldPreset>("world_presets");
    }
}

fn read_optional_dir<P: AsRef<std::path::Path>>(path: P) -> Option<std::fs::ReadDir> {
    match std::fs::read_dir(path) {
        Ok(read_dir) => Some(read_dir),
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                None
            } else {
                panic!("{err:?}");
            }
        }
    }
}

fn write<T: serde::Serialize>(path: &str, contents: &T) {
    use std::{env, fs::File, path::Path};

    let file = File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join(path)).unwrap();

    ciborium::into_writer(contents, file).unwrap();
}
