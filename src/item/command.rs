use std::fmt;

use super::{Item, Resource};
use crate::util::{UberIdentifier, UberState};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Command {
    Autosave,
    Resource { resource: Resource, amount: i16 },
    Checkpoint,
    Magic,
    StopEqual { uber_state: UberState },
    StopGreater { uber_state: UberState },
    StopLess { uber_state: UberState },
    Toggle { target: ToggleCommand, on: bool },
    Warp { x: i16, y: i16 },
    StartTimer { identifier: UberIdentifier },
    StopTimer { identifier: UberIdentifier },
    StateRedirect { intercept: i32, set: i32 },
    SetHealth { amount: i16 },
    SetEnergy { amount: i16 },
    SetSpiritLight { amount: i16 },
    Equip { slot: u8, ability: u16 },
    AhkSignal { signal: String },
    IfEqual { uber_state: UberState, item: Box<Item> },
    IfGreater { uber_state: UberState, item: Box<Item> },
    IfLess { uber_state: UberState, item: Box<Item> },
    DisableSync { uber_state: UberState },
    EnableSync { uber_state: UberState },
    CreateWarp { id: u8, x: i16, y: i16 },
    DestroyWarp { id: u8 },
    IfBox { x1: i16, y1: i16, x2: i16, y2: i16, item: Box<Item> },
    IfSelfEqual { item: Box<Item> },
    IfSelfGreater { item: Box<Item> },
    IfSelfLess { item: Box<Item> },
}
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Autosave => write!(f, "0"),
            Command::Resource { resource, amount } => write!(f, "1|{}|{}", resource.to_id(), amount),
            Command::Checkpoint => write!(f, "2"),
            Command::Magic => write!(f, "3"),
            Command::StopEqual { uber_state } => write!(f, "4|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopGreater { uber_state } => write!(f, "5|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopLess { uber_state } => write!(f, "6|{}|{}", uber_state.identifier, uber_state.value),
            Command::Toggle { target, on } => write!(f, "7|{}|{}", target, u8::from(*on)),
            Command::Warp { x, y } => write!(f, "8|{}|{}", x, y),
            Command::StartTimer { identifier } => write!(f, "9|{}", identifier),
            Command::StopTimer { identifier } => write!(f, "10|{}", identifier),
            Command::StateRedirect { intercept, set } => write!(f, "11|{}|{}", intercept, set),
            Command::SetHealth { amount } => write!(f, "12|{}", amount),
            Command::SetEnergy { amount } => write!(f, "13|{}", amount),
            Command::SetSpiritLight { amount } => write!(f, "14|{}", amount),
            Command::Equip { slot, ability } => write!(f, "15|{}|{}", slot, ability),
            Command::AhkSignal { signal } => write!(f, "16|{}", signal),
            Command::IfEqual { uber_state, item } => write!(f, "17|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfGreater { uber_state, item } => write!(f, "18|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfLess { uber_state, item } => write!(f, "19|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::DisableSync { uber_state } => write!(f, "20|{}", uber_state.identifier),
            Command::EnableSync { uber_state } => write!(f, "21|{}", uber_state.identifier),
            Command::CreateWarp { id, x, y } => write!(f, "22|{}|{}|{}", id, x, y),
            Command::DestroyWarp { id } => write!(f, "23|{}", id),
            Command::IfBox { x1, y1, x2, y2, item } => write!(f, "24|{}|{}|{}|{}|{}", x1, y1, x2, y2, item.code()),
            Command::IfSelfEqual { item } => write!(f, "25|{}", item.code()),
            Command::IfSelfGreater { item } => write!(f, "26|{}", item.code()),
            Command::IfSelfLess { item } => write!(f, "27|{}", item.code()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
impl ToggleCommand {
    pub fn from_id(id: u8) -> Option<ToggleCommand> {
        match id {
            0 => Some(ToggleCommand::KwolokDoor),
            1 => Some(ToggleCommand::Rain),
            2 => Some(ToggleCommand::Howl),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            ToggleCommand::KwolokDoor => 0,
            ToggleCommand::Rain => 1,
            ToggleCommand::Howl => 2,
        }
    }
}
