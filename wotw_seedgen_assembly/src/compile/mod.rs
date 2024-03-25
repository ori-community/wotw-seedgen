mod args;
mod command;
mod package;

pub use package::compile_intermediate_output;

use self::command::MemoryUsed;
use crate::{Command, Event, Trigger};
use wotw_seedgen_seed_language::output::{self as input, StringOrPlaceholder};

// TODO dedup functions?

pub trait Compile {
    type Output;

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output;
}

// TODO if the command is just one single execute, we should follow it and insert the execute index directly in the event
impl Compile for input::Event {
    type Output = Event;

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        Event(
            self.trigger.compile(command_lookup),
            compile_into_lookup(self.command, command_lookup),
        )
    }
}

impl Compile for input::Trigger {
    type Output = Trigger;

    fn compile(self, command_lookup: &mut Vec<Vec<Command>>) -> Self::Output {
        match self {
            Self::ClientEvent(trigger) => Trigger::ClientEvent(trigger),
            Self::Binding(uber_identifier) => Trigger::Binding(uber_identifier.into()),
            Self::Condition(command) => {
                Trigger::Condition(compile_into_lookup(command, command_lookup))
            }
        }
    }
}

fn compile_into_lookup<I: Compile<Output = (Vec<Command>, MemoryUsed)>>(
    input: I,
    command_lookup: &mut Vec<Vec<Command>>,
) -> usize {
    // TODO are we allowed to ignore memoryused here?
    let (command, _) = input.compile(command_lookup);
    let index = command_lookup.len();
    command_lookup.push(command);
    index
}

fn unwrap_string_placeholder(value: StringOrPlaceholder) -> String {
    match value {
        StringOrPlaceholder::Value(value) => value,
        _ => panic!("Unresolved string placeholder"),
    }
}
