use std::collections::HashSet;
use rand::prelude::IteratorRandom;
use indexmap::IndexSet;
use itertools::Itertools;
use rand::prelude::StdRng;
use rustc_hash::{FxHashMap, FxHashSet};
use crate::{World};
use rand::seq::SliceRandom;
use crate::item::{UberStateItem, UberStateValue};
use crate::uber_state::{UberIdentifier, UberStateTrigger, UberType};

type DoorId = u16;

pub fn generate_door_headers(world: &mut World, rng: &mut StdRng) -> String {
    let mut header_lines: Vec<String> = vec![];
    let door_groups: Vec<Vec<DoorId>> = vec![
        vec![1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 32],
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
    ];

    // Create group index lookup map
    let mut group_index_by_door_id = FxHashMap::default();
    for (group_index, door_ids) in door_groups.iter().enumerate() {
        for door_id in door_ids {
            group_index_by_door_id.insert(door_id, group_index.to_owned());
        }
    }

    let mut reachable_doors_without_outgoing_connection: HashSet<DoorId, _> = FxHashSet::default();

    let mut all_door_ids = door_groups.iter().flatten().collect_vec();
    all_door_ids.shuffle(rng);

    let mut doors_without_incoming_connection: IndexSet<DoorId> = IndexSet::from_iter(all_door_ids.iter().map(|d| **d));
    let mut remaining_groups = FxHashSet::from_iter(0..door_groups.len());

    let mut door = *doors_without_incoming_connection.first().unwrap();
    let mut current_circle_start_group_index: Option<usize> = None;

    loop {
        let group_index = group_index_by_door_id[&door];

        if current_circle_start_group_index.is_none() {
            current_circle_start_group_index = Some(group_index);
        }

        // Mark group of door as reachable
        if remaining_groups.remove(&group_index) {
            // If this is the first time we're entering this group, mark all doors in this group as reachable
            for door_id in &door_groups[group_index] {
                reachable_doors_without_outgoing_connection.insert(*door_id);
                
                #[cfg(feature = "log")]
                log::trace!("Door {} is now reachable", door_id);
            }
        }

        // Create an outgoing connection

        // Pick random target door
        let mut target_door = *doors_without_incoming_connection.iter().filter(|d| **d != door).choose(rng).unwrap();
        let mut target_group_index = *group_index_by_door_id.get(&target_door).unwrap();

        // If this is the last reachable door without an outgoing connection and there are unreached groups, connect to an unreached group
        if reachable_doors_without_outgoing_connection.len() <= 1 && remaining_groups.len() > 0 && !remaining_groups.contains(&target_group_index) {
            // Pick a door from an unreached group

            let mut possible_doors: Vec<DoorId> = vec![];
            for possible_group_index in &remaining_groups {
                for possible_door in &door_groups[possible_group_index.to_owned()] {
                    possible_doors.push(*possible_door);
                }
            }

            target_door = possible_doors.choose(rng).unwrap().to_owned();
            target_group_index = group_index_by_door_id[&target_door].to_owned();
        }

        let current_circle_is_closed = current_circle_start_group_index.is_some_and(|g| g == target_group_index);

        world.preplace(
            UberStateTrigger::spawn(),
            UberStateItem::simple_setter(
                UberIdentifier::new(27, door),
                UberType::Int,
                UberStateValue::Number((target_door as f32).into()),
            )
        );
        header_lines.push(format!("3|0|8|27|{}|int|{}", door, target_door));
        
        #[cfg(feature = "log")]
        log::trace!("Connecting door {} → {}", door, target_door);

        // Now mark the target group as reachable
        if remaining_groups.remove(&target_group_index) {
            for door_id in &door_groups[target_group_index] {
                reachable_doors_without_outgoing_connection.insert(*door_id);
                
                #[cfg(feature = "log")]
                log::trace!("Door {} is now reachable", door_id);
            }
        }

        reachable_doors_without_outgoing_connection.remove(&door);
        doors_without_incoming_connection.shift_remove(&target_door);

        if doors_without_incoming_connection.is_empty() {
            break;
        }

        // Select next outgoing door that is in the same group as the current target door
        let mut possible_next_doors: Vec<DoorId> = vec![];
        for possible_next_door in &reachable_doors_without_outgoing_connection {
            if current_circle_is_closed || group_index_by_door_id[possible_next_door] == target_group_index {
                possible_next_doors.push(*possible_next_door);
            }
        }

        door = *possible_next_doors.choose(rng).unwrap();

        // Start a new circle if the current circle is closed
        if current_circle_is_closed {
            current_circle_start_group_index = None;
        }
    }
    
    header_lines.join("\n")
}
