use super::{command::MemoryUsed, Compile};
use crate::{assembly::Command, compile::CompileContext};
use arrayvec::ArrayVec;
use wotw_seedgen_seed_language::{
    compile::RESERVED_MEMORY,
    output::{CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandZone},
};

pub struct Args<'a> {
    context: &'a mut CompileContext,
    args: ArrayVec<Arg, 4>,
    args_in_progress: Vec<ArgDestination>,
}

impl<'a> Args<'a> {
    pub fn new(context: &'a mut CompileContext) -> Self {
        Self {
            context,
            args: Default::default(),
            args_in_progress: Default::default(),
        }
    }

    fn arg(mut self, arg_destination: ArgDestination, arg: (Vec<Command>, MemoryUsed)) -> Self {
        self.args.push(Arg::new(arg_destination, arg));
        self
    }

    pub fn boolean(self, index: usize, arg: CommandBoolean) -> Self {
        let arg = arg.compile(self.context);
        self.arg(ArgDestination::Boolean(index), arg)
    }

    pub fn integer(self, index: usize, arg: CommandInteger) -> Self {
        let arg = arg.compile(self.context);
        self.arg(ArgDestination::Integer(index), arg)
    }

    pub fn float(self, index: usize, arg: CommandFloat) -> Self {
        let arg = arg.compile(self.context);
        self.arg(ArgDestination::Float(index), arg)
    }

    pub fn string(self, index: usize, arg: CommandString) -> Self {
        let arg = arg.compile(self.context);
        self.arg(ArgDestination::String(index), arg)
    }

    pub fn zone(self, index: usize, arg: CommandZone) -> Self {
        let arg = arg.compile(self.context);
        self.arg(ArgDestination::Integer(index), arg)
    }

    pub fn call(mut self, command: Command) -> (Vec<Command>, MemoryUsed) {
        self.build_overwrite_relations();
        let mut call_builder = CallBuilder::new(&self);

        while let Some(index) = self.select_next_arg() {
            self.commit_arg(index, &mut call_builder);
        }

        call_builder.finish(command)
    }

    pub fn call_multiple<I>(self, commands: I) -> (Vec<Command>, MemoryUsed)
    where
        I: IntoIterator<Item = Command>,
    {
        let mut iter = commands.into_iter();

        let mut out = self.call(iter.next().unwrap());

        out.0.extend(iter);

        out
    }

    fn total_memory_used(&self) -> MemoryUsed {
        self.args.iter().fold(MemoryUsed::ZERO, |mut total, arg| {
            total.combine(arg.compile_output.1.clone());
            arg.destination.reserve(&mut total);
            total
        })
    }

    fn select_next_arg(&self) -> Option<usize> {
        self.args
            .iter()
            .enumerate()
            .min_by_key(|(_, arg)| arg.gets_overwritten_by.len())
            .map(|(index, _)| index)
    }

    fn build_overwrite_relations(&mut self) {
        for index in 0..self.args.len() {
            let arg = &self.args[index];
            self.args[index].gets_overwritten_by = self
                .args
                .iter()
                .filter(|other_arg| {
                    arg.destination != other_arg.destination
                        && arg
                            .destination
                            .gets_overwritten(&other_arg.compile_output.1)
                })
                .map(|other_arg| other_arg.destination)
                .collect()
        }
    }

    fn commit_arg(&mut self, index: usize, call_builder: &mut CallBuilder) {
        let mut arg = self.args.swap_remove(index);
        let mut may_copy_directly = true;

        self.args_in_progress.push(arg.destination);

        for overwriting_arg_destination in arg.gets_overwritten_by {
            if let Some(index) = self
                .args
                .iter()
                .position(|arg| arg.destination == overwriting_arg_destination)
            {
                self.commit_arg(index, call_builder);
            } else if self.args_in_progress.contains(&overwriting_arg_destination) {
                may_copy_directly = false;
            }
        }

        self.args_in_progress.pop();

        call_builder.out.append(&mut arg.compile_output.0);
        if may_copy_directly {
            call_builder.copy_directly(arg.destination);
        } else {
            call_builder.copy_with_intermediate(arg.destination);
        }
    }
}

