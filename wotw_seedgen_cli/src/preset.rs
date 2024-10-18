use std::fs;

use wotw_seedgen_assets::{
    PresetGroup, PresetInfo, UniversePreset, WorldPreset, CURRENT_ASSETS_VERSION,
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
    let contents = serde_json::to_string_pretty(&universe_preset)?;

    fs::create_dir_all("universe_presets")?;
    fs::write(format!("universe_presets/{identifier}.json"), contents)?;

    Ok(())
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
    let contents = serde_json::to_string_pretty(&world_preset)?;

    fs::create_dir_all("world_presets")?;
    fs::write(format!("world_presets/{identifier}.json"), contents)?;

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
