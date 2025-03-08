use clap::Args;
use std::path::PathBuf;

use super::GenerationArgs;

#[derive(Args)]
pub struct PlandoArgs {
    /// Path to your plandomizer source
    ///
    /// If the path leads to a file, it will be used as entry point.
    /// If it leads to a folder, "main.wotws" in that folder will be used as entry point.
    pub path: PathBuf,
    /// Destination for the compiled seed
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
    // TODO watch functionality?
    #[command(flatten)]
    pub generation_args: GenerationArgs,
}
