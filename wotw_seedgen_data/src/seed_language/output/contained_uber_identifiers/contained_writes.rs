use std::fmt::{self, Display};

use crate::{
    seed_language::output::{
        ArithmeticOperator, CommandBoolean, CommandFloat, CommandInteger, CommandVoid, Operation,
    },
    CommonUberIdentifier, Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
};
use ordered_float::OrderedFloat;
use strum::EnumTryAs;

use super::{none, some};

fn nested<'a, T>(nested: &'a T) -> Box<dyn Iterator<Item = UberStateWrite<'a>> + 'a>
where
    T: ContainedWrites,
{
    Box::new(nested.contained_writes())
}

pub trait ContainedWrites {
    fn contained_writes(&self) -> impl Iterator<Item = UberStateWrite<'_>>;

    fn contained_writes_owned(&self) -> impl Iterator<Item = UberStateWriteOwned> {
        self.contained_writes().map(UberStateWriteOwned::new)
    }

    fn contained_write_identifiers(&self) -> impl Iterator<Item = UberIdentifier> {
        self.contained_writes().map(|write| write.uber_identifier)
    }

    fn contained_common_writes(&self) -> impl Iterator<Item = CommonUberStateWrite> {
        self.contained_writes()
            .filter_map(CommonUberStateWrite::from_write)
    }

    fn contained_common_write_identifiers(&self) -> impl Iterator<Item = CommonUberIdentifier> {
        self.contained_write_identifiers()
            .filter_map(CommonUberIdentifier::from_uber_identifier)
    }

    fn contained_common_items(&self) -> impl Iterator<Item = CommonItem> {
        self.contained_common_writes()
            .filter_map(CommonItem::from_common_write)
    }
}

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

impl<T: ContainedWrites> ContainedWrites for Vec<T> {
    fn contained_writes(&self) -> impl Iterator<Item = UberStateWrite<'_>> {
        self.iter().flat_map(T::contained_writes)
    }
}

impl ContainedWrites for Vec<UberStateWriteOwned> {
    fn contained_writes(&self) -> impl Iterator<Item = UberStateWrite<'_>> {
        self.iter().map(UberStateWriteOwned::as_ref)
    }
}

impl ContainedWrites for CommandVoid {
    fn contained_writes(&self) -> impl Iterator<Item = UberStateWrite<'_>> {
        match self {
            CommandVoid::Multi { commands } => nested(commands),
            // TODO Lookup could be relevant both here and in reads
            // TODO this might be fine for the current use case, but an exhaustive list would be safer
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                ..
            } => some(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Boolean(value),
            }),
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                ..
            } => some(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Integer(value),
            }),
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                ..
            } => some(UberStateWrite {
                uber_identifier: *uber_identifier,
                command: WriteCommand::Float(value),
            }),
            _ => none(),
        }
    }
}

impl CommonUberStateWrite {
    pub fn from_write(write: UberStateWrite) -> Option<Self> {
        let uber_identifier = CommonUberIdentifier::from_uber_identifier(write.uber_identifier)?;
        let command = CommonWriteCommand::from_write(&write)?;

        Some(Self {
            uber_identifier,
            command,
        })
    }
}

impl CommonWriteCommand {
    pub fn from_write(write: &UberStateWrite) -> Option<Self> {
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
