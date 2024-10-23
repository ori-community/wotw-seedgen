pub mod analyzers;
pub mod storage_access;

mod handle_errors;

use std::{
    cmp,
    fmt::Write,
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};

use analyzers::Analyzer;
use handle_errors::HandleErrors;
use itertools::Itertools;
use rustc_hash::FxHashMap;
use storage_access::SeedStorageAccess;
use wotw_seedgen::{
    assets::{SnippetAccess, UberStateData},
    logic_language::output::Graph,
    settings::UniverseSettings,
    spoiler::SeedSpoiler,
};

pub type Result<T> = std::result::Result<T, String>;

pub struct StatsGenerator<
    'settings,
    'graph,
    'uberstates,
    'snippet_access,
    'storage_access,
    A1: SnippetAccess,
    A2: SeedStorageAccess,
> {
    /// The [`UniverseSettings`] to generate seeds with
    pub settings: &'settings UniverseSettings,
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
    /// The logical [`Graph`] passed to seedgen
    ///
    /// You can obtain this from the logic language crate using [`Graph::compile`]
    pub graph: &'graph Graph,
    /// The [`UberStateData`] passed to seedgen
    ///
    /// TODO how can you obtain this
    pub uber_state_data: &'uberstates UberStateData,
    /// The [`SnippetAccess`] passed to seedgen
    ///
    /// TODO how can you obtain this
    pub snippet_access: &'snippet_access A1,
    /// A [`SeedStorageAccess`] implementation
    ///
    /// TODO how can you obtain this
    pub storage_access: &'storage_access A2,
    /// How many errors during seed generation should be tolerated before aborting
    pub tolerated_errors: usize,
    /// How many error messages should be displayed after aborting due to `tolerated_errors` being exceeded
    pub error_message_limit: usize,
    /// If `true`, cleans the seed storage for the provided `settings` and generates new seeds from scratch
    pub overwrite_seed_storage: bool,
}

/// Multiple [`Analyzer`]s chained together
pub type ChainedAnalyzers = Vec<Box<dyn Analyzer>>;

