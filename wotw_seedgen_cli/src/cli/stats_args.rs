use super::{SeedSettings, LITERAL};
use clap::{builder::styling::Reset, error::ErrorKind, Args, Parser};
use std::{num::NonZeroUsize, str::FromStr};
use wotw_seedgen::data::Zone;

#[derive(Args)]
pub struct StatsArgs {
    #[command(flatten)]
    pub settings: SeedSettings,
    /// How many samples (seeds) to use
    #[arg(short = 'z', long, value_name = "NUMBER", default_value = "10000")]
    pub sample_size: usize,
    #[arg(short, long, required = true, num_args = 1.., long_help = format!(
        "Any amount of analyzers that will provide statistics\n\n\
        Some analyzers take additional arguments. You can provide those like <ANALYZER>:<ARG>[,<ARG>]...\n\
        For instance, '{literal}--analyzers item-unlock:Launch{reset}' will create a 'Reachables on launch unlock.csv'\n\n\
        Multiple analyzers separated with spaces will create separate stats files analyzing their individual criteria\n\
        For instance, '{literal}--spawn random --analyzers spawn-location spawn-items{reset}' will create one set of stats 'Spawn Location.csv' and another 'Spawn Items.csv'\n\n\
        However, you can also chain analyzers together by separating them with plus signs (no spaces)\n\
        For instance, '{literal}--spawn random --analyzers spawn-location+spawn-items{reset}' will create a 'Spawn Location and Spawn Items.csv' that contains stats on spawn items for each individual spawn location\n\n\
        To learn more about the available analyzers, try '{literal}--analyzers help{reset}'",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    pub analyzers: Vec<ChainedAnalyzers>,
    // TODO folder_name, tolerated_errors, error_message_limit, overwrite_cache
}

#[derive(Clone)]
pub struct ChainedAnalyzers(pub Vec<Analyzer>);
#[derive(Parser, Clone)]
pub enum Analyzer {
    #[command(about = format!(
        "Analyzes the skills placed early on. By default this means placed in the first 50 reachable locations\n\
        Optionally use '{literal}early-skills:<reachable-limit>{reset}' to override the default value",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    EarlySkills {
        #[arg(default_value = "50")]
        reachable_limit: usize,
    },
    /// Analyzes which weapon gets placed first
    FirstWeapon,
    #[command(about = format!(
        "Analyzes what location an item get placed on\n\
        Use '{literal}item-location:<item-name>{reset}' to specify which item to analyze (Example: '{literal}item-location:Launch{reset}')",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    ItemLocation { item: String },
    #[command(about = format!(
        "Analyzes how many locations are reachable when an item unlocks\n\
        Use '{literal}item-unlock:<item-name>{reset}' to specify which item to analyze (Example: '{literal}item-name:Launch{reset}')\n\
        Optionally use '{literal}item-unlock:<item-name>,<result-bucket-size>{reset}' to group results together in buckets",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    ItemUnlock {
        item: String,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    #[command(about = format!(
        "Analyzes what zone an item is placed in\n\
        Use '{literal}item-zone:<item-name>{reset}' to specify which item to analyze (Example: '{literal}item-zone:Launch{reset}')",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    ItemZone { item: String },
    #[command(about = format!(
        "Analyzes what item gets placed on a location\n\
        Use '{literal}location-item:<location>{reset}' to specify which location to analyze (Example: '{literal}location-item:GladesTown.RebuildTheGlades{reset}')",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    LocationItem { location: String },
    /// Analyzes which items get placed as forced progression
    Progression,
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
        Optionally use '{literal}step-size:<result-bucket-size>{reset}' to group results together in buckets",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    StepSize {
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    // TODO test
    // TODO available values
    #[command(about = format!(
        "Analyzes how much Spirit Light is in a zone\n\
        Use '{literal}zone-spirit-light:<zone>{reset}' to specify which zone to analyze (Example: '{literal}zone-spirit-light:wellspring{reset}')\n\
        Optionally use '{literal}zone-spirit-light:<zone>,<result-bucket-size>{reset}' to group results together in buckets",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    ZoneSpiritLight {
        zone: Zone,
        #[arg(default_value = "1")]
        result_bucket_size: NonZeroUsize,
    },
    #[command(about = format!(
        "Analyzes how many locations are reachable when a zone unlocks\n\
        Use '{literal}zone-unlock:<zone>{reset}' to specify which zone to analyze (Example: '{literal}zone-unlock:wellspring{reset}')\n\
        Optionally use '{literal}zone-unlock:<zone>,<result-bucket-size>{reset}' to group results together in buckets",
        literal = LITERAL.render(),
        reset = Reset.render(),
    ))]
    ZoneUnlock {
        zone: Zone,
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
