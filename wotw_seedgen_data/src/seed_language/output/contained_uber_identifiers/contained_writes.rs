use std::{
    fmt::{self, Display},
    slice,
};

use crate::{
    seed_language::output::{
        ArithmeticOperator, CommandBoolean, CommandFloat, CommandInteger, CommandVoid,
        CommandsOutput, Operation,
    },
    CommonUberIdentifier, Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
};
use ordered_float::OrderedFloat;
use rustc_hash::{FxBuildHasher, FxHashSet};
use strum::EnumTryAs;

pub trait ContainedWrites {
    type Iter<'a>: Iterator<Item = UberStateWrite<'a>>
    where
        Self: 'a;

    fn direct_contained_writes<'a>(&'a self, lookup: &'a [CommandVoid]) -> Self::Iter<'a>;

    fn contained_writes<'a>(
        &'a self,
        commands: &'a CommandsOutput,
    ) -> ContainedWritesIter<'a, Self::Iter<'a>> {
        ContainedWritesIter::new(self.direct_contained_writes(&commands.lookup), commands)
    }
}

impl ContainedWrites for CommandVoid {
    type Iter<'a> = CommandVoidWrites<'a>;

    fn direct_contained_writes<'a>(&'a self, lookup: &'a [CommandVoid]) -> Self::Iter<'a> {
        CommandVoidWrites::new(self, lookup)
    }
}

pub struct CommandVoidWrites<'a> {
    state: Vec<CommandVoidWritesState<'a>>,
    lookup: &'a [CommandVoid],
    visited_functions: FxHashSet<usize>,
}

impl<'a> CommandVoidWrites<'a> {
    fn new(command: &'a CommandVoid, lookup: &'a [CommandVoid]) -> Self {
        let mut visited_functions = FxHashSet::with_hasher(FxBuildHasher);
        let state = CommandVoidWritesState::new(command, lookup, &mut visited_functions);

        Self {
            state: state.into_iter().collect(),
            lookup,
            visited_functions,
        }
    }
}

impl<'a> Iterator for CommandVoidWrites<'a> {
    type Item = UberStateWrite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.last_mut()? {
                CommandVoidWritesState::One(write) => {
                    let write = write.clone();
                    self.state.pop();
                    return Some(write);
                }
                CommandVoidWritesState::Multi(nested) => {
                    match nested.find_map(|command| {
                        CommandVoidWritesState::new(
                            command,
                            self.lookup,
                            &mut self.visited_functions,
                        )
                    }) {
                        None => {
                            self.state.pop();
                        }
                        Some(state) => self.state.push(state),
                    }
                }
            }
        }
    }
}

enum CommandVoidWritesState<'a> {
    One(UberStateWrite<'a>),
    Multi(slice::Iter<'a, CommandVoid>),
}

impl<'a> CommandVoidWritesState<'a> {
    fn new(
        command: &'a CommandVoid,
        lookup: &'a [CommandVoid],
        visited_functions: &mut FxHashSet<usize>,
    ) -> Option<Self> {
        match command {
            CommandVoid::Multi { commands } => Some(Self::Multi(commands.iter())),
            CommandVoid::CallFunction { index, .. } => {
                if visited_functions.insert(*index) {
                    Self::new(&lookup[*index], lookup, visited_functions)
                } else {
                    None
                }
            }
            CommandVoid::If { command, .. } => Self::new(command, lookup, visited_functions),
            // TODO this might be fine for the current use case, but an exhaustive list would be safer
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                ..
            } => Some(Self::One(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Boolean(value),
            })),
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                ..
            } => Some(Self::One(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Integer(value),
            })),
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                ..
            } => Some(Self::One(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Float(value),
            })),
            _ => None,
        }
    }
}

pub struct ContainedWritesIter<'a, I> {
    inner: I,
    in_progress: Vec<CommandVoidWrites<'a>>,
    visited_events: FxHashSet<usize>,
    commands: &'a CommandsOutput,
}

