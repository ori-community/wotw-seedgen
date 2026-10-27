use std::path::PathBuf;

use clap::{Args, ValueHint};

#[derive(Args)]
pub struct FixArgs {
    #[arg(short, long, num_args = 1.., value_name = "PATH", value_hint = ValueHint::FilePath)]
    pub paths: Vec<PathBuf>,
}
