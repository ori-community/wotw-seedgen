use super::{SeedSettingsArgs, LITERAL};
use clap::{builder::styling::Reset, error::ErrorKind, Args, Parser, ValueHint};
use std::{num::NonZeroUsize, str::FromStr};
use wotw_seedgen::data::Area;

#[derive(Args)]
pub struct StatsArgs {
    #[command(flatten)]
    pub settings_args: SeedSettingsArgs,
    /// How many samples (seeds) to use
    #[arg(short = 'z', long, value_name = "NUMBER", value_hint = ValueHint::Other, default_value = "10000")]
    pub sample_size: usize,
    #[arg(short, long, value_hint = ValueHint::Other, required = true, num_args = 1.., long_help = format!(
        "Any amount of analyzers that will provide statistics\n\n\
        Some analyzers take additional arguments. You can provide those like <ANALYZER>:<ARG>[,<ARG>]...\n\
        For instance, '{LITERAL}--analyzers item-unlock:Launch{Reset}' will create a 'Reachables on launch unlock.csv'\n\n\
        Multiple analyzers separated with spaces will create separate stats files analyzing their individual criteria\n\
        For instance, '{LITERAL}--spawn random --analyzers spawn-location spawn-items{Reset}' will create one set of stats 'Spawn Location.csv' and another 'Spawn Items.csv'\n\n\
        However, you can also chain analyzers together by separating them with plus signs (no spaces)\n\
        For instance, '{LITERAL}--spawn random --analyzers spawn-location+spawn-items{Reset}' will create a 'Spawn Location and Spawn Items.csv' that contains stats on spawn items for each individual spawn location\n\n\
        To learn more about the available analyzers, try '{LITERAL}--analyzers help{Reset}'",
    ))]
    pub analyzers: Vec<ChainedAnalyzers>,
    // TODO dir_name, tolerated_errors, error_message_limit, overwrite_cache
}

#[derive(Clone)]
pub struct ChainedAnalyzers(pub Vec<Analyzer>);
#[derive(Parser, Clone)]
pub enum Analyzer {
    #[command(about = format!(
        "Analyzes the skills placed early on. By default this means placed in the first 50 reachable locations\n\
        Optionally use '{LITERAL}early-skills:<reachable-limit>{Reset}' to override the default value",
    ))]
    EarlySkills {
        #[arg(default_value = "50")]
        reachable_limit: usize,
    },
    /// Analyzes which weapon gets placed first
    FirstWeapon,
    #[command(about = format!(
        "Analyzes what location an item get placed on\n\
        Use '{LITERAL}item-location:<item-name>{Reset}' to specify which item to analyze (Example: '{LITERAL}item-location:Launch{Reset}')",
    ))]
    ItemLocation { item: String },
    #[command(about = format!(
        "Analyzes how many locations are reachable when an item unlocks\n\
        Use '{LITERAL}item-unlock:<item-name>{Reset}' to specify which item to analyze (Example: '{LITERAL}item-unlock:Launch{Reset}')\n\
        Optionally use '{LITERAL}item-unlock:<item-name>,<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    ItemUnlock {
        item: String,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    #[command(about = format!(
        "Analyzes what area an item is placed in\n\
        Use '{LITERAL}item-area:<item-name>{Reset}' to specify which item to analyze (Example: '{LITERAL}item-area:Launch{Reset}')",
    ))]
    ItemArea { item: String },
    #[command(about = format!(
        "Analyzes what item gets placed on a location\n\
        Use '{LITERAL}location-item:<location>{Reset}' to specify which location to analyze (Example: '{LITERAL}location-item:Glades.RebuildGlades.Completed{Reset}')",
    ))]
    LocationItem { location: String },
    #[command(about = format!(
        "Analyzes how many locations are reachable before a location unlocks\n\
        Use '{LITERAL}location-unlock:<location>{Reset}' to specify which location to analyze (Example: '{LITERAL}location-item:Glades.RebuildGlades.Completed{Reset}')\n\
        Optionally use '{LITERAL}item-unlock:<item-name>,<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    LocationUnlock {
        location: String,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    /// Analyzes which items get placed as forced progression
    Progression,
    /// Analyzes what locations forced progression gets placed on
    ProgressionLocations,
    /// Analyzes the amount of spawn items. Mostly makes sense with random spawn, since with the default spawn usually no spawn items are given
    SpawnItemCount,
    /// Analyzes the spawn items. Mostly makes sense with random spawn, since with the default spawn usually no spawn items are given
    SpawnItems,
    /// Analyzes the spawn locations. Only makes sense with random spawn, otherwise this will always be the same
    SpawnLocation,
    /// Analyzes the spawn region. Useful for fully random spawn where there are a lot of spawn locations
    SpawnRegion,
    #[command(about = format!(
        "Analyzes how big the steps of progression are\n\
        Optionally use '{LITERAL}step-size:<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    StepSize {
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    #[command(about = format!(
        "Analyzes how much Spirit Light is in a seed\n\
        Optionally use '{LITERAL}total-spirit-light:<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    TotalSpiritLight {
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    // TODO test
    // TODO available values
    #[command(about = format!(
        "Analyzes how much Spirit Light is in an area\n\
        Use '{LITERAL}area-spirit-light:<area>{Reset}' to specify which area to analyze (Example: '{LITERAL}area-spirit-light:wellspring{Reset}')\n\
        Optionally use '{LITERAL}area-spirit-light:<area>,<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    AreaSpiritLight {
        area: Area,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    #[command(about = format!(
        "Analyzes how many skills are in an area\n\
        Use '{LITERAL}area-skills:<area>{Reset}' to specify which area to analyze (Example: '{LITERAL}area-skills:marsh{Reset}'",
    ))]
    AreaSkills { area: Area },
    #[command(about = format!(
        "Analyzes how many locations are reachable when an area unlocks\n\
        Use '{LITERAL}area-unlock:<area>{Reset}' to specify which area to analyze (Example: '{LITERAL}area-unlock:wellspring{Reset}')\n\
        Optionally use '{LITERAL}area-unlock:<area>,<result-bucket-size>{Reset}' to group results together in buckets",
    ))]
    AreaUnlock {
        area: Area,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
}

impl FromStr for ChainedAnalyzers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('+')
            .map(|s| match s.split_once(':') {
                // The first arg is the "executable name"
                Some((identifier, args)) if !args.is_empty() => Analyzer::try_parse_from(
                    ["--analyzers", identifier]
                        .into_iter()
                        .chain(args.split(',')),
                ),
                _ => Analyzer::try_parse_from(["--analyzers", s]),
            })
            .collect::<Result<_, _>>()
            .map(Self)
            .map_err(|err| match err.kind() {
                ErrorKind::DisplayHelp => err.exit(),
                _ => err
                    .render()
                    .ansi()
                    .to_string()
                    .replace("--help", "--analyzers help"),
            })
    }
}
