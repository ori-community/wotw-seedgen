use clap::Args;
use humantime::Duration;

use crate::cli::SeedSettingsArgs;

#[derive(Args)]
pub struct SlowArgs {
    /// Minimum duration the seed has to take
    #[arg(short = 'D', long, default_value = "10s")]
    pub min_duration: Duration,
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
}
