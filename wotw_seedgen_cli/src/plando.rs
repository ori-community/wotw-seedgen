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
        seed_language::compile::Compiler,
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

    let cache = Cache::new(PlandoFileAccess::new(root))?;

    let out = match out {
        None => {
            let mut out: PathBuf = root.join("out");
            assets::create_dir_all(&out)?;
            out.push(path.file_stem().unwrap_or_else(|| OsStr::new("plando")));
            out.set_extension("wotwr");
            out
        }
        Some(out) => out,
    };

    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);

    let result = compile(&mut rng, &cache, entry, &out, debug);

    if launch {
        launch_seed(&out)?;
    }

    if watch {
        let mut watcher = Watcher::new(Duration::from_millis(10))?;

        cache.watch(&mut watcher)?;

        let canonical_out = assets::canonicalize(&out)?;

        for res in watcher {
            if res?.into_iter().all(|event| {
                event
                    .event
                    .paths
                    .into_iter()
                    .flat_map(fs::canonicalize)
                    .all(|path| path == canonical_out)
            }) {
                continue;
            }

            let _ = compile(&mut rng, &cache, entry, &out, debug);
        }
    }

    result
}

fn compile(
    rng: &mut Pcg64Mcg,
    cache: &Cache,
    entry: &str,
    out: &Path,
    debug: bool,
) -> Result<(), Error> {
    let start = Instant::now();

    let mut compiler = Compiler::new(
        rng,
        cache,
        &cache.uber_state_data,
        Default::default(),
        debug,
    );

    compiler.compile_snippet(entry)?;
    let mut output = compiler
        .finish()
        .eprint_errors()
        .ok_or_else(|| Error(format!("failed to compile \"{entry}\"")))?;

    let string_placeholder_map = output.postprocess(&cache.loc_data, rng);

    let seed = Seed::new(output, string_placeholder_map, debug);

    let mut file = assets::file_create(out)?;
    seed.package(&mut file, !debug)?;

    eprintln!(
        "compiled in {:.2}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        out.display()
    );

    Ok(())
}
