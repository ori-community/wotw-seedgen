use clap::Args;
use std::path::PathBuf;

use super::seed_args::VerboseArgs;

#[derive(Args)]
pub struct RegenerateArgs {
    /// Path to the existing seed
    pub path: PathBuf,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
