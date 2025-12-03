#[cfg(test)]
mod tests;

use std::{path::Path, sync::LazyLock};

use rustc_hash::FxHashMap;
use wotw_seedgen_assets::{
    LocData, PresetAccess, SnippetAccess, Source, StateData, UberStateData, UniversePreset,
    WorldPreset,
};
use wotw_seedgen_logic_language::{ast::Areas, output::Graph};

pub static LOC_DATA: LazyLock<LocData> = LazyLock::new(|| {
    ciborium::from_reader(include_bytes!(concat!(env!("OUT_DIR"), "/loc_data")).as_slice()).unwrap()
});

pub static STATE_DATA: LazyLock<StateData> = LazyLock::new(|| {
    ciborium::from_reader(include_bytes!(concat!(env!("OUT_DIR"), "/state_data")).as_slice())
        .unwrap()
});

pub static UBER_STATE_DATA: LazyLock<UberStateData> = LazyLock::new(|| {
    ciborium::from_reader(include_bytes!(concat!(env!("OUT_DIR"), "/uber_state_data")).as_slice())
        .unwrap()
});

pub static AREAS: LazyLock<Areas> = LazyLock::new(|| {
    Areas::parse(include_str!("../../wotw_seedgen/areas.wotw"))
        .parsed
        .unwrap()
});

pub static GRAPH: LazyLock<Graph> = LazyLock::new(|| {
    Graph::compile(AREAS.clone(), LOC_DATA.clone(), STATE_DATA.clone(), &[])
        .parsed
        .unwrap()
});

pub struct StaticSnippetAccess {
    snippets: FxHashMap<String, (String, String)>,
}

pub static SNIPPET_ACCESS: LazyLock<StaticSnippetAccess> = LazyLock::new(|| StaticSnippetAccess {
    snippets: ciborium::from_reader(
        include_bytes!(concat!(env!("OUT_DIR"), "/snippets")).as_slice(),
    )
    .unwrap(),
});

impl SnippetAccess for StaticSnippetAccess {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.snippets
            .get(identifier)
            .cloned()
            .map(|(id, content)| Source::new(id, content))
            .ok_or_else(|| format!("unknown snippet \"{identifier}\""))
    }

    fn read_file(&self, _path: &Path) -> Result<Vec<u8>, String> {
        Err("cannot read arbitrary files with static file access".to_string())
    }

    fn available_snippets(&self) -> Vec<String> {
        self.snippets.keys().map(String::clone).collect()
    }
}

pub struct StaticPresetAccess {
    world_presets: FxHashMap<String, WorldPreset>,
}

pub static PRESET_ACCESS: LazyLock<StaticPresetAccess> = LazyLock::new(|| StaticPresetAccess {
    world_presets: ciborium::from_reader(
        include_bytes!(concat!(env!("OUT_DIR"), "/world_presets")).as_slice(),
    )
    .unwrap(),
});

impl PresetAccess for StaticPresetAccess {
    fn universe_preset(&self, _identifier: &str) -> Result<UniversePreset, String> {
        unimplemented!()
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        self.world_presets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown world preset \"{identifier}\""))
    }

    fn available_universe_presets(&self) -> Vec<String> {
        vec![]
    }

    fn available_world_presets(&self) -> Vec<String> {
        self.world_presets.keys().map(String::clone).collect()
    }
}
