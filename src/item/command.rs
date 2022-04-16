use std::fmt;

use num_enum::TryFromPrimitive;
use seedgen_derive::VVariant;

use super::{Item, VItem, Resource};
use crate::util::{UberIdentifier, UberState, VUberState, Position, VPosition, NumericBool};
use crate::header::{V, VResolve, VString};

#[derive(Debug, PartialEq, Eq, Hash, Clone, VVariant)]
pub enum Command {
    Autosave,
    Resource { resource: Resource, #[VWrap] amount: i16 },
    Checkpoint,
    Magic,
    StopEqual { #[VType] uber_state: UberState },
    StopGreater { #[VType] uber_state: UberState },
    StopLess { #[VType] uber_state: UberState },
    Toggle { target: ToggleCommand, #[VWrap] on: NumericBool },
    Warp { #[VType] position: Position },
    StartTimer { identifier: UberIdentifier },
    StopTimer { identifier: UberIdentifier },
    StateRedirect { intercept: i32, set: i32 },
    SetHealth { #[VWrap] amount: i16 },
    SetEnergy { #[VWrap] amount: i16 },
    SetSpiritLight { #[VWrap] amount: i16 },
    Equip { #[VWrap] slot: EquipSlot, ability: u16 },
    AhkSignal { signal: String },
    IfEqual { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    IfGreater { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    IfLess { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    DisableSync { uber_state: UberState },
    EnableSync { uber_state: UberState },
    CreateWarp { id: u8, #[VType] position: Position },
    DestroyWarp { id: u8 },
    IfBox { #[VType] position1: Position, #[VType] position2: Position, #[VType] item: Box<Item> },
    IfSelfEqual { #[VWrap] value: String, #[VType] item: Box<Item> },
    IfSelfGreater { #[VWrap] value: String, #[VType] item: Box<Item> },
    IfSelfLess { #[VWrap] value: String, #[VType] item: Box<Item> },
    UnEquip { ability: u16 },
    SaveString { id: i32, #[VType] string: String },
    AppendString { id: i32, #[VType] string: String },
}
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Autosave => write!(f, "0"),
            Command::Resource { resource, amount } => write!(f, "1|{}|{}", *resource as u8, amount),
            Command::Checkpoint => write!(f, "2"),
            Command::Magic => write!(f, "3"),
            Command::StopEqual { uber_state } => write!(f, "4|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopGreater { uber_state } => write!(f, "5|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopLess { uber_state } => write!(f, "6|{}|{}", uber_state.identifier, uber_state.value),
            Command::Toggle { target, on } => write!(f, "7|{}|{}", target, *on as u8),
            Command::Warp { position } => write!(f, "8|{}", position.code()),
            Command::StartTimer { identifier } => write!(f, "9|{}", identifier),
            Command::StopTimer { identifier } => write!(f, "10|{}", identifier),
            Command::StateRedirect { intercept, set } => write!(f, "11|{}|{}", intercept, set),
            Command::SetHealth { amount } => write!(f, "12|{}", amount),
            Command::SetEnergy { amount } => write!(f, "13|{}", amount),
            Command::SetSpiritLight { amount } => write!(f, "14|{}", amount),
            Command::Equip { slot, ability } => write!(f, "15|{}|{}", *slot as u8, ability),
            Command::AhkSignal { signal } => write!(f, "16|{}", signal),
            Command::IfEqual { uber_state, item } => write!(f, "17|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfGreater { uber_state, item } => write!(f, "18|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfLess { uber_state, item } => write!(f, "19|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::DisableSync { uber_state } => write!(f, "20|{}", uber_state.identifier),
            Command::EnableSync { uber_state } => write!(f, "21|{}", uber_state.identifier),
            Command::CreateWarp { id, position } => write!(f, "22|{}|{}", id, position.code()),
            Command::DestroyWarp { id } => write!(f, "23|{}", id),
            Command::IfBox { position1, position2, item } => write!(f, "24|{}|{}|{}", position1.code(), position2.code(), item.code()),
            Command::IfSelfEqual { value, item } => write!(f, "25|{}|{}", value, item.code()),
            Command::IfSelfGreater { value, item } => write!(f, "26|{}|{}", value, item.code()),
            Command::IfSelfLess { value, item } => write!(f, "27|{}|{}", value, item.code()),
            Command::UnEquip { ability } => write!(f, "28|{}", ability),
            Command::SaveString { id, string } => write!(f, "29|{}|{}", id, string),
            Command::AppendString { id, string } => write!(f, "30|{}|{}", id, string),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum ToggleCommand {
    KwolokDoor,
    Rain,
    Howl,
}
impl fmt::Display for ToggleCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToggleCommand::KwolokDoor => write!(f, "0"),
            ToggleCommand::Rain => write!(f, "1"),
            ToggleCommand::Howl => write!(f, "2"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, seedgen_derive::FromStr)]
#[repr(u8)]
pub enum EquipSlot {
    Ability1,
    Ability2,
    Ability3,
}
