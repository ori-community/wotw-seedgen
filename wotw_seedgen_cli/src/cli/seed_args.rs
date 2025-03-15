use clap::{builder::styling::Reset, Args, ValueEnum};

use super::{SeedSettings, LITERAL};

#[derive(Args, Debug, Default)]
pub struct SeedArgs {
    #[command(flatten)]
    pub settings: SeedSettings,
    #[command(flatten)]
    pub generation_args: GenerationArgs,
    #[command(flatten)]
    pub verbose_args: VerboseArgs,
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

const VERBOSE_HELP: &str = "Write a detailed log into seedgen_log.txt";
#[derive(Args, Debug, Default)]
pub struct VerboseArgs {
    #[arg(
        short,
        long,
        value_name = "target",
        num_args = 0..,
        help = VERBOSE_HELP,
        long_help = format!(
            "{VERBOSE_HELP}.\nOne or more targets can be provided for additional logging.\n'{literal}-v{reset}' without any arguments defaults to '{literal}-v placement{reset}'",
            literal = LITERAL.render(),
            reset = Reset.render()
        )
    )]
    pub verbose: Option<Vec<VerboseTarget>>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum VerboseTarget {
    Placement,
    Reached,
    Doors,
}
