mod plando_args;
mod preset_args;
mod seed_args;
mod seed_settings;
mod stats_args;

pub use plando_args::PlandoArgs;
pub use preset_args::{PresetInfoArgs, UniversePresetArgs, WorldPresetArgs};
pub use seed_args::{GenerationArgs, SeedArgs};
pub use seed_settings::{SeedSettings, SeedWorldSettings};
pub use stats_args::{ChainedAnalyzers, StatsArgs};

use clap::{
    builder::{styling::Style, Styles},
    Parser,
};

const STYLES: Styles = Styles::styled();
const LITERAL: Style = *STYLES.get_literal();
const LINK: Style = Style::new().underline();
const INVALID: Style = *STYLES.get_invalid();

// TODO configure assets file paths

// TODO documentation for the commands
#[derive(Parser)]
pub enum Cli {
    /// Generate a seed
    Seed {
        #[command(flatten)]
        args: SeedArgs,
    },
    /// Create a world preset
    UniversePreset {
        #[command(flatten)]
        args: UniversePresetArgs,
    },
    /// Create a world preset
    WorldPreset {
        #[command(flatten)]
        args: WorldPresetArgs,
    },
    /// Compile a plandomizer
    Plando {
        #[command(flatten)]
        args: PlandoArgs,
    },
    /// Generate seed statistics
    ///
    /// The resulting statistics will be written into a 'stats' folder, you can read them out there
    ///
    /// This command also maintains a cache of seeds in a 'seed_storage' folder, you do not need to interact with this folder (although you won't break anything either if you delete it or such)
    Stats {
        #[command(flatten)]
        args: StatsArgs,
    },
}

#[cfg(test)]
#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}
