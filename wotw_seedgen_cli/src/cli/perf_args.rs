use clap::{Args, ValueEnum};
use humantime::Duration;

use crate::cli::SeedSettingsArgs;

#[derive(Args)]
pub struct PerfArgs {
    /// Which target should be measured
    pub target: PerfTarget,
    /// How long to measure
    #[arg(short = 'D', long, default_value = "100s")]
    pub duration: Duration,
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum PerfTarget {
    Reached,
}
