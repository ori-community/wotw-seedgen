//! This crate contains low-level data structures used in the Ori and the Will of the Wisps randomizer
//!
//! Note that some of the contained data is only applicable to the randomizer, not the base game

pub use strum::VariantArray;

mod equipment;
mod icon;
mod message;
mod position;
mod shard;
mod skill;
mod teleporter;
mod uber_identifier;
mod weapon_upgrade;
mod wheel;
mod zone;

pub use equipment::{EquipSlot, Equipment};
pub use icon::{GromIcon, Icon, LupoIcon, MapIcon, OpherIcon, TuleyIcon};
pub use message::{Alignment, ScreenPosition};
pub use position::Position;
pub use shard::Shard;
pub use skill::Skill;
pub use teleporter::Teleporter;
pub use uber_identifier::{CommonUberIdentifier, UberIdentifier};
pub use weapon_upgrade::WeaponUpgrade;
pub use wheel::{WheelBind, WheelItemPosition};
pub use zone::Zone;
