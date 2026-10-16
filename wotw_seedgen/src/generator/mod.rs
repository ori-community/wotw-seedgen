pub mod item_pool;
pub mod perf_data;
pub mod spoiler;

mod entrances;
mod placement;
mod solutions;
mod spawn;
mod spirit_light;

use crate::{
    generator::{placement::Context, spoiler::SeedSpoiler},
    perf_data::PerfData,
    world::World,
};
use itertools::Itertools;
use log::{info, trace, warn, Level};
use rand_pcg::Pcg64Mcg;
use rand_seeder::Seeder;
use std::{fmt::Display, sync::LazyLock};
use wotw_seedgen_data::{
    assets::{ChainedSnippetAccess, LocData, LocDataEntry, SnippetAccess, UberStateData},
    env_or,
    logic_language::output::Graph,
    parse::Severity,
    seed_language::{compile::Compiler, output::IntermediateOutput, simulate::UberStates},
    UniverseSettings, WorldSettings,
};
use wotw_seedgen_log_capture::{LogCapture, NO_LOG_CAPTURE};
use wotw_seedgen_seed::Seed;

/// How many retries to allow when generating a seed
///
/// Retries are mostly expected when using items that seedgen cannot force-place such as launch fragments.
static RETRIES: LazyLock<u32> = LazyLock::new(|| env_or("RETRIES", 10));

pub struct Generator<'graph, 'loc_data, 'uber_state_data, 'access, 'settings, 'perf, 'log, A> {
    pub graph: &'graph Graph,
    pub loc_data: &'loc_data LocData,
    pub uber_state_data: &'uber_state_data UberStateData,
    pub snippet_access: &'access A,
    pub settings: &'settings UniverseSettings,
    pub debug: bool,
    pub perf_data: Option<&'perf PerfData<'graph>>,
    pub log_capture: &'log LogCapture,
}

impl<'graph, 'loc_data, 'uber_state_data, 'access, 'settings, 'perf, 'log, A>
    Generator<'graph, 'loc_data, 'uber_state_data, 'access, 'settings, 'perf, 'log, A>
where
    A: SnippetAccess,
{
    pub fn new(
        graph: &'graph Graph,
        loc_data: &'loc_data LocData,
        uber_state_data: &'uber_state_data UberStateData,
        snippet_access: &'access A,
        settings: &'settings UniverseSettings,
    ) -> Self {
        Self {
            graph,
            loc_data,
            uber_state_data,
            snippet_access,
            settings,
            debug: false,
            perf_data: None,
            log_capture: &NO_LOG_CAPTURE,
        }
    }

    pub fn with_debug_symbols(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_perf_data(mut self, perf_data: &'perf PerfData<'graph>) -> Self {
        self.perf_data = Some(perf_data);
        self
    }

    pub fn with_log_capture(mut self, log_capture: &'log LogCapture) -> Self {
        self.log_capture = log_capture;
        self
    }

    pub fn generate(&self) -> Result<SeedUniverse, String> {
        self.settings.verify()?;

        trace!(
            logger: self.log_capture,
            "Generating with settings: {}",
            serde_json::to_string(self.settings).unwrap(),
        );

        let mut rng: Pcg64Mcg = Seeder::from(&self.settings.seed).into_rng();

        let snippet_outputs = self
            .settings
            .world_settings
            .iter()
            .map(|world_settings| {
                let snippet_access =
                    ChainedSnippetAccess::new(&self.settings.inline_snippets, self.snippet_access);

                let compiler = Compiler::new(
                    &mut rng,
                    &snippet_access,
                    self.loc_data,
                    self.uber_state_data,
                )
                .with_config(world_settings.snippet_config.clone())
                .with_lint(true)
                .with_debug(self.debug)
                .with_log_capture(self.log_capture);

                // TODO this is inefficient because we probably do a lot of redundant work between the worlds
                let output = parse_snippets(
                    world_settings,
                    compiler,
                    self.log_capture.capture_enabled(Level::Error),
                )?;

                Ok((world_settings, output))
            })
            .collect::<Result<Vec<_>, String>>()?;

        let retries = *RETRIES;
        for attempt in 1..=retries {
            trace!(logger: self.log_capture, "Attempt #{attempt} to generate");

            let worlds = snippet_outputs
                .iter()
                .map(|(world_settings, output)| {
                    if output.preload.spawn.is_some() {
                        warn!(logger: self.log_capture, "A Snippet attempted to set spawn");
                    }

                    let mut output = output.clone();
                    let uber_states = UberStates::new(self.uber_state_data);

                    let world = World::new(
                        self.graph,
                        0,
                        world_settings,
                        uber_states,
                        output.commands.events_mut(),
                    )
                    .with_perf_data(self.perf_data)
                    .with_log_capture(self.log_capture);

                    Ok((world, output))
                })
                .collect::<Result<Vec<_>, String>>()?;

            let context = Context::new(&mut rng, worlds, self.settings, self.log_capture)?;

            match context.attempt_placements(self.loc_data, self.debug) {
                Ok(seed) => {
                    if attempt > 1 {
                        info!(
                            logger: self.log_capture,
                            "Generated seed after {attempt} attempts{}",
                            if attempt <= retries / 2 {
                                ""
                            } else {
                                " (phew)"
                            }
                        );
                    }

                    return Ok(seed);
                }
                Err(err) => warn!(logger: self.log_capture, "{err}"),
            }
        }

        Err(format!(
            "All {retries} attempts to generate a seed failed :("
        ))
    }
}

/// End Result of seed generation
pub struct SeedUniverse {
    /// Seed data per world
    pub worlds: Vec<Seed>,
    /// Spoiler data for the generation process
    pub spoiler: SeedSpoiler,
}

fn parse_snippets<'log>(
    world_settings: &WorldSettings,
    mut compiler: Compiler<'_, '_, 'log>,
    error_capture_enabled: bool,
) -> Result<IntermediateOutput<'log>, String> {
    compiler.compile_snippet("seed_core")?;

    for identifier in &world_settings.snippets {
        compiler.compile_snippet(identifier)?;
    }

    let result = compiler.finish();

    let failed_snippets = result
        .errors
        .iter()
        .filter(|(_, (_, errors))| {
            errors
                .iter()
                .any(|error| error.kind.severity() == Severity::Error)
        })
        .map(|(identifier, _)| identifier.clone())
        .collect::<Vec<_>>();

    let output = if error_capture_enabled {
        failed_snippets.is_empty().then_some(result.output)
    } else {
        result.eprint_errors()
    };

    output.ok_or_else(|| {
        if failed_snippets.len() == 1 {
            format!("failed to compile \"{}\"", failed_snippets.first().unwrap())
        } else {
            format!(
                "failed to compile {amount} snippets: \"{identifiers}\"",
                amount = failed_snippets.len(),
                identifiers = failed_snippets.iter().format("\", \"")
            )
        }
    })
}

fn format_pickups<'a, 'graph>(
    pickups: &'a [&'graph LocDataEntry],
) -> impl Display + use<'a, 'graph> {
    pickups.iter().map(|pickup| &pickup.identifier).format(", ")
}
