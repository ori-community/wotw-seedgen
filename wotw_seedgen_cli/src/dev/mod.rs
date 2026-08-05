mod find;
mod import_uber_states;
mod optimize_graph;
mod paths;
mod perf;
mod regenerate;

use find::find;
use import_uber_states::import_uber_states;
use optimize_graph::optimize_graph;
use paths::paths;
use perf::perf;
use regenerate::regenerate;

use crate::{cli::dev::Dev, Error};

pub fn dev(command: Dev) -> Result<(), Error> {
    match command {
        Dev::Regenerate { args } => regenerate(args),
        Dev::Paths => paths(),
        Dev::ImportUberStates => import_uber_states(),
        Dev::OptimizeGraph { args } => optimize_graph(args),
        Dev::Find { args, command } => find(args, command),
        Dev::Perf { args } => perf(args),
    }
}
