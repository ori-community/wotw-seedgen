mod find;
mod generate_uber_state_log;
mod import_uber_states;
mod optimize_graph;
mod paths;
mod perf;
mod regenerate;

use find::find;
use generate_uber_state_log::generate_uber_state_log;
use import_uber_states::import_uber_states;
use optimize_graph::optimize_graph;
use paths::paths;
use perf::perf;
use regenerate::regenerate;

use crate::{cli::dev::Dev, Error};

pub fn dev(command: Dev) -> Result<(), Error> {
    match command {
        Dev::ImportUberStates => import_uber_states(),
        Dev::Regenerate { args } => regenerate(args),
        Dev::Paths => paths(),
        Dev::GenerateUberStateLog { args } => generate_uber_state_log(args),
        Dev::OptimizeGraph { args } => optimize_graph(args),
        Dev::Find { args, command } => find(args, command),
        Dev::Perf { args } => perf(args),
    }
}
