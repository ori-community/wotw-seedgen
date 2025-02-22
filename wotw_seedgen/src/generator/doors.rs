use arrayvec::ArrayVec;
use indexmap::IndexSet;
use itertools::Itertools;
use rand::prelude::IteratorRandom;
use rand::seq::SliceRandom;
use rustc_hash::FxHashMap;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_logic_language::output::DoorId;
use wotw_seedgen_seed_language::{
    ast::ClientEvent,
    compile::{set_boolean_value, set_integer_value},
    output::{CommandBoolean, Event, Trigger},
};

use super::placement::WorldContext;

type DoorGroups = [ArrayVec<DoorId, 15>; 16];

struct DoorRandomizerConfig {
    max_loop_size: u8,
    door_groups: DoorGroups,
    group_index_by_door_id: FxHashMap<DoorId, usize>,
}

impl DoorRandomizerConfig {
    pub fn new(max_loop_size: u8, door_groups: DoorGroups) -> Result<Self, String> {
        if max_loop_size < 2 {
            return Err("Max loop size for door randomization has to be 2 or higher".to_string());
        }

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

        Ok(config)
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
    connections: FxHashMap<DoorId, DoorId>,
    recursion_level: u8,
}

impl WorldContext<'_, '_> {
    pub fn generate_doors(&mut self) -> Result<(), String> {
        let (connections, loop_size) = match self.world.player.settings.randomize_doors {
            None => {
                log::trace!("Using default door connections");
                (self.world.graph.default_door_connections.clone(), 2)
            }
            Some(loop_size) => {
                log::trace!("Randomizing door connections");

                let door_groups: DoorGroups = [
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

                self.output.events.push(Event {
                    trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                    command: set_boolean_value(UberIdentifier::new(7, 200), true),
                });
                self.output.events.push(Event {
                    trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                    command: set_boolean_value(UberIdentifier::new(7, 201), true),
                });

                let config = DoorRandomizerConfig::new(loop_size.get(), door_groups)?;
                let connections = self.generate_door_connections(&config)?;
                (connections, loop_size.get())
            }
        };

        for (door_id, target_door_id) in connections {
            log::trace!("Connected door {} → {}", door_id, target_door_id);

            let set_connection =
                set_integer_value(UberIdentifier::new(27, door_id), target_door_id);
            self.world.simulate(&set_connection, &self.output);
            self.output.events.push(Event {
                trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                command: set_connection,
            });

            // This is only for seedgen simulation to make it think
            // we have gone through all doors
            self.world
                .set_boolean(UberIdentifier::new(28, door_id), true, &self.output);

            // If the target door is known to connect back to this door, mark
            // the target door as visited too once we went through this door
            if loop_size == 2 {
                self.output.events.push(Event {
                    trigger: Trigger::Condition(CommandBoolean::FetchBoolean {
                        uber_identifier: UberIdentifier::new(28, door_id),
                    }),
                    command: set_boolean_value(UberIdentifier::new(28, target_door_id), true),
                });
            }
        }

        log::trace!("Doors generated");

        Ok(())
    }

    fn generate_door_connections(
        &mut self,
        config: &DoorRandomizerConfig,
    ) -> Result<FxHashMap<DoorId, DoorId>, String> {
        let initial_door = *config
            .door_groups
            .iter()
            .flatten()
            .choose(&mut self.rng)
            .unwrap();
        let initial_door_group = config.group_index_by_door_id[&initial_door];

        let initial_state = DoorRandomizerState {
            next_door_id: initial_door,
            doors_without_incoming_connection: IndexSet::from_iter(
                config.door_groups.iter().flatten().copied().collect_vec(),
            ),
            reachable_doors: IndexSet::from_iter(
                config.door_groups[initial_door_group].iter().copied(),
            ),
            remaining_groups: IndexSet::from_iter(
                (0..config.door_groups.len()).filter(|g| *g != initial_door_group),
            ),
            ..DoorRandomizerState::default()
        };

        let final_state = self.generate_door_connections_recursively(&initial_state, config)?;
        Ok(final_state.connections)
    }

    fn generate_door_connections_recursively(
        &mut self,
        state: &DoorRandomizerState,
        config: &DoorRandomizerConfig,
    ) -> Result<DoorRandomizerState, String> {
        let log_indent_level = state.recursion_level;
        let log_indent = "  ".repeat(log_indent_level as usize);

        let mut state = state.clone();
        state.recursion_level += 1;

        let door_id = state.next_door_id;

        if state.current_loop_start.is_none() {
            log::trace!("{log_indent}Started new loop");
            state.current_loop_start = Some(door_id);
        }

        state.current_loop_size += 1;

        log::trace!(
            "{log_indent}Door: {door_id}, Loop Size: {}",
            state.current_loop_size
        );

        let mut possible_target_doors: IndexSet<DoorId> = IndexSet::new();

        if state.current_loop_size >= config.max_loop_size {
            log::trace!("{log_indent}Reached max loop size, force closing loop");
            possible_target_doors.insert(state.current_loop_start.unwrap());
        } else {
            let mut shuffled_remaining_groups = state.remaining_groups.iter().collect_vec();
            shuffled_remaining_groups.shuffle(&mut self.rng);

            // Add remaining groups first
            for remaining_group in &shuffled_remaining_groups {
                for possible_target_door_id in &config.door_groups[**remaining_group] {
                    possible_target_doors.insert(*possible_target_door_id);
                }
            }

            let mut other_doors_without_incoming_connections =
                state.doors_without_incoming_connection.clone();
            other_doors_without_incoming_connections.shift_remove(&door_id);
            possible_target_doors.append(&mut other_doors_without_incoming_connections);
        }

        log::trace!(
            "{log_indent}Possible doors: {}",
            possible_target_doors
                .iter()
                .map(|d| d.to_string())
                .join(", ")
        );

        // TODO: Remove these special cases when there is a mechanism for
        //       defining required pickups.
        // Prevent second floor to connecting to third floor in wellspring
        // 21 == Wellspring third floor entrance
        if door_id == 21 {
            possible_target_doors.shift_remove(&20); // 20 == Wellspring second floor exit
        }

        // Prevent the Moki Father hut connecting to the Teddy hut
        // 9 == Moki Father hut entrance
        if door_id == 9 {
            possible_target_doors.shift_remove(&26); // 26 == Teddy hut exit
        }

        if possible_target_doors.is_empty() {
            return Err("No possible target door".to_string());
        }

        for possible_target_door in possible_target_doors {
            let mut state = state.clone();

            let target_door_id = possible_target_door;
            let target_door_group_index = config.group_index_by_door_id[&target_door_id];

            state.connections.insert(door_id, target_door_id);
            state
                .doors_without_incoming_connection
                .shift_remove(&target_door_id);

            if state.doors_without_incoming_connection.is_empty() {
                return Ok(state);
            }

            // Mark all doors in same group as reachable
            state
                .remaining_groups
                .shift_remove(&target_door_group_index);
            for door_in_same_group_id in &config.door_groups[target_door_group_index] {
                state.reachable_doors.insert(*door_in_same_group_id);
            }

            if state.current_loop_start.unwrap() == target_door_id {
                log::trace!("{log_indent}Ended loop");
                state.current_loop_start = None;
                state.current_loop_size = 0;

                let possible_next_doors = state
                    .reachable_doors
                    .iter()
                    .filter(|d| **d != target_door_id)
                    .filter(|d| !state.connections.contains_key(*d))
                    .copied()
                    .collect_vec();
                {
                    log::trace!("{log_indent}Current connections:");
                    for (from_door, to_door) in &state.connections {
                        log::trace!("{log_indent}  {from_door} → {to_door}");
                    }
                }

                log::trace!(
                    "{log_indent}Possible next doors: {}",
                    possible_next_doors.iter().map(|d| d.to_string()).join(", ")
                );

                for possible_next_door_id in possible_next_doors {
                    state.next_door_id = possible_next_door_id;

                    log::trace!("{log_indent}Trying {possible_next_door_id} as next door...");
                    if let Ok(state) = self.generate_door_connections_recursively(&state, config) {
                        log::trace!("{log_indent}Worked! {door_id} → {target_door_id}");
                        return Ok(state);
                    }

                    log::trace!("{log_indent}Failed");
                }
            } else {
                state.next_door_id = target_door_id;

                log::trace!("{log_indent}Trying target door as next door: {target_door_id}");
                if let Ok(state) = self.generate_door_connections_recursively(&state, config) {
                    log::trace!("{log_indent}Worked! {door_id} → {target_door_id}");
                    return Ok(state);
                }

                log::trace!("{log_indent}Failed");
            }
        }

        Err("Found no possible solution".to_string())
    }
}
