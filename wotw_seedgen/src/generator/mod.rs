pub mod item_pool;
pub mod spoiler;

mod cost;
mod placement;
mod spirit_light;
mod string_placeholders;

use self::spoiler::SeedSpoiler;
use crate::{
    generator::placement::generate_placements,
    log::{info, trace, warning},
    logical_difficulty,
    world::World,
    UberStates,
};
use rand::{seq::IteratorRandom, Rng};
use rand_pcg::Pcg64Mcg;
use rand_seeder::Seeder;
use std::iter;
use wotw_seedgen_assets::{SnippetAccess, UberStateData};
use wotw_seedgen_data::uber_identifier;
use wotw_seedgen_logic_language::output::Graph;
use wotw_seedgen_seed::Seed;
use wotw_seedgen_seed_language::{
    compile::{self, Compiler},
    output::{ClientEvent, Event, IntermediateOutput, Trigger},
};
use wotw_seedgen_settings::{Spawn, UniverseSettings, WorldSettings};

/// End Result of seed generation
pub struct SeedUniverse {
    /// Seed data per world
    pub worlds: Vec<Seed>,
    /// Spoiler data for the generation process
    pub spoiler: SeedSpoiler,
}

const RETRIES: u16 = 10; // How many retries to allow when generating a seed

/// Entry point for seed generation
pub fn generate_seed<F: SnippetAccess>(
    graph: &Graph,
    uber_state_data: &UberStateData,
    snippet_access: &F,
    settings: &UniverseSettings,
    debug: bool,
) -> Result<SeedUniverse, String> {
    let mut rng: Pcg64Mcg = Seeder::from(&settings.seed).make_rng();
    trace!("Seeded RNG with \"{}\"", settings.seed);

    let snippet_outputs = settings
        .world_settings
        .iter()
        .map(|world_settings| {
            let compiler = Compiler::new(
                &mut rng,
                snippet_access,
                uber_state_data,
                world_settings.snippet_config.clone(),
                debug,
            );
            // TODO this is inefficient because we probably do a lot of redundant work between the worlds
            let output = parse_snippets(&world_settings.snippets, compiler)?;
            Ok((world_settings, output))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let uber_states = UberStates::new(uber_state_data);

    for attempt in 1..=RETRIES {
        trace!("Attempt #{attempt} to generate");

        let worlds = snippet_outputs
            .iter()
            .map(|(world_settings, output)| {
                let spawn = choose_spawn(graph, world_settings, &mut rng)?;
                if output.spawn.is_some() {
                    warning!("A Snippet attempted to set spawn");
                }
                let mut output = output.clone();
                // TODO something less specialized?
                if graph.nodes[spawn].identifier() == "EastPools.Teleporter" {
                    output.events.push(Event {
                        trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                        command: compile::set_boolean_value(
                            uber_identifier::teleporter::CENTRAL_POOLS,
                            true,
                        ),
                    })
                }
                let world = World::new_spawn(graph, spawn, world_settings, uber_states.clone());
                Ok((world, output))
            })
            .collect::<Result<Vec<_>, String>>()?;

        match generate_placements(&mut rng, worlds, debug) {
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
            #[cfg_attr(not(feature = "log"), allow(unused_variables))]
            Err(err) => warning!("{err}"),
        }
    }

    Err(format!(
        "All {RETRIES} attempts to generate a seed failed :("
    ))
}

const SEED_FAILED_MESSAGE: &str = "Failed to seed child RNG";

fn parse_snippets(
    snippets: &[String],
    mut compiler: Compiler,
) -> Result<IntermediateOutput, String> {
    for identifier in iter::once("seed_core").chain(snippets.iter().map(String::as_str)) {
        compiler
            .compile_snippet(identifier)
            .map_err(|err| format!("Failed to read snippet \"{identifier}\": {err}"))?;
    }

    compiler.finish().into_result()
}

fn choose_spawn(
    graph: &Graph,
    world_settings: &WorldSettings,
    rng: &mut impl Rng,
) -> Result<usize, String> {
    let spawn = match &world_settings.spawn {
        Spawn::Random => {
            let spawns = logical_difficulty::spawn_locations(world_settings.difficulty);
            graph
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| spawns.contains(&node.identifier()))
                .choose(rng)
                .ok_or_else(|| String::from("No valid spawn locations available"))?
                .0
        }
        Spawn::FullyRandom => {
            graph
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.can_spawn())
                .choose(rng)
                .ok_or_else(|| String::from("No valid spawn locations available"))?
                .0
        }
        Spawn::Set(spawn_loc) => {
            let (index, node) = graph
                .nodes
                .iter()
                .enumerate()
                .find(|(_, node)| node.identifier() == spawn_loc)
                .ok_or_else(|| format!("Spawn {} not found", spawn_loc))?;
            if !node.can_spawn() {
                return Err(format!("{} is not a valid spawn", spawn_loc));
            }
            index
        }
    };
    Ok(spawn)
}
