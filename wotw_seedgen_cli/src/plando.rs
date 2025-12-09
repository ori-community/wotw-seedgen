use crate::{
    cli::{GenerationArgs, PlandoArgs},
    seed::launch_seed,
    Error,
};
use rand_pcg::Pcg64Mcg;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    iter::{self, Chain, Map, Once},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use wotw_seedgen::{seed::Seed, seed_language::compile::Compiler};
use wotw_seedgen_assets::{
    file_err, AssetCache, AssetFileAccess, DefaultAssetCacheValues, DefaultFileAccess,
    PresetFileAccess, SnippetFileAccess, Watcher,
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
            fs::create_dir_all(&out)?;
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

        let canonical_out = fs::canonicalize(&out)?;

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

    let mut file = File::create(out).map_err(|err| file_err("create", out, err))?;
    seed.package(&mut file, !debug)?;

    eprintln!(
        "compiled in {:.2}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        out.display()
    );

    Ok(())
}

struct PlandoFileAccess<'a> {
    root: &'a Path,
}

impl<'a> PlandoFileAccess<'a> {
    fn new(root: &'a Path) -> Self {
        Self { root }
    }
}

impl AssetFileAccess for PlandoFileAccess<'_> {
    type Folders = <DefaultFileAccess as AssetFileAccess>::Folders;
    type Path = <DefaultFileAccess as AssetFileAccess>::Path;

    fn folders(&self) -> Self::Folders {
        AssetFileAccess::folders(&DefaultFileAccess)
    }
}

impl<'a> SnippetFileAccess for PlandoFileAccess<'a> {
    type Folders = Chain<
        Once<Cow<'a, Path>>,
        Map<<DefaultFileAccess as SnippetFileAccess>::Folders, fn(PathBuf) -> Cow<'a, Path>>,
    >;
    type Path = Cow<'a, Path>;

    fn folders(&self) -> Self::Folders {
        iter::once(Cow::Borrowed(self.root))
            .chain(SnippetFileAccess::folders(&DefaultFileAccess).map(Cow::Owned as fn(_) -> _))
    }
}

impl PresetFileAccess for PlandoFileAccess<'_> {
    type Folders = <DefaultFileAccess as PresetFileAccess>::Folders;
    type Path = <DefaultFileAccess as PresetFileAccess>::Path;

    fn universe_folders(&self) -> Self::Folders {
        DefaultFileAccess.universe_folders()
    }

    fn world_folders(&self) -> Self::Folders {
        DefaultFileAccess.world_folders()
    }
}
