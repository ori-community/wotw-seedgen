use std::iter;

use wotw_seedgen::{generator::SeedSpoiler, settings::UniverseSettings};

use crate::Result;

/// Access seed files across stats runs
///
/// When generating stats multiple times with the same settings, seeds generated for previous runs can be reused  
/// These trait methods will be used to store and reuse seeds across stats runs
///
/// You can work without this behaviour by using [`NoFileAccess`], generating all seeds frmo scratch every time
///
/// If the `fs` feature is enabled, you may use [`FileSystemAccess`] to store seeds in the local filesystem
pub trait FileAccess {
    type Iter: Iterator<Item = Result<SeedSpoiler>>;

    /// fetch seeds that have been previously generated with these settings
    fn read_seeds(settings: &UniverseSettings, limit: usize) -> Result<Self::Iter>;
    /// write a seed generated from these settings for later use
    fn write_seed(seed: &SeedSpoiler, settings: &UniverseSettings) -> Result<()>;
    /// clean all seeds that have previously been generated with these settings
    fn clean_seeds(settings: &UniverseSettings) -> Result<()>;
    /// clean all seeds that have previously been generated
    fn clean_all_seeds() -> Result<()>;
}

/// A [`FileAccess`] implementation prohibiting access to any files
///
/// This prevents storing seeds for future stats, so all the seeds will have to be generated from scratch every time
pub struct NoFileAccess;
impl FileAccess for NoFileAccess {
    type Iter = iter::Empty<Result<SeedSpoiler>>;

    fn read_seeds(_: &UniverseSettings, _: usize) -> Result<Self::Iter> {
        Ok(iter::empty())
    }
    fn write_seed(_: &SeedSpoiler, _: &UniverseSettings) -> Result<()> {
        Ok(())
    }
    fn clean_seeds(_: &UniverseSettings) -> Result<()> {
        Ok(())
    }
    fn clean_all_seeds() -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "fs")]
pub use fs_access::*;
#[cfg(feature = "fs")]
mod fs_access {
    use crate::handle_errors::HandleErrors;

    use super::*;

    use std::{
        fs::{self, DirEntry, ReadDir},
        hash::{Hash, Hasher},
        io::{self, Write},
        path::{Path, PathBuf},
    };

    use rustc_hash::FxHasher;

    const SEED_STORAGE_FOLDER: &str = "seed_storage";

    /// A [`FileAccess`] implementation storing and fetching seeds using the local filesystem
    pub struct FileSystemAccess;
    impl FileAccess for FileSystemAccess {
        type Iter = ReadSeeds;

        fn read_seeds(settings: &UniverseSettings, limit: usize) -> Result<Self::Iter> {
            let path = path_from_settings(settings);

            ReadSeeds::new(path, limit)
        }

        fn write_seed(seed: &SeedSpoiler, settings: &UniverseSettings) -> Result<()> {
            let bytes = bincode::serialize(seed).expect("Failed to serialize spoiler");
            let base_path = path_from_settings(settings);
            fs::create_dir_all(&base_path)
                .map_err(|err| format!("Failed to create folder for seed storage: {err}"))?;
            let mut index = 0;
            loop {
                let mut path = base_path.to_path_buf();
                path.push(index.to_string());

                match fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                {
                    Ok(mut file) => {
                        file.write_all(bytes.as_ref())
                            .map_err(|err| format!("Failed to write seed to storage: {err}"))?;
                        return Ok(());
                    }
                    Err(err) => {
                        if err.kind() == io::ErrorKind::AlreadyExists {
                            index += 1
                        } else {
                            return Err(format!("Failed to write seed to storage: {err}"));
                        }
                    }
                }
            }
        }
        fn clean_seeds(settings: &UniverseSettings) -> Result<()> {
            let path = path_from_settings(settings);
            fs::remove_dir_all(path).map_err(|err| format!("Failed to clean seed storage: {err}"))
        }
        fn clean_all_seeds() -> Result<()> {
            fs::remove_dir_all(SEED_STORAGE_FOLDER)
                .map_err(|err| format!("Failed to clean seed storage: {err}"))
        }
    }

