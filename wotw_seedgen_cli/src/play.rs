use super::log_init;

use std::fs;

use log::LevelFilter;

pub fn play() -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Info, false)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {}", err));

    play_last_seed()
}

pub fn play_last_seed() -> Result<(), String> {
    let last_seed = fs::read_to_string(".currentseedpath").map_err(|err| {
        format!(
            "Failed to read last generated seed from .currentseedpath: {}",
            err
        )
    })?;
    log::info!("Launching seed {}", last_seed);
    open::that(last_seed).map_err(|err| format!("Failed to launch seed: {}", err))?;
    Ok(())
}
