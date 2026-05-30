use clap::{builder::styling::Reset, Args, ValueEnum};
use wotw_seedgen::data::UberIdentifier;

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

const VERBOSE_HELP: &str = "Write a detailed log into seedgen_log.txt";
const TRACE_UBER_STATES_HELP: &str = "Trace UberState changes";

#[derive(Args, Debug, Default)]
pub struct VerboseArgs {
    #[arg(
        short,
        long,
        value_name = "TARGET",
        num_args = 0..,
        help = VERBOSE_HELP,
        long_help = format!(
            "{VERBOSE_HELP}.\nOne or more targets can be provided for additional logging.\n'{literal}-v{reset}' without any arguments defaults to '{literal}-v placement{reset}'",
            literal = LITERAL.render(),
            reset = Reset.render()
        )
    )]
    pub verbose: Option<Vec<VerboseTarget>>,
    #[arg(
        long,
        value_name = "UBER_IDENTIFIER",
        requires = "verbose",
        num_args = 0..,
        help = TRACE_UBER_STATES_HELP,
        long_help = format!(
            "{TRACE_UBER_STATES_HELP}.\nOne or more UberStates can be provided in {literal}<group>|<member>{reset} format for filtering.\nWithout filters, all changes will be traced",
            literal = LITERAL.render(),
            reset = Reset.render()
        )
    )]
    pub trace_uber_states: Option<Vec<UberIdentifier>>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum VerboseTarget {
    Placement,
    Reached,
    IsMet,
    Solutions,
    Weight,
    Spawn,
    Entrances,
    OptimizeGraph,
}
