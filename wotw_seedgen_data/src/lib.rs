//! This crate contains data structures used in the Ori and the Will of the Wisps randomizer
//!
//! Note that some of the contained data is only applicable to the randomizer, not the base game

pub use wotw_seedgen_parse as parse;

pub use strum::{VariantArray, VariantNames};

pub mod assets;
mod equipment;
mod icon;
pub mod logic_language;
mod message;
mod position;
pub mod seed_language;
mod settings;
mod shard;
mod skill;
mod teleporter;
mod uber_identifier;
mod weapon_upgrade;
mod wheel;
mod zone;

pub use equipment::{EquipSlot, Equipment};
pub use icon::{GromIcon, Icon, LupoIcon, MapIcon, OpherIcon, TuleyIcon};
pub use message::{Alignment, CoordinateSystem, HorizontalAnchor, ScreenPosition, VerticalAnchor};
pub use position::Position;
pub use settings::{
    Difficulty, GreaterOneU8, Spawn, Trick, UniverseSettings, WorldSettings, WorldSettingsHelpers,
    DEFAULT_SPAWN,
};
pub use shard::Shard;
pub use skill::Skill;
pub use teleporter::Teleporter;
pub use uber_identifier::{CommonUberIdentifier, ShopKind, UberIdentifier};
pub use weapon_upgrade::WeaponUpgrade;
pub use wheel::{WheelBind, WheelItemPosition};
pub use zone::Zone;
