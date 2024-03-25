use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct PlandoArgs {
    /// Path to your plandomizer source
    ///
    /// If the path points to a folder, "main.wotws" in that folder will be used as entry point.
    /// Otherwise, the file at the path will be used as entry point directly.
    pub path: PathBuf,
    /// Write information useful for debugging into the seed
    #[arg(long)]
    pub debug: bool,
}
