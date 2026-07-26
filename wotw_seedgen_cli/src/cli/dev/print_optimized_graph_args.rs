use clap::Args;

use crate::cli::{SeedSettingsArgs, VerboseArgs};

#[derive(Args)]
pub struct PrintOptimizedGraphArgs {
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
