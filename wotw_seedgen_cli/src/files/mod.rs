use lazy_static::lazy_static;
use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    time::Instant,
};
use wotw_seedgen::SeedUniverse;
use wotw_seedgen_assets::{file_err, FileAccess};

use crate::Error;

lazy_static! {
    static ref EXECUTABLE_DIR: Result<PathBuf, String> = env::current_exe()
        .map(|mut executable| {
            executable.pop();
            executable
        })
        .map_err(|err| format!("failed to read executable path: {err}"));
}

pub fn logic_access<P: AsRef<Path>>(root: P) -> Result<FileAccess, String> {
    folder_access(root, "logic")
}

pub fn snippet_access<P: AsRef<Path>>(root: P) -> Result<FileAccess, String> {
    folder_access(root, "snippets")
}

pub fn preset_access<P: AsRef<Path>>(root: P) -> Result<FileAccess, String> {
    Ok(FileAccess::new([root.as_ref(), EXECUTABLE_DIR.as_ref()?]))
}

fn folder_access<P: AsRef<Path>>(root: P, folder: &str) -> Result<FileAccess, String> {
    let root = root.as_ref();
    let executable_dir = EXECUTABLE_DIR.as_ref()?;
    Ok(FileAccess::new([
        root.to_path_buf(),
        root.join(folder),
        executable_dir.to_path_buf(),
        executable_dir.join(folder),
    ]))
}

pub fn write_seed(
    mut seed_universe: SeedUniverse,
    name: &str,
    debug: bool,
    launch: bool,
    start: Instant,
) -> Result<(), Error> {
    fs::create_dir_all("seeds")?;

    let path = if seed_universe.worlds.len() == 1 {
        let (mut file, path) = create_unique_file(&format!("seeds/{name}"))?;
        let seed = seed_universe.worlds.pop().unwrap();
        seed.package(&mut file, !debug)?;

        if launch {
            launch_seed(&path)?;
        }

        let spoiler_path = format!("{}_spoiler.txt", &path[..path.len() - ".wotwr".len()]);
        fs::write(&spoiler_path, seed_universe.spoiler.to_string())
            .map_err(|err| file_err("write", &spoiler_path, err))?;

        path
    } else {
        let path = create_unique_dir(&format!("seeds/{name}"))?;

        for (index, seed) in seed_universe.worlds.into_iter().enumerate() {
            let path = format!("{path}/world_{index}.wotwr");
            let mut file = File::create(&path).map_err(|err| file_err("create", path, err))?;
            seed.package(&mut file, !debug)?;
        }

        let spoiler_path = format!("{path}/spoiler.txt");
        fs::write(&spoiler_path, seed_universe.spoiler.to_string())
            .map_err(|err| file_err("write", &spoiler_path, err))?;

        path
    };

    eprintln!(
        "Generated seed in {:.1}s to \"{path}\"",
        start.elapsed().as_secs_f32()
    );

    Ok(())
}

fn create_unique_file(path: &str) -> Result<(File, String), Error> {
    create_unique::<_, File>(path, ".wotwr", |path| File::create_new(path))
}

fn create_unique_dir(path: &str) -> Result<String, Error> {
    create_unique::<_, ()>(path, "", |path| fs::create_dir(path)).map(|(_, path)| path)
}

fn create_unique<F, T>(path: &str, extension: &str, mut f: F) -> Result<(T, String), Error>
where
    F: FnMut(&str) -> io::Result<T>,
{
    for attempt in 0_u32.. {
        let path = if attempt == 0 {
            format!("{path}{extension}")
        } else {
            format!("{path}_{attempt}{extension}")
        };

        match f(&path) {
            Ok(t) => return Ok((t, path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(Error(file_err("create", path, err))),
        }
    }

    unreachable!()
}

pub fn launch_seed<P: AsRef<Path> + AsRef<OsStr>>(path: P) -> Result<(), Error> {
    Ok(open::that_detached(&path).map_err(|err| file_err("launch", path, err))?)
}
