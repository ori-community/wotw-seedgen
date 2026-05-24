use crate::{
    cli::{GenerationArgs, PlandoArgs},
    seed::launch_seed,
    Error,
};
use rand_pcg::Pcg64Mcg;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use wotw_seedgen::{
    data::{
        assets::{self, AssetCache, DefaultAssetCacheValues, PlandoFileAccess, Watcher},
        seed_language::{compile::Compiler, output::postprocess},
    },
    seed::Seed,
};

type Cache<'a> = AssetCache<PlandoFileAccess<'a>, DefaultAssetCacheValues>;

pub fn plando(args: PlandoArgs) -> Result<(), Error> {
    let PlandoArgs {
        path,
        out,
        watch,
        generation_args: GenerationArgs { debug, launch },
    } = args;

    let path = assets::canonicalize(path)?;

    let (root, entry, lockfile) = if assets::metadata(&path)?.is_dir() {
        (path.as_path(), "main", path.join(".id_lock.json"))
    } else if path.extension() == Some(OsStr::new("wotws")) {
        let file_stem = path.file_stem().unwrap();
        let identifier = file_stem
            .to_str()
            .ok_or_else(|| format!("\"{}\" is not valid unicode", file_stem.display()))?;

        let root = path.parent().unwrap();

        (
            root,
            identifier,
            path.with_file_name(format!(".{identifier}.id_lock.json")),
        )
    } else {
        return Err(Error(format!(
            "\"{}\" is not a .wotws file or directory",
            path.display()
        )));
    };

    let mut cache = Cache::new(PlandoFileAccess::new(root))?;

    let out = match out {
        None => {
            let mut out: PathBuf = root.join("out");
            assets::create_dir_all(&out)?;
            out.push(path.file_stem().unwrap_or_else(|| OsStr::new("plando")));
            out.set_extension("wotwr");
            out
        }
        Some(out) => {
            if let Some(parent) = out.parent() {
                assets::create_dir_all(parent)?;
            }
            assets::file_create(&out)?;
            assets::canonicalize(out)?
        }
    };

    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);

    let result = compile(&mut rng, &cache, entry, &out, lockfile.clone(), debug);

    launch_seed(&out, launch)?;

    if watch {
        if let Err(err) = result {
            err.eprint();
        }

        let mut watcher = Watcher::new(Duration::from_millis(50))?;

        cache.watch(&mut watcher)?;

        for res in watcher {
            let mut events = res?;

            events.retain_mut(|event| {
                event.event.paths.retain(|path| {
                    fs::canonicalize(path)
                        .ok()
                        .is_some_and(|path| !ignore_file_event(&path, &out))
                });

                !event.event.paths.is_empty()
            });

            if events.is_empty() {
                continue;
            }

            cache.update_from_watcher_event(&events)?;

            if let Err(err) = compile(&mut rng, &cache, entry, &out, lockfile.clone(), debug) {
                err.eprint();
            }
        }

        Ok(())
    } else {
        result
    }
}

fn compile(
    rng: &mut Pcg64Mcg,
    cache: &Cache,
    entry: &str,
    out: &Path,
    lockfile: PathBuf,
    debug: bool,
) -> Result<(), Error> {
    let start = Instant::now();

    let mut compiler = Compiler::new(
        rng,
        cache,
        &cache.uber_state_data,
        Default::default(),
        Some(lockfile),
        debug,
    );

    compiler.compile_snippet(entry)?;
    let mut output = compiler
        .finish()
        .eprint_errors()
        .ok_or_else(|| Error(format!("failed to compile \"{entry}\"")))?;

    let placeholder_map = postprocess(&mut [&mut output], &cache.loc_data, rng)
        .pop()
        .unwrap();

    let seed = Seed::new(output, placeholder_map, debug);

    let mut file = assets::file_create(out)?;
    seed.package(&mut file, !debug)?;

    eprintln!(
        "compiled in {:.2}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        out.display()
    );

    Ok(())
}

fn ignore_file_event(path: &Path, out: &Path) -> bool {
    path.ends_with(".id_lock.json") || path == out
}
