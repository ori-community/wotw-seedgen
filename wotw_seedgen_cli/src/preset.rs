use serde::Serialize;
use wotw_seedgen::data::assets::{
    self, PresetGroup, PresetInfo, UniversePreset, WorldPreset, CURRENT_ASSETS_VERSION, SEEDGEN_USER_DATA_DIR,
};

use crate::{
    cli::{PresetInfoArgs, SeedSettings, SeedWorldSettings, UniversePresetArgs, WorldPresetArgs},
    Error,
};

pub fn universe_preset(args: UniversePresetArgs) -> Result<(), Error> {
    let UniversePresetArgs {
        identifier,
        settings: SeedSettings(settings),
        info_args,
    } = args;

    let universe_preset = UniversePreset {
        assets_version: CURRENT_ASSETS_VERSION,
        info: info_args.into_preset_info(),
        settings,
    };

    write_preset(&identifier, &universe_preset, "universe_presets")
}

pub fn world_preset(args: WorldPresetArgs) -> Result<(), Error> {
    let WorldPresetArgs {
        identifier,
        settings: SeedWorldSettings(settings),
        info_args,
    } = args;

    let world_preset = WorldPreset {
        assets_version: CURRENT_ASSETS_VERSION,
        info: info_args.into_preset_info(),
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

impl PresetInfoArgs {
    fn into_preset_info(self) -> Option<PresetInfo> {
        let preset_info = PresetInfo {
            name: self.display_name,
            description: self.description,
            group: self.base_preset.then_some(PresetGroup::Base),
        };
        (preset_info != PresetInfo::default()).then_some(preset_info)
    }
}
