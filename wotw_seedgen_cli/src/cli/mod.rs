pub mod dev;

mod display;
mod http_server_args;
mod interactive;
mod open_args;
mod plando_args;
mod preset_args;
mod seed_args;
mod seed_settings;
mod stats_args;
mod verbose_args;

use dev::Dev;
pub use display::{
    AvailablePreset, AvailableSnippet, AVAILABLE_SNIPPETS, AVAILABLE_UNIVERSE_PRESETS,
    AVAILABLE_WORLD_PRESETS,
};
pub use http_server_args::HttpServerArgs;
pub use open_args::{OpenArgs, OpenDirectory};
pub use plando_args::PlandoArgs;
pub use preset_args::{PresetInfoArgs, UniversePresetArgs, WorldPresetArgs};
pub use seed_args::{CompileArgs, GenerationArgs, LaunchArgs, SeedArgs, SeedSettingsArgs};
pub use seed_settings::{SeedSettings, SeedWorldSettings};
pub use stats_args::{Analyzer, ChainedAnalyzers, StatsArgs};
pub use verbose_args::{VerboseArgs, VerboseTarget};

use crate::shell_completions::CompletionGenerator;
use clap::{
    builder::{styling::Style, Styles},
    Parser,
};

const STYLES: Styles = Styles::styled();
const LITERAL: Style = *STYLES.get_literal();
const LINK: Style = Style::new().underline();
const INVALID: Style = *STYLES.get_invalid();

// TODO configure assets file paths

#[derive(Parser)]
pub enum Cli {
    /// Generate a seed
    Seed {
        #[command(flatten)]
        args: SeedArgs,
    },
    /// Create a universe preset
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
    /// Open directories related to seedgen
    Open {
        #[command(flatten)]
        args: OpenArgs,
    },
    /// Generate shell completions
    ///
    /// This only outputs the completion file, installation depends on your shell
    ShellCompletions {
        /// Explicitly request completions for the given shell
        shell: Option<CompletionGenerator>,
    },
    /// Various utilities primarily intended for seedgen development
    Dev {
        #[command(subcommand)]
        command: Dev,
    },
    /// Start the http server
    HttpServer {
        #[command(flatten)]
        args: HttpServerArgs,
    },
    /// Start the language server for snippets and plandos
    SeedLsp,
    /// Start the language server for paths.wotwl
    LogicLsp,
}

#[cfg(test)]
#[test]
fn verify_cli() {
    use clap::CommandFactory;

    Cli::command().debug_assert();
}
