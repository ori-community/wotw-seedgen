/// Access preset and header files
/// 
/// You will have to provide an implementation of `FileAccess` whenever a preset or header identifier needs to be resolved
/// 
/// The [`files`](crate::files) module contains an implementations of the `FileAccess` trait based on the local file system if you have the "fs" feature flag enabled
pub trait FileAccess {
    /// Read a [`UniversePreset`](crate::preset::UniversePreset) with the given identifier, returning its contents
    fn read_universe_preset(&self, identifier: &str) -> Result<String, String>;
    /// Read a [`WorldPreset`](crate::preset::WorldPreset) with the given identifier, returning its contents
    fn read_world_preset(&self, identifier: &str) -> Result<String, String>;
    /// Read a [`Header`](crate::header::Header) with the given identifier, returning its contents
    fn read_header(&self, identifier: &str) -> Result<String, String>;
}

#[cfg(any(feature = "fs", test))]
pub use fs_access::*;
#[cfg(any(feature = "fs", test))]
mod fs_access {
    use super::*;

    use std::ffi::OsStr;
    use std::fs;
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};

    const UNIVERSE_PRESET_FOLDER: &str = "universe_presets";
    const WORLD_PRESET_FOLDER: &str = "world_presets";
    const HEADER_FOLDER: &str = "headers";

    /// A [`FileAccess`] implementation searching for identifiers in the local filesystem
    /// 
    /// It will append the appropriate file extensions, e.g. requesting "black_market" will search for "black_market.wotwrh"
    /// 
    /// All reads will first be attempted in defined subfolders:
    /// - "universe_presets" for [`UniversePreset`](crate::preset::UniversePreset)s
    /// - "world_presets" for [`WorldPreset`](crate::preset::WorldPreset)s
    /// - "headers" for [`Header`](crate::header::Header)s
    /// 
    /// If unable to perform the operation in the subfolder, it will be attempted in the current directory instead
    pub struct FileSystemAccess;
    impl FileAccess for FileSystemAccess {
        fn read_universe_preset(&self, identifier: &str) -> Result<String, String> {
            read_file(identifier, "json", UNIVERSE_PRESET_FOLDER)
        }
        fn read_world_preset(&self, identifier: &str) -> Result<String, String> {
            read_file(identifier, "json", WORLD_PRESET_FOLDER)
        }
        fn read_header(&self, identifier: &str) -> Result<String, String> {
            read_file(identifier, "wotwrh", HEADER_FOLDER)
        }
    }
    impl FileSystemAccess {
        pub fn write_universe_preset(&self, identifier: &str, contents: &str) -> Result<(), String> {
            write_file(identifier, "json", contents, UNIVERSE_PRESET_FOLDER)
        }
        pub fn write_world_preset(&self, identifier: &str, contents: &str) -> Result<(), String> {
            write_file(identifier, "json", contents, WORLD_PRESET_FOLDER)
        }
        pub fn write_header(&self, identifier: &str, contents: &str) -> Result<(), String> {
            write_file(identifier, "wotwrh", contents, HEADER_FOLDER)
        }
    }
    /// Instance of [`FileSystemAccess`]
    pub const FILE_SYSTEM_ACCESS: FileSystemAccess = FileSystemAccess;

    /// Read a file
    /// 
    /// The read will first be attempted in the `default_folder` subfolder. If unable to perform the operation in the subfolder, it will be attempted in the current directory instead
    pub fn read_file(identifier: &str, extension: impl AsRef<OsStr>, default_folder: impl AsRef<Path>) -> Result<String, String> {
        let path = with_extension(identifier, extension);
        fs::read_to_string(in_folder(&path, default_folder)).or_else(|_| {
            fs::read_to_string(&path).map_err(|err| format!("Failed to read file {}: {}", path.display(), err))
        })
    }
    /// Write a file
    /// 
    /// The write will first be attempted in the `default_folder` subfolder. If unable to perform the operation in the subfolder, it will be attempted in the current directory instead
    pub fn write_file(identifier: &str, extension: impl AsRef<OsStr>, contents: &str, default_folder: impl AsRef<Path>) -> Result<(), String> {
        let path = with_extension(identifier, extension);
        write_in_folder(in_folder(&path, default_folder), contents).or_else(|_| {
            write_in_folder(&path, contents).map_err(|err| format!("Failed to write file {}: {}", path.display(), err))
        })
    }

    /// Find all [`Header`](crate::header::Header)s in the current directory and "headers" subdirectory
    pub fn find_headers() -> Result<Vec<PathBuf>, String> {
        let mut current_directory = PathBuf::from(".");
        let mut headers = files_in_directory(&current_directory, "wotwrh")?;
        current_directory.push(&HEADER_FOLDER);
        if let Ok(mut more) = files_in_directory(&current_directory, "wotwrh") {
            headers.append(&mut more);
        }

        Ok(headers)
    }

    fn with_extension(identifier: &str, extension: impl AsRef<OsStr>) -> PathBuf {
        Path::new(identifier).with_extension(extension)
    }
    fn in_folder(path: impl AsRef<Path>, default_folder: impl AsRef<Path>) -> PathBuf {
        let mut folder = default_folder.as_ref().to_path_buf();
        folder.push(path.as_ref());
        folder
    }
    fn write_in_folder(file: impl AsRef<Path>, contents: &str) -> Result<(), io::Error> {
        let path = file.as_ref();
        match fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path) {
                Ok(mut file) => file.write_all(contents.as_bytes()),
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir_all(path.parent().unwrap())?;
                    write_in_folder(file, contents)
                },
                Err(err) => Err(err),
            }
    }
    fn files_in_directory(directory: &Path, extension: &str) -> Result<Vec<PathBuf>, String> {
        fs::read_dir(directory)
            .map_err(|err| format!("Failed to read directory {}: {}", directory.display(), err))
            .map(|dir| dir
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if let Some(path_extension) = path.extension() {
                            if path_extension == extension {
                                return Some(path);
                            }
                        }
                    }
                    None
                })
                .collect())
    }
}
