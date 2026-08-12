use clap::{Args, ValueHint};
use std::path::PathBuf;

use super::{CompileArgs, VerboseArgs};

#[derive(Args)]
pub struct PlandoArgs {
    /// Path to your plandomizer source
    ///
    /// If the path leads to a file, it will be used as entry point.
    /// If it leads to a folder, "main.wotws" in that folder will be used as entry point.
    #[arg(value_hint = ValueHint::FilePath)]
    pub path: PathBuf,
    /// Destination for the compiled seed
    #[arg(long, value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub out: Option<PathBuf>,
    /// Recompile when the source changes
    #[arg(short, long)]
    pub watch: bool,
    #[command(flatten)]
    pub compile_args: CompileArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
