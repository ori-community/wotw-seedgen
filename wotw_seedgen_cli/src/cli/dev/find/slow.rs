use clap::{Args, ValueHint};
use humantime::Duration;

use crate::cli::SeedSettingsArgs;

#[derive(Args)]
pub struct SlowArgs {
    /// Minimum duration the seed has to take
    #[arg(short = 'D', long, value_name = "DURATION", value_hint = ValueHint::Other, default_value = "10s")]
    pub min_duration: Duration,
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
}
