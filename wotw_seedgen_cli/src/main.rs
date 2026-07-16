mod cli;
mod http_server;
mod import_uber_states;
mod log_config;
mod paths;
mod perf;
mod plando;
mod preset;
mod print_optimized_graph;
mod regenerate;
mod seed;
mod stats;

use bugsalot::debugger;
use clap::Parser;
use cli::Cli;
use import_uber_states::import_uber_states;
use paths::paths;
use perf::perf;
use plando::plando;
use preset::{universe_preset, world_preset};
use print_optimized_graph::print_optimized_graph;
use regenerate::regenerate;
use seed::seed;
use stats::stats;
use std::{
    env,
    fmt::{self, Debug},
};

use crate::http_server::http_server;

fn main() -> Result<(), Error> {
    if env::var_os("ATTACH").is_some() {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).unwrap();
        eprintln!("debugger attached");
    }

    let cli = Cli::parse();
    match cli {
        Cli::Seed { args } => seed(args),
        Cli::UniversePreset { args } => universe_preset(args),
        Cli::WorldPreset { args } => world_preset(args),
        Cli::Plando { args } => plando(args),
        Cli::Stats { args } => stats(args),
        Cli::Regenerate { args } => regenerate(args),
        Cli::Paths => paths(),
        Cli::ImportUberStates => import_uber_states(),
        Cli::PrintOptimizedGraph { args } => print_optimized_graph(args),
        Cli::Perf { args } => perf(args),
        Cli::HttpServer { args } => http_server(args),
        Cli::SeedLsp => {
            wotw_seedgen_lsp::start_seed();
            Ok(())
        }
        Cli::LogicLsp => {
            wotw_seedgen_lsp::start_logic();
            Ok(())
        }
    }
}

pub struct Error(String);

impl Error {
    pub fn eprint(&self) {
        eprintln!("{}", self.0);
    }
}

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
