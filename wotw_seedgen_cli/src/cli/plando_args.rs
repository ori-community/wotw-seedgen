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
    /// Filename of the output file
    ///
    /// .wotwr will be appended as the file extension
    #[arg(short = 'n', long, value_name = "STRING")]
    pub out_name: Option<String>,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
}
