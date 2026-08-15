pub mod find;

mod optimize_graph_args;
mod patch_args;
mod perf_args;
mod regenerate_args;

use find::{Find, FindArgs};
pub use optimize_graph_args::OptimizeGraphArgs;
pub use patch_args::PatchArgs;
pub use perf_args::{PerfArgs, PerfTarget};
pub use regenerate_args::RegenerateArgs;

use clap::Subcommand;

use crate::cli::LaunchArgs;

#[derive(Subcommand)]
pub enum Dev {
    /// Import an UberState dump
    ImportUberStates,
    /// Regenerate an existing seed for debugging
    Regenerate {
        #[command(flatten)]
        args: RegenerateArgs,
    },
    /// Patch a seed by regenerating it in place with any changes made to seedgen or assets
    Patch {
        #[command(flatten)]
        args: PatchArgs,
    },
    /// Display which local paths seedgen is using
    Paths,
    /// Generate a toolseed to log UberState changes
    GenerateUberStateLog {
        #[command(flatten)]
        args: LaunchArgs,
    },
    /// Compiles and decompiles the logic graph
    OptimizeGraph {
        #[command(flatten)]
        args: OptimizeGraphArgs,
    },
    /// Find a problematic seed
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
