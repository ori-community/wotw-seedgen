// TODO mod seed_storage;

use lazy_static::lazy_static;
use std::{
    env,
    path::{Path, PathBuf},
};
use wotw_seedgen_assets::FileAccess;

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