impl<
        'settings,
        'graph,
        'uberstates,
        'snippet_access,
        'storage_access,
        A1: SnippetAccess + Sync,
        A2: SeedStorageAccess + Sync,
    > StatsGenerator<'settings, 'graph, 'uberstates, 'snippet_access, 'storage_access, A1, A2>
{
    pub fn new(
        settings: &'settings UniverseSettings,
        graph: &'graph Graph,
        snippet_access: &'snippet_access A1,
        storage_access: &'storage_access A2,
        uber_state_data: &'uberstates UberStateData,
    ) -> Self {
        Self {
            settings,
            sample_size: 10000,
            analyzers: vec![],
            graph,
            snippet_access,
            storage_access,
            uber_state_data,
            tolerated_errors: 4,
            error_message_limit: 10,
            overwrite_seed_storage: false,
        }
    }

    pub fn sample_size(mut self, sample_size: usize) -> Self {
        self.sample_size = sample_size;
        self
    }
    pub fn tolerated_errors(mut self, tolerated_errors: usize) -> Self {
        self.tolerated_errors = tolerated_errors;
        self
    }
    pub fn error_message_limit(mut self, error_message_limit: usize) -> Self {
        self.error_message_limit = error_message_limit;
        self
    }
    pub fn overwrite_seed_storage(mut self, overwrite_seed_storage: bool) -> Self {
        self.overwrite_seed_storage = overwrite_seed_storage;
        self
    }
    pub fn analyzer(mut self, analyzer: ChainedAnalyzers) -> Self {
        self.analyzers.push(analyzer);
        self
    }

    pub fn generate(self) -> Result<Vec<Stats>> {
        if self.overwrite_seed_storage {
            self.storage_access.clean_all_seeds()?;
            eprintln!("Cleaned seed storage for these settings");
        }

        if self.settings.world_count() > 1 {
            return Err("Multiworld seeds aren't supported yet".to_string());
        }

        let mut data = vec![SeedData::default(); self.analyzers.len()];

        let existing = self.analyze_existing_seeds(&mut data)?;

        if existing < self.sample_size {
            self.generate_missing_seeds(existing, &mut data)?;
        }

        let stats = iter::zip(self.analyzers, data)
            .map(|(analyzers, data)| Stats { analyzers, data })
            .collect();

        Ok(stats)
    }

    fn analyze_existing_seeds(&self, data: &mut [SeedData]) -> Result<usize> {
        let mut existing = HandleErrors::new_print_errors(
            self.storage_access
                .read_seeds(&self.settings, self.sample_size)?,
        );
        let mut existing_amount = 0;

        for seed in &mut existing {
            existing_amount += 1;
            self.analyze_seed(&seed, data);
        }
        print_feedback_for_unusable_seeds(existing.errors);

        Ok(existing_amount)
    }

    fn analyze_seed(&self, seed: &SeedSpoiler, data: &mut [SeedData]) {
        for (data, chained_analyzers) in data.iter_mut().zip(self.analyzers.iter()) {
            chained_analyzers
                .iter()
                .map(|analyzer| analyzer.analyze(seed).into_iter().map(Arc::new))
                .multi_cartesian_product()
                .for_each(|key| *data.entry(key).or_default() += 1);
        }
    }

    fn generate_missing_seeds(&self, existing: usize, data: &mut [SeedData]) -> Result<()> {
        let available = thread::available_parallelism().map_or(4, NonZeroUsize::get);
        let count = AtomicUsize::new(existing);
        let data = Mutex::new(data);
        let errors = AtomicUsize::new(0);
        let error_messages = Mutex::<Vec<String>>::new(vec![]);

        thread::scope(|scope| {
            let count = &count;
            let data = &data;
            let errors = &errors;
            let error_messages = &error_messages;

            iter::repeat_with(|| {
                thread::Builder::new()
                    .name("seedgen".to_string())
                    .spawn_scoped(scope, move || -> Result<()> {
                        let mut settings = self.settings.clone();

                        loop {
                            let count = count.fetch_add(1, Ordering::Relaxed);

                            if count > self.sample_size {
                                break;
                            }

                            // TODO maybe this crate shouldn't print
                            eprint!("Generating seed {count}/{}\r", self.sample_size);

                            settings.seed = rand::random::<u64>().to_string();

                            let seed =
                                self.generate_seed(count, &settings, &errors, &error_messages)?;

                            self.analyze_seed(
                                &seed,
                                &mut data.lock().expect("Another thread panicked"),
                            );
                        }

                        Ok(())
                    })
                    .expect("failed to spawn thread")
            })
            .take(available)
            .collect::<Vec<_>>()
            .into_iter()
            .try_for_each(|handle| handle.join().expect("A seedgen thread panicked"))
        })?;

        print_feedback_for_generated_seeds(self.sample_size - existing, self.sample_size);

        data.into_inner().expect("data was poisoned");
        Ok(())
    }

    fn generate_seed(
        &self,
        count: usize,
        settings: &UniverseSettings,
        errors: &AtomicUsize,
        error_messages: &Mutex<Vec<String>>,
    ) -> Result<SeedSpoiler> {
        let seed = loop {
            match wotw_seedgen::generate_seed(
                self.graph,
                self.uber_state_data,
                self.snippet_access,
                settings,
                false,
            ) {
                Ok(seed) => break seed.spoiler,
                Err(err) => {
                    let mut error_messages_lock =
                        error_messages.lock().expect("Another thread panicked");

                    if error_messages_lock.len() < self.error_message_limit {
                        error_messages_lock.push(err);
                    }
                    let errors = errors.fetch_add(1, Ordering::Relaxed) + 1;
                    if errors > self.tolerated_errors {
                        let more = errors - error_messages_lock.len();
                        let mut error_message = format!(
                            "Too many errors while generating seeds\nSample of some errors:\n{}",
                            error_messages_lock.join("\n")
                        );
                        if more > 0 {
                            write!(error_message, "\n...{more} more").unwrap();
                        }
                        return Err(error_message);
                    }
                }
            }
        };

        if let Err(err) = self.storage_access.write_seed(&seed, &settings, count) {
            eprintln!("{err}");
        };

        Ok(seed)
    }
}

type SeedData = FxHashMap<Vec<Arc<String>>, u32>;

fn print_feedback_for_unusable_seeds(unusable_amount: usize) {
    if unusable_amount > 0 {
        let singular = unusable_amount == 1;
        let plural_s = if singular { "" } else { "s" };
        eprintln!(
            "{} seed{} in storage couldn't be reused, {}replacement{} will be generated",
            unusable_amount,
            plural_s,
            if singular { "a " } else { "" },
            plural_s,
        );
    }
}
fn print_feedback_for_generated_seeds(generated: usize, total: usize) {
    let any_reused = generated < total;
    let mut message = format!(
        "Generated {}{} seed{}",
        if any_reused { "another " } else { "" },
        generated,
        if generated == 1 { "" } else { "s" },
    );
    if any_reused {
        let _ = write!(message, " ({total} total)");
    }

    let length_of_line_to_clear =
        (17 + generated.to_string().len() * 2).saturating_sub(message.len());
    eprintln!("{}{}", message, " ".repeat(length_of_line_to_clear));
}

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
                    cmp::Ordering::Equal => (),
                    non_eq => return non_eq,
                }
            }

            cmp::Ordering::Equal
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
