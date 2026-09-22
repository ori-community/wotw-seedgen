use std::{
    convert::Infallible,
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
    type Iter<'a, E>: Iterator<Item = Write<'a, E>>
    where
        Self: 'a,
        E: WriteExtra<'a>;

    fn direct_contained_writes<'a, E>(&'a self, lookup: &'a [CommandVoid]) -> Self::Iter<'a, E>
    where
        E: WriteExtra<'a>;

    fn contained_writes<'a, E>(
        &'a self,
        commands: &'a CommandsOutput,
    ) -> ContainedWritesIter<'a, Self::Iter<'a, E>, E>
    where
        E: WriteExtra<'a>,
    {
        ContainedWritesIter::new(self.direct_contained_writes(&commands.lookup), commands)
    }

    fn direct_contained_uber_state_writes<'a>(
        &'a self,
        lookup: &'a [CommandVoid],
    ) -> ContainedUberStateWrites<Self::Iter<'a, Infallible>> {
        ContainedUberStateWrites::new(self.direct_contained_writes(lookup))
    }

    fn contained_uber_state_writes<'a>(
        &'a self,
        commands: &'a CommandsOutput,
    ) -> ContainedUberStateWrites<ContainedWritesIter<'a, Self::Iter<'a, Infallible>, Infallible>>
    {
        ContainedUberStateWrites::new(self.contained_writes(commands))
    }

    fn contained_writes_with_shops<'a>(
        &'a self,
        commands: &'a CommandsOutput,
    ) -> ContainedWritesIter<'a, Self::Iter<'a, ShopWrite<'a>>, ShopWrite<'a>> {
        self.contained_writes(commands)
    }
}

impl ContainedWrites for CommandVoid {
    type Iter<'a, E: WriteExtra<'a>> = CommandVoidWrites<'a, E>;

    fn direct_contained_writes<'a, E>(&'a self, lookup: &'a [CommandVoid]) -> Self::Iter<'a, E>
    where
        E: WriteExtra<'a>,
    {
        CommandVoidWrites::new(self, lookup)
    }
}

impl ContainedWrites for Option<CommandVoid> {
    type Iter<'a, E: WriteExtra<'a>> = CommandVoidWrites<'a, E>;

    fn direct_contained_writes<'a, E>(&'a self, lookup: &'a [CommandVoid]) -> Self::Iter<'a, E>
    where
        E: WriteExtra<'a>,
    {
        match self {
            None => CommandVoidWrites {
                state: Vec::new(),
                lookup,
                visited_functions: FxHashSet::default(),
            },
            Some(command) => command.direct_contained_writes(lookup),
        }
    }
}

pub struct CommandVoidWrites<'a, E> {
    state: Vec<CommandVoidWritesState<'a, E>>,
    lookup: &'a [CommandVoid],
    visited_functions: FxHashSet<usize>,
}

