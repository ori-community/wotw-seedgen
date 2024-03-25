use crate::{
    cli::{self, SeedSettings, StatsArgs},
    files::{compile_graph, read_assets, PresetFileAccess},
    Error,
};
use wotw_seedgen::settings::UniverseSettings;
use wotw_seedgen_stats::ChainedAnalyzers;

pub fn stats(args: StatsArgs) -> Result<(), Error> {
    let StatsArgs {
        settings: SeedSettings(universe_preset),
        sample_size,
        analyzers,
    } = args;

    let mut settings = UniverseSettings::new("".to_string());
    settings.apply_preset(universe_preset, &PresetFileAccess)?;

    let assets = read_assets()?;
    let graph = compile_graph(assets.loc_data, assets.state_data, &settings.world_settings)?;

    // TODO
    // let x = wotw_seedgen_stats::stats(wotw_seedgen_stats::StatsArgs {
    //     settings,
    //     sample_size: args.sample_size,
    //     analyzers: args.analyzers.into_iter().map(Into::into).collect(),
    //     graph: &graph,
    //     snippet_access: &SnippetFileAccess,
    //     uber_state_data: &uber_state_data,
    //     tolerated_errors: None,
    //     error_message_limit: None,
    //     overwrite_seed_storage: false,
    // });

    Ok(())
}

impl Into<ChainedAnalyzers> for cli::ChainedAnalyzers {
    fn into(self) -> ChainedAnalyzers {
        todo!()
    }
}
