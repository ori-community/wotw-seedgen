use super::cli;
use super::log_init;

use log::LevelFilter;

use wotw_seedgen::files::FILE_SYSTEM_ACCESS;

pub fn create_world_preset(args: cli::WorldPresetArgs) -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Info, false)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    let preset = args.settings.into_world_preset();
    let preset = preset.to_json_pretty();

    FILE_SYSTEM_ACCESS.write_world_preset(&args.filename, &preset)?;
    log::info!("Created world preset {}", args.filename);

    Ok(())
}
