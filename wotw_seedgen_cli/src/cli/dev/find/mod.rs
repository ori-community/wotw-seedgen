mod slow;

pub use slow::SlowArgs;

use clap::Subcommand;

use crate::cli::SeedSettingsArgs;

#[derive(Subcommand)]
pub enum Find {
    /// Find a seed that panics
    Panic {
        #[command(flatten)]
        args: SeedSettingsArgs,
    },
    /// Find a seed that logs a warning or error
    Warning {
        #[command(flatten)]
        args: SeedSettingsArgs,
    },
    /// Find a seed that's slow to generate
    Slow {
        #[command(flatten)]
        args: SlowArgs,
    },
}
