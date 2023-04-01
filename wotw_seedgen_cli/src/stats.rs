use std::fmt::Write;
use std::{fs, path::PathBuf};

use log::LevelFilter;
use sanitize_filename::sanitize;
use wotw_seedgen::settings::Spawn;
use wotw_seedgen::util::constants::DEFAULT_SPAWN;
use wotw_seedgen::world::Graph;
use wotw_seedgen::{files::FILE_SYSTEM_ACCESS, logic, settings::UniverseSettings};
use wotw_seedgen_stats::analyzers::{self, Analyzer};
use wotw_seedgen_stats::{
    files::{FileAccess, FileSystemAccess},
    StatsArgs,
};

use crate::{cli, log_init};

pub fn generate_stats(args: cli::StatsArgs) -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Warn, false)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {err}"));

    let cli::StatsArgs {
        areas,
        locations,
        uber_states,
        sample_size,
        analyzers,
        folder_name,
        tolerated_errors,
        error_message_limit,
        overwrite_cache,
        settings: settings_args,
    } = args;

    let mut settings = UniverseSettings {
        seed: String::default(),
        ..UniverseSettings::default()
    };
    settings
        .apply_preset(settings_args.into_universe_preset()?, &FILE_SYSTEM_ACCESS)
        .map_err(|err| err.to_string())?;

    let areas = fs::read_to_string(&areas)
        .map_err(|err| format!("Failed to read {}: {}", areas.display(), err))?;
    let locations = fs::read_to_string(&locations)
        .map_err(|err| format!("Failed to read {}: {}", locations.display(), err))?;
    let states = fs::read_to_string(&uber_states)
        .map_err(|err| format!("Failed to read {}: {}", uber_states.display(), err))?;
    let graph = logic::parse_logic(&areas, &locations, &states, &settings, false)?;

    let settings_json = settings.to_json();
    let settings_summary = folder_name.unwrap_or_else(|| summarize_settings(&settings, &graph));

    let mut path = PathBuf::from("stats");

    for index in 0.. {
        let mut unique_settings_summary = settings_summary.clone();
        if index > 0 {
            write!(unique_settings_summary, " ({index})").unwrap()
        }

        path.push(&unique_settings_summary);
        fs::create_dir_all(&path).map_err(|err| {
            format!(
                "failed to create folder for statistics at \"{}\": {}",
                path.display(),
                err
            )
        })?;

        let mut settings_path = path.clone();
        settings_path.push("settings.json");

        let may_write =
            fs::read_to_string(settings_path).map_or(true, |previous| previous == settings_json);
        if may_write {
            if index > 0 {
                eprintln!("Encountered previous stats with the same folder name but different settings, renaming to \"{unique_settings_summary}\"");
            }
            break;
        }

        path.pop();
    }

    // Convenience function so we don't have to upcast into the trait object every time
    #[inline]
    fn box_analyzer<'a, A: Analyzer + 'a>(analyzer: A) -> Box<dyn Analyzer + 'a> {
        Box::new(analyzer)
    }

    let analyzers = analyzers
        .into_iter()
        .map(|chained_analyzers| {
            chained_analyzers
                .0
                .into_iter()
                .map(|analyzer| match analyzer {
                    cli::Analyzer::EarlySkills { reachable_limit } => {
                        box_analyzer(analyzers::EarlySkillsStats {
                            reachable_limit: reachable_limit.unwrap_or(50),
                        })
                    }
                    cli::Analyzer::ItemUnlock { item } => {
                        box_analyzer(analyzers::ItemUnlockStats { item })
                    }
                    cli::Analyzer::ItemZone { item } => {
                        box_analyzer(analyzers::ItemZoneStats { item })
                    }
                    cli::Analyzer::Progression => box_analyzer(analyzers::ProgressionStats),
                    cli::Analyzer::SpawnItems => box_analyzer(analyzers::SpawnItemStats),
                    cli::Analyzer::SpawnLocation => box_analyzer(analyzers::SpawnLocationStats),
                    cli::Analyzer::SpawnRegion => box_analyzer(analyzers::SpawnRegionStats),
                    cli::Analyzer::ZoneUnlock { zone } => {
                        box_analyzer(analyzers::ZoneUnlockStats { zone })
                    }
                })
                .collect()
        })
        .collect();

    let args = StatsArgs {
        settings,
        sample_size,
        analyzers,
        graph: &graph,
        tolerated_errors,
        error_message_limit: Some(error_message_limit),
        overwrite_seed_storage: overwrite_cache,
    };
    let stats = wotw_seedgen_stats::stats::<FileSystemAccess>(args)?;

    fs::create_dir_all(&path).map_err(|err| {
        format!(
            "failed to create folder for statistics at \"{}\": {}",
            path.display(),
            err
        )
    })?; // It might've been a while ago that we created this folder, lets check if we need to recreate it in case the user deleted it in the meantime
    for stats in stats {
        let csv = stats.csv();
        let mut path = path.clone();
        path.push(sanitize(stats.title()));
        path.set_extension("csv");
        fs::write(&path, csv).map_err(|err| {
            format!(
                "failed to write statistics to \"{}\": {}",
                path.display(),
                err
            )
        })?;
        eprintln!("Wrote statistics to \"{}\"", path.display());
    }

    path.push("settings.json");
    fs::write(&path, settings_json).map_err(|err| {
        format!(
            "failed to write settings to \"{}\": {}",
            path.display(),
            err
        )
    })?;

    Ok(())
}

pub fn clean_stats_cache() -> Result<(), String> {
    log_init::initialize_log(None, LevelFilter::Warn, false)
        .unwrap_or_else(|err| eprintln!("Failed to initialize log: {err}"));

    wotw_seedgen_stats::files::FileSystemAccess::clean_all_seeds()
}

fn summarize_settings(settings: &UniverseSettings, graph: &Graph) -> String {
    let world_count = settings.world_count();
    let multiworld = world_count > 1;

    let mut summary = String::new();

    macro_rules! write_summary {
        ($($arg:tt)*) => {
            write!(summary, $($arg)*).unwrap();
        };
    }

    if multiworld {
        write_summary!("{world_count} world ");
    }

    let world_settings = &settings.world_settings;
    macro_rules! all {
        ($f:expr) => {
            world_settings.iter().all($f)
        };
    }

    let mut difficulties = world_settings
        .iter()
        .map(|world_settings| world_settings.difficulty.to_string())
        .collect::<Vec<_>>();
    difficulties.sort();
    difficulties.dedup();

    if difficulties.len() <= 3 {
        write_summary!("{} ", difficulties.join("/"));
    }

    if all!(|world_settings| !world_settings.tricks.is_empty()) {
        summary.push_str("Glitches ")
    }

    if all!(|world_settings| matches!(world_settings.spawn, Spawn::Random | Spawn::FullyRandom)) {
        if all!(|world_settings| matches!(world_settings.spawn, Spawn::FullyRandom)) {
            summary.push_str("Fully Random Spawn ")
        } else {
            summary.push_str("Random Spawn ")
        }
    } else if let Spawn::Set(spawn) = &world_settings[0].spawn {
        if spawn != DEFAULT_SPAWN
            && all!(|world_settings| matches!(&world_settings.spawn, Spawn::Set(s) if s == spawn))
        {
            if let Ok(node) = graph.find_spawn(spawn) {
                if let Some(zone) = node.zone() {
                    write_summary!("{zone} Spawn ");
                }
            }
        }
    }

    summary.pop(); // Remove the space we always added for the next piece

    summary
}
