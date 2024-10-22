use clap::Args;

use super::SeedSettings;

#[derive(Args, Debug, Default)]
pub struct SeedArgs {
    #[command(flatten)]
    pub settings: SeedSettings,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
    /// Write a detailed log into seedgen_log.txt
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Args, Debug, Default)]
pub struct GenerationArgs {
    /// Write information useful for debugging into the seed
    #[arg(long)]
    pub debug: bool,
    /// Load the seed into the randomizer after finishing
    ///
    /// Ignored when generating multiworld seeds
    #[arg(short, long)]
    pub launch: bool,
}
