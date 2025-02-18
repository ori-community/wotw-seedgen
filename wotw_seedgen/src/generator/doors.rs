use crate::item::{UberStateItem, UberStateValue};
use crate::uber_state::{UberIdentifier, UberStateTrigger, UberType};
use crate::World;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rustc_hash::{FxHashMap};

type DoorId = u16;

struct DoorRandomizerConfig {
    max_loop_size: u8,
    door_groups: Vec<Vec<DoorId>>,
    group_index_by_door_id: FxHashMap<DoorId, usize>,
}

impl DoorRandomizerConfig {
    pub fn new(max_loop_size: u8, door_groups: Vec<Vec<DoorId>>) -> Self {
        let mut group_index_by_door_id: FxHashMap<DoorId, usize> = FxHashMap::default();

        for (group_index, door_ids) in door_groups.iter().enumerate() {
            for door_id in door_ids {
                group_index_by_door_id.insert(*door_id, group_index.to_owned());
            }
        }

        let config = Self {
            max_loop_size,
            door_groups,
            group_index_by_door_id,
        };

        config
    }
}

#[derive(Default, Clone)]
struct DoorRandomizerState {
    current_loop_start: Option<DoorId>,
    next_door_id: DoorId,
    current_loop_size: u8,
    doors_without_incoming_connection: IndexSet<DoorId>,
    reachable_doors: IndexSet<DoorId>,
    remaining_groups: IndexSet<usize>,
    connections: IndexMap<DoorId, DoorId>,
    recursion_level: u8,
}

fn generate_door_connections(config: &DoorRandomizerConfig, rng: &mut StdRng) -> Result<DoorRandomizerState, String> {
    let initial_door = *config.door_groups.iter().flatten().choose(rng).unwrap();
    let initial_door_group = config.group_index_by_door_id[&initial_door];

    let initial_state = DoorRandomizerState {
        next_door_id: initial_door,
        doors_without_incoming_connection: IndexSet::from_iter(config.door_groups.iter().flatten().copied().collect_vec()),
        reachable_doors: IndexSet::from_iter(config.door_groups[initial_door_group].iter().copied()),
        remaining_groups: IndexSet::from_iter((0..config.door_groups.len()).filter(|g| *g != initial_door_group)),
        ..DoorRandomizerState::default()
    };

    generate_door_connections_recursively(&initial_state, config, rng)
}

