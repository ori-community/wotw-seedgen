use std::time::Instant;

use crate::{cli::dev::PrintOptimizedGraphArgs, log_config::LogConfig, seed::logic_assets, Error};

pub fn print_optimized_graph(args: PrintOptimizedGraphArgs) -> Result<(), Error> {
    let PrintOptimizedGraphArgs {
        settings_args,
        verbose_args,
    } = args;

    // TODO maybe move the start times a bit after the purely cli-related work?
    let start = Instant::now();

    LogConfig::from_args(verbose_args).apply()?;

    let settings = settings_args.into_universe_settings()?;

    let (graph, _, _) = logic_assets(&settings.world_settings)?;

    println!("{}", graph.decompile());

    eprintln!("Printed graph in {:.1}s", start.elapsed().as_secs_f32());

    Ok(())
}