impl<'a, I> ContainedWritesIter<'a, I>
where
    I: Iterator<Item = UberStateWrite<'a>>,
{
    fn new(inner: I, commands: &'a CommandsOutput) -> Self {
        Self {
            inner,
            in_progress: Vec::new(),
            visited_events: FxHashSet::with_hasher(FxBuildHasher),
            commands,
        }
    }
}

impl<'a, I> Iterator for ContainedWritesIter<'a, I>
where
    I: Iterator<Item = UberStateWrite<'a>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = loop {
            match self.in_progress.last_mut() {
                None => break self.inner.next()?,
                Some(in_progress) => match in_progress.next() {
                    None => {
                        self.in_progress.pop();
                    }
                    Some(in_progress_next) => break in_progress_next,
                },
            }
        };

        if let Some(triggers) = self.commands.trigger_map.get(&next.uber_identifier) {
            for &index in triggers {
                if self.visited_events.insert(index) {
                    self.in_progress.push(
                        self.commands.events[index]
                            .command
                            .direct_contained_writes(&self.commands.lookup),
                    );
                }
            }
        }

        Some(next)
    }
}

pub trait ContainedWritesExt<'a>: Sized + IntoIterator<Item = UberStateWrite<'a>> {
    fn owned(self) -> impl Iterator<Item = UberStateWriteOwned> {
        self.into_iter().map(UberStateWriteOwned::new)
    }

    fn identifiers(self) -> impl Iterator<Item = UberIdentifier> {
        self.into_iter().map(|write| write.uber_identifier)
    }

    fn common(self) -> impl Iterator<Item = CommonUberStateWrite> {
        self.into_iter()
            .filter_map(CommonUberStateWrite::from_write)
    }

    fn common_identifiers(self) -> impl Iterator<Item = CommonUberIdentifier> {
        self.identifiers()
            .filter_map(CommonUberIdentifier::from_uber_identifier)
    }

    fn common_items(self) -> impl Iterator<Item = CommonItem> {
        self.common().filter_map(CommonItem::from_common_write)
    }
}

impl<'a, I> ContainedWritesExt<'a> for I where I: IntoIterator<Item = UberStateWrite<'a>> {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UberStateWriteGeneric<U, C> {
    pub uber_identifier: U,
    pub command: C,
}

pub type UberStateWrite<'a> = UberStateWriteGeneric<UberIdentifier, WriteCommand<'a>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumTryAs)]
pub enum WriteCommand<'a> {
    Boolean(&'a CommandBoolean),
    Integer(&'a CommandInteger),
    Float(&'a CommandFloat),
}

impl Display for WriteCommand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(command) => command.fmt(f),
            Self::Integer(command) => command.fmt(f),
            Self::Float(command) => command.fmt(f),
        }
    }
}

pub type UberStateWriteOwned = UberStateWriteGeneric<UberIdentifier, WriteCommandOwned>;

impl UberStateWriteOwned {
    pub fn new(write: UberStateWrite) -> Self {
        Self {
            uber_identifier: write.uber_identifier,
            command: WriteCommandOwned::new(write.command),
        }
    }

    pub fn as_ref(&self) -> UberStateWrite<'_> {
        UberStateWrite {
            uber_identifier: self.uber_identifier,
            command: self.command.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumTryAs)]
pub enum WriteCommandOwned {
    Boolean(CommandBoolean),
    Integer(CommandInteger),
    Float(CommandFloat),
}

impl Display for WriteCommandOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(command) => command.fmt(f),
            Self::Integer(command) => command.fmt(f),
            Self::Float(command) => command.fmt(f),
        }
    }
}

impl WriteCommandOwned {
    pub fn new(write_command: WriteCommand) -> Self {
        match write_command {
            WriteCommand::Boolean(command) => Self::Boolean(command.clone()),
            WriteCommand::Integer(command) => Self::Integer(command.clone()),
            WriteCommand::Float(command) => Self::Float(command.clone()),
        }
    }

