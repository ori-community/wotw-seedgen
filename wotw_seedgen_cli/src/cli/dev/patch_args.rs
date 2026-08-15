use clap::Args;
use std::path::PathBuf;

use crate::cli::{GenerationArgs, VerboseArgs};

#[derive(Args)]
pub struct PatchArgs {
    /// Path to the existing seed
    ///
    /// If empty, patch the last seed from .newgameseedsource
    pub path: Option<PathBuf>,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
