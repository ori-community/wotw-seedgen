use std::{fmt::Write as _, marker::PhantomData};
use wotw_seedgen::{
    files::FILE_SYSTEM_ACCESS, generator::SeedSpoiler, settings::UniverseSettings, world::Graph,
};

use crate::{files::FileAccess, handle_errors::HandleErrors, Result};

const DEFAULT_ERROR_MESSAGE_LIMIT: usize = 10;

pub(crate) struct Seeds<'graph, F: FileAccess> {
    sample_size: usize,
    existing: HandleErrors<SeedSpoiler, String, F::Iter, fn(String)>,
    existing_amount: usize,
    exhausted_existing: bool,
    tolerated_errors: Option<usize>,
    seed_factory: SeedFactory<'graph, F>,
    missing: usize,
    finished: bool,
}
impl<'graph, F: FileAccess> Seeds<'graph, F> {
    pub(crate) fn new(
        settings: UniverseSettings,
        sample_size: usize,
        tolerated_errors: Option<usize>,
        error_message_limit: Option<usize>,
        graph: &'graph Graph,
    ) -> Result<Seeds<'graph, F>> {
        let existing = HandleErrors::new(
            F::read_seeds(&settings, sample_size)?,
            (|err| {
                eprintln!("{err}");
            }) as fn(String),
        );
        let error_message_limit = error_message_limit.unwrap_or(DEFAULT_ERROR_MESSAGE_LIMIT);
        let seed_factory = SeedFactory::<F>::new(
            0, // Initialized after exhausting existing
            0, // Initialized after exhausting existing
            error_message_limit,
            settings,
            graph,
        );
        Ok(Self {
            sample_size,
            existing,
            existing_amount: 0,
            exhausted_existing: false,
            tolerated_errors,
            seed_factory,
            missing: 0, // Initialized after exhausting existing
            finished: false,
        })
    }
}
impl<F: FileAccess> Iterator for Seeds<'_, F> {
    type Item = SeedSpoiler;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if !self.exhausted_existing {
            let next = self.existing.next();
            match next {
                Some(seed) => {
                    self.existing_amount += 1;
                    return Some(seed);
                }
                None => {
                    self.exhausted_existing = true;
                    print_feedback_for_unusable_seeds(self.existing.errors);
                    self.missing = self.sample_size.saturating_sub(self.existing_amount);
                    self.seed_factory.amount = self.missing;
                    self.seed_factory.tolerated_errors = self
                        .tolerated_errors
                        .unwrap_or_else(|| usize::max(self.missing.saturating_mul(4), 100));
                }
            }
        }
        let next = self.seed_factory.next();
        if next.is_none() && self.missing > 0 {
            print_feedback_for_generated_seeds(self.missing, self.sample_size);
        }
        next
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
struct SeedFactory<'graph, F: FileAccess> {
    amount: usize,
    successes: usize,
    errors: usize,
    error_messages: Vec<String>,
    error_message_limit: usize,
    tolerated_errors: usize,
    failed: bool,
    write_errors: usize,
    printed_write_error_count: bool,
    settings: UniverseSettings,
    graph: &'graph Graph,
    file_access: PhantomData<F>,
}
impl<'graph, F: FileAccess> SeedFactory<'graph, F> {
    fn new(
        amount: usize,
        tolerated_errors: usize,
        error_message_limit: usize,
        settings: UniverseSettings,
        graph: &'graph Graph,
    ) -> SeedFactory<'graph, F> {
        SeedFactory {
            amount,
            successes: 0,
            errors: 0,
            error_messages: vec![],
            error_message_limit,
            tolerated_errors,
            failed: false,
            write_errors: 0,
            printed_write_error_count: false,
            settings,
            graph,
            file_access: PhantomData::default(),
        }
    }
}
impl<F: FileAccess> Iterator for SeedFactory<'_, F> {
    type Item = SeedSpoiler;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.failed {
            while self.successes < self.amount {
                eprint!("Generating seed {}/{}\r", self.successes + 1, self.amount);
                self.settings.seed = rand::random::<u64>().to_string();
                let seed = match wotw_seedgen::generate_seed(
                    self.graph,
                    &FILE_SYSTEM_ACCESS,
                    &self.settings,
                ) {
                    Ok(seed) => seed.spoiler,
                    Err(err) => {
                        if self.error_messages.len() < self.error_message_limit {
                            self.error_messages.push(err);
                        }
                        self.errors += 1;
                        if self.errors >= self.tolerated_errors {
                            let more = self.errors - self.error_messages.len();
                            eprint!(
                            "Too many errors while generating seeds\nSample of some errors:\n{}",
                            self.error_messages.join("\n")
                        );
                            if more > 0 {
                                eprint!("\n...{more} more");
                            }
                            eprintln!();
                            self.failed = true;
                            return None;
                        }
                        continue;
                    }
                };
                if let Err(err) = F::write_seed(&seed, &self.settings) {
                    if self.write_errors < 10 {
                        eprintln!("{err}");
                    }
                    self.write_errors += 1;
                };
                self.successes += 1;
                return Some(seed);
            }
        }
        if !self.printed_write_error_count && self.write_errors > 10 {
            let more = self.write_errors - 10;
            eprintln!(
                "...{} more error{} omitted",
                more,
                if more == 1 { "" } else { "s" }
            );
            self.printed_write_error_count = true;
        }
        None
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
