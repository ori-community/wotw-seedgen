use super::{compile_into_lookup, Compile};
use crate::assembly::{Command, Event};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use wotw_seedgen_seed_language::output::{CommandVoid, Event as IntermediateEvent};

pub fn compile_command_lookup(intermediate_command_lookup: Vec<CommandVoid>) -> Vec<Vec<Command>> {
    let mut command_lookup = vec![];
    command_lookup.resize_with(intermediate_command_lookup.len(), Default::default);
    for (index, command) in intermediate_command_lookup.into_iter().enumerate() {
        command_lookup[index] = command.compile(&mut command_lookup).0;
    }
    command_lookup
}

pub fn compile_events(
    intermediate_events: Vec<IntermediateEvent>,
    command_lookup: &mut Vec<Vec<Command>>,
) -> Vec<Event> {
    let mut events = FxHashMap::<_, usize>::default();
    events.reserve(intermediate_events.len());
    for event in intermediate_events {
        let trigger = event.trigger.compile(command_lookup);
        match events.entry(trigger) {
            Entry::Occupied(occupied) => {
                let (new, _) = event.command.compile(command_lookup);
                let existing = &mut command_lookup[*occupied.get()];
                existing.extend(new);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(compile_into_lookup(event.command, command_lookup));
            }
        }
    }
    events
        .into_iter()
        .map(|(trigger, command)| Event(trigger, command))
        .collect()
}
