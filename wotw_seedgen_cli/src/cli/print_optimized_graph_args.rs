use clap::Args;

use crate::cli::SeedSettingsArgs;

use super::seed_args::VerboseArgs;

#[derive(Args)]
pub struct PrintOptimizedGraphArgs {
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}
