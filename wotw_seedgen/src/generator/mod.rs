pub mod item_pool;
pub mod perf_data;
pub mod spoiler;

mod entrances;
mod placement;
mod solutions;
mod spawn;
mod spirit_light;

use crate::{generator::spoiler::SeedSpoiler, perf_data::PerfData, world::World};
use log::{info, trace, warn};
use rand_pcg::Pcg64Mcg;
use rand_seeder::Seeder;
use std::iter;
use wotw_seedgen_data::{
    assets::{ChainedSnippetAccess, LocData, SnippetAccess, UberStateData},
    logic_language::output::Graph,
    seed_language::{compile::Compiler, output::IntermediateOutput, simulate::UberStates},
    UniverseSettings, WorldSettings,
};
use wotw_seedgen_log_capture::{LogCapture, NO_LOG_CAPTURE};
use wotw_seedgen_seed::Seed;

const RETRIES: u16 = 10; // How many retries to allow when generating a seed
const SEED_FAILED_MESSAGE: &str = "Failed to seed child RNG";

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
        let mut rng: Pcg64Mcg = Seeder::from(&self.settings.seed).make_rng();
        trace!(
            logger: self.log_capture,
            "Seeded RNG with \"{}\"",
            self.settings.seed
        );

        let snippet_outputs = self
            .settings
            .world_settings
            .iter()
            .map(|world_settings| {
                let snippet_access =
                    ChainedSnippetAccess::new(&world_settings.inline_snippets, self.snippet_access);

                let compiler = Compiler::new(&mut rng, &snippet_access, self.uber_state_data)
                    .with_config(world_settings.snippet_config.clone())
                    .with_lint(true)
                    .with_debug(self.debug)
                    .with_log_capture(self.log_capture);

                // TODO this is inefficient because we probably do a lot of redundant work between the worlds
                let output = parse_snippets(world_settings, compiler)?;

                Ok((world_settings, output))
            })
            .collect::<Result<Vec<_>, String>>()?;

        for attempt in 1..=RETRIES {
            trace!(logger: self.log_capture, "Attempt #{attempt} to generate");

            let worlds = snippet_outputs
                .iter()
                .map(|(world_settings, output)| {
                    if output.preload.spawn.is_some() {
                        warn!(logger: self.log_capture, "A Snippet attempted to set spawn");
                    }

                    let mut output = output.clone();
                    let uber_states =
                        UberStates::new(self.uber_state_data, &output.commands.events);

                    // TODO technically we shouldn't have to change our spawn choice between attempts anymore?
                    let world = World::new(
                        self.graph,
                        0,
                        world_settings,
                        uber_states,
                        &mut output.commands.events,
                    )
                    .with_perf_data(self.perf_data)
                    .with_log_capture(self.log_capture);

                    Ok((world, output))
                })
                .collect::<Result<Vec<_>, String>>()?;

            match self.generate_placements(&mut rng, worlds) {
                Ok(seed) => {
                    if attempt > 1 {
                        info!(
                            logger: self.log_capture,
                            "Generated seed after {attempt} attempts{}",
                            if attempt <= RETRIES / 2 {
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
            "All {RETRIES} attempts to generate a seed failed :("
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
) -> Result<IntermediateOutput<'log>, String> {
    for identifier in iter::once("seed_core")
        .chain(world_settings.inline_snippets.keys().map(String::as_str))
        .chain(world_settings.snippets.iter().map(String::as_str))
    {
        compiler
            .compile_snippet(identifier)
            .map_err(|err| format!("Failed to read snippet \"{identifier}\": {err}"))?;
    }

    compiler
        .finish()
        .eprint_errors()
        .ok_or_else(|| "failed to compile snippets".to_string())
}
