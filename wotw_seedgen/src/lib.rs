//! # Ori and the Will of the Wisps Seed Generator
//!
//! This library can generate seeds for the [Ori and the Will of the Wisps Randomizer](https://wotw.orirando.com/).
//!
//! The main entry point is [`generate_seed`], which holds further documentation.
//!
//! # Re-exports
//!
//! Relevant crates are re-exported here, e.g. you can access the [`wotw_seedgen_settings`] crate as `wotw_seedgen::settings`

#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::match_bool)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]

pub use wotw_seedgen_data as data;
pub use wotw_seedgen_seed as seed;

pub mod orbs;

mod generator;
mod logical_difficulty;
#[cfg(test)]
mod tests;
mod world;

pub use generator::{generate_seed, item_pool, spoiler, SeedUniverse};
pub use world::World;

// TODO use this and also set the other metadata: current world, format version, settings
// TODO look into having the commit hash again
pub const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"));
