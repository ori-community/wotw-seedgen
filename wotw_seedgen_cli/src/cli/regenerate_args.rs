use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct RegenerateArgs {
    /// Path to the existing seed
    pub path: PathBuf,
    /// Write a detailed log into seedgen_log.txt
    #[arg(short, long)]
    pub verbose: bool,
}
