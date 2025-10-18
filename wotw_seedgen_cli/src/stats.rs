use std::{fmt::Write, fs, path::PathBuf};

use crate::{
    cli::{self, StatsArgs},
    files,
    seed::logic_assets,
    Error,
};
use itertools::Itertools;
use sanitize_filename::sanitize;
use wotw_seedgen::settings::{Spawn, UniverseSettings, DEFAULT_SPAWN};
use wotw_seedgen_assets::file_err;
use wotw_seedgen_stats::{
    analyzers::{
        Analyzer, EarlySkillsStats, FirstWeaponStats, ItemLocationStats, ItemUnlockStats,
        ItemZoneStats, LocationItemStats, ProgressionStats, SpawnItemCountStats, SpawnItemStats,
        SpawnLocationStats, SpawnRegionStats, StepSizeStats, ZoneSpiritLightStats, ZoneUnlockStats,
    },
    storage_access, ChainedAnalyzers, Stats, StatsGenerator,
};

pub fn stats(args: StatsArgs) -> Result<(), Error> {
    let StatsArgs {
        settings,
        sample_size,
        analyzers,
    } = args;

    let settings = settings.into_universe_settings()?;

    let (graph, loc_data, uber_state_data) = logic_assets(&settings.world_settings)?;
    let snippet_access = files::snippet_access("")?;

    let mut generator = StatsGenerator::new(
        &settings,
        &graph,
        &snippet_access,
        &storage_access::FileAccess,
        &loc_data,
        &uber_state_data,
    )
    .sample_size(sample_size);

    generator.analyzers = analyzers.into_iter().map(From::from).collect();

    let stats = generator.generate()?;
    write_stats(stats, &settings)
}

fn write_stats(stats: Vec<Stats>, settings: &UniverseSettings) -> Result<(), Error> {
    let settings_json = serde_json::to_string(settings)?;
    let settings_summary = summarize_settings(settings);

    let mut path = PathBuf::from("stats");

    for index in 0.. {
        let mut unique_settings_summary = settings_summary.clone();
        if index > 0 {
            write!(unique_settings_summary, " ({index})").unwrap()
        }

        path.push(&unique_settings_summary);
        fs::create_dir_all(&path).map_err(|err| file_err("create", &path, err))?;

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

    for stats in stats {
        let csv = stats.csv();
        let mut path = path.clone();
        path.push(format!("{}.csv", sanitize(stats.title())));
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

fn summarize_settings(settings: &UniverseSettings) -> String {
    let world_count = settings.world_count();
    let multiworld = world_count > 1;

    let mut summary = String::new();

    if multiworld {
        let _ = write!(&mut summary, "{world_count} world ");
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
        let _ = write!(&mut summary, "{} ", difficulties.into_iter().format("/"));
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
            let _ = write!(&mut summary, "{spawn} Spawn ");
        }
    }

    summary.pop(); // Remove the space we always added for the next piece

    summary
}

impl From<cli::ChainedAnalyzers> for ChainedAnalyzers {
    fn from(value: cli::ChainedAnalyzers) -> Self {
        value.0.into_iter().map(From::from).collect()
    }
}

impl From<cli::Analyzer> for Box<dyn Analyzer> {
    fn from(value: cli::Analyzer) -> Self {
        match value {
            cli::Analyzer::EarlySkills { reachable_limit } => {
                Box::new(EarlySkillsStats { reachable_limit })
            }
            cli::Analyzer::FirstWeapon => Box::new(FirstWeaponStats),
            cli::Analyzer::ItemLocation { item } => Box::new(ItemLocationStats { item }),
            cli::Analyzer::ItemUnlock {
                item,
                result_bucket_size,
            } => Box::new(ItemUnlockStats {
                item,
                result_bucket_size,
            }),
            cli::Analyzer::ItemZone { item } => Box::new(ItemZoneStats { item }),
            cli::Analyzer::LocationItem { location } => Box::new(LocationItemStats { location }),
            cli::Analyzer::Progression => Box::new(ProgressionStats),
            cli::Analyzer::SpawnItemCount => Box::new(SpawnItemCountStats),
            cli::Analyzer::SpawnItems => Box::new(SpawnItemStats),
            cli::Analyzer::SpawnLocation => Box::new(SpawnLocationStats),
            cli::Analyzer::SpawnRegion => Box::new(SpawnRegionStats),
            cli::Analyzer::StepSize { result_bucket_size } => {
                Box::new(StepSizeStats { result_bucket_size })
            }
            cli::Analyzer::ZoneSpiritLight {
                zone,
                result_bucket_size,
            } => Box::new(ZoneSpiritLightStats {
                zone,
                result_bucket_size,
            }),
            cli::Analyzer::ZoneUnlock {
                zone,
                result_bucket_size,
            } => Box::new(ZoneUnlockStats {
                zone,
                result_bucket_size,
            }),
        }
    }
}
