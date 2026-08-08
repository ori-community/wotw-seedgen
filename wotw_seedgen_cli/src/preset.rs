use serde::Serialize;
use wotw_seedgen::data::assets::{
    self, PresetInfo, UniversePreset, WorldPreset, CURRENT_ASSETS_VERSION, SEEDGEN_USER_DATA_DIR,
};

use crate::{
    cli::{PresetInfoArgs, SeedSettings, SeedWorldSettings, UniversePresetArgs, WorldPresetArgs},
    Error,
};

pub fn universe_preset(args: UniversePresetArgs) -> Result<(), Error> {
    let UniversePresetArgs {
        settings: SeedSettings(settings),
        info_args,
    } = args;

    let (identifier, info) = info_args.into_inner();

    let universe_preset = UniversePreset {
        assets_version: CURRENT_ASSETS_VERSION,
        info,
        settings,
    };

    write_preset(&identifier, &universe_preset, "universe_presets")
}

pub fn world_preset(args: WorldPresetArgs) -> Result<(), Error> {
    let WorldPresetArgs {
        settings: SeedWorldSettings(settings),
        info_args,
    } = args;

    let (identifier, info) = info_args.into_inner();

    let world_preset = WorldPreset {
        assets_version: CURRENT_ASSETS_VERSION,
        info,
        settings,
    };

    write_preset(&identifier, &world_preset, "world_presets")
}

fn write_preset<T: Serialize>(identifier: &str, preset: &T, dir: &str) -> Result<(), Error> {
    let contents = serde_json::to_string_pretty(preset)?;

    let mut preset_dir = SEEDGEN_USER_DATA_DIR.join(dir);
    assets::create_dir_all(&preset_dir)?;

    preset_dir.push(format!("{identifier}.json"));
    assets::write(&preset_dir, contents)?;

    Ok(())
}

impl<const UNIVERSE: bool> PresetInfoArgs<UNIVERSE> {
    fn into_inner(self) -> (String, Option<PresetInfo>) {
        (
            self.identifier,
            (self.info != PresetInfo::default()).then_some(self.info),
        )
    }
}
