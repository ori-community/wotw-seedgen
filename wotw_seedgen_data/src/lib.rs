//! This crate contains low-level data structures used in the Ori and the Will of the Wisps randomizer
//!
//! Note that some of the contained data is only applicable to the randomizer, not the base game

pub use strum::VariantArray;

mod _uber_identifier;
mod equipment;
mod icon;
mod message;
mod position;
mod shard;
mod skill;
mod teleporter;
mod weapon_upgrade;
mod wheel;
mod zone;

pub use _uber_identifier::{uber_identifier, UberIdentifier};
pub use equipment::{EquipSlot, Equipment};
pub use icon::{GromIcon, LupoIcon, MapIcon, OpherIcon, TuleyIcon};
pub use message::{Alignment, ScreenPosition};
pub use position::Position;
pub use shard::Shard;
pub use skill::Skill;
pub use teleporter::Teleporter;
pub use weapon_upgrade::WeaponUpgrade;
pub use wheel::{WheelBind, WheelItemPosition};
pub use zone::Zone;
