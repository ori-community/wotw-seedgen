mod slow;

pub use slow::SlowArgs;

use clap::{Args, Subcommand};

use crate::cli::SeedSettingsArgs;

#[derive(Args)]
pub struct FindArgs {
    #[arg(short, long, default_value = "0")]
    pub start: u32,
}

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
