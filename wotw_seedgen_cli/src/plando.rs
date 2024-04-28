use crate::{cli::PlandoArgs, files, Error};
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::{
    ffi::OsStr,
    fs::{self, File},
    mem,
};
use wotw_seedgen::{
    seed::{assembly::Command, Seed},
    seed_language::{compile::Compiler, output::DebugOutput},
};

pub fn plando(args: PlandoArgs) -> Result<(), Error> {
    let PlandoArgs {
        path,
        out_name,
        debug,
    } = args;

    let (root, entry) = if path
        .metadata()
        .map_err(|err| format!("{err}: {}", path.display()))?
        .is_dir()
    {
        (path.as_path(), "main")
    } else {
        let mut file_name = path
            .file_name()
            .ok_or_else(|| format!("{} is not a file or directory", path.display()))?
            .to_str()
            .ok_or_else(|| format!("{} is not valid unicode", path.display()))?
            .rsplit('.');
        match (file_name.next(), file_name.next()) {
            (Some("wotws"), Some(identifier)) => {
                let root = path.parent().unwrap();
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

    let logic_access = files::logic_access(root)?;
    let uber_state_data =
        logic_access.uber_state_data(logic_access.loc_data()?, logic_access.state_data()?)?;

    let mut rng = rand::thread_rng();
    let snippet_access = files::snippet_access(root)?;
    let mut compiler = Compiler::new(
        &mut rng,
        &snippet_access,
        &uber_state_data,
        Default::default(),
    );
    if debug {
        compiler.debug();
    }
    compiler.compile_snippet(entry)?;
    let (mut output, success) = compiler.finish().eprint_errors();
    if !success {
        return Err("Compilation failed".into());
    }

    let debug_output = mem::take(&mut output.debug);

    let mut seed = Seed::new(output);

    if debug {
        let metadata = Metadata {
            compiler_data: debug_output.unwrap(),
            indexed_lookup: seed
                .assembly
                .command_lookup
                .iter()
                .cloned()
                .enumerate()
                .collect(),
        };
        seed.assets.insert(
            "debug.json".to_string(),
            serde_json::to_vec_pretty(&metadata)?,
        );
    }

    let mut out = root.join("out");
    fs::create_dir_all(&out)?;
    match out_name {
        None => {
            out.push(path.file_stem().unwrap_or_else(|| OsStr::new("plando")));
            out.set_extension("wotwr");
        }
        Some(name) => out.push(format!("{name}.wotwr")),
    }
    let mut file = File::create(&out)
        .map_err(|err| format!("failed to create \"{}\": {err}", out.display()))?;
    seed.package(&mut file, !debug)?;

    eprintln!("compiled successfully to \"{}\"", out.display());

    Ok(())
}

#[derive(Serialize)]
struct Metadata {
    compiler_data: DebugOutput,
    indexed_lookup: FxHashMap<usize, Vec<Command>>,
}
