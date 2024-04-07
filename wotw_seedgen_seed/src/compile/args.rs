use super::{command::MemoryUsed, Compile};
use crate::assembly::Command;
use rustc_hash::FxHashMap;
use wotw_seedgen_seed_language::{
    compile::RESERVED_MEMORY,
    output::{CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandZone},
};

pub struct Args<'a> {
    command_lookup: &'a mut Vec<Vec<Command>>,
    commands: Vec<(usize, ArgType, (Vec<Command>, MemoryUsed))>,
}
impl<'a> Args<'a> {
    #[inline]
    pub fn new(_arg_count: usize, command_lookup: &'a mut Vec<Vec<Command>>) -> Self {
        Self {
            command_lookup,
            commands: vec![], // TODO can we estimate the needed capacity? kept arg_count in case we want it for that
        }
    }

    // TODO once deduplication exists, check if moving commands out into function calls is ever worth it
    fn arg<T>(mut self, arg: T, arg_type: ArgType) -> Self
    where
        T: Compile<Output = (Vec<Command>, MemoryUsed)>,
    {
        self.commands.push((
            self.commands.len(),
            arg_type,
            arg.compile(self.command_lookup),
        ));
        self
    }
    #[inline]
    pub fn boolean(self, arg: CommandBoolean) -> Self {
        self.arg(arg, ArgType::Boolean)
    }
    #[inline]
    pub fn integer(self, arg: CommandInteger) -> Self {
        self.arg(arg, ArgType::Integer)
    }
    #[inline]
    pub fn float(self, arg: CommandFloat) -> Self {
        self.arg(arg, ArgType::Float)
    }
    #[inline]
    pub fn string(self, arg: CommandString) -> Self {
        self.arg(arg, ArgType::String)
    }
    #[inline]
    pub fn zone(self, arg: CommandZone) -> Self {
        self.arg(arg, ArgType::Integer)
    }

    pub fn call(mut self, command: Command) -> (Vec<Command>, MemoryUsed) {
        // TODO sometimes reordering operands is allowed and would result in a more optimal output

        // The order in which we calculate the arguments matters a lot for how many copies we have to do
        // After all is done, the arguments have to be in memory 0, 1, 2... etc.
        // However, the calculation of arguments may need this memory itself
        // Sometimes this is unavoidable and we have to copy results out of the way and get them back later
        // But we still want to copy as many results as possible directly into their final destination
        // In the optimal scenario we can calculate all arguments last to first and copy them into their destination directly
        // Calculating arguments in reverse order isn't always the best strategy though
        // For example if the first of four integer arguments requires 4 integer memory to calculate and everything else requires 0 memory,
        // calculating the first argument last would require moving all others out of the way and copying them back (3 extra copies),
        // while calculating the first argument first would only require copying that out of the way and back later (2 extra copies)
        // Specifically, we employ the following strategies:
        // - Calculating the first argument last saves 2 copies (since it naturally lands in memory 0)
        // - Copying any other argument directly into its destination saves 1 copy

        let mut output = vec![]; // TODO capacity
        let mut back_copies = vec![];
        let mut occupied_intermediate_locations = FxHashMap::<ArgType, Vec<usize>>::default();
        let mut total_memory_used = self.commands.iter().fold(
            MemoryUsed::ZERO,
            |mut memory_used, (memory_destination, arg_type, (_, other))| {
                memory_used.combine(other.clone());
                let arg_type_memory_used = arg_type.memory_used_mut(&mut memory_used);
                *arg_type_memory_used = usize::max(*arg_type_memory_used, *memory_destination);
                memory_used
            },
        );

        while !self.commands.is_empty() {
            let args_except_last = &self.commands[..self.commands.len() - 1];
            let prioritized_arg_index = args_except_last.iter().enumerate().rev().find_map(
                |(index, (_memory_destination, _, (_, memory_used)))| {
                    let args_after = &self.commands[index + 1..];
                    let mut args_which_this_overwrites =
                        args_after
                            .iter()
                            .filter(|(memory_destination, other_arg_type, _)| {
                                other_arg_type.memory_used(memory_used) >= *memory_destination
                            });
                    // TODO implement strategy to avoid copies on the first arg
                    // let is_first_arg = *memory_destination == 0;
                    // let overwrite_threshold = if is_first_arg { 1 } else { 0 };
                    let overwrite_threshold = 0;
                    let should_prioritize = args_which_this_overwrites
                        .nth(overwrite_threshold)
                        .is_some();
                    if should_prioritize {
                        Some(index)
                    } else {
                        None
                    }
                },
            );
            match prioritized_arg_index {
                None => {
                    for (memory_destination, arg_type, (commands, _)) in
                        self.commands.into_iter().rev()
                    {
                        output.extend(commands);
                        if memory_destination != 0 {
                            output.push(arg_type.copy_command()(0, memory_destination));
                        }
                    }
                    break;
                }
                Some(index) => {
                    // TODO can we swap_remove?
                    let (memory_destination, arg_type, (commands, _)) = self.commands.remove(index);
                    output.extend(commands);
                    let max_memory_used = self
                        .commands
                        .iter()
                        .map(|(memory_destination, other_arg_type, (_, memory_used))| {
                            let mut used = arg_type.memory_used(memory_used);
                            if arg_type == *other_arg_type {
                                used = usize::max(used, *memory_destination);
                            }
                            used
                        })
                        .max()
                        .unwrap_or_default();
                    let occupied_intermediate_locations =
                        occupied_intermediate_locations.entry(arg_type).or_default();
                    let intermediate_location = ((max_memory_used + 1)..)
                        .find(|location| !occupied_intermediate_locations.contains(location))
                        .unwrap();
                    assert!(
                        intermediate_location < RESERVED_MEMORY,
                        "insufficient memory to perform calculation"
                    );
                    occupied_intermediate_locations.push(intermediate_location);
                    let copy_command = arg_type.copy_command();
                    output.push(copy_command(0, intermediate_location));
                    back_copies.push(copy_command(intermediate_location, memory_destination));
                    let total_memory_used = arg_type.memory_used_mut(&mut total_memory_used);
                    *total_memory_used = usize::max(*total_memory_used, intermediate_location);
                }
            }
        }

        output.extend(back_copies);
        output.push(command);

        (output, total_memory_used)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ArgType {
    Boolean,
    Integer,
    Float,
    String,
}
impl ArgType {
    #[inline]
    fn memory_used(self, memory_used: &MemoryUsed) -> usize {
        match self {
            ArgType::Boolean => memory_used.boolean,
            ArgType::Integer => memory_used.integer,
            ArgType::Float => memory_used.float,
            ArgType::String => memory_used.string,
        }
    }
    #[inline]
    fn memory_used_mut(self, memory_used: &mut MemoryUsed) -> &mut usize {
        match self {
            ArgType::Boolean => &mut memory_used.boolean,
            ArgType::Integer => &mut memory_used.integer,
            ArgType::Float => &mut memory_used.float,
            ArgType::String => &mut memory_used.string,
        }
    }
    #[inline]
    fn copy_command(self) -> fn(usize, usize) -> Command {
        match self {
            ArgType::Boolean => Command::CopyBoolean,
            ArgType::Integer => Command::CopyInteger,
            ArgType::Float => Command::CopyFloat,
            ArgType::String => Command::CopyString,
        }
    }
}
