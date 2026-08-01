//! # Ori and the Will of the Wisps Seed Generator
//!
//! This library can generate seeds for the [Ori and the Will of the Wisps Randomizer](https://wotw.orirando.com/).
//!
//! The main entry point is [`generate_seed`], which holds further documentation.
//!
//! # Re-exports
//!
//! Relevant crates are re-exported here, e.g. you can access the [`wotw_seedgen_settings`] crate as `wotw_seedgen::settings`

pub use wotw_seedgen_data as data;
pub use wotw_seedgen_seed as seed;

pub mod orbs;

mod generator;
mod logical_difficulty;
#[cfg(test)]
mod tests;
mod world;

pub use generator::{generate_seed, item_pool, perf_data, spoiler, SeedUniverse};
pub use logical_difficulty::LogicalDifficulty;
pub use world::World;
