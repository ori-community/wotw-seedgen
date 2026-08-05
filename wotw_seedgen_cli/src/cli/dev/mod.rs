pub mod find;

mod optimize_graph_args;
mod perf_args;
mod regenerate_args;

use find::{Find, FindArgs};
pub use optimize_graph_args::OptimizeGraphArgs;
pub use perf_args::{PerfArgs, PerfTarget};
pub use regenerate_args::RegenerateArgs;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Dev {
    /// Import an UberState dump
    ImportUberStates,
    /// Regenerate an existing seed for debugging
    Regenerate {
        #[command(flatten)]
        args: RegenerateArgs,
    },
    /// Display which local paths seedgen is using
    Paths,
    /// Compiles and decompiles the logic graph
    OptimizeGraph {
        #[command(flatten)]
        args: OptimizeGraphArgs,
    },
    Find {
        #[command(flatten)]
        args: FindArgs,
        #[command(subcommand)]
        command: Find,
    },
    /// Performance measurement tools
    Perf {
        #[command(flatten)]
        args: PerfArgs,
    },
}
