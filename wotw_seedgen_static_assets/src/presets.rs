use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use wotw_seedgen_assets::{PresetAccess, UniversePreset, WorldPreset};

pub struct StaticPresetAccess {
    universe_presets: FxHashMap<String, UniversePreset>,
    world_presets: FxHashMap<String, WorldPreset>,
}

lazy_static! {
    pub static ref PRESET_ACCESS: StaticPresetAccess = StaticPresetAccess {
        universe_presets: ciborium::from_reader(
            include_bytes!(concat!(env!("OUT_DIR"), "/universe_presets")).as_slice()
        )
        .unwrap(),
        world_presets: ciborium::from_reader(
            include_bytes!(concat!(env!("OUT_DIR"), "/world_presets")).as_slice()
        )
        .unwrap()
    };
}

impl PresetAccess for StaticPresetAccess {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        self.universe_presets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown universe preset \"{identifier}\""))
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        self.world_presets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown world preset \"{identifier}\""))
    }

    fn available_universe_presets(&self) -> Vec<String> {
        self.universe_presets.keys().map(String::clone).collect()
    }

    fn available_world_presets(&self) -> Vec<String> {
        self.world_presets.keys().map(String::clone).collect()
    }
}
