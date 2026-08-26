//! When compiling functions, we make two levels of distinction:
//!
//! Level 1: Is the argument calculated before or after stack push?
//!
//! We always prefer to calculate after the stack push, then we can move the result into the stack directly.
//! But this is only valid if the calculation requires no values from the previous stack!
//! If anything from the previous stack is used, the value has to be calculated before the stack push
//! and kept somewhere in memory before it is pushed onto the stack in a later step.
//!
//! Level 2: If it is calculated before stack push, does it need an intermediate copy?
//!
//! Preferrably, we let the argument sit in memory 0 to be pushed onto the stack directly later.
//! But this is only valid if no other argument calculation overwrites it.
//! If it would be overwritten, we make an intermediate copy and don't really care much about how far away it is.

use std::{collections::VecDeque, mem};

use itertools::Itertools;
use wotw_seedgen_data::seed_language::output::{
    CommandBoolean, CommandFloat, CommandInteger, CommandString,
};

use crate::{
    assembly::Command,
    compile::{command::MemoryUsed, Compile, CompileContext},
};

pub struct FunctionCompiler<'ctx> {
    context: &'ctx mut CompileContext,
    pre_push_args: PrePushArgs,
    post_push_args: PostPushArgs,
}

impl<'ctx> FunctionCompiler<'ctx> {
    pub fn new(context: &'ctx mut CompileContext) -> Self {
        Self {
            context,
            pre_push_args: PrePushArgs::new(),
            post_push_args: PostPushArgs::new(),
        }
    }

    pub fn boolean(&mut self, index: usize, boolean: CommandBoolean) {
        self.arg(index, boolean);
    }

    pub fn integer(&mut self, index: usize, integer: CommandInteger) {
        self.arg(index, integer);
    }

    pub fn float(&mut self, index: usize, float: CommandFloat) {
        self.arg(index, float);
    }

    pub fn string(&mut self, index: usize, string: CommandString) {
        self.arg(index, string);
    }

    pub fn finish(mut self, index: usize) -> (Vec<Command>, MemoryUsed) {
        let mut commands = vec![];

        let (pre_push_args, mut memory_used) = self.pre_push_args.finish();
        memory_used.combine(self.post_push_args.memory_used.clone());

        for pre_push_arg in pre_push_args {
            let needs_intermediate_copy =
                pre_push_arg.needs_intermediate_copy(&self.post_push_args.memory_used);

            commands.extend(pre_push_arg.commands);

            let mut post_push_commands =
                Vec::with_capacity(1 + usize::from(needs_intermediate_copy));

            if needs_intermediate_copy {
                let (copy_away, copy_back) =
                    pre_push_arg.destination.intermediate_copy(&mut memory_used);

                commands.push(copy_away);
                post_push_commands.push(copy_back);
            }

            let post_push = &mut self.post_push_args.args[pre_push_arg.index.0];
            debug_assert!(matches!(post_push, PostPushArg::CopyPlaceholder));

            post_push_commands.push(pre_push_arg.destination.stack_push());
            *post_push = PostPushArg::Finished(post_push_commands);
        }

        commands.push(Command::StackPush);

        for post_push_arg in self.post_push_args.args {
            let PostPushArg::Finished(arg) = post_push_arg else {
                unreachable!()
            };

            commands.extend(arg);
        }

        commands.extend([Command::Execute(index), Command::StackPop]);

        (commands, memory_used)
    }

    fn arg<T: FunctionArg>(&mut self, index: usize, arg: T) {
        let (commands, memory_used) = arg.compile(self.context);

        if uses_current_stack(&commands) {
            self.pre_push_args.arg::<T>(index, commands, memory_used);
            self.post_push_args.copy_placeholder();
        } else {
            self.post_push_args.arg::<T>(commands, memory_used);
        }
    }
}

struct PrePushArgs {
    args: Vec<PrePushArg>,
    memory_used: MemoryUsed,
}

impl PrePushArgs {
    fn new() -> Self {
        Self {
            args: Vec::new(),
            memory_used: MemoryUsed::ZERO,
        }
    }

    fn arg<T: FunctionArg>(
        &mut self,
        index: usize,
        commands: Vec<Command>,
        memory_used: MemoryUsed,
    ) {
        self.args
            .push(PrePushArg::new::<T>(index, commands, memory_used.clone()));

        self.memory_used.combine(memory_used);
    }

    fn finish(mut self) -> (PrePushArgSelector, MemoryUsed) {
        self.build_overwrite_relations();

        (PrePushArgSelector::new(self.args), self.memory_used)
    }

    fn build_overwrite_relations(&mut self) {
        for index in 0..self.args.len() {
            let destination = self.args[index].destination;

            self.args[index].gets_overwritten_by = (0..index)
                .chain(index + 1..self.args.len())
                .map(|other| &self.args[other])
                .filter(|other| destination.gets_overwritten_by(&other.memory_used))
                .map(|other| other.index)
                .collect();
        }
    }
}

struct PrePushArg {
    index: ArgIndex,
    destination: FunctionArgDestination,
    commands: Vec<Command>,
    memory_used: MemoryUsed,
    gets_overwritten_by: Vec<ArgIndex>,
}

impl PrePushArg {
    fn new<T: FunctionArg>(index: usize, commands: Vec<Command>, memory_used: MemoryUsed) -> Self {
        Self {
            index: ArgIndex(index),
            destination: T::DESTINATION,
            commands,
            memory_used,
            gets_overwritten_by: vec![],
        }
    }

