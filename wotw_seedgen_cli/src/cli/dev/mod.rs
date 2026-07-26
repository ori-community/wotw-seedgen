mod perf_args;
mod print_optimized_graph_args;
mod regenerate_args;

pub use perf_args::{PerfArgs, PerfTarget};
pub use print_optimized_graph_args::PrintOptimizedGraphArgs;
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
    PrintOptimizedGraph {
        #[command(flatten)]
        args: PrintOptimizedGraphArgs,
    },
    /// Performance measurement tools
    Perf {
        #[command(flatten)]
        args: PerfArgs,
    },
}
