mod cli;
mod headers;
mod log_init;
mod play;
mod reach_check;
mod seed;
mod stats;
mod universe_preset;
mod world_preset;

use std::process::ExitCode;

use bugsalot::debugger;
use structopt::StructOpt;

fn main() -> ExitCode {
    let args = cli::SeedGen::from_args();

    if args.wait_on_debugger {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).expect("state() not implemented on this platform");
    }

    match args.command {
        cli::SeedGenCommand::Seed { args } => seed::generate_seeds(args),
        cli::SeedGenCommand::Play => play::play(),
        cli::SeedGenCommand::UniversePreset { args } => {
            universe_preset::create_universe_preset(args)
        }
        cli::SeedGenCommand::WorldPreset { args } => world_preset::create_world_preset(args),
        cli::SeedGenCommand::Stats { args } => stats::generate_stats(args),
        cli::SeedGenCommand::CleanStatsCache => stats::clean_stats_cache(),
        cli::SeedGenCommand::Headers {
            headers,
            subcommand,
        } => headers::headers(headers, subcommand),
        cli::SeedGenCommand::ReachCheck { args } => reach_check::reach_check(args),
        cli::SeedGenCommand::Regenerate { args } => seed::regenerate_seed(args),
    }
    .map_or_else(
        |err| {
            log::error!("{err}");
            ExitCode::FAILURE
        },
        |()| ExitCode::SUCCESS,
    )
}
