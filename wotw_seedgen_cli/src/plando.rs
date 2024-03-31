use crate::{cli::PlandoArgs, files::read_assets, Error};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{
    fs,
    io::ErrorKind,
    mem,
    path::{Path, PathBuf},
};
use wotw_seedgen::assembly::{
    compile_intermediate_output,
    seed_language::{
        assets::{SnippetAccess, Source},
        compile::Compiler,
        output::DebugOutput,
    },
    Command, Package,
};

pub fn plando(args: PlandoArgs) -> Result<(), Error> {
    let PlandoArgs { path, debug } = args;

    let uber_state_data = read_assets()?.uber_state_data;

    fs::create_dir_all("seeds/out")?;

    let (root, entry) = if path
        .metadata()
        .map_err(|err| format!("{err}: {}", path.display()))?
        .is_dir()
    {
        (path, "main")
    } else {
        let mut file_name = path
            .file_name()
            .ok_or_else(|| format!("{} is not a file or directory", path.display()))?
            .to_str()
            .ok_or_else(|| format!("{} is not valid unicode", path.display()))?
            .rsplit('.');
        match (file_name.next(), file_name.next()) {
            (Some("wotws"), Some(identifier)) => {
                let root = path.parent().unwrap().to_path_buf();
                (root, identifier)
            }
            _ => {
                return Err(Error(format!(
                    "{} is not a .wotws file or directory",
                    path.display()
                )))
            }
        }
    };

    let mut rng = rand::thread_rng();
    let files = Files { root };
    let mut compiler = Compiler::new(&mut rng, &files, &uber_state_data, Default::default());
    if debug {
        compiler.debug();
    }
    compiler.compile_snippet(entry)?;
    let (mut output, success) = compiler.finish().eprint_errors();
    if !success {
        return Err("Compilation failed".into());
    }

    let debug_output = mem::take(&mut output.debug);

    let mut package = Package::new("seeds/out/out.wotwr")?;
    let (seed_world, icons) = compile_intermediate_output(output);
    package.add_seed(&seed_world, debug)?;
    for (name, icon) in icons {
        let mut path = PathBuf::from("assets");
        path.push(name);
        package.add_data(path, icon)?;
    }

    if debug {
        let metadata = Metadata {
            compiler_data: debug_output.unwrap(),
            indexed_lookup: seed_world
                .command_lookup
                .iter()
                .cloned()
                .enumerate()
                .collect(),
        };
        package.add_data("debug", serde_json::to_vec_pretty(&metadata)?)?;
    }

    package.finish()?;

    Ok(())
}

#[derive(Serialize)]
struct Metadata {
    compiler_data: DebugOutput,
    indexed_lookup: FxHashMap<usize, Vec<Command>>,
}

struct Files {
    root: PathBuf,
}
impl SnippetAccess for Files {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        let mut filename = PathBuf::from(identifier);
        filename.set_extension("wotws");

        let mut path_plando = self.root.clone();
        path_plando.push(&filename);
        if let Some(result) = try_read(&path_plando) {
            return result;
        }

        let mut path_snippet = PathBuf::from("snippets");
        path_snippet.push(&filename);
        if let Some(result) = try_read(&path_snippet) {
            return result;
        }

        Err(format!(
            "failed to find \"{}\" at \"{}\" or \"{}\"",
            identifier,
            path_plando.display(),
            path_snippet.display(),
        ))
    }
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        let mut full_path = self.root.clone();
        full_path.push(path);
        fs::read(full_path).map_err(|err| err.to_string())
    }
}

fn try_read(path: &Path) -> Option<Result<Source, String>> {
    match fs::read_to_string(&path) {
        Ok(content) => Some(Ok(Source::new(path.to_string_lossy().to_string(), content))),
        Err(err) => match err.kind() {
            ErrorKind::NotFound => None,
            _ => Some(Err(format!(
                "failed to read \"{}\": {}",
                path.display(),
                err
            ))),
        },
    }
}
