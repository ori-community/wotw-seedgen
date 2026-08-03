use clap::Args;

use super::{SeedSettings, VerboseArgs};

#[derive(Args, Debug, Default)]
pub struct SeedArgs {
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
}

#[derive(Args, Debug, Default)]
pub struct SeedSettingsArgs {
    #[arg(long)]
    pub stdin_settings: bool,
    #[command(flatten)]
    pub settings: SeedSettings,
}

#[derive(Args, Debug, Default)]
pub struct GenerationArgs {
    /// Write information useful for debugging into the seed
    #[arg(long)]
    pub debug: bool,
    #[command(flatten)]
    pub launch: LaunchArgs,
}

#[derive(Args, Debug, Default)]
pub struct LaunchArgs {
    /// Load the seed into the randomizer after finishing
    ///
    /// Ignored when generating multiworld seeds
    #[arg(short, long)]
    pub launch: bool,
    /// Update .newgameseedsource without trying to launch
    #[arg(long = "ngss")]
    pub new_game_seed_source: bool,
}
