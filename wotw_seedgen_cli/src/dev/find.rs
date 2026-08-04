use std::{
    iter,
    num::NonZeroUsize,
    panic,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    thread,
    time::{Duration as StdDuration, Instant},
};

use log::LevelFilter;
use wotw_seedgen::{
    data::{assets::DefaultFileAccess, UniverseSettings},
    log_capture::LogCapture,
    Generator,
};

use crate::{
    cli::{
        dev::find::{Find, SlowArgs},
        SeedSettingsArgs,
    },
    seed::logic_assets,
    Error,
};

pub fn find(command: Find) -> Result<(), Error> {
    match command {
        Find::Panic { args } => panic(args),
        Find::Warning { args } => warning(args),
        Find::Slow { args } => slow(args),
    }
}

fn panic(args: SeedSettingsArgs) -> Result<(), Error> {
    let seed = find_with(args, |generator| {
        panic::catch_unwind(|| {
            let _ = generator.generate();
        })
        .is_err()
    })?;

    println!("panicking seed: {seed}");

    Ok(())
}

fn warning(args: SeedSettingsArgs) -> Result<(), Error> {
    log::set_max_level(LevelFilter::Trace);

    let seed = find_with(args, |generator| {
        let log_capture = LogCapture::new().with_max_level(LevelFilter::Warn);

        generator.with_log_capture(&log_capture).generate().is_err()
            || !log_capture.finish().is_empty()
    })?;

    println!("seed with warning: {seed}");

    Ok(())
}

fn slow(args: SlowArgs) -> Result<(), Error> {
    let SlowArgs {
        min_duration,
        settings_args,
    } = args;

    let min_duration = StdDuration::from(min_duration);

    let seed = find_with(settings_args, |generator| {
        let start = Instant::now();

        let _ = generator.generate();

        start.elapsed() > min_duration
    })?;

    println!("slow seed: {seed}");

    Ok(())
}

fn find_with<F>(settings_args: SeedSettingsArgs, f: F) -> Result<String, Error>
where
    F: Fn(Generator<DefaultFileAccess>) -> bool + Send + Sync,
{
    let start = Instant::now();

    let settings = settings_args.into_universe_settings()?;
    let (graph, loc_data, uber_state_data) = logic_assets(&settings.world_settings)?;

    let available = thread::available_parallelism().map_or(4, NonZeroUsize::get);

    let finished = AtomicBool::new(false);
    let count = AtomicU32::new(0);

    let seed = thread::scope(|scope| {
        iter::repeat_with(|| {
            thread::Builder::new()
                .name("seedgen".to_string())
                .spawn_scoped(scope, || loop {
                    let count = count.fetch_add(1, Ordering::Relaxed);
                    let settings = UniverseSettings {
                        seed: count.to_string(),
                        ..settings.clone()
                    };

                    eprint!("Generating seed {}\r", settings.seed);

                    let generator = Generator::new(
                        &graph,
                        &loc_data,
                        &uber_state_data,
                        &DefaultFileAccess,
                        &settings,
                    );

                    if f(generator) {
                        finished.store(true, Ordering::Relaxed);

                        return Some(settings.seed);
                    }

                    if finished.load(Ordering::Relaxed) {
                        return None;
                    }
                })
                .expect("failed to spawn thread")
        })
        .take(available)
        .collect::<Vec<_>>()
        .into_iter()
        .fold(None, |seed, handle| {
            seed.or(handle.join().expect("a seedgen thread panicked"))
        })
    })
    .unwrap();

    eprintln!("found seed in {:.2}s", start.elapsed().as_secs_f32());

    Ok(seed)
}
