pub mod analyzers;
pub mod files;
mod handle_errors;
mod seed_storage;

use std::{cmp::Ordering, fmt::Write, sync::Arc, time::Instant};

use analyzers::Analyzer;
use files::FileAccess;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use wotw_seedgen::{settings::UniverseSettings, world::Graph};

type Result<T> = std::result::Result<T, String>;

/// Arguments passed to [`stats`]
pub struct StatsArgs<'graph> {
    /// The [`UniverseSettings`] to generate seeds with
    pub settings: UniverseSettings,
    /// How many seeds to analyze
    pub sample_size: usize,
    /// Any number of [`Analyzer`]s that will analyze the seeds
    ///
    /// Each instance of [`ChainedAnalyzers`] will be treated separately (as though you would call [`stats`] multiple times with each of them),
    /// but the [`Analyzer`]s within one [`ChainedAnalyzers`] will be chained together, e.g. chaining [`SpawnLocationStats`] with [`ZoneUnlockStats`]
    /// would analyze the zone unlocks for each spawn
    ///
    /// [`SpawnLocationStats`]: (analyzers::SpawnLocationStats)
    /// [`ZoneUnlockStats`]: (analyzers::ZoneUnlockStats)
    pub analyzers: Vec<ChainedAnalyzers>,
    /// The logical [`Graph`]
    ///
    /// You can obtain this from the seedgen library using [`wotw_seedgen::logic::parse_logic`]
    pub graph: &'graph Graph,
    /// How many errors during seed generation should be tolerated before aborting
    ///
    /// If `None`, this will default to a value based on `sample_size`
    pub tolerated_errors: Option<usize>,
    /// How many error messages should be displayed after aborting due to `tolerated_errors` being exceeded
    ///
    /// If `None`, defaults to 10
    pub error_message_limit: Option<usize>,
    /// If `true`, cleans the seed storage for the provided `settings` and generates new seeds from scratch
    pub overwrite_seed_storage: bool,
}
/// Multiple [`Analyzer`]s chained together
pub type ChainedAnalyzers = Vec<Box<dyn Analyzer>>;

pub struct Stats {
    analyzers: ChainedAnalyzers,
    pub data: FxHashMap<Vec<Arc<String>>, u32>,
}
impl Stats {
    pub fn title(&self) -> String {
        self.analyzers
            .iter()
            .map(|analyzer| analyzer.title())
            .join(" and ")
    }
    pub fn csv(&self) -> String {
        let mut csv = self.title();
        csv.push_str(", Count\n");

        let mut data = self.data.iter().collect::<Vec<_>>();
        data.sort_unstable_by(|(a, _), (b, _)| {
            for ((x, y), analyzer) in a.iter().zip(b.iter()).zip(self.analyzers.iter()) {
                match analyzer.compare_keys()(x, y) {
                    Ordering::Equal => (),
                    non_eq => return non_eq,
                }
            }

            Ordering::Equal
        });

        csv.extend(Itertools::intersperse_with(
            data.into_iter().map(|(keys, value)| {
                let mut data_line = keys.iter().join(", ");
                write!(data_line, ", {value}").unwrap();
                data_line
            }),
            || "\n".to_string(),
        ));

        csv
    }
}

/// Generates a set of stats
///
/// See [`StatsArgs`] for more details on the passed arguments
pub fn stats<F: FileAccess>(args: StatsArgs) -> Result<Vec<Stats>> {
    let now = Instant::now();

    let StatsArgs {
        settings,
        sample_size,
        analyzers,
        graph,
        tolerated_errors,
        error_message_limit,
        overwrite_seed_storage,
    } = args;

    if overwrite_seed_storage {
        F::clean_seeds(&settings)?;
        eprintln!("Cleaned seed storage for these settings");
    }

    if settings.world_count() > 1 {
        return Err("Multiworld seeds aren't well supported yet".to_string());
    }

    let data = seed_storage::analyze::<F>(
        &analyzers,
        &settings,
        sample_size,
        tolerated_errors,
        error_message_limit,
        graph,
    )?;

    let stats = data
        .into_iter()
        .zip(analyzers)
        .map(|(data, analyzers)| Stats { analyzers, data })
        .collect();

    let elapsed = now.elapsed();
    eprintln!("Generated stats in {:.1}s", elapsed.as_secs_f32());

    Ok(stats)
}
