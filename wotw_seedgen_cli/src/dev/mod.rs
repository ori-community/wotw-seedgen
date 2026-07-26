mod import_uber_states;
mod paths;
mod perf;
mod print_optimized_graph;
mod regenerate;

use import_uber_states::import_uber_states;
use paths::paths;
use perf::perf;
use print_optimized_graph::print_optimized_graph;
use regenerate::regenerate;

use crate::{cli::dev::Dev, Error};

pub fn dev(command: Dev) -> Result<(), Error> {
    match command {
        Dev::Regenerate { args } => regenerate(args),
        Dev::Paths => paths(),
        Dev::ImportUberStates => import_uber_states(),
        Dev::PrintOptimizedGraph { args } => print_optimized_graph(args),
        Dev::Perf { args } => perf(args),
    }
}
