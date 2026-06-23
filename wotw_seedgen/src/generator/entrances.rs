use arrayvec::ArrayVec;
use indexmap::IndexSet;
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use wotw_seedgen_data::{
    logic_language::output::EntranceId,
    seed_language::{
        ast::ClientEvent,
        compile::{store_boolean, store_integer},
        output::{CommandBoolean, CommandsOutput, Event, Trigger, TriggerCondition},
        simulate::Simulation,
    },
    UberIdentifier,
};

use crate::World;

type EntranceGroups = [ArrayVec<EntranceId, 15>; 16];

struct EntranceRandomizerConfig {
    max_loop_size: u8,
    entrance_groups: EntranceGroups,
    group_index_by_entrance_id: FxHashMap<EntranceId, usize>,
}

impl EntranceRandomizerConfig {
    pub fn new(max_loop_size: u8, entrance_groups: EntranceGroups) -> Result<Self, String> {
        if max_loop_size < 2 {
            return Err(
                "Max loop size for entrance randomization has to be 2 or higher".to_string(),
            );
        }

        let mut group_index_by_entrance_id: FxHashMap<EntranceId, usize> = FxHashMap::default();

        for (group_index, entrance_ids) in entrance_groups.iter().enumerate() {
            for entrance_id in entrance_ids {
                group_index_by_entrance_id.insert(*entrance_id, group_index.to_owned());
            }
        }

        let config = Self {
            max_loop_size,
            entrance_groups,
            group_index_by_entrance_id,
        };

        Ok(config)
    }
}

#[derive(Default, Clone)]
struct EntranceRandomizerState {
    current_loop_start: Option<EntranceId>,
    next_entrance_id: EntranceId,
    current_loop_size: u8,
    entrances_without_incoming_connection: IndexSet<EntranceId>,
    reachable_entrances: IndexSet<EntranceId>,
    remaining_groups: IndexSet<usize>,
    connections: FxHashMap<EntranceId, EntranceId>,
    recursion_level: u8,
}

pub fn generate_entrances(
    world: &mut World,
    output: &mut CommandsOutput,
    rng: &mut Pcg64Mcg,
) -> Result<(), String> {
    let (connections, loop_size) = match world.settings.randomize_entrances {
        None => {
            log::trace!("Using default entrance connections");
            (world.graph.default_entrance_connections.clone(), 2)
        }
        Some(loop_size) => {
            log::trace!("Randomizing entrance connections");

            let entrance_groups: EntranceGroups = [
                ArrayVec::from([1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29]),
                ArrayVec::from_iter([2]),
                ArrayVec::from_iter([4]),
                ArrayVec::from_iter([6]),
                ArrayVec::from_iter([8]),
                ArrayVec::from_iter([10]),
                ArrayVec::from_iter([12]),
                ArrayVec::from_iter([14]),
                ArrayVec::from_iter([16, 18]),
                ArrayVec::from_iter([20]),
                ArrayVec::from_iter([22]),
                ArrayVec::from_iter([24]),
                ArrayVec::from_iter([26]),
                ArrayVec::from_iter([28]),
                ArrayVec::from_iter([30, 31]),
                ArrayVec::from_iter([32]),
            ];

            // enable randoConfig.showSmallEntrances
            output.events.push(Event {
                trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                command: store_boolean(UberIdentifier::rando_config(200), true),
            });
            // mark entrance connections as unknown
            output.events.extend((1..=32).map(|entrance_id| Event {
                trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                command: store_boolean(
                    UberIdentifier::known_entrance_connections(entrance_id),
                    false,
                ),
            }));

            let config = EntranceRandomizerConfig::new(loop_size.get(), entrance_groups)?;
            let connections = generate_entrance_connections(&config, rng)?;
            (connections, loop_size.get())
        }
    };

    for (entrance_id, target_entrance_id) in connections {
        log::trace!(
            "Connected entrance {} → {}",
            entrance_id,
            target_entrance_id
        );

        let uber_identifier = UberIdentifier::entrances(entrance_id);

        world.store_integer(uber_identifier, target_entrance_id, output);

        output.events.push(Event {
            trigger: Trigger::ClientEvent(ClientEvent::Spawn),
            command: store_integer(uber_identifier, target_entrance_id),
        });

        // If the target entrance is known to connect back to this entrance, mark
        // the target entrance as visited too once we went through this entrance
        if loop_size == 2 {
            output.events.push(Event {
                trigger: Trigger::Condition(TriggerCondition::new(CommandBoolean::FetchBoolean {
                    uber_identifier: UberIdentifier::known_entrance_connections(entrance_id),
                })),
                command: store_boolean(
                    UberIdentifier::known_entrance_connections(target_entrance_id),
                    true,
                ),
            });
        }
    }

    log::trace!("Entrances generated");

    Ok(())
}

fn generate_entrance_connections(
    config: &EntranceRandomizerConfig,
    rng: &mut Pcg64Mcg,
) -> Result<FxHashMap<EntranceId, EntranceId>, String> {
    let initial_entrance = *config.entrance_groups.iter().flatten().choose(rng).unwrap();
    let initial_entrance_group = config.group_index_by_entrance_id[&initial_entrance];

    let initial_state = EntranceRandomizerState {
        next_entrance_id: initial_entrance,
        entrances_without_incoming_connection: IndexSet::from_iter(
            config
                .entrance_groups
                .iter()
                .flatten()
                .copied()
                .collect_vec(),
        ),
        reachable_entrances: IndexSet::from_iter(
            config.entrance_groups[initial_entrance_group]
                .iter()
                .copied(),
        ),
        remaining_groups: IndexSet::from_iter(
            (0..config.entrance_groups.len()).filter(|g| *g != initial_entrance_group),
        ),
        ..EntranceRandomizerState::default()
    };

    let final_state = generate_entrance_connections_recursively(&initial_state, config, rng)?;
    Ok(final_state.connections)
}