fn generate_door_connections_recursively(state: &DoorRandomizerState, config: &DoorRandomizerConfig, rng: &mut StdRng) -> Result<DoorRandomizerState, String> {
    #[cfg(feature = "log")]
    let log_indent_level = state.recursion_level;
    #[cfg(feature = "log")]
    let log_indent = "  ".repeat(log_indent_level as usize);

    let mut state = state.clone();
    state.recursion_level += 1;

    let door_id = state.next_door_id;

    if state.current_loop_start.is_none() {
        #[cfg(feature = "log")]
        log::trace!("{}Started new loop", log_indent);
        state.current_loop_start = Some(door_id);
    }

    state.current_loop_size += 1;

    #[cfg(feature = "log")]
    log::trace!("{}Door: {}, Loop Size: {}", log_indent, door_id, state.current_loop_size);

    let mut possible_target_doors: IndexSet<DoorId> = IndexSet::new();

    if state.current_loop_size >= config.max_loop_size {
        #[cfg(feature = "log")]
        log::trace!("{}Force closing loop", log_indent);
        possible_target_doors.insert(state.current_loop_start.unwrap());
    } else {
        let mut shuffled_remaining_groups = state.remaining_groups.iter().collect_vec();
        shuffled_remaining_groups.shuffle(rng);

        // Add remaining groups first
        for remaining_group in &shuffled_remaining_groups {
            for possible_target_door_id in &config.door_groups[**remaining_group] {
                possible_target_doors.insert(*possible_target_door_id);
            }
        }

        let mut other_doors_without_incoming_connections = state.doors_without_incoming_connection.clone();
        other_doors_without_incoming_connections.shift_remove(&door_id);
        possible_target_doors.append(&mut other_doors_without_incoming_connections);
    }

    #[cfg(feature = "log")]
    log::trace!("{}Possible doors: {}", log_indent, possible_target_doors.iter().map(|d| d.to_string()).join(", "));

    if possible_target_doors.is_empty() {
        return Err("No possible target door".to_string());
    }

    for possible_target_door in possible_target_doors {
        let mut state = state.clone();

        let target_door_id = possible_target_door;
        let target_door_group_index = config.group_index_by_door_id[&target_door_id];

        state.connections.insert(door_id, target_door_id);
        state.doors_without_incoming_connection.shift_remove(&target_door_id);

        if state.doors_without_incoming_connection.is_empty() {
            return Ok(state);
        }

        // Mark all doors in same group as reachable
        state.remaining_groups.shift_remove(&target_door_group_index);
        for door_in_same_group_id in &config.door_groups[target_door_group_index] {
            state.reachable_doors.insert(*door_in_same_group_id);
        }

        if state.current_loop_start.unwrap() == target_door_id {
            #[cfg(feature = "log")]
            log::trace!("{}Ended loop", log_indent);
            state.current_loop_start = None;
            state.current_loop_size = 0;

            let possible_next_doors = state.reachable_doors
                .iter()
                .filter(|d| **d != target_door_id)
                .filter(|d| !state.connections.contains_key(*d))
                .copied()
                .collect_vec();

            #[cfg(feature = "log")]
            for (d1, d2) in &state.connections {
                log::trace!("{}Conn: {} > {}", log_indent, d1, d2);
            }

            #[cfg(feature = "log")]
            log::trace!("{}Possible next doors: {}", log_indent, possible_next_doors.iter().map(|d| d.to_string()).join(", "));

            for possible_next_door_id in possible_next_doors {
                state.next_door_id = possible_next_door_id;

                #[cfg(feature = "log")]
                log::trace!("{}Trying {} as next door...", log_indent, possible_next_door_id);
                if let Ok(state) = generate_door_connections_recursively(&state, config, rng) {

                    #[cfg(feature = "log")]
                    log::trace!("{}Worked! {} -> {}", log_indent, door_id, target_door_id);
                    return Ok(state);
                }

                #[cfg(feature = "log")]
                log::trace!("{}Failed", log_indent);
            }
        } else {
            state.next_door_id = target_door_id;

            #[cfg(feature = "log")]
            log::trace!("{}Trying target door as next door: {}", log_indent, target_door_id);
            if let Ok(state) = generate_door_connections_recursively(&state, config, rng) {
                #[cfg(feature = "log")]
                log::trace!("{}Worked! {} -> {}", log_indent, door_id, target_door_id);
                return Ok(state);
            }

            #[cfg(feature = "log")]
            log::trace!("{}Failed", log_indent);
        }
    }

    Err("Found no possible solution".to_string())
}

pub fn generate_door_headers(world: &mut World, rng: &mut StdRng) -> String {
    let mut header_lines: Vec<String> = vec![];
    let door_groups: Vec<Vec<DoorId>> = vec![
        vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29],
        vec![2],
        vec![4],
        vec![6],
        vec![8],
        vec![10],
        vec![12],
        vec![14],
        vec![16, 18],
        vec![20],
        vec![22],
        vec![24],
        vec![26],
        vec![28],
        vec![30, 31],
        vec![32],
    ];

    let config = DoorRandomizerConfig::new(2, door_groups);
    let result = generate_door_connections(&config, rng);

    if result.is_err() {
        panic!("Door error: {}", result.err().unwrap());
    }

    for (door_id, target_door_id) in result.unwrap().connections {
        #[cfg(feature = "log")]
        log::trace!("Connected door {} → {}", door_id, target_door_id);

        world.preplace(
            UberStateTrigger::spawn(),
            UberStateItem::simple_setter(
                UberIdentifier::new(27, door_id),
                UberType::Int,
                UberStateValue::Number((target_door_id as f32).into()),
            ),
        );
        header_lines.push(format!("3|0|8|27|{}|int|{}", door_id, target_door_id));
    }

    #[cfg(feature = "log")]
    log::trace!("Doors generated");

    header_lines.join("\n")
}