impl<'a, E> CommandVoidWrites<'a, E>
where
    E: WriteExtra<'a>,
{
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

impl<'a, E> Iterator for CommandVoidWrites<'a, E>
where
    E: WriteExtra<'a>,
{
    type Item = Write<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.last_mut()? {
                CommandVoidWritesState::One(_) => {
                    let Some(CommandVoidWritesState::One(write)) = self.state.pop() else {
                        unreachable!()
                    };

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

pub enum Write<'a, E> {
    UberState(UberStateWrite<'a>),
    Extra(E),
}

impl<'a> Write<'a, Infallible> {
    pub fn into_uber_state(self) -> UberStateWrite<'a> {
        match self {
            Write::UberState(uber_state_write) => uber_state_write,
            Write::Extra(_) => unreachable!(),
        }
    }
}

pub enum ShopWrite<'a> {
    Hidden(ShopBooleanWrite<'a>),
    Locked(ShopBooleanWrite<'a>),
}

pub type ShopBooleanWrite<'a> = UberStateWriteGeneric<UberIdentifier, &'a CommandBoolean>;

pub type ShopBooleanWriteOwned = UberStateWriteGeneric<UberIdentifier, CommandBoolean>;

impl ShopBooleanWriteOwned {
    pub fn new(write: ShopBooleanWrite) -> Self {
        Self {
            uber_identifier: write.uber_identifier,
            command: write.command.clone(),
        }
    }
}

pub trait WriteExtra<'a>: 'a + Sized {
    fn extract(command: &'a CommandVoid) -> Option<Self>;
}

impl<'a> WriteExtra<'a> for Infallible {
    fn extract(_command: &CommandVoid) -> Option<Self> {
        None
    }
}

impl<'a> WriteExtra<'a> for ShopWrite<'a> {
    fn extract(command: &'a CommandVoid) -> Option<Self> {
        match command {
            CommandVoid::SetShopItemHidden {
                uber_identifier,
                hidden,
            } => Some(Self::Hidden(ShopBooleanWrite {
                uber_identifier: *uber_identifier,
                command: hidden,
            })),
            CommandVoid::SetShopItemLocked {
                uber_identifier,
                locked,
            } => Some(Self::Locked(ShopBooleanWrite {
                uber_identifier: *uber_identifier,
                command: locked,
            })),
            _ => None,
        }
    }
}

enum CommandVoidWritesState<'a, E> {
    One(Write<'a, E>),
    Multi(slice::Iter<'a, CommandVoid>),
}

impl<'a, E> CommandVoidWritesState<'a, E>
where
    E: WriteExtra<'a>,
{
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
            } => Some(Self::One(Write::UberState(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Boolean(value),
            }))),
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                ..
            } => Some(Self::One(Write::UberState(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Integer(value),
            }))),
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                ..
            } => Some(Self::One(Write::UberState(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Float(value),
            }))),
            other => E::extract(other).map(|e| Self::One(Write::Extra(e))),
        }
    }
}

pub struct ContainedWritesIter<'a, I, E> {
    inner: I,
    in_progress: Vec<CommandVoidWrites<'a, E>>,
    visited_events: FxHashSet<usize>,
    commands: &'a CommandsOutput,
}

impl<'a, I, E> ContainedWritesIter<'a, I, E>
where
    I: Iterator<Item = Write<'a, E>>,
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

impl<'a, I, E> Iterator for ContainedWritesIter<'a, I, E>
where
    I: Iterator<Item = Write<'a, E>>,
    E: WriteExtra<'a>,
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

        if let Write::UberState(next) = &next {
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
        }

        Some(next)
    }
}

pub struct ContainedUberStateWrites<I> {
    inner: I,
}

impl<I> ContainedUberStateWrites<I> {
    fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<'a, I> Iterator for ContainedUberStateWrites<I>
where
    I: Iterator<Item = Write<'a, Infallible>>,
{
    type Item = UberStateWrite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Write::into_uber_state)
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

    pub fn log_name(&self) -> CommonItemLogName<'_> {
        CommonItemLogName { inner: self }
    }
}

pub struct CommonItemLogName<'a> {
    inner: &'a CommonItem,
}

impl Display for CommonItemLogName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.inner {
            CommonItem::SpiritLight(amount) => write!(f, "{amount} Spirit Light"),
            CommonItem::HealthFragment => f.write_str("Health Fragment"),
            CommonItem::EnergyFragment => f.write_str("Energy Fragment"),
            CommonItem::GorlekOre => f.write_str("Gorlek Ore"),
            CommonItem::Keystone => f.write_str("Keystone"),
            CommonItem::ShardSlot => f.write_str("Shard Slot"),
            CommonItem::WeaponUpgrade(weapon_upgrade) => weapon_upgrade.fmt(f),
            CommonItem::Shard(shard) => shard.fmt(f),
            CommonItem::Teleporter(teleporter) => teleporter.display::<true>().fmt(f),
            CommonItem::Skill(skill) => skill.fmt(f),
            CommonItem::CleanWater => f.write_str("Clean Water"),
        }
    }
}
