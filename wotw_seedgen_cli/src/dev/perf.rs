use std::{
    iter,
    num::NonZeroUsize,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Instant,
};

use wotw_seedgen::{data::assets::DefaultFileAccess, perf_data::PerfData, Generator};

use crate::{
    cli::dev::{PerfArgs, PerfTarget},
    seed::logic_assets,
    Error,
};

pub fn perf(args: PerfArgs) -> Result<(), Error> {
    let PerfArgs {
        target,
        settings_args,
        duration,
    } = args;

    let settings = settings_args.into_universe_settings()?;
    let (graph, loc_data, uber_state_data) = logic_assets(&settings.world_settings)?;
    let mut perf_data = PerfData::new();

    match target {
        PerfTarget::Reached => perf_data.record_reached(),
    }

    let available = thread::available_parallelism().map_or(4, NonZeroUsize::get);

    eprintln!("Generating seeds for {duration} on {available} threads...");

    let start = Instant::now();
    let count = AtomicU32::new(0);

    thread::scope(|scope| {
        iter::repeat_with(|| {
            thread::Builder::new()
                .name("seedgen".to_string())
                .spawn_scoped(scope, || {
                    let generator = Generator::new(
                        &graph,
                        &loc_data,
                        &uber_state_data,
                        &DefaultFileAccess,
                        &settings,
                    )
                    .with_perf_data(&perf_data);

                    loop {
                        if let Err(err) = generator.generate() {
                            eprintln!("{err}");
                        }

                        count.fetch_add(1, Ordering::Relaxed);

                        if start.elapsed() >= *duration {
                            return;
                        }
                    }
                })
                .expect("failed to spawn thread")
        })
        .take(available)
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|handle| handle.join().expect("a seedgen thread panicked"));
    });

    eprintln!("Generated {} seeds", count.into_inner());

    println!("{}", perf_data.display(&graph));

    Ok(())
}
