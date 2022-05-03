use std::fmt;

use num_enum::TryFromPrimitive;
use seedgen_derive::{VVariant, FromStr, Display};

use super::{Item, VItem, Resource};
use crate::util::{UberIdentifier, UberState, VUberState, Position, VPosition, NumericBool, Spell};
use crate::header::VString;

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
    Equip { #[VWrap] slot: EquipSlot, ability: Spell },
    AhkSignal { signal: String },
    IfEqual { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    IfGreater { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    IfLess { #[VType] uber_state: UberState, #[VType] item: Box<Item> },
    DisableSync { uber_identifier: UberIdentifier },
    EnableSync { uber_identifier: UberIdentifier },
    CreateWarp { id: u8, #[VType] position: Position, label: Option<String> },
    DestroyWarp { id: u8 },
    IfBox { #[VType] position1: Position, #[VType] position2: Position, #[VType] item: Box<Item> },
    IfSelfEqual { #[VWrap] value: String, #[VType] item: Box<Item> },
    IfSelfGreater { #[VWrap] value: String, #[VType] item: Box<Item> },
    IfSelfLess { #[VWrap] value: String, #[VType] item: Box<Item> },
    UnEquip { ability: Spell },
    SaveString { id: i32, #[VType] string: String },
    AppendString { id: i32, #[VType] string: String },
}
impl Command {
    pub fn code(&self) -> String {
        match self {
            Command::Autosave => format!("0"),
            Command::Resource { resource, amount } => format!("1|{}|{}", *resource as u8, amount),
            Command::Checkpoint => format!("2"),
            Command::Magic => format!("3"),
            Command::StopEqual { uber_state } => format!("4|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopGreater { uber_state } => format!("5|{}|{}", uber_state.identifier, uber_state.value),
            Command::StopLess { uber_state } => format!("6|{}|{}", uber_state.identifier, uber_state.value),
            Command::Toggle { target, on } => format!("7|{}|{}", *target as u8, *on as u8),
            Command::Warp { position } => format!("8|{}", position.code()),
            Command::StartTimer { identifier } => format!("9|{}", identifier),
            Command::StopTimer { identifier } => format!("10|{}", identifier),
            Command::StateRedirect { intercept, set } => format!("11|{}|{}", intercept, set),
            Command::SetHealth { amount } => format!("12|{}", amount),
            Command::SetEnergy { amount } => format!("13|{}", amount),
            Command::SetSpiritLight { amount } => format!("14|{}", amount),
            Command::Equip { slot, ability } => format!("15|{}|{}", *slot as u8, ability),
            Command::AhkSignal { signal } => format!("16|{}", signal),
            Command::IfEqual { uber_state, item } => format!("17|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfGreater { uber_state, item } => format!("18|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::IfLess { uber_state, item } => format!("19|{}|{}|{}", uber_state.identifier, uber_state.value, item.code()),
            Command::DisableSync { uber_identifier } => format!("20|{}", uber_identifier),
            Command::EnableSync { uber_identifier } => format!("21|{}", uber_identifier),
            Command::CreateWarp { id, position, label } => format!("22|{}|{}{}", id, position.code(), label.iter().map(|label| format!("|{label}")).collect::<String>()),
            Command::DestroyWarp { id } => format!("23|{}", id),
            Command::IfBox { position1, position2, item } => format!("24|{}|{}|{}", position1.code(), position2.code(), item.code()),
            Command::IfSelfEqual { value, item } => format!("25|{}|{}", value, item.code()),
            Command::IfSelfGreater { value, item } => format!("26|{}|{}", value, item.code()),
            Command::IfSelfLess { value, item } => format!("27|{}|{}", value, item.code()),
            Command::UnEquip { ability } => format!("28|{}", ability),
            Command::SaveString { id, string } => format!("29|{}|{}", id, string),
            Command::AppendString { id, string } => format!("30|{}|{}", id, string),
        }
    }
}
impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Autosave => write!(f, "Autosave"),
            Command::Resource { resource, amount } => write!(f, "Set {resource} to {amount}"),
            Command::Checkpoint => write!(f, "Checkpoint"),
            Command::Magic => write!(f, "✨Magic✨"),
            Command::StopEqual { uber_state } => write!(f, "Stop the multipickup if {} is {}", uber_state.identifier, uber_state.value),
            Command::StopGreater { uber_state } => write!(f, "Stop the multipickup if {} is greater than {}", uber_state.identifier, uber_state.value),
            Command::StopLess { uber_state } => write!(f, "Stop the multipickup if {} is less than {}", uber_state.identifier, uber_state.value),
            Command::Toggle { target, on } => write!(f, "Toggle the {target} {}", if *on as u8 == 0 { "on" } else { "off" }),
            Command::Warp { position } => write!(f, "Warp to {position}"),
            Command::StartTimer { identifier } => write!(f, "Start a timer on {identifier}"),
            Command::StopTimer { identifier } => write!(f, "Stop any timer on {identifier}"),
            Command::StateRedirect { intercept, set } => write!(f, "Override state applier {intercept} with {set}"),
            Command::SetHealth { amount } => write!(f, "Set current health to {amount}"),
            Command::SetEnergy { amount } => write!(f, "Set current energy to {amount}"),
            Command::SetSpiritLight { amount } => write!(f, "Set current spirit light to {amount}"),
            Command::Equip { slot, ability } => write!(f, "Equip {ability} to slot {slot}"),
            Command::AhkSignal { signal } => write!(f, "Trigger the \"{signal}\" keybind"),
            Command::IfEqual { uber_state, item } => write!(f, "Grant this item if {} is {}: {}", uber_state.identifier, uber_state.value, item),
            Command::IfGreater { uber_state, item } => write!(f, "Grant this item if {} is greater than {}: {}", uber_state.identifier, uber_state.value, item),
            Command::IfLess { uber_state, item } => write!(f, "Grant this item if {} is less than {}: {}", uber_state.identifier, uber_state.value, item),
            Command::DisableSync { uber_identifier } => write!(f, "Disable multiplayer sync for {uber_identifier}"),
            Command::EnableSync { uber_identifier } => write!(f, "Enable multiplayer sync for {uber_identifier}"),
            Command::CreateWarp { id, position, label } => write!(f, "Create a warp icon with identifier {id} at {position}{}", label.iter().map(|label| format!(" labelled \"{label}\"")).collect::<String>()),
            Command::DestroyWarp { id } => write!(f, "Destroy the warp icon with identifier {id}"),
            Command::IfBox { position1, position2, item } => write!(f, "Grant this item if Ori is within the rectangle defined by {position1}/{position2}: {item}"),
            Command::IfSelfEqual { value, item } => write!(f, "Grant this item if the trigger state's value is {value}: {item}"),
            Command::IfSelfGreater { value, item } => write!(f, "Grant this item if the trigger state's value is greater than {value}: {item}"),
            Command::IfSelfLess { value, item } => write!(f, "Grant this item if the trigger state's value is less than {value}: {item}"),
            Command::UnEquip { ability } => write!(f, "Unequip {ability}"),
            Command::SaveString { id, string } => write!(f, "Stores the string \"{string}\" under the identifier {id}"),
            Command::AppendString { id, string } => write!(f, "Appends the string \"{string}\" to the current value stored under the identifier {id}"),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum ToggleCommand {
    KwolokDoor = 0,
    Rain = 1,
    Howl = 2,
}

#[derive(Debug, Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum EquipSlot {
    Ability1,
    Ability2,
    Ability3,
}