struct Arg {
    destination: ArgDestination,
    compile_output: (Vec<Command>, MemoryUsed),
    gets_overwritten_by: ArrayVec<ArgDestination, 4>,
}

impl Arg {
    fn new(destination: ArgDestination, compile_output: (Vec<Command>, MemoryUsed)) -> Self {
        Self {
            destination,
            compile_output,
            gets_overwritten_by: Default::default(),
        }
    }
}

struct CallBuilder {
    out: Vec<Command>,
    final_commands: Vec<Command>,
    total_memory_used: MemoryUsed,
}

impl CallBuilder {
    fn new(args: &Args) -> Self {
        Self {
            out: vec![],
            final_commands: vec![],
            total_memory_used: args.total_memory_used(),
        }
    }

    fn copy_directly(&mut self, arg_destination: ArgDestination) {
        self.out.extend(arg_destination.copy_directly());
    }

    fn copy_with_intermediate(&mut self, arg_destination: ArgDestination) {
        let (copy_away, copy_back) =
            arg_destination.copy_with_intermediate(&mut self.total_memory_used);
        self.out.push(copy_away);
        self.final_commands.push(copy_back);
    }

    fn finish(mut self, command: Command) -> (Vec<Command>, MemoryUsed) {
        self.out.append(&mut self.final_commands);
        self.out.push(command);
        (self.out, self.total_memory_used)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ArgDestination {
    Boolean(usize),
    Integer(usize),
    Float(usize),
    String(usize),
}

impl ArgDestination {
    fn reserve(&self, memory_used: &mut MemoryUsed) {
        match self {
            ArgDestination::Boolean(index) => {
                memory_used.boolean = usize::max(memory_used.boolean, *index)
            }
            ArgDestination::Integer(index) => {
                memory_used.integer = usize::max(memory_used.integer, *index)
            }
            ArgDestination::Float(index) => {
                memory_used.float = usize::max(memory_used.float, *index)
            }
            ArgDestination::String(index) => {
                memory_used.string = usize::max(memory_used.string, *index)
            }
        }
    }

    // TODO inaccuracy: we don't differentiate whether something overwrites memory zero or overwrites nothing
    // this might lead to unnecessary copies
    fn gets_overwritten(&self, memory_used: &MemoryUsed) -> bool {
        match self {
            ArgDestination::Boolean(index) => memory_used.boolean >= *index,
            ArgDestination::Integer(index) => memory_used.integer >= *index,
            ArgDestination::Float(index) => memory_used.float >= *index,
            ArgDestination::String(index) => memory_used.string >= *index,
        }
    }

    fn copy_directly(self) -> Option<Command> {
        match self {
            ArgDestination::Boolean(index) => (index > 0).then_some(Command::CopyBoolean(0, index)),
            ArgDestination::Integer(index) => (index > 0).then_some(Command::CopyInteger(0, index)),
            ArgDestination::Float(index) => (index > 0).then_some(Command::CopyFloat(0, index)),
            ArgDestination::String(index) => (index > 0).then_some(Command::CopyString(0, index)),
        }
    }

    fn copy_with_intermediate(self, total_memory_used: &mut MemoryUsed) -> (Command, Command) {
        let (used, command, index): (_, fn(_, _) -> _, _) = match self {
            ArgDestination::Boolean(index) => {
                (&mut total_memory_used.boolean, Command::CopyBoolean, index)
            }
            ArgDestination::Integer(index) => {
                (&mut total_memory_used.integer, Command::CopyInteger, index)
            }
            ArgDestination::Float(index) => {
                (&mut total_memory_used.float, Command::CopyFloat, index)
            }
            ArgDestination::String(index) => {
                (&mut total_memory_used.string, Command::CopyString, index)
            }
        };
        *used += 1;
        debug_assert!(*used < RESERVED_MEMORY);
        (command(0, *used), command(*used, index))
    }
}
