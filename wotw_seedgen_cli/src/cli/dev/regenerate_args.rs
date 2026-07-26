use clap::Args;
use std::path::PathBuf;

use crate::cli::{GenerationArgs, VerboseArgs};

#[derive(Args)]
pub struct RegenerateArgs {
    /// Path to the existing seed
    pub path: PathBuf,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