    // An Iterator reading stored seeds from the filesystem
    pub struct ReadSeeds {
        inner: Option<iter::Map<HandleErrorsReadDir, fn(DirEntry) -> Result<SeedSpoiler>>>,
    }
    impl ReadSeeds {
        fn new(path: PathBuf, limit: usize) -> Result<Self> {
            fn format_read_dir_err(err: io::Error, path: &Path) -> String {
                format!(
                    "Failed to access seed storage at \"{}\": {}",
                    path.display(),
                    err
                )
            }

            match read_dir(&path, limit) {
                Ok(dir) => print_feedback_for_existing_seeds(dir),
                Err(err) => {
                    return if err.kind() == io::ErrorKind::NotFound {
                        Ok(Self { inner: None })
                    } else {
                        Err(format_read_dir_err(err, &path))
                    }
                }
            }

            let inner = read_dir(&path, limit)
                .map_err(|err| format_read_dir_err(err, &path))?
                .map(
                    (|entry| {
                        let path = entry.path();
                        fs::read(&path)
                            .map_err(|err| {
                                (
                                    format!(
                                        "Failed to read seed from seed storage at \"{}\": {}",
                                        path.display(),
                                        err
                                    ),
                                    path.clone(),
                                )
                            })
                            .map(|bytes| (bytes, path))
                            .and_then(|(bytes, path)| {
                                bincode::deserialize::<SeedSpoiler>(&bytes).map_err(|err| {
                                    (
                                        format!(
                                            "Failed to deserialize seed from seed storage at \"{}\": {}",
                                            path.display(),
                                            err
                                        ),
                                        path,
                                    )
                                })
                            })
                            .map_err(|(err, path)| {
                                match fs::remove_file(&path) {
                                    Ok(()) => {
                                        eprintln!(
                                            "Removed \"{}\" from seed storage",
                                            path.display()
                                        )
                                    }
                                    Err(err) => {
                                        eprintln!(
                                            "Failed to remove \"{}\" from seed storage: {}",
                                            path.display(),
                                            err
                                        )
                                    }
                                }
                                err
                            })
                    }) as fn(DirEntry) -> Result<SeedSpoiler>,
                );

            Ok(Self { inner: Some(inner) })
        }
    }
    impl Iterator for ReadSeeds {
        type Item = Result<SeedSpoiler>;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.as_mut().and_then(Iterator::next)
        }
    }
    type HandleErrorsReadDir =
        HandleErrors<DirEntry, io::Error, iter::Take<ReadDir>, fn(io::Error)>;
    fn read_dir<P: AsRef<Path>>(path: P, limit: usize) -> io::Result<HandleErrorsReadDir> {
        fs::read_dir(&path).map(|read_dir| {
            HandleErrors::new(
                read_dir.take(limit),
                (|err| eprintln!("Failed to read from seed storage: {err}")) as fn(io::Error),
            )
        })
    }

    fn path_from_settings(settings: &UniverseSettings) -> PathBuf {
        let folder = format!("{:x}", hash_settings(settings));
        let mut path = PathBuf::from(SEED_STORAGE_FOLDER);
        path.push(folder);
        path
    }
    fn hash_settings(settings: &UniverseSettings) -> u64 {
        let mut hasher = FxHasher::default();
        let bytes =
            bincode::serialize(&settings.world_settings).expect("Failed to serialize settings"); // We deliberately ignore the seed
        bytes.hash(&mut hasher);
        hasher.finish()
    }
    fn print_feedback_for_existing_seeds(seeds: HandleErrorsReadDir) {
        let modify_timestamps = HandleErrors::new_print_errors(
            HandleErrors::new_print_errors(seeds.map(|entry| {
                let path = entry.path();
                entry
                    .metadata()
                    .map_err(|err| {
                        format!(
                            "Failed to read metadata for \"{}\": {}",
                            path.display(),
                            err
                        )
                    })
                    .map(|metadata| (metadata, path))
            }))
            .map(|(metadata, path)| {
                metadata.modified().map_err(|err| {
                    format!(
                        "Failed to read modified timestamp for \"{}\": {}",
                        path.display(),
                        err
                    )
                })
            }),
        )
        .map(chrono::DateTime::<chrono::Local>::from)
        .collect::<Vec<_>>();

        let amount = modify_timestamps.len();
        let oldest = modify_timestamps.iter().min().copied();
        let newest = modify_timestamps.into_iter().max();

        if let Some((oldest, newest)) = Option::zip(oldest, newest) {
            let fmt = "%c";
            eprintln!(
                "Reusing {} seed{} with these settings from a previous run, generated between {} and {}",
                amount,
                if amount == 1 { "" } else { "s" },
                oldest.format(fmt),
                newest.format(fmt)
            )
        }
    }
}
