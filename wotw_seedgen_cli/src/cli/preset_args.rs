use clap::Args;

use super::{SeedSettings, SeedWorldSettings};

#[derive(Args, Debug, Default)]
pub struct WorldPresetArgs {
    /// The preset's identifier which you can later pass like `seedgen seed -p <identifier>`
    #[arg(short, long, value_name = "STRING")]
    pub identifier: String,
    #[command(flatten)]
    pub settings: SeedWorldSettings,
    #[command(flatten)]
    pub info_args: PresetInfoArgs,
}

#[derive(Args, Debug, Default)]
pub struct UniversePresetArgs {
    /// The preset's identifier which you can later pass like `seedgen seed -P <identifier>`
    #[arg(short, long, value_name = "STRING")]
    pub identifier: String,
    #[command(flatten)]
    pub settings: SeedSettings,
    #[command(flatten)]
    pub info_args: PresetInfoArgs,
}

#[derive(Args, Debug, Default)]
pub struct PresetInfoArgs {
    /// The preset's display name
    #[arg(short = 'n', long, value_name = "STRING")]
    pub display_name: Option<String>,
    /// The preset's extended description
    #[arg(short = 'D', long, value_name = "STRING")]
    pub description: Option<String>,
    /// Whether the preset should be displayed as a base preset
    #[arg(short, long)]
    pub base_preset: bool,
}
