use super::{compile_into_lookup, Compile};
use crate::{Event, SeedWorld, Spawn};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use wotw_seedgen_seed_language::output::IntermediateOutput;

pub fn compile_intermediate_output(
    output: IntermediateOutput,
) -> (SeedWorld, Vec<(String, Vec<u8>)>) {
    let mut flags = output.flags;
    flags.sort();
    let spawn = output
        .spawn
        .map(|position| Spawn {
            position,
            identifier: "Custom Spawn".to_string(), // TODO is this important?
        })
        .unwrap_or_default();

    let mut command_lookup = vec![];
    command_lookup.resize_with(output.command_lookup.len(), Default::default);
    for (index, command) in output.command_lookup.into_iter().enumerate() {
        command_lookup[index] = command.compile(&mut command_lookup).0;
    }

    let mut events = FxHashMap::<_, usize>::default();
    events.reserve(output.events.len());
    for event in output.events {
        let trigger = event.trigger.compile(&mut command_lookup);
        match events.entry(trigger) {
            Entry::Occupied(occupied) => {
                let (new, _) = event.command.compile(&mut command_lookup);
                let existing = &mut command_lookup[*occupied.get()];
                existing.extend(new);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(compile_into_lookup(event.command, &mut command_lookup));
            }
        }
    }
    let events = events
        .into_iter()
        .map(|(trigger, command)| Event(trigger, command))
        .collect();

    let seed_world = SeedWorld {
        flags,
        spawn,
        timers: output.timers,
        events,
        command_lookup,
    };

    (seed_world, output.icons)
}
