//! Data structures to represent the settings when generating a seed
//!
//! See the [`UniverseSettings`] struct for more information
//!
//! ## Features
//!
//! - `serde`: Enables [`Deserialize`] and [`Serialize`] implementations on all types
//! - `strum`: Enables [`Display`], [`FromStr`] and [`VariantNames`] implementations on [`Difficulty`] and [`Trick`]
//!
//! [`Deserialize`]: serde::Deserialize
//! [`Serialize`]: serde::Serialize
//! [`Display`]: std::fmt::Display
//! [`FromStr`]: std::str::FromStr
//! [`VariantNames`]: strum::VariantNames

mod preset;
mod settings;

pub use settings::{Difficulty, Spawn, Trick, UniverseSettings, WorldSettings, DEFAULT_SPAWN};
