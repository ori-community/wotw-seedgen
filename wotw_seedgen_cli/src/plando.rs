use crate::{
    cli::{GenerationArgs, PlandoArgs},
    files::{self, launch_seed},
    Error,
};
use notify_debouncer_full::{
    new_debouncer,
    notify::{EventKind, RecursiveMode},
};
use rand::rngs::ThreadRng;
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};
use wotw_seedgen::{seed::Seed, seed_language::compile::Compiler};
use wotw_seedgen_assets::{file_err, FileAccess, LocData, UberStateData};

pub fn plando(args: PlandoArgs) -> Result<(), Error> {
    let PlandoArgs {
        path,
        out,
        watch,
        generation_args: GenerationArgs { debug, launch },
    } = args;

    let (root, entry) = if path
        .metadata()
        .map_err(|err| format!("{err}: {}", path.display()))?
        .is_dir()
    {
        (path.as_path(), "main")
    } else {
        if path.extension() != Some(OsStr::new("wotws")) {
            return Err(Error(format!(
                "\"{}\" is not a .wotws file or directory",
                path.display()
            )));
        }

        let file_stem = path.file_stem().unwrap();
        let identifier = file_stem
            .to_str()
            .ok_or_else(|| format!("\"{}\" is not valid unicode", file_stem.display()))?;

        let root = path.parent().unwrap();

        (root, identifier)
    };

    let logic_access = files::logic_access("")?;
    let loc_data = logic_access.loc_data()?;
    let uber_state_data = logic_access.uber_state_data(&loc_data, &logic_access.state_data()?)?;

    let mut rng = rand::thread_rng();
    let snippet_access = files::snippet_access(root)?;

    let out = match out {
        None => {
            let mut out: PathBuf = root.join("out");
            fs::create_dir_all(&out)?;
            out.push(path.file_stem().unwrap_or_else(|| OsStr::new("plando")));
            out.set_extension("wotwr");
            out
        }
        Some(out) => out,
    };

    let result = compile(
        &mut rng,
        &snippet_access,
        &loc_data,
        &uber_state_data,
        entry,
        &out,
        debug,
    );

    if launch {
        launch_seed(&out)?;
    }

    if watch {
        let (tx, rx) = mpsc::channel();

        let mut watcher = new_debouncer(Duration::from_millis(10), None, tx)?;

        watcher.watch(Path::new(root), RecursiveMode::Recursive)?;

        for res in rx {
            let events = res.map_err(|mut errors| errors.pop().unwrap())?;

            if !events.into_iter().any(|event| {
                matches!(
                    event.event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                )
            }) {
                continue;
            }

            let _ = compile(
                &mut rng,
                &snippet_access,
                &loc_data,
                &uber_state_data,
                entry,
                &out,
                debug,
            );
        }
    }

    result
}

fn compile(
    rng: &mut ThreadRng,
    snippet_access: &FileAccess,
    loc_data: &LocData,
    uber_state_data: &UberStateData,
    entry: &str,
    out: &Path,
    debug: bool,
) -> Result<(), Error> {
    let start = Instant::now();

    let mut compiler = Compiler::new(
        rng,
        snippet_access,
        uber_state_data,
        Default::default(),
        debug,
    );

    compiler.compile_snippet(entry)?;
    let (output, success) = compiler.finish().eprint_errors();
    if !success {
        return Err("compilation failed".into());
    }

    let seed = Seed::new(output, loc_data, debug);

    let mut file = File::create(out).map_err(|err| file_err("create", out, err))?;
    seed.package(&mut file, !debug)?;

    eprintln!(
        "compiled in {:.1}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        out.display()
    );

    Ok(())
}
