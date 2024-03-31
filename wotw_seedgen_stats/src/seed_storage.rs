use std::{
    fmt::Write,
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
};

use itertools::Itertools;
use wotw_seedgen::{
    assets::{SnippetAccess, UberStateData},
    logic_language::output::Graph,
    settings::UniverseSettings,
    spoiler::SeedSpoiler,
};

use rustc_hash::FxHashMap;

use crate::{files::SeedStorageAccess, handle_errors::HandleErrors, ChainedAnalyzers, Result};

const DEFAULT_ERROR_MESSAGE_LIMIT: usize = 10;
const ANOTHER_THREAD_PANICKED: &str = "Another thread panicked";

pub(crate) type SeedData = FxHashMap<Vec<Arc<String>>, u32>;
pub(crate) fn analyze<F: SeedStorageAccess, SA: SnippetAccess + Sync>(
    analyzers: &[ChainedAnalyzers],
    settings: &UniverseSettings,
    sample_size: usize,
    tolerated_errors: Option<usize>,
    error_message_limit: Option<usize>,
    graph: &Graph,
    snippet_access: &SA,
    uber_state_data: &UberStateData,
) -> Result<Vec<SeedData>> {
    let mut data = iter::repeat(FxHashMap::<_, u32>::default())
        .take(analyzers.len())
        .collect::<Vec<_>>();

    let existing_amount = analyze_existing_seeds::<F>(analyzers, settings, sample_size, &mut data)?;

    let missing = sample_size.saturating_sub(existing_amount);
    if missing > 0 {
        let available = thread::available_parallelism().map_or(4, NonZeroUsize::get);
        // TODO simplify this to a shared usize that all threads take from
        let mut remainder = iter::repeat(1).take(missing % available);
        let sample_size_per_thread = iter::repeat(missing / available)
            .map(|sample_size| sample_size + remainder.next().unwrap_or(0))
            .take(available);

        let data = Mutex::new(data);
        let count = AtomicUsize::new(0);
        let tolerated_errors =
            tolerated_errors.unwrap_or_else(|| usize::max(missing.saturating_mul(4), 100));
        let errors = AtomicUsize::new(0);
        let error_message_limit = error_message_limit.unwrap_or(DEFAULT_ERROR_MESSAGE_LIMIT);
        let error_messages = Mutex::new(vec![]);
        let write_errors = AtomicUsize::new(0);

        std::thread::scope(|scope| {
            let data = &data;
            let count = &count;
            let errors = &errors;
            let error_messages = &error_messages;
            let write_errors = &write_errors;

            // we collect to spawn all the threads
            #[allow(clippy::needless_collect)]
            let handles = sample_size_per_thread.map(|thread_sample_size| {
                thread::Builder::new().name("seedgen".to_string()).spawn_scoped(scope, move || {
                    let mut settings = settings.clone();

                    for _ in 0..thread_sample_size {
                        let count = count.fetch_add(1, Ordering::Relaxed) + 1;
                        eprint!("Generating seed {}/{}\r", count, missing);

                        settings.seed = rand::random::<u64>().to_string();

                        let seed = loop {
                            match wotw_seedgen::generate_seed(
                                graph,
                                uber_state_data,
                                snippet_access,
                                &settings,
                            ) {
                                Ok(seed) => break seed.spoiler,
                                Err(err) => {
                                    let mut error_messages_lock = error_messages.lock().expect(ANOTHER_THREAD_PANICKED);

                                    if error_messages_lock.len() < error_message_limit {
                                        error_messages_lock.push(err);
                                    }
                                    let errors = errors.fetch_add(1, Ordering::Relaxed) + 1;
                                    if errors > tolerated_errors {
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

                        if let Err(err) =
                            F::write_seed(&seed, &settings, existing_amount + count)
                        {
                            let write_errors = write_errors.fetch_add(1, Ordering::Relaxed);
                            if write_errors < 10 {
                                eprintln!("{err}");
                            }
                        };

                        analyze_seed(&seed, analyzers, &mut data.lock().expect(ANOTHER_THREAD_PANICKED));
                    }

                    Ok(())
                }).expect("failed to create thread")
            }).collect::<Vec<_>>();

            handles
                .into_iter()
                .try_for_each(|handle| handle.join().expect("A seedgen thread panicked"))
        })?;

        print_feedback_for_generated_seeds(missing, sample_size);

        Ok(data.into_inner().expect("data was poisoned"))
    } else {
        Ok(data)
    }
}

fn analyze_existing_seeds<F: SeedStorageAccess>(
    analyzers: &[ChainedAnalyzers],
    settings: &UniverseSettings,
    sample_size: usize,
    data: &mut [SeedData],
) -> Result<usize> {
    let mut existing = HandleErrors::new_print_errors(F::read_seeds(settings, sample_size)?);
    let mut existing_amount = 0;

    for seed in existing.by_ref() {
        existing_amount += 1;
        analyze_seed(&seed, analyzers, data);
    }
    print_feedback_for_unusable_seeds(existing.errors);

    Ok(existing_amount)
}
fn analyze_seed(seed: &SeedSpoiler, analyzers: &[ChainedAnalyzers], data: &mut [SeedData]) {
    for (data, chained_analyzers) in data.iter_mut().zip(analyzers.iter()) {
        chained_analyzers
            .iter()
            .map(|analyzer| analyzer.analyze(seed).into_iter().map(Arc::new))
            .multi_cartesian_product()
            .for_each(|key| *data.entry(key).or_default() += 1);
    }
}

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
