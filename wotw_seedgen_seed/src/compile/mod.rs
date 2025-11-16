mod args;
mod command;

use std::mem;

use self::command::MemoryUsed;
use crate::assembly::{Command, Event, Trigger};
use indexmap::{map::Entry, IndexMap};
use rustc_hash::FxBuildHasher;
use wotw_seedgen_seed_language::output::{self as input, PlaceholderMap};

// TODO dedup functions?

pub struct CompileContext {
    events: IndexMap<Trigger, usize, FxBuildHasher>,
    pub command_lookup: Vec<Vec<Command>>,
    placeholder_map: PlaceholderMap,
}

impl CompileContext {
    pub fn new(placeholder_map: PlaceholderMap) -> Self {
        Self {
            events: IndexMap::default(),
            command_lookup: vec![],
            placeholder_map,
        }
    }

    pub fn compile_command_lookup(&mut self, intermediate_command_lookup: Vec<input::CommandVoid>) {
        self.command_lookup
            .resize_with(intermediate_command_lookup.len(), Default::default);

        for (index, command) in intermediate_command_lookup.into_iter().enumerate() {
            self.command_lookup[index] = command.compile(self).0;
        }
    }

    pub fn compile_events(&mut self, intermediate_events: Vec<input::Event>) -> Vec<Event> {
        self.events.reserve(intermediate_events.len());

        for event in intermediate_events {
            let trigger = event.trigger.compile(self);
            let command = event.command.compile(self).0;

            match self.events.entry(trigger) {
                Entry::Occupied(occupied) => {
                    let existing = &mut self.command_lookup[*occupied.get()];
                    existing.extend(command);
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(self.command_lookup.len());
                    self.command_lookup.push(command);
                }
            }
        }

        mem::take(&mut self.events)
            .into_iter()
            .map(|(trigger, command)| Event(trigger, command))
            .collect()
    }

    fn compile_into_lookup<I: Compile<Output = (Vec<Command>, MemoryUsed)>>(
        &mut self,
        input: I,
    ) -> usize {
        // TODO are we allowed to ignore memoryused here?
        let (command, _) = input.compile(self);

        if let [Command::Execute(index)] = command.as_slice() {
            return *index;
        }

        let index = self.command_lookup.len();
        self.command_lookup.push(command);

        index
    }
}

pub trait Compile {
    type Output;

    fn compile(self, context: &mut CompileContext) -> Self::Output;
}

// TODO if the command is just one single execute, we should follow it and insert the execute index directly in the event
impl Compile for input::Event {
    type Output = Event;

    // TODO this is unused
    fn compile(self, context: &mut CompileContext) -> Self::Output {
        Event(
            self.trigger.compile(context),
            context.compile_into_lookup(self.command),
        )
    }
}

impl Compile for input::Trigger {
    type Output = Trigger;

    fn compile(self, context: &mut CompileContext) -> Self::Output {
        match self {
            Self::ClientEvent(trigger) => Trigger::ClientEvent(trigger),
            Self::Binding(uber_identifier) => Trigger::Binding(uber_identifier),
            Self::Condition(command) => Trigger::Condition(context.compile_into_lookup(command)),
        }
    }
}
