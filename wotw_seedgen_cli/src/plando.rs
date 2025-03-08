use crate::{
    cli::{GenerationArgs, PlandoArgs},
    files::{self, launch_seed},
    Error,
};
use std::{
    ffi::OsStr,
    fs::{self, File},
};
use wotw_seedgen::{seed::Seed, seed_language::compile::Compiler};
use wotw_seedgen_assets::file_err;

pub fn plando(args: PlandoArgs) -> Result<(), Error> {
    let PlandoArgs {
        path,
        out,
        generation_args: GenerationArgs { debug, launch },
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
        debug,
    );
    compiler.compile_snippet(entry)?;
    let (output, success) = compiler.finish().eprint_errors();
    if !success {
        return Err("Compilation failed".into());
    }

    let seed = Seed::new(output, debug);

    let out = match out {
        None => {
            let mut out = root.join("out");
            fs::create_dir_all(&out)?;
            out.push(path.file_stem().unwrap_or_else(|| OsStr::new("plando")));
            out.set_extension("wotwr");
            out
        }
        Some(out) => out,
    };

    let mut file = File::create(&out).map_err(|err| file_err("create", &out, err))?;
    seed.package(&mut file, !debug)?;

    eprintln!("compiled successfully to \"{}\"", out.display());

    if launch {
        launch_seed(out)?;
    }

    Ok(())
}