    pub fn as_ref(&self) -> WriteCommand<'_> {
        match self {
            Self::Boolean(command) => WriteCommand::Boolean(command),
            Self::Integer(command) => WriteCommand::Integer(command),
            Self::Float(command) => WriteCommand::Float(command),
        }
    }
}

pub type CommonUberStateWrite = UberStateWriteGeneric<CommonUberIdentifier, CommonWriteCommand>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonWriteCommand {
    SetBooleanTrue,
    AddInteger(i32),
    AddFloat(OrderedFloat<f32>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonItem {
    SpiritLight(i32),
    HealthFragment,
    EnergyFragment,
    GorlekOre,
    Keystone,
    ShardSlot,
    WeaponUpgrade(WeaponUpgrade),
    Shard(Shard),
    Teleporter(Teleporter),
    Skill(Skill),
    CleanWater,
}

impl CommonUberStateWrite {
    pub fn from_write(write: UberStateWrite) -> Option<Self> {
        let uber_identifier = CommonUberIdentifier::from_uber_identifier(write.uber_identifier)?;
        let command = CommonWriteCommand::from_write(write)?;

        Some(Self {
            uber_identifier,
            command,
        })
    }
}

impl CommonWriteCommand {
    pub fn from_write(write: UberStateWrite) -> Option<Self> {
        match write.command {
            WriteCommand::Boolean(CommandBoolean::Constant { value: true }) => {
                Some(CommonWriteCommand::SetBooleanTrue)
            }
            WriteCommand::Integer(CommandInteger::Arithmetic { operation }) => match &**operation {
                Operation {
                    left: CommandInteger::FetchInteger { uber_identifier },
                    operator: ArithmeticOperator::Add,
                    right: CommandInteger::Constant { value },
                } if *uber_identifier == write.uber_identifier => {
                    Some(CommonWriteCommand::AddInteger(*value))
                }
                _ => None,
            },
            WriteCommand::Float(CommandFloat::Arithmetic { operation }) => match &**operation {
                Operation {
                    left: CommandFloat::FetchFloat { uber_identifier },
                    operator: ArithmeticOperator::Add,
                    right: CommandFloat::Constant { value },
                } if *uber_identifier == write.uber_identifier => {
                    Some(CommonWriteCommand::AddFloat(*value))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

impl CommonItem {
    pub fn from_common_write(write: CommonUberStateWrite) -> Option<Self> {
        match write {
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::SpiritLight,
                command: CommonWriteCommand::AddInteger(amount),
            } => Some(Self::SpiritLight(amount)),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::GorlekOre,
                command: CommonWriteCommand::AddInteger(1),
            } => Some(Self::GorlekOre),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::Keystones,
                command: CommonWriteCommand::AddInteger(1),
            } => Some(Self::Keystone),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::ShardSlots,
                command: CommonWriteCommand::AddInteger(1),
            } => Some(Self::ShardSlot),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::CleanWater,
                command: CommonWriteCommand::SetBooleanTrue,
            } => Some(Self::CleanWater),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::BaseMaxHealth,
                command: CommonWriteCommand::AddInteger(5),
            } => Some(Self::HealthFragment),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::BaseMaxEnergy,
                command: CommonWriteCommand::AddFloat(OrderedFloat(0.5)),
            } => Some(Self::EnergyFragment),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::Skill(skill),
                command: CommonWriteCommand::SetBooleanTrue,
            } => Some(Self::Skill(skill)),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::Shard(shard),
                command: CommonWriteCommand::SetBooleanTrue,
            } => Some(Self::Shard(shard)),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::Teleporter(teleporter),
                command: CommonWriteCommand::SetBooleanTrue,
            } => Some(Self::Teleporter(teleporter)),
            CommonUberStateWrite {
                uber_identifier: CommonUberIdentifier::WeaponUpgrade(weapon_upgrade),
                command: CommonWriteCommand::SetBooleanTrue,
            } => Some(Self::WeaponUpgrade(weapon_upgrade)),
            _ => None,
        }
    }
}
