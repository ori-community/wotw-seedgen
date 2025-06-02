mod cli;
mod daemon;
mod files;
mod log_config;
mod plando;
mod preset;
mod regenerate;
mod seed;
#[cfg(feature = "stats")]
mod stats;

use bugsalot::debugger;
use clap::Parser;
use cli::Cli;
use daemon::daemon;
use plando::plando;
use preset::{universe_preset, world_preset};
use regenerate::regenerate;
use seed::seed;
#[cfg(feature = "stats")]
use stats::stats;
use std::{
    env,
    fmt::{self, Debug},
};

fn main() -> Result<(), Error> {
    if env::var_os("ATTACH").is_some() {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).unwrap();
    }

    let cli = Cli::parse();
    match cli {
        Cli::Seed { args } => seed(args),
        Cli::UniversePreset { args } => universe_preset(args),
        Cli::WorldPreset { args } => world_preset(args),
        Cli::Plando { args } => plando(args),
        #[cfg(feature = "stats")]
        Cli::Stats { args } => stats(args),
        Cli::Regenerate { args } => regenerate(args),
        Cli::Daemon { args } => daemon(args),
        #[cfg(feature = "lsp")]
        Cli::Lsp => Ok(wotw_seedgen_lsp::start()),
    }
}

pub struct Error(String);
impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<T: ToString> From<T> for Error {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}