    fn needs_intermediate_copy(&self, post_push_memory_used: &MemoryUsed) -> bool {
        !self.gets_overwritten_by.is_empty()
            || self.destination.gets_overwritten_by(post_push_memory_used)
    }
}

/// Newtype to denote which of the function's argument this is without risk of confusion with other indices
#[derive(Clone, Copy, PartialEq, Eq)]
struct ArgIndex(usize);

#[derive(Clone, Copy)]
enum FunctionArgDestination {
    Boolean,
    Integer,
    Float,
    String,
}

impl FunctionArgDestination {
    fn gets_overwritten_by(self, memory_used: &MemoryUsed) -> bool {
        match self {
            Self::Boolean => memory_used.boolean > 0,
            Self::Integer => memory_used.integer > 0,
            Self::Float => memory_used.float > 0,
            Self::String => memory_used.string > 0,
        }
    }

    fn intermediate_copy(self, memory_used: &mut MemoryUsed) -> (Command, Command) {
        let (copy, index): (fn(usize, usize) -> Command, _) = match self {
            Self::Boolean => {
                let previous = memory_used.boolean;
                memory_used.boolean += 1;
                (Command::CopyBoolean, previous)
            }
            Self::Integer => {
                let previous = memory_used.integer;
                memory_used.integer += 1;
                (Command::CopyInteger, previous)
            }
            Self::Float => {
                let previous = memory_used.float;
                memory_used.float += 1;
                (Command::CopyFloat, previous)
            }
            Self::String => {
                let previous = memory_used.string;
                memory_used.string += 1;
                (Command::CopyString, previous)
            }
        };

        (copy(0, index), copy(index, 0))
    }

    const fn stack_push(self) -> Command {
        match self {
            Self::Boolean => Command::StackPushBoolean,
            Self::Integer => Command::StackPushInteger,
            Self::Float => Command::StackPushFloat,
            Self::String => Command::StackPushString,
        }
    }
}

struct PrePushArgSelector {
    args: Vec<PrePushArg>,
    queue: VecDeque<PrePushArg>,
}

impl PrePushArgSelector {
    fn new(args: Vec<PrePushArg>) -> Self {
        Self {
            args,
            queue: VecDeque::new(),
        }
    }
}

impl Iterator for PrePushArgSelector {
    type Item = PrePushArg;

    fn next(&mut self) -> Option<Self::Item> {
        fn select(remaining_args: &mut [PrePushArg], arg: PrePushArg) -> PrePushArg {
            for remaining in remaining_args {
                remaining
                    .gets_overwritten_by
                    .retain(|index| *index != arg.index);
            }

            arg
        }

        if let queued @ Some(_) = self.queue.pop_front() {
            return queued;
        }

        let index = self
            .args
            .iter()
            .position_min_by_key(|arg| arg.gets_overwritten_by.len())?;

        let mut arg = self.args.swap_remove(index);

        if arg.gets_overwritten_by.is_empty() {
            return Some(select(&mut self.args, arg));
        }

        self.queue.reserve_exact(arg.gets_overwritten_by.len());

        let mut next_args = mem::take(&mut arg.gets_overwritten_by)
            .into_iter()
            .rev()
            .map(|index| {
                let index = self.args.iter().position(|arg| arg.index == index).unwrap();
                let arg = self.args.swap_remove(index);
                select(&mut self.args, arg)
            });

        let next = next_args.next().unwrap();

        self.queue.extend(next_args);
        self.queue.push_back(select(&mut self.args, arg));

        Some(next)
    }
}

struct PostPushArgs {
    args: Vec<PostPushArg>,
    memory_used: MemoryUsed,
}

impl PostPushArgs {
    fn new() -> Self {
        Self {
            args: Vec::new(),
            memory_used: MemoryUsed::ZERO,
        }
    }

    fn arg<T: FunctionArg>(&mut self, mut commands: Vec<Command>, memory_used: MemoryUsed) {
        commands.push(T::STACK_PUSH);
        self.args.push(PostPushArg::Finished(commands));

        self.memory_used.combine(memory_used.clone());
    }

    fn copy_placeholder(&mut self) {
        self.args.push(PostPushArg::CopyPlaceholder);
    }
}

enum PostPushArg {
    Finished(Vec<Command>),
    CopyPlaceholder,
}

fn uses_current_stack(commands: &[Command]) -> bool {
    let mut depth = 0;

    for command in commands {
        match command {
            Command::StackPush => depth += 1,
            Command::StackPop => depth -= 1,
            Command::StackCopyBoolean(_)
            | Command::StackCopyInteger(_)
            | Command::StackCopyFloat(_)
            | Command::StackCopyString(_)
                if depth == 0 =>
            {
                return true
            }
            _ => {}
        }
    }

    false
}

trait FunctionArg: Compile<Output = (Vec<Command>, MemoryUsed)> {
    const DESTINATION: FunctionArgDestination;
    const STACK_PUSH: Command = Self::DESTINATION.stack_push();
}

impl FunctionArg for CommandBoolean {
    const DESTINATION: FunctionArgDestination = FunctionArgDestination::Boolean;
}

impl FunctionArg for CommandInteger {
    const DESTINATION: FunctionArgDestination = FunctionArgDestination::Integer;
}

impl FunctionArg for CommandFloat {
    const DESTINATION: FunctionArgDestination = FunctionArgDestination::Float;
}

impl FunctionArg for CommandString {
    const DESTINATION: FunctionArgDestination = FunctionArgDestination::String;
}
