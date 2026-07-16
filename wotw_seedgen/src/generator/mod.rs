pub mod entrances;
pub mod item_pool;
pub mod perf_data;
pub mod spoiler;

mod placement;
mod solutions;
mod spawn;
mod spirit_light;

use self::spoiler::SeedSpoiler;
use crate::{generator::placement::generate_placements, perf_data::PerfData, world::World};
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
use wotw_seedgen_seed::Seed;

/// End Result of seed generation
pub struct SeedUniverse {
    /// Seed data per world
    pub worlds: Vec<Seed>,
    /// Spoiler data for the generation process
    pub spoiler: SeedSpoiler,
}

const RETRIES: u16 = 10; // How many retries to allow when generating a seed

/// Entry point for seed generation
pub fn generate_seed<'graph, F: SnippetAccess>(
    graph: &'graph Graph,
    loc_data: &LocData,
    uber_state_data: &UberStateData,
    snippet_access: &F,
    settings: &UniverseSettings,
    debug: bool,
    perf_data: Option<&PerfData<'graph>>,
) -> Result<SeedUniverse, String> {
    let mut rng: Pcg64Mcg = Seeder::from(&settings.seed).make_rng();
    trace!("Seeded RNG with \"{}\"", settings.seed);

    let snippet_outputs = settings
        .world_settings
        .iter()
        .map(|world_settings| {
            let snippet_access =
                ChainedSnippetAccess::new(&world_settings.inline_snippets, snippet_access);

            let compiler = Compiler::new(
                &mut rng,
                &snippet_access,
                uber_state_data,
                world_settings.snippet_config.clone(),
                None,
                true,
                debug,
            );

            // TODO this is inefficient because we probably do a lot of redundant work between the worlds
            let output = parse_snippets(world_settings, compiler)?;

            Ok((world_settings, output))
        })
        .collect::<Result<Vec<_>, String>>()?;

    for attempt in 1..=RETRIES {
        trace!("Attempt #{attempt} to generate");

        let worlds = snippet_outputs
            .iter()
            .map(|(world_settings, output)| {
                if output.preload.spawn.is_some() {
                    warn!("A Snippet attempted to set spawn");
                }

                let mut output = output.clone();
                let uber_states = UberStates::new(uber_state_data, &output.commands.events);

                // TODO technically we shouldn't have to change our spawn choice between attempts anymore?
                let world = World::new(
                    graph,
                    0,
                    world_settings,
                    uber_states,
                    &mut output.commands.events,
                    perf_data,
                );

                Ok((world, output))
            })
            .collect::<Result<Vec<_>, String>>()?;

        match generate_placements(&mut rng, worlds, settings, loc_data, debug) {
            Ok(seed) => {
                if attempt > 1 {
                    info!(
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
            Err(err) => warn!("{err}"),
        }
    }

    Err(format!(
        "All {RETRIES} attempts to generate a seed failed :("
    ))
}

const SEED_FAILED_MESSAGE: &str = "Failed to seed child RNG";

fn parse_snippets(
    world_settings: &WorldSettings,
    mut compiler: Compiler,
) -> Result<IntermediateOutput, String> {
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
