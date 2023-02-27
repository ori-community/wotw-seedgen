use super::cli;
use super::log_init;

use log::LevelFilter;

use wotw_seedgen::files::FILE_SYSTEM_ACCESS;

pub fn create_universe_preset(args: cli::UniversePresetArgs) -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Info, false).unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    let mut preset = args.settings.into_universe_preset()?;
    preset.info = args.info.into_preset_info();
    let preset = preset.to_json_pretty();

    FILE_SYSTEM_ACCESS.write_universe_preset(&args.filename, &preset)?;
    log::info!("Created universe preset {}", args.filename);

    Ok(())
}