fn generate_entrance_connections_recursively(
    state: &EntranceRandomizerState,
    config: &EntranceRandomizerConfig,
    rng: &mut Pcg64Mcg,
) -> Result<EntranceRandomizerState, String> {
    let log_indent_level = state.recursion_level;
    let log_indent = "  ".repeat(log_indent_level as usize);

    let mut state = state.clone();
    state.recursion_level += 1;

    let entrance_id = state.next_entrance_id;

    if state.current_loop_start.is_none() {
        log::trace!("{log_indent}Started new loop");
        state.current_loop_start = Some(entrance_id);
    }

    state.current_loop_size += 1;

    log::trace!(
        "{log_indent}Entrance: {entrance_id}, Loop Size: {}",
        state.current_loop_size
    );

    let mut possible_target_entrances: IndexSet<EntranceId> = IndexSet::new();

    if state.current_loop_size >= config.max_loop_size {
        log::trace!("{log_indent}Reached max loop size, force closing loop");
        possible_target_entrances.insert(state.current_loop_start.unwrap());
    } else {
        let mut shuffled_remaining_groups = state.remaining_groups.iter().collect_vec();
        shuffled_remaining_groups.shuffle(rng);

        // Add remaining groups first
        for remaining_group in &shuffled_remaining_groups {
            for possible_target_entrance_id in &config.entrance_groups[**remaining_group] {
                possible_target_entrances.insert(*possible_target_entrance_id);
            }
        }

        let mut other_entrances_without_incoming_connections =
            state.entrances_without_incoming_connection.clone();
        other_entrances_without_incoming_connections.shift_remove(&entrance_id);
        possible_target_entrances.append(&mut other_entrances_without_incoming_connections);
    }

    log::trace!(
        "{log_indent}Possible entrances: {}",
        possible_target_entrances
            .iter()
            .map(|d| d.to_string())
            .join(", ")
    );

    // TODO: Remove these special cases when there is a mechanism for
    //       defining required pickups.
    // Prevent second floor to connecting to third floor in wellspring
    // 21 == Wellspring third floor entrance
    if entrance_id == 21 {
        possible_target_entrances.shift_remove(&20); // 20 == Wellspring second floor exit
    }

    // Prevent the Moki Father hut connecting to the Teddy hut
    // 9 == Moki Father hut entrance
    if entrance_id == 9 {
        possible_target_entrances.shift_remove(&26); // 26 == Teddy hut exit
    }

    if possible_target_entrances.is_empty() {
        return Err("No possible target entrance".to_string());
    }

    for possible_target_entrance in possible_target_entrances {
        let mut state = state.clone();

        let target_entrance_id = possible_target_entrance;
        let target_entrance_group_index = config.group_index_by_entrance_id[&target_entrance_id];

        state.connections.insert(entrance_id, target_entrance_id);
        state
            .entrances_without_incoming_connection
            .shift_remove(&target_entrance_id);

        if state.entrances_without_incoming_connection.is_empty() {
            return Ok(state);
        }

        // Mark all entrances in same group as reachable
        state
            .remaining_groups
            .shift_remove(&target_entrance_group_index);
        for entrance_in_same_group_id in &config.entrance_groups[target_entrance_group_index] {
            state.reachable_entrances.insert(*entrance_in_same_group_id);
        }

        if state.current_loop_start.unwrap() == target_entrance_id {
            log::trace!("{log_indent}Ended loop");
            state.current_loop_start = None;
            state.current_loop_size = 0;

            let possible_next_entrances = state
                .reachable_entrances
                .iter()
                .filter(|d| **d != target_entrance_id)
                .filter(|d| !state.connections.contains_key(*d))
                .copied()
                .collect_vec();
            {
                log::trace!("{log_indent}Current connections:");
                for (from_entrance, to_entrance) in &state.connections {
                    log::trace!("{log_indent}  {from_entrance} → {to_entrance}");
                }
            }

            log::trace!(
                "{log_indent}Possible next entrances: {}",
                possible_next_entrances
                    .iter()
                    .map(|d| d.to_string())
                    .join(", ")
            );

            for possible_next_entrance_id in possible_next_entrances {
                state.next_entrance_id = possible_next_entrance_id;

                log::trace!("{log_indent}Trying {possible_next_entrance_id} as next entrance...");
                if let Ok(state) = generate_entrance_connections_recursively(&state, config, rng) {
                    log::trace!("{log_indent}Worked! {entrance_id} → {target_entrance_id}");
                    return Ok(state);
                }

                log::trace!("{log_indent}Failed");
            }
        } else {
            state.next_entrance_id = target_entrance_id;

            log::trace!(
                "{log_indent}Trying target entrance as next entrance: {target_entrance_id}"
            );
            if let Ok(state) = generate_entrance_connections_recursively(&state, config, rng) {
                log::trace!("{log_indent}Worked! {entrance_id} → {target_entrance_id}");
                return Ok(state);
            }

            log::trace!("{log_indent}Failed");
        }
    }

    Err("Found no possible solution".to_string())
}
